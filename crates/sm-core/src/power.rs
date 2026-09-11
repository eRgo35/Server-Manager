use crate::model::{Machine, PowerAction};

pub const DEFAULT_SHUTDOWN: &str = "shutdown -h now";
pub const DEFAULT_REBOOT: &str = "shutdown -r now";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerPlan {
    pub primary: String,
    pub sudo_fallback: Option<String>,
}

pub fn resolve_power(machine: &Machine, action: PowerAction) -> PowerPlan {
    let primary = match action {
        PowerAction::Shutdown => machine.shutdown_cmd.as_deref().unwrap_or(DEFAULT_SHUTDOWN),
        PowerAction::Reboot => machine.reboot_cmd.as_deref().unwrap_or(DEFAULT_REBOOT),
    };

    let sudo_fallback = if machine.sudo_password.is_some() {
        Some(format!("sudo -S -p '' {}", primary))
    } else {
        None
    };

    PowerPlan {
        primary: primary.to_string(),
        sudo_fallback,
    }
}

pub fn needs_sudo(exit_code: i32, stderr: &str) -> bool {
    if exit_code == 0 {
        return false;
    }

    let stderr_lower = stderr.to_lowercase();
    [
        "permission denied",
        "must be root",
        "operation not permitted",
        "not authorized",
    ]
    .iter()
    .any(|msg| stderr_lower.contains(msg))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    fn m() -> Machine {
        Machine {
            id: "x".into(),
            name: "x".into(),
            mac: "AA:BB:CC:DD:EE:FF".into(),
            broadcast_addr: None,
            os_host: "h".into(),
            ssh_port: 22,
            ssh_user: "u".into(),
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

    #[test]
    fn default_shutdown_no_fallback_without_password() {
        let p = resolve_power(&m(), PowerAction::Shutdown);
        assert_eq!(p.primary, "shutdown -h now");
        assert!(p.sudo_fallback.is_none());
    }

    #[test]
    fn override_and_sudo_fallback() {
        let mut mm = m();
        mm.reboot_cmd = Some("systemctl reboot".into());
        mm.sudo_password = Some("pw".into());
        let p = resolve_power(&mm, PowerAction::Reboot);
        assert_eq!(p.primary, "systemctl reboot");
        assert_eq!(
            p.sudo_fallback.as_deref(),
            Some("sudo -S -p '' systemctl reboot")
        );
    }

    #[test]
    fn detects_permission_failure() {
        assert!(needs_sudo(1, "shutdown: Permission denied"));
        assert!(!needs_sudo(0, ""));
        assert!(!needs_sudo(1, "command not found"));
    }
}
