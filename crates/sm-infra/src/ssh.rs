//! Russh-based SSH runner with TOFU host-key verification.
//!
//! Connect timeout, host-key trust and auth method selection are the runner's
//! concerns; non-zero remote exit codes are passed through as data (the
//! service layer decides what to do with them).

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;

use russh::keys::{HashAlg, PrivateKeyWithHashAlg};
use russh::{client, ChannelMsg};
use sm_core::{Machine, SecretMode};
use sm_services::{
    CommandOutput, HostKeyStore, HostKeyVerdict, SecretKind, SecretStore, ServiceError, SshRunner,
};

/// Default TCP + SSH handshake timeout used by [`RusshRunner::new`]
/// (no `with_connect_timeout` configurator exists in M1).
const DEFAULT_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// One authentication attempt, in the order the runner tries methods.
#[derive(Debug, PartialEq)]
pub(crate) enum AuthStep {
    Agent,
    Key { path: String, has_passphrase: bool },
    Password,
}

/// Effective secret mode: machine override or app default (shared
/// `RwLock`, so `save_settings` changes apply without a restart). The
/// `Keyring` → `Prompt` mapping (user decision 2026-09-11) happens in
/// [`auth_plan`].
fn effective_mode(machine: &Machine, default_mode: &RwLock<SecretMode>) -> SecretMode {
    machine.secret_mode.unwrap_or(*default_mode.read().unwrap())
}

/// Pure auth-method selection.
///
/// - `Agent` first only when the agent is available and no explicit key is
///   configured.
/// - `Key` when `machine.key_path` is set (passphrase is expected to come from
///   the secret store).
/// - `Password` when a password source may exist: the machine holds a
///   plaintext password, or the effective mode is `Prompt` (the store may hold
///   one).
pub(crate) fn auth_plan(
    machine: &Machine,
    mode: SecretMode,
    agent_available: bool,
) -> Vec<AuthStep> {
    // Keyring is deferred beyond M1: a prompt-mode store lookup is attempted.
    let mode = match mode {
        SecretMode::Keyring => SecretMode::Prompt,
        m => m,
    };
    let mut plan = Vec::new();
    match machine.key_path.as_deref() {
        Some(path) => plan.push(AuthStep::Key {
            path: path.to_owned(),
            has_passphrase: true,
        }),
        None if agent_available => plan.push(AuthStep::Agent),
        None => {}
    }
    if machine.ssh_password.is_some() || mode == SecretMode::Prompt {
        plan.push(AuthStep::Password);
    }
    plan
}

/// Fingerprint + verdict handed back from the `check_server_key` handler
/// (russh's connect consumes the handler, so this side-channel is the only way
/// to get the fingerprint and structured error out).
#[derive(Default, Debug)]
struct CapturedKey {
    fingerprint: Mutex<Option<String>>,
    error: Mutex<Option<ServiceError>>,
}

impl CapturedKey {
    fn take_fingerprint(&self) -> Option<String> {
        self.fingerprint.lock().unwrap().take()
    }

    fn take_error(&self) -> Option<ServiceError> {
        self.error.lock().unwrap().take()
    }
}

/// russh's client `Handler::Error` must be `From<russh::Error>`; the local
/// wrapper carries the structured host-key error the handler stored.
#[derive(Debug)]
enum ConnectError {
    HostKey(ServiceError),
    Russh(russh::Error),
}

impl From<russh::Error> for ConnectError {
    fn from(e: russh::Error) -> Self {
        match e {
            russh::Error::UnknownKey => ConnectError::HostKey(ServiceError::HostKeyUntrusted),
            e => ConnectError::Russh(e),
        }
    }
}

fn map_russh_error(e: &russh::Error) -> ServiceError {
    match e {
        russh::Error::UnknownKey => ServiceError::HostKeyUntrusted,
        russh::Error::ConnectionTimeout
        | russh::Error::KeepaliveTimeout
        | russh::Error::InactivityTimeout => ServiceError::Timeout,
        e => ServiceError::Network(e.to_string()),
    }
}

/// Minimal client handler: computes the `SHA256:` fingerprint of the server
/// key, records it, and either verifies TOFU (`host_keys = Some`) or accepts
/// any key (`trust_pending`).
struct HostKeyHandler<H: HostKeyStore> {
    host: String,
    port: u16,
    host_keys: Option<Arc<H>>,
    captured: Arc<CapturedKey>,
}

