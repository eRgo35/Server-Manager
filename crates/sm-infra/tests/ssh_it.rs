//! Docker-backed SSH integration tests (`just test-integration`).
//!
//! Each test targets the sshd container from
//! `tests/integration/docker-compose.yml` (127.0.0.1:2222, user `tester`,
//! password `testpass`). They are `#[ignore]`-gated so a plain `cargo test`
//! never touches docker; the justfile recipe runs them with `-- --ignored`.

#![cfg(test)]

use std::sync::Arc;

use sm_core::{Machine, MachineId, SecretMode};
use sm_infra::{FileHostKeyStore, InMemorySecretStore, RusshRunner, SshStatsProbe};
use sm_services::{ServiceError, SshRunner, StatsProbe};

fn machine() -> Machine {
    Machine {
        id: MachineId::from("test-host"),
        name: "test-host".into(),
        mac: "00:00:00:00:00:00".into(),
        broadcast_addr: None,
        os_host: "127.0.0.1".into(),
        ssh_port: 2222,
        ssh_user: "tester".into(),
        key_path: None,
        secret_mode: None,
        shutdown_cmd: None,
        reboot_cmd: None,
        stats_cmd: None,
        share_path: None,
        ssh_password: Some("testpass".into()),
        key_passphrase: None,
        sudo_password: None,
    }
}

#[tokio::test]
#[ignore = "needs the docker sshd from `just test-integration`"]
async fn runs_a_command_after_trusting_host() {
    let dir = tempfile::tempdir().unwrap();
    let hk = Arc::new(FileHostKeyStore::new(dir.path().join("known_hosts")));
    let secrets = Arc::new(InMemorySecretStore::default());
    let runner = RusshRunner::new(secrets, hk, SecretMode::Plaintext);

    // First contact: TOFU must reject the untrusted host key.
    let err = match runner.run(&machine(), "echo hi").await {
        Err(e) => e,
        Ok(_) => panic!("first run against an untrusted host should fail"),
    };
    assert!(matches!(err, ServiceError::HostKeyUntrusted), "got {err:?}");

    runner.trust_pending("127.0.0.1", 2222).await.unwrap();

    let out = match runner.run(&machine(), "echo hi").await {
        Ok(out) => out,
        Err(e) => panic!("run after trust_pending should succeed, got {e:?}"),
    };
    assert_eq!(out.stdout.trim(), "hi");
    assert_eq!(out.code, 0);
}

#[tokio::test]
#[ignore = "needs the docker sshd from `just test-integration`"]
async fn samples_proc_stats() {
    let dir = tempfile::tempdir().unwrap();
    let hk = Arc::new(FileHostKeyStore::new(dir.path().join("known_hosts")));
    let runner = Arc::new(RusshRunner::new(
        Arc::new(InMemorySecretStore::default()),
        hk,
        SecretMode::Plaintext,
    ));

    runner.trust_pending("127.0.0.1", 2222).await.unwrap();

    let probe = SshStatsProbe::new(runner);
    let stats = probe.sample(&machine()).await.unwrap();
    assert!(stats.mem_total > 0, "mem_total was 0");
    assert!(stats.uptime_secs > 0, "uptime_secs was 0");
}
