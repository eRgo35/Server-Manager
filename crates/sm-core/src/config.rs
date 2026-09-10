use serde::{Deserialize, Serialize};
use crate::model::{Machine, MachineId, Settings};

pub const CURRENT_SCHEMA: u32 = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "def_schema")]
    pub schema_version: u32,
    #[serde(default)]
    pub settings: Settings,
    #[serde(default, rename = "machine")]
    pub machines: Vec<Machine>,
}
fn def_schema() -> u32 { CURRENT_SCHEMA }

impl Default for Config {
    fn default() -> Self {
        Config { schema_version: CURRENT_SCHEMA, settings: Settings::default(), machines: vec![] }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("config parse error: {0}")]
    Parse(String),
    #[error("config validation error: {0}")]
    Validation(String),
}

pub fn parse_config(text: &str) -> Result<Config, ConfigError> {
    let cfg: Config = toml::from_str(text).map_err(|e| ConfigError::Parse(e.to_string()))?;
    validate(&cfg)?;
    Ok(cfg)
}

pub fn serialize_config(cfg: &Config) -> String {
    toml::to_string_pretty(cfg).expect("Config is always serializable")
}

fn valid_mac(s: &str) -> bool {
    let parts: Vec<&str> = s.split([':', '-']).collect();
    parts.len() == 6 && parts.iter().all(|p| p.len() == 2 && p.chars().all(|c| c.is_ascii_hexdigit()))
}

pub fn validate(cfg: &Config) -> Result<(), ConfigError> {
    use std::collections::HashSet;
    let mut seen = HashSet::new();
    if cfg.settings.poll_base_secs < 1 {
        return Err(ConfigError::Validation("poll_base_secs must be >= 1".into()));
    }
    for m in &cfg.machines {
        if !seen.insert(&m.id.0) {
            return Err(ConfigError::Validation(format!("duplicate machine id: {}", m.id)));
        }
        if !valid_mac(&m.mac) {
            return Err(ConfigError::Validation(format!("invalid mac for {}", m.id)));
        }
        if m.ssh_port == 0 {
            return Err(ConfigError::Validation(format!("ssh_port is 0 for {}", m.id)));
        }
        if m.os_host.trim().is_empty() || m.ssh_user.trim().is_empty() {
            return Err(ConfigError::Validation(format!("os_host/ssh_user empty for {}", m.id)));
        }
    }
    Ok(())
}

impl Config {
    pub fn machine(&self, id: &MachineId) -> Option<&Machine> {
        self.machines.iter().find(|m| &m.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
schema_version = 2
[settings]
language = "pl"
[[machine]]
id = "nas"
name = "Home NAS"
mac = "AA:BB:CC:DD:EE:FF"
os_host = "192.168.1.10"
ssh_user = "mike"
"#;

    #[test]
    fn parses_sample_and_applies_defaults() {
        let cfg = parse_config(SAMPLE).unwrap();
        assert_eq!(cfg.schema_version, 2);
        assert_eq!(cfg.settings.language, "pl");
        assert_eq!(cfg.settings.poll_base_secs, 5); // default
        assert_eq!(cfg.machines.len(), 1);
        assert_eq!(cfg.machines[0].ssh_port, 22); // default
    }

    #[test]
    fn round_trips() {
        let cfg = parse_config(SAMPLE).unwrap();
        let text = serialize_config(&cfg);
        let again = parse_config(&text).unwrap();
        assert_eq!(again.machines[0].name, "Home NAS");
    }

    #[test]
    fn rejects_bad_mac() {
        let bad = SAMPLE.replace("AA:BB:CC:DD:EE:FF", "nope");
        let err = parse_config(&bad).unwrap_err();
        assert!(matches!(err, ConfigError::Validation(_)));
    }

    #[test]
    fn rejects_duplicate_ids() {
        let dup = format!("{SAMPLE}\n[[machine]]\nid=\"nas\"\nname=\"x\"\nmac=\"AA:BB:CC:DD:EE:00\"\nos_host=\"h\"\nssh_user=\"u\"\n");
        assert!(matches!(parse_config(&dup).unwrap_err(), ConfigError::Validation(_)));
    }

    #[test]
    fn parse_error_on_garbage() {
        assert!(matches!(parse_config("this is not toml =").unwrap_err(), ConfigError::Parse(_)));
    }
}