fn fingerprint_of(server_public_key: &russh::keys::PublicKeyOrCertificate) -> String {
    match server_public_key {
        russh::keys::PublicKeyOrCertificate::PublicKey { key, .. } => {
            key.fingerprint(HashAlg::Sha256).to_string()
        }
        russh::keys::PublicKeyOrCertificate::Certificate(cert) => {
            cert.public_key().fingerprint(HashAlg::Sha256).to_string()
        }
    }
}

impl<H: HostKeyStore> client::Handler for HostKeyHandler<H> {
    type Error = ConnectError;

    async fn check_server_key(
        &mut self,
        server_public_key: &russh::keys::PublicKeyOrCertificate,
    ) -> Result<bool, Self::Error> {
        let fp = fingerprint_of(server_public_key);
        *self.captured.fingerprint.lock().unwrap() = Some(fp.clone());

        let Some(store) = &self.host_keys else {
            return Ok(true); // trust_pending: accept, capture only
        };

        match store.verify(&self.host, self.port, &fp) {
            HostKeyVerdict::TrustedMatch => Ok(true),
            HostKeyVerdict::Unknown => {
                *self.captured.error.lock().unwrap() = Some(ServiceError::HostKeyUntrusted);
                Ok(false)
            }
            HostKeyVerdict::Changed { stored_fp } => {
                *self.captured.error.lock().unwrap() =
                    Some(ServiceError::HostKeyChanged(self.host.clone()));
                tracing::warn!(host = %self.host, stored_fp, "host key changed");
                Ok(false)
            }
        }
    }
}
/// TCP + SSH handshake to `host:port`, bounded by `connect_timeout`.
///
/// On failure the `CapturedKey` side-channel is folded into the error, but
/// the fingerprint the server presented (if any) is returned alongside so
/// callers can surface it in a trust prompt.
async fn connect_session<H: HostKeyStore + 'static>(
    host: &str,
    port: u16,
    host_keys: Option<Arc<H>>,
    connect_timeout: Duration,
) -> Result<(client::Handle<HostKeyHandler<H>>, Arc<CapturedKey>), (ServiceError, Option<String>)> {
    let captured = Arc::new(CapturedKey::default());
    let handler = HostKeyHandler {
        host: host.to_owned(),
        port,
        host_keys,
        captured: captured.clone(),
    };
    let config = Arc::new(client::Config::default());
    let handshake = client::connect(config, (host, port), handler);
    match tokio::time::timeout(connect_timeout, handshake).await {
        Ok(Ok(handle)) => Ok((handle, captured)),
        Ok(Err(e)) => {
            // The handler's structured verdict (e.g. HostKeyChanged) is more
            // specific than the generic UnknownKey russh surfaces; prefer it.
            let mapped = match e {
                ConnectError::HostKey(e) => e,
                ConnectError::Russh(e) => map_russh_error(&e),
            };
            let fp = captured.take_fingerprint();
            Err((captured.take_error().unwrap_or(mapped), fp))
        }
        Err(_) => Err((ServiceError::Timeout, None)),
    }
}

/// Try each planned method in order; all failing is `Auth`.
async fn authenticate<S: SecretStore>(
    session: &mut client::Handle<HostKeyHandler<impl HostKeyStore + 'static>>,
    machine: &Machine,
    plan: &[AuthStep],
    secrets: &S,
) -> Result<(), ServiceError> {
    for step in plan {
        match step {
            AuthStep::Agent => {
                if std::env::var_os("SSH_AUTH_SOCK").is_none() {
                    continue;
                }
                let Ok(mut agent) = russh::keys::agent::client::AgentClient::connect_env().await
                else {
                    continue;
                };
                let Ok(identities) = agent.request_identities().await else {
                    continue;
                };
                for identity in &identities {
                    let key = identity.public_key().into_owned();
                    match session
                        .authenticate_publickey_with(&machine.ssh_user, key, None, &mut agent)
                        .await
                    {
                        Ok(res) if res.success() => return Ok(()),
                        _ => continue,
                    }
                }
            }
            AuthStep::Key {
                path,
                has_passphrase,
            } => {
                let passphrase = if *has_passphrase {
                    machine
                        .key_passphrase
                        .clone()
                        .or_else(|| secrets.get(&machine.id, SecretKind::KeyPassphrase))
                } else {
                    None
                };
                match russh::keys::load_secret_key(path, passphrase.as_deref()) {
                    Ok(key) => {
                        let key = PrivateKeyWithHashAlg::new(Arc::new(key), None);
                        match session.authenticate_publickey(&machine.ssh_user, key).await {
                            Ok(res) if res.success() => return Ok(()),
                            _ => continue,
                        }
                    }
                    Err(e) => {
                        tracing::debug!(path, "cannot load key: {e}");
                        continue;
                    }
                }
            }
            AuthStep::Password => {
                let password = machine
                    .ssh_password
                    .clone()
                    .or_else(|| secrets.get(&machine.id, SecretKind::SshPassword));
                let Some(password) = password else {
                    continue;
                };
                match session
                    .authenticate_password(&machine.ssh_user, password)
                    .await
                {
                    Ok(res) if res.success() => return Ok(()),
                    _ => continue,
                }
            }
        }
    }
    Err(ServiceError::Auth)
}

