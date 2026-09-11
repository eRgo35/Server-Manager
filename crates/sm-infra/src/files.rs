//! Linux file-manager opener: opens SMB shares via `xdg-open`.

use std::process::Command;

use sm_core::Machine;
use sm_services::{FileOpener, ServiceError};

/// URL for the machine's share: `share_path` as an SMB URL when set
/// (`\\host\share` → `smb://host/share`), else `smb://<os_host>`.
pub(crate) fn smb_url(machine: &Machine) -> String {
    match &machine.share_path {
        Some(path) => {
            let trimmed = path.trim_start_matches(r"\\").trim_start_matches('/');
            format!("smb://{}", trimmed.replace('\\', "/"))
        }
        None => format!("smb://{}", machine.os_host),
    }
}

/// Opens the machine's share in the desktop file manager (fire-and-forget).
pub struct LinuxFileOpener;

impl FileOpener for LinuxFileOpener {
    fn open_files(&self, machine: &Machine) -> Result<(), ServiceError> {
        Command::new("xdg-open")
            .arg(smb_url(machine))
            .spawn()
            .map(|_| ())
            .map_err(|e| ServiceError::Other(format!("xdg-open: {e}")))
    }

    fn map_drive(&self, _machine: &Machine) -> Result<(), ServiceError> {
        Err(ServiceError::Other("map drive is Windows-only".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sm_core::{Machine, MachineId};

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

    #[test]
    fn smb_url_prefers_share_path() {
        let mut m = test_machine();
        m.share_path = Some(r"\\192.168.1.10\media".into());
        assert_eq!(smb_url(&m), "smb://192.168.1.10/media");
    }

    // Brief's `smb://host.example` adapted: the real test_machine `os_host`
    // is `192.168.1.10`.
    #[test]
    fn smb_url_falls_back_to_host() {
        assert_eq!(smb_url(&test_machine()), "smb://192.168.1.10");
    }
}
