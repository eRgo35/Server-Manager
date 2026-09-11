//! Trait contracts for the `infra` layer and the Tauri app to implement.
//!
//! Async traits use return-position `impl Future` in trait (RPITIT), stable
//! since Rust 1.98. No `async fn` in traits, no `async-trait` crate.

use sm_core::{Machine, MachineId, Stats};

/// Errors surfaced by service implementations.
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("network: {0}")]
    Network(String),
    #[error("auth failed")]
    Auth,
    #[error("host key changed for {0}")]
    HostKeyChanged(String),
    #[error("host key not trusted")]
    HostKeyUntrusted,
    #[error("timeout")]
    Timeout,
    #[error("remote command failed ({code}): {stderr}")]
    Remote { code: i32, stderr: String },
    #[error("{0}")]
    Other(String),
}

/// Wake-on-LAN sender.
pub trait Waker: Send + Sync {
    fn wake(
        &self,
        mac: [u8; 6],
        broadcast_addr: &str,
    ) -> impl std::future::Future<Output = Result<(), ServiceError>> + Send;
}

/// Reachability probe.
pub trait StatusProbe: Send + Sync {
    fn is_up(
        &self,
        host: &str,
        port: u16,
        timeout: std::time::Duration,
    ) -> impl std::future::Future<Output = bool> + Send;
}

/// Result of running a remote command.
pub struct CommandOutput {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Runs commands on a machine over SSH.
pub trait SshRunner: Send + Sync {
    fn run(
        &self,
        machine: &Machine,
        command: &str,
    ) -> impl std::future::Future<Output = Result<CommandOutput, ServiceError>> + Send;
}

/// Samples runtime stats from a machine.
pub trait StatsProbe: Send + Sync {
    fn sample(
        &self,
        machine: &Machine,
    ) -> impl std::future::Future<Output = Result<Stats, ServiceError>> + Send;
}

/// Kinds of secret held per machine.
///
/// `Hash` is required because infra uses this as a map key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SecretKind {
    SshPassword,
    KeyPassphrase,
    SudoPassword,
}

/// Persistent secret storage (sync).
pub trait SecretStore: Send + Sync {
    fn get(&self, machine_id: &MachineId, kind: SecretKind) -> Option<String>;
    fn set(
        &self,
        machine_id: &MachineId,
        kind: SecretKind,
        value: &str,
    ) -> Result<(), ServiceError>;
    fn clear(&self, machine_id: &MachineId, kind: SecretKind) -> Result<(), ServiceError>;
}

/// Outcome of verifying a host key against stored trust.
#[derive(Debug, PartialEq)]
pub enum HostKeyVerdict {
    TrustedMatch,
    Unknown,
    Changed { stored_fp: String },
}

/// Known-hosts style trust store (sync).
pub trait HostKeyStore: Send + Sync {
    fn verify(&self, host: &str, port: u16, fingerprint: &str) -> HostKeyVerdict;
    fn trust(&self, host: &str, port: u16, fingerprint: &str) -> Result<(), ServiceError>;
}

/// Opens remote resources in the host OS (sync).
pub trait FileOpener: Send + Sync {
    fn open_files(&self, machine: &Machine) -> Result<(), ServiceError>;
    fn map_drive(&self, machine: &Machine) -> Result<(), ServiceError>;
}