/// Run `command` on the existing session, collecting stdout/stderr/exit code.
async fn exec(
    session: &mut client::Handle<HostKeyHandler<impl HostKeyStore + 'static>>,
    command: &str,
) -> Result<CommandOutput, ServiceError> {
    let mut channel = session
        .channel_open_session()
        .await
        .map_err(|e| ServiceError::Other(format!("open channel: {e}")))?;
    channel
        .exec(true, command)
        .await
        .map_err(|e| ServiceError::Other(format!("exec: {e}")))?;

    let mut stdout = String::new();
    let mut stderr = String::new();
    let mut code = 0;
    while let Some(msg) = channel.wait().await {
        match msg {
            ChannelMsg::Data { data } => stdout.push_str(&String::from_utf8_lossy(&data)),
            ChannelMsg::ExtendedData { data, ext: 1 } => {
                stderr.push_str(&String::from_utf8_lossy(&data))
            }
            ChannelMsg::ExitStatus { exit_status } => code = exit_status as i32,
            ChannelMsg::Close => break,
            _ => {}
        }
    }
    Ok(CommandOutput {
        code,
        stdout,
        stderr,
    })
}

/// SSH runner over russh: TOFU host keys, agent/key/password auth, exec.
pub struct RusshRunner<S: SecretStore, H: HostKeyStore> {
    secrets: Arc<S>,
    host_keys: Arc<H>,
    /// Default mode for machines with no per-machine `secret_mode`; shared
    /// with the tauri shell so `save_settings` takes effect immediately
    /// (a captured value would need a restart).
    default_secret_mode: Arc<RwLock<SecretMode>>,
    connect_timeout: Duration,
    /// Fingerprint from the most recent failed handshake against a host,
    /// keyed by `host:port`. `run` records it when the host-key check
    /// rejects; `pending_host_key` exposes it to the UI so the trust
    /// modal can show what the user is agreeing to.
    pending_fp: Mutex<HashMap<String, String>>,
}

impl<S: SecretStore, H: HostKeyStore + 'static> RusshRunner<S, H> {
    pub fn new(
        secrets: Arc<S>,
        host_keys: Arc<H>,
        default_secret_mode: Arc<RwLock<SecretMode>>,
    ) -> Self {
        Self {
            secrets,
            host_keys,
            default_secret_mode,
            connect_timeout: DEFAULT_CONNECT_TIMEOUT,
            pending_fp: Mutex::new(HashMap::new()),
        }
    }

    /// The fingerprint the server presented on the most recent rejected
    /// handshake to `host:port`, or `None` when nothing was captured.
    pub fn pending_host_key(&self, host: &str, port: u16) -> Option<String> {
        self.pending_fp
            .lock()
            .unwrap()
            .get(&format!("{host}:{port}"))
            .cloned()
    }

    /// Handshake while accepting whatever key the server presents, then
    /// record its fingerprint in the trust store (the "trust this host"
    /// action after a `HostKeyUntrusted` result).
    pub async fn trust_pending(&self, host: &str, port: u16) -> Result<(), ServiceError> {
        let (session, captured) = connect_session::<H>(host, port, None, self.connect_timeout)
            .await
            .map_err(|(e, _)| e)?;
        drop(session);
        let fp = captured
            .take_fingerprint()
            .ok_or_else(|| ServiceError::Other("server presented no host key".into()))?;
        self.host_keys.trust(host, port, &fp)
    }
}

