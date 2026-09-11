use std::collections::HashMap;
use std::sync::Mutex;

use sm_core::MachineId;
use sm_services::{SecretKind, SecretStore, ServiceError};

/// Stable storage name for a secret kind (plaintext config files, keyrings).
pub fn secret_key(kind: SecretKind) -> &'static str {
    match kind {
        SecretKind::SshPassword => "ssh_password",
        SecretKind::KeyPassphrase => "key_passphrase",
        SecretKind::SudoPassword => "sudo_password",
    }
}

/// Process-lifetime in-memory secret store (`prompt` secret mode).
pub struct InMemorySecretStore {
    map: Mutex<HashMap<(String, SecretKind), String>>,
}

impl Default for InMemorySecretStore {
    fn default() -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
        }
    }
}

impl SecretStore for InMemorySecretStore {
    fn get(&self, machine_id: &MachineId, kind: SecretKind) -> Option<String> {
        self.map
            .lock()
            .unwrap()
            .get(&(machine_id.to_string(), kind))
            .cloned()
    }

    fn set(&self, machine_id: &MachineId, kind: SecretKind, value: &str) -> Result<(), ServiceError> {
        self.map
            .lock()
            .unwrap()
            .insert((machine_id.to_string(), kind), value.to_owned());
        Ok(())
    }

    fn clear(&self, machine_id: &MachineId, kind: SecretKind) -> Result<(), ServiceError> {
        self.map.lock().unwrap().remove(&(machine_id.to_string(), kind));
        Ok(())
    }
}

/// Secret store backed by the OS keyring.
///
/// Entries are keyed as `{service}` / `{machine_id}:{kind:?}`. On Linux this
/// uses the kernel keyutils backend; on macOS and Windows the native
/// keychain/credential manager.
pub struct KeyringSecretStore {
    service: String,
}

impl Default for KeyringSecretStore {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyringSecretStore {
    pub fn new() -> Self {
        Self {
            service: "server-manager".to_owned(),
        }
    }

    fn entry(
        &self,
        machine_id: &MachineId,
        kind: SecretKind,
    ) -> Result<keyring::Entry, ServiceError> {
        keyring::Entry::new(&self.service, &format!("{machine_id}:{kind:?}"))
            .map_err(|e| ServiceError::Other(format!("keyring entry: {e}")))
    }
}

impl SecretStore for KeyringSecretStore {
    fn get(&self, machine_id: &MachineId, kind: SecretKind) -> Option<String> {
        let entry = match self.entry(machine_id, kind) {
            Ok(entry) => entry,
            Err(e) => {
                tracing::warn!(error = %e, "keyring: no entry handle");
                return None;
            }
        };
        match entry.get_password() {
            Ok(password) => Some(password),
            Err(keyring::Error::NoEntry) => None,
            Err(e) => {
                tracing::warn!(error = %e, "keyring get failed");
                None
            }
        }
    }

    fn set(&self, machine_id: &MachineId, kind: SecretKind, value: &str) -> Result<(), ServiceError> {
        self.entry(machine_id, kind)?
            .set_password(value)
            .map_err(|e| ServiceError::Other(format!("keyring set: {e}")))
    }

    fn clear(&self, machine_id: &MachineId, kind: SecretKind) -> Result<(), ServiceError> {
        match self.entry(machine_id, kind)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(ServiceError::Other(format!("keyring clear: {e}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sm_services::{SecretStore, SecretKind};

    #[test]
    fn in_memory_roundtrip() {
        let s = InMemorySecretStore::default();
        let id = sm_core::MachineId::from("nas");
        assert!(s.get(&id, SecretKind::SshPassword).is_none());
        s.set(&id, SecretKind::SshPassword, "hunter2").unwrap();
        assert_eq!(s.get(&id, SecretKind::SshPassword).as_deref(), Some("hunter2"));
        s.clear(&id, SecretKind::SshPassword).unwrap();
        assert!(s.get(&id, SecretKind::SshPassword).is_none());
    }

    /// Clearing an absent key must be Ok, not an error.
    #[test]
    fn in_memory_clear_missing_is_ok() {
        let s = InMemorySecretStore::default();
        s.clear(&sm_core::MachineId::from("ghost"), SecretKind::SudoPassword)
            .unwrap();
    }

    /// Stable names for plaintext config files.
    #[test]
    fn secret_key_names() {
        assert_eq!(secret_key(SecretKind::SshPassword), "ssh_password");
        assert_eq!(secret_key(SecretKind::KeyPassphrase), "key_passphrase");
        assert_eq!(secret_key(SecretKind::SudoPassword), "sudo_password");
    }

    /// Roundtrip against the real OS keyring. Ignored because it depends on
    /// a user session (keyutils session keyring on Linux, keychain on macOS)
    /// and persists state outside the workspace.
    #[test]
    #[ignore = "needs a user session and touches the real OS keyring"]
    fn keyring_roundtrip() {
        let s = KeyringSecretStore::new();
        let id = sm_core::MachineId::from("sm-infra-selftest");
        assert!(s.get(&id, SecretKind::SudoPassword).is_none());
        s.set(&id, SecretKind::SudoPassword, "pw").unwrap();
        assert_eq!(s.get(&id, SecretKind::SudoPassword).as_deref(), Some("pw"));
        s.clear(&id, SecretKind::SudoPassword).unwrap();
        assert!(s.get(&id, SecretKind::SudoPassword).is_none());
    }
}