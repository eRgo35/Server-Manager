//! SSH stats probe: custom `stats_cmd` or the built-in proc-stats command.

use std::sync::Arc;

use sm_core::{parse_proc_stats, stats::PROC_STATS_CMD, Machine, Stats};
use sm_services::{ServiceError, SshRunner, StatsProbe};

/// Samples stats over SSH. When the machine defines a `stats_cmd`, its
/// `key=value` output is parsed; otherwise the built-in [`PROC_STATS_CMD`]
/// is run and parsed by `sm_core::parse_proc_stats`.
pub struct SshStatsProbe<R: SshRunner> {
    runner: Arc<R>,
}

impl<R: SshRunner> SshStatsProbe<R> {
    pub fn new(runner: Arc<R>) -> Self {
        SshStatsProbe { runner }
    }
}

/// Parse the documented `key=value` keys into `Stats`; any missing key is an
/// error.
fn parse_stats_cmd(stdout: &str) -> Result<Stats, ServiceError> {
    let mut values = std::collections::HashMap::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            values.insert(key.trim(), value.trim());
        }
    }
    let get = |key: &str| {
        values
            .get(key)
            .copied()
            .ok_or_else(|| ServiceError::Other(format!("stats_cmd missing key: {key}")))
    };
    let f32_of = |key: &str| -> Result<f32, ServiceError> {
        let raw = get(key)?;
        raw.parse()
            .map_err(|_| ServiceError::Other(format!("stats_cmd bad value for {key}: {raw}")))
    };
    let u64_of = |key: &str| -> Result<u64, ServiceError> {
        let raw = get(key)?;
        raw.parse()
            .map_err(|_| ServiceError::Other(format!("stats_cmd bad value for {key}: {raw}")))
    };
    Ok(Stats {
        cpu_pct: f32_of("cpu_pct")?,
        mem_used: u64_of("mem_used")?,
        mem_total: u64_of("mem_total")?,
        disk_used: u64_of("disk_used")?,
        disk_total: u64_of("disk_total")?,
        uptime_secs: u64_of("uptime_secs")?,
    })
}

impl<R: SshRunner> StatsProbe for SshStatsProbe<R> {
    async fn sample(&self, machine: &Machine) -> Result<Stats, ServiceError> {
        {
            let command = machine
                .stats_cmd
                .as_deref()
                .filter(|c| !c.trim().is_empty())
                .unwrap_or(PROC_STATS_CMD);
            let out = self.runner.run(machine, command).await?;
            if out.code != 0 {
                return Err(ServiceError::Remote {
                    code: out.code,
                    stderr: out.stderr,
                });
            }
            match &machine.stats_cmd {
                Some(c) if !c.trim().is_empty() => parse_stats_cmd(&out.stdout),
                _ => parse_proc_stats(&out.stdout).map_err(|e| ServiceError::Other(e.to_string())),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sm_core::{Machine, MachineId};
    use sm_services::{CommandOutput, ServiceError, SshRunner};
    use std::sync::Mutex;

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

    /// Fake SshRunner: returns one canned result (taken, so no Clone needed).
    struct FakeRunner {
        out: Mutex<Option<Result<CommandOutput, ServiceError>>>,
    }

    impl FakeRunner {
        fn new(out: Result<CommandOutput, ServiceError>) -> Self {
            FakeRunner {
                out: Mutex::new(Some(out)),
            }
        }
    }

    impl SshRunner for FakeRunner {
        fn run(
            &self,
            _machine: &Machine,
            _command: &str,
        ) -> impl std::future::Future<Output = Result<CommandOutput, ServiceError>> + Send {
            let out = self.out.lock().unwrap().take();
            async move { out.expect("runner invoked twice") }
        }
    }

    #[tokio::test]
    async fn custom_cmd_parses_key_value() {
        let mut m = test_machine();
        m.stats_cmd = Some("my-stats".into());
        let runner = Arc::new(FakeRunner::new(Ok(CommandOutput {
            code: 0,
            stdout: "\n# comment\ncpu_pct=12.5\nmem_used=1000\nmem_total=2000\ndisk_used=3000\ndisk_total=4000\nuptime_secs=123456\n".into(),
            stderr: String::new(),
        })));
        let probe = SshStatsProbe::new(runner);
        let s = probe.sample(&m).await.unwrap();
        assert_eq!(s.cpu_pct, 12.5);
        assert_eq!(s.mem_used, 1000);
        assert_eq!(s.mem_total, 2000);
        assert_eq!(s.disk_used, 3000);
        assert_eq!(s.disk_total, 4000);
        assert_eq!(s.uptime_secs, 123456);
    }

    #[tokio::test]
    async fn custom_cmd_missing_key_errs() {
        let mut m = test_machine();
        m.stats_cmd = Some("my-stats".into());
        let runner = Arc::new(FakeRunner::new(Ok(CommandOutput {
            code: 0,
            stdout: "cpu_pct=1.0\nmem_used=2\nmem_total=3\n".into(),
            stderr: String::new(),
        })));
        let probe = SshStatsProbe::new(runner);
        let err = probe.sample(&m).await.unwrap_err();
        assert!(matches!(err, ServiceError::Other(msg) if msg.contains("disk_used")));
    }

    // No `stats_cmd` → the built-in proc-stats command, parsed by
    // `sm_core::parse_proc_stats`.
    #[tokio::test]
    async fn falls_back_to_proc_stats_cmd() {
        let m = test_machine();
        let runner = Arc::new(FakeRunner::new(Ok(CommandOutput {
            code: 0,
            stdout: "\
---SM-UPTIME
12345.67 98765.43
---SM-MEM
MemTotal:       16384000 kB
MemAvailable:    8192000 kB
---SM-DISK
   500107862016   123456789012
---SM-CPU1
cpu  100 0 100 800 0 0 0 0 0 0
---SM-CPU2
cpu  110 0 110 860 0 0 0 0 0 0
"
            .into(),
            stderr: String::new(),
        })));
        let probe = SshStatsProbe::new(runner);
        let s = probe.sample(&m).await.unwrap();
        assert_eq!(s.uptime_secs, 12345);
        assert_eq!(s.mem_total, 16_384_000 * 1024);
        assert_eq!(s.disk_total, 500_107_862_016);
    }
}