impl<S: SecretStore, H: HostKeyStore + 'static> SshRunner for RusshRunner<S, H> {
    async fn run(&self, machine: &Machine, command: &str) -> Result<CommandOutput, ServiceError> {
        let mode = effective_mode(machine, &self.default_secret_mode);
        let plan = auth_plan(machine, mode, true);
        let key = format!("{}:{}", machine.os_host, machine.ssh_port);
        match connect_session(
            &machine.os_host,
            machine.ssh_port,
            Some(self.host_keys.clone()),
            self.connect_timeout,
        )
        .await
        {
            Ok((mut session, _captured)) => {
                self.pending_fp.lock().unwrap().remove(&key);
                authenticate(&mut session, machine, &plan, &*self.secrets).await?;
                exec(&mut session, command).await
            }
            Err((e, fp)) => {
                // Host-key rejection is the trust-prompt path: cache the
                // presented fingerprint (if any) so `pending_host_key` can
                // show it. Any other failure clears a stale entry.
                let mut pending = self.pending_fp.lock().unwrap();
                match fp {
                    Some(fp)
                        if matches!(
                            e,
                            ServiceError::HostKeyUntrusted | ServiceError::HostKeyChanged(_)
                        ) =>
                    {
                        pending.insert(key, fp);
                    }
                    _ => {
                        pending.remove(&key);
                    }
                }
                Err(e)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FileHostKeyStore, InMemorySecretStore};
    use sm_core::MachineId;

    fn test_machine() -> Machine {
        Machine {
            id: MachineId::from("nas"),
            name: "NAS".into(),
            mac: "00:11:22:33:44:55".into(),
            broadcast_addr: None,
            os_host: "192.168.1.10".into(),
            ssh_port: 22,
            ssh_user: "admin".into(),
            key_path: None,
            secret_mode: None,
            shutdown_cmd: None,
            reboot_cmd: None,
            stats_cmd: None,
            share_path: None,
            ssh_password: None,
            key_passphrase: None,
            sudo_password: None,
        }
    }

    // Brief test 1, verbatim (Machine already has the fields it names).
    #[test]
    fn password_only_when_no_key_configured() {
        let mut m = test_machine();
        m.ssh_password = Some("p".into());
        let plan = auth_plan(&m, SecretMode::Plaintext, false);
        assert_eq!(plan, vec![AuthStep::Password]);
    }

    // Brief test 2, verbatim. `Keyring` resolves as `Prompt`, so the plan is
    // [Agent, Password] with the agent first.
    #[test]
    fn agent_first_when_available_and_no_explicit_key() {
        let plan = auth_plan(&test_machine(), SecretMode::Keyring, true);
        assert_eq!(plan.first(), Some(&AuthStep::Agent));
    }

    // Brief test 3 (added): explicit key beats the agent.
    #[test]
    fn explicit_key_skips_agent() {
        let mut m = test_machine();
        m.key_path = Some("/home/admin/.ssh/id_ed25519".into());
        let plan = auth_plan(&m, SecretMode::Plaintext, true);
        assert_eq!(
            plan.first(),
            Some(&AuthStep::Key {
                path: "/home/admin/.ssh/id_ed25519".into(),
                has_passphrase: true,
            })
        );
        assert!(!plan.contains(&AuthStep::Agent));
    }

    // `Keyring` resolves as `Prompt` at resolution time, so a store-held
    // password is a possible source and the plan ends with `Password`.
    #[test]
    fn keyring_resolves_as_prompt_for_password_fallback() {
        let plan = auth_plan(&test_machine(), SecretMode::Keyring, false);
        assert_eq!(plan, vec![AuthStep::Password]);
    }
    // A TCP server that closes immediately is not an SSH server: the
    // handshake fails with a network error and no host key is presented,
    // so no pending fingerprint is recorded (the stale-entry clear path).
    #[tokio::test]
    async fn non_ssh_host_leaves_no_pending_fingerprint() {
        let l = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = l.local_addr().unwrap().port();
        tokio::spawn(async move {
            while let Ok((s, _)) = l.accept().await {
                drop(s);
            }
        });
        let secrets = Arc::new(InMemorySecretStore::default());
        let dir = tempfile::tempdir().unwrap();
        let hk = Arc::new(FileHostKeyStore::new(dir.path().join("known_hosts")));
        let runner = RusshRunner::new(secrets, hk, default_mode(SecretMode::Prompt));
        let mut m = test_machine();
        m.os_host = "127.0.0.1".into();
        m.ssh_port = port;
        assert!(runner.run(&m, "echo hi").await.is_err());
        assert_eq!(runner.pending_host_key("127.0.0.1", port), None);
    }

    #[test]
    fn pending_host_key_starts_empty_and_is_scoped_by_host_port() {
        let secrets = Arc::new(InMemorySecretStore::default());
        let dir = tempfile::tempdir().unwrap();
        let hk = Arc::new(FileHostKeyStore::new(dir.path().join("known_hosts")));
        let runner = RusshRunner::new(secrets, hk, default_mode(SecretMode::Plaintext));
        assert_eq!(runner.pending_host_key("h", 1), None);
    }

    /// Test helper: wraps a mode in the shared-lock shape `RusshRunner::new`
    /// expects.
    fn default_mode(mode: SecretMode) -> Arc<RwLock<SecretMode>> {
        Arc::new(RwLock::new(mode))
    }
}
