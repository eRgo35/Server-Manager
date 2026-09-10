use crate::config::{Config, ConfigError, CURRENT_SCHEMA, validate};

#[derive(Debug)]
pub enum LoadOutcome {
    Loaded(Config),
    MigratedFrom { from: u32, config: Config },
    BackedUpAndReset { reason: String, config: Config },
    TooNew { found: u32 },
}

pub fn backup_filename(now_unix: i64) -> String {
    format!("config.toml.bak-{now_unix}")
}

fn read_schema(text: &str) -> Option<u32> {
    let v: toml::Value = toml::from_str(text).ok()?;
    v.get("schema_version").and_then(|s| s.as_integer()).map(|i| i as u32)
}

pub fn load_from_text(text: &str) -> LoadOutcome {
    let parsed: Result<toml::Value, _> = toml::from_str(text);
    let Ok(_value) = parsed else {
        return LoadOutcome::BackedUpAndReset {
            reason: "config.toml is not valid TOML".into(),
            config: Config::default(),
        };
    };
    let found = read_schema(text).unwrap_or(CURRENT_SCHEMA);
    if found > CURRENT_SCHEMA {
        return LoadOutcome::TooNew { found };
    }
    if found < CURRENT_SCHEMA {
        match migrate_chain(text, found) {
            Ok(config) => return LoadOutcome::MigratedFrom { from: found, config },
            Err(e) => return LoadOutcome::BackedUpAndReset {
                reason: format!("migration from v{found} failed: {e}"),
                config: Config::default(),
            },
        }
    }
    match crate::config::parse_config(text) {
        Ok(config) => LoadOutcome::Loaded(config),
        Err(e) => LoadOutcome::BackedUpAndReset { reason: e.to_string(), config: Config::default() },
    }
}

/// Applies migrate_vN_to_vN+1 in sequence. No migrations registered for M1.
fn migrate_chain(text: &str, _from: u32) -> Result<Config, ConfigError> {
    // future: for v in from..CURRENT_SCHEMA { value = step(v, value)?; }
    let cfg: Config = toml::from_str(text).map_err(|e| ConfigError::Parse(e.to_string()))?;
    validate(&cfg)?;
    Ok(cfg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_current_schema() {
        let text = "schema_version = 2\n[settings]\n";
        assert!(matches!(load_from_text(text), LoadOutcome::Loaded(_)));
    }

    #[test]
    fn garbage_backs_up_and_resets() {
        match load_from_text("== not toml ==") {
            LoadOutcome::BackedUpAndReset { config, .. } => assert_eq!(config.schema_version, 2),
            other => panic!("got {other:?}"),
        }
    }

    #[test]
    fn future_schema_is_too_new() {
        assert!(matches!(load_from_text("schema_version = 99"), LoadOutcome::TooNew { found: 99 }));
    }

    #[test]
    fn backup_name_format() {
        assert_eq!(backup_filename(1_700_000_000), "config.toml.bak-1700000000");
    }
}
