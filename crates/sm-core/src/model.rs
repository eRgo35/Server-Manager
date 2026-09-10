use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MachineId(pub String);

impl std::fmt::Display for MachineId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl From<&str> for MachineId {
    fn from(s: &str) -> Self {
        MachineId(s.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SecretMode {
    Plaintext,
    Keyring,
    Prompt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    System,
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StatsDisplay {
    Graph,
    Numbers,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MountProtocol {
    Smb,
    Sshfs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerAction {
    Shutdown,
    Reboot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Online,
    Offline,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Stats {
    pub cpu_pct: f32,
    pub mem_used: u64,
    pub mem_total: u64,
    pub disk_used: u64,
    pub disk_total: u64,
    pub uptime_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Machine {
    pub id: MachineId,
    pub name: String,
    pub mac: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub broadcast_addr: Option<String>,
    pub os_host: String,
    #[serde(default = "default_ssh_port")]
    pub ssh_port: u16,
    pub ssh_user: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret_mode: Option<SecretMode>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shutdown_cmd: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reboot_cmd: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stats_cmd: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub share_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssh_password: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_passphrase: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sudo_password: Option<String>,
}

fn default_ssh_port() -> u16 {
    22
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "def_lang")]
    pub language: String,
    #[serde(default)]
    pub theme: Theme,
    #[serde(default)]
    pub stats_display: StatsDisplay,
    #[serde(default = "def_poll")]
    pub poll_base_secs: u64,
    #[serde(default = "def_secret")]
    pub default_secret_mode: SecretMode,
    #[serde(default)]
    pub mount_protocol: MountProtocol,
}

fn def_lang() -> String {
    "en".into()
}
fn def_poll() -> u64 {
    5
}
fn def_secret() -> SecretMode {
    SecretMode::Keyring
}

impl Default for Theme {
    fn default() -> Self {
        Theme::System
    }
}
impl Default for StatsDisplay {
    fn default() -> Self {
        StatsDisplay::Graph
    }
}
impl Default for MountProtocol {
    fn default() -> Self {
        MountProtocol::Smb
    }
}
impl Default for Settings {
    fn default() -> Self {
        Settings {
            language: def_lang(),
            theme: Theme::System,
            stats_display: StatsDisplay::Graph,
            poll_base_secs: def_poll(),
            default_secret_mode: def_secret(),
            mount_protocol: MountProtocol::Smb,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_default_matches_spec() {
        let s = Settings::default();
        assert_eq!(s.language, "en");
        assert_eq!(s.theme, Theme::System);
        assert_eq!(s.stats_display, StatsDisplay::Graph);
        assert_eq!(s.poll_base_secs, 5);
        assert_eq!(s.default_secret_mode, SecretMode::Keyring);
        assert_eq!(s.mount_protocol, MountProtocol::Smb);
    }

    #[derive(serde::Serialize, serde::Deserialize)]
    struct Wrap {
        v: SecretMode,
    }

    #[test]
    fn secret_mode_serde_is_lowercase() {
        let s = toml::to_string(&Wrap { v: SecretMode::Keyring }).unwrap();
        assert!(s.contains("\"keyring\""));
        let back: Wrap = toml::from_str("v = \"prompt\"").unwrap();
        assert_eq!(back.v, SecretMode::Prompt);
    }
}
