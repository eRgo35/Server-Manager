//! Compile-lock test: a zero-behavior implementation of every `sm-services`
//! trait. If any trait signature drifts, this file stops compiling.

use sm_core::{Machine, MachineId, Stats};
use sm_services::{
    CommandOutput, FileOpener, HostKeyStore, HostKeyVerdict, SecretKind, SecretStore, ServiceError,
    SshRunner, StatsProbe, StatusProbe, Waker,
};

struct Dummy;

impl Waker for Dummy {
    async fn wake(&self, _mac: [u8; 6], _broadcast_addr: &str) -> Result<(), ServiceError> {
        Ok(())
    }
}

impl StatusProbe for Dummy {
    async fn is_up(&self, _host: &str, _port: u16, _timeout: std::time::Duration) -> bool {
        false
    }
}

impl SshRunner for Dummy {
    async fn run(&self, _machine: &Machine, _command: &str) -> Result<CommandOutput, ServiceError> {
        Ok(CommandOutput {
            code: 0,
            stdout: String::new(),
            stderr: String::new(),
        })
    }
}

impl StatsProbe for Dummy {
    async fn sample(&self, _machine: &Machine) -> Result<Stats, ServiceError> {
        Ok(Stats {
            cpu_pct: 0.0,
            mem_used: 0,
            mem_total: 0,
            disk_used: 0,
            disk_total: 0,
            uptime_secs: 0,
        })
    }
}

impl SecretStore for Dummy {
    fn get(&self, _machine_id: &MachineId, _kind: SecretKind) -> Option<String> {
        None
    }
    fn set(
        &self,
        _machine_id: &MachineId,
        _kind: SecretKind,
        _value: &str,
    ) -> Result<(), ServiceError> {
        Ok(())
    }
    fn clear(&self, _machine_id: &MachineId, _kind: SecretKind) -> Result<(), ServiceError> {
        Ok(())
    }
}

impl HostKeyStore for Dummy {
    fn verify(&self, _host: &str, _port: u16, _fingerprint: &str) -> HostKeyVerdict {
        HostKeyVerdict::Unknown
    }
    fn trust(&self, _host: &str, _port: u16, _fingerprint: &str) -> Result<(), ServiceError> {
        Ok(())
    }
}

impl FileOpener for Dummy {
    fn open_files(&self, _machine: &Machine) -> Result<(), ServiceError> {
        Ok(())
    }
    fn map_drive(&self, _machine: &Machine) -> Result<(), ServiceError> {
        Ok(())
    }
}

fn machine() -> Machine {
    Machine {
        id: MachineId("m1".to_string()),
        name: "m1".to_string(),
        mac: "00:00:00:00:00:00".to_string(),
        broadcast_addr: None,
        os_host: "localhost".to_string(),
        ssh_port: 22,
        ssh_user: "root".to_string(),
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

// Assert trait-object / generic usability and Send bounds.
fn _assert_send<T: Send>(_: T) {}

#[tokio::test]
async fn async_traits_await_on_dummy() {
    let d = Dummy;
    d.wake([0u8; 6], "255.255.255.255").await.unwrap();
    assert!(
        !d.is_up("localhost", 22, std::time::Duration::from_millis(1))
            .await
    );
    let m = machine();
    let out = d.run(&m, "true").await.unwrap();
    assert_eq!(out.code, 0);
    let stats = d.sample(&m).await.unwrap();
    assert_eq!(stats.uptime_secs, 0);
    _assert_send(d.wake([0u8; 6], "255.255.255.255"));
}

#[test]
fn sync_traits_lock() {
    let d = Dummy;
    let id = MachineId("m1".to_string());
    assert!(d.get(&id, SecretKind::SshPassword).is_none());
    d.set(&id, SecretKind::KeyPassphrase, "x").unwrap();
    d.clear(&id, SecretKind::SudoPassword).unwrap();

    assert_eq!(d.verify("h", 22, "fp"), HostKeyVerdict::Unknown);
    d.trust("h", 22, "fp").unwrap();

    let m = machine();
    d.open_files(&m).unwrap();
    d.map_drive(&m).unwrap();

    // SecretKind is Hash + Eq (infra uses it as a map key).
    let mut map = std::collections::HashMap::new();
    map.insert(SecretKind::SshPassword, 1);
    assert_eq!(map.get(&SecretKind::SshPassword), Some(&1));

    // ServiceError Display / thiserror wiring.
    assert_eq!(
        ServiceError::Remote {
            code: 2,
            stderr: "boom".to_string()
        }
        .to_string(),
        "remote command failed (2): boom"
    );
}
