//! App state: filesystem paths, shared config, and config load/save I/O.

use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use sm_core::{BackoffState, Config, LoadOutcome, MachineId, SecretMode};
use sm_infra::files::LinuxFileOpener;
use sm_infra::probe::TcpStatusProbe;
use sm_infra::wol::UdpWaker;
use sm_infra::{FileHostKeyStore, InMemorySecretStore, RusshRunner, SshStatsProbe};

/// Locations of everything the app keeps on disk, rooted at `dir`
/// (Linux: `~/.config/server-manager/`, spec §4.1).
pub struct Paths {
    pub dir: PathBuf,
    pub config: PathBuf,
    pub known_hosts: PathBuf,
    pub log: PathBuf,
}

/// Resolves the config directory via `directories::ProjectDirs` and creates it.
pub fn resolve_paths() -> Paths {
    let dir = directories::ProjectDirs::from("", "", "server-manager")
        .expect("could not determine user config directory")
        .config_dir()
        .to_path_buf();
    let _ = std::fs::create_dir_all(&dir);
    Paths {
        config: dir.join("config.toml"),
        known_hosts: dir.join("known_hosts"),
        log: dir.join("latest.log"),
        dir,
    }
}

/// Shared mutable state handed to Tauri commands (Task 18).
pub struct AppState {
    pub cfg: RwLock<Config>,
    pub secrets_prompt: Arc<InMemorySecretStore>,
    pub paths: Paths,
    pub active: RwLock<Option<MachineId>>,
    /// Startup message from `load_config` (migration/reset), shown once by the UI.
    pub notice: RwLock<Option<String>>,
    pub waker: UdpWaker,
    pub probe: TcpStatusProbe,
    pub opener: LinuxFileOpener,
    pub runner: Arc<RusshRunner<InMemorySecretStore, FileHostKeyStore>>,
    pub stats: Arc<SshStatsProbe<RusshRunner<InMemorySecretStore, FileHostKeyStore>>>,
    /// Default secret mode for machines without an override; shared with the
    /// runner so `save_settings` takes effect without a restart.
    pub default_secret_mode: Arc<RwLock<SecretMode>>,
    /// `true` when `config.toml` uses a schema newer than this build
    /// supports (`TooNew`): the app runs on defaults and must not write
    /// config (all mutating commands fail with this message).
    pub read_only_reason: RwLock<Option<String>>,
    /// Signals the background poller (Task 19) to re-poll immediately.
    pub notify: Arc<tokio::sync::Notify>,
    /// Backoff of the active machine's poll loop; reset on
    /// `set_active`/`refresh_now`, ramped by consecutive offline polls.
    pub backoff: RwLock<BackoffState>,
}

impl AppState {
    /// Wires the infra implementations: the TOFU store points at
    /// `paths.known_hosts`, and the runner/probe share the prompt secret store.
    pub fn new(
        paths: Paths,
        cfg: Config,
        notice: Option<String>,
        read_only_reason: Option<String>,
    ) -> Self {
        let secrets_prompt = Arc::new(InMemorySecretStore::default());
        let host_keys = Arc::new(FileHostKeyStore::new(paths.known_hosts.clone()));
        // The runner shares the default secret mode through this lock: a
        // later `save_settings` changing it takes effect immediately.
        let default_secret_mode = Arc::new(RwLock::new(cfg.settings.default_secret_mode));
        let runner = Arc::new(RusshRunner::new(
            secrets_prompt.clone(),
            host_keys,
            default_secret_mode.clone(),
        ));
        let stats = Arc::new(SshStatsProbe::new(runner.clone()));
        let poll_base = cfg.settings.poll_base_secs.max(1);
        AppState {
            notify: Arc::new(tokio::sync::Notify::new()),
            backoff: RwLock::new(BackoffState::new(poll_base)),
            cfg: RwLock::new(cfg),
            secrets_prompt,
            paths,
            active: RwLock::new(None),
            notice: RwLock::new(notice),
            waker: UdpWaker,
            read_only_reason: RwLock::new(read_only_reason),
            probe: TcpStatusProbe,
            opener: LinuxFileOpener,
            runner,
            stats,
            default_secret_mode,
        }
    }
}

/// Loads `config.toml` through `sm_core::load_from_text`, applying the spec §4.3
/// side effects: on garbage/migration the original is copied to
/// `config.toml.bak-<unix-ts>` and the fresh/upgraded file is written.
pub struct LoadedConfig {
    pub cfg: Config,
    pub notice: Option<String>,
    /// `Some(reason)` for `TooNew`: config writes are refused while set.
    pub read_only_reason: Option<String>,
}

/// Loads `config.toml` through `sm_core::load_from_text`, applying the spec §4.3
/// side effects: on garbage/migration the original is copied to
/// `config.toml.bak-<unix-ts>` and the fresh/upgraded file is written.
/// IO errors on the side-effect writes surface in the notice instead of
/// being swallowed — the user sees that the backup/fresh-file step failed.
pub fn load_config(paths: &Paths) -> LoadedConfig {
    let text = match std::fs::read_to_string(&paths.config) {
        Ok(text) => text,
        Err(_) => {
            return LoadedConfig {
                cfg: Config::default(),
                notice: None,
                read_only_reason: None,
            };
        }
    };
    let io_err = |what: &str, e: std::io::Error| Some(format!("{what} failed: {e}"));
    match sm_core::load_from_text(&text) {
        LoadOutcome::Loaded(config) => LoadedConfig {
            cfg: config,
            notice: None,
            read_only_reason: None,
        },
        LoadOutcome::MigratedFrom { from, config } => {
            let mut notice: Option<String> = Some(format!(
                "config.toml was upgraded from schema v{from} to v{}; \
                 the old file was kept as config.toml.bak-*",
                sm_core::CURRENT_SCHEMA
            ));
            backup_current(paths, &text);
            if let Err(e) = std::fs::write(&paths.config, sm_core::serialize_config(&config)) {
                notice = io_err("rewriting upgraded config.toml", e);
            }
            LoadedConfig {
                cfg: config,
                notice,
                read_only_reason: None,
            }
        }
        LoadOutcome::BackedUpAndReset { reason, config } => {
            let mut notice: Option<String> = Some(format!(
                "config.toml is not valid TOML — it was backed up as \
                 config.toml.bak-* and reset to defaults ({reason})"
            ));
            backup_current(paths, &text);
            if let Err(e) = std::fs::write(&paths.config, sm_core::serialize_config(&config)) {
                notice = io_err("resetting config.toml", e);
            }
            LoadedConfig {
                cfg: config,
                notice,
                read_only_reason: None,
            }
        }
        LoadOutcome::TooNew { found } => LoadedConfig {
            cfg: Config::default(),
            notice: Some(format!(
                "config.toml uses schema v{found}, newer than this app \
                 supports (v{}); running with defaults and refusing to write \
                 until the app is upgraded",
                sm_core::CURRENT_SCHEMA
            )),
            read_only_reason: Some(format!(
                "config.toml uses schema v{found} (newer than supported v{})",
                sm_core::CURRENT_SCHEMA
            )),
        },
    }
}

/// Copies the current `config.toml` text to `config.toml.bak-<unix-ts>`.
/// Best-effort: a failed backup is logged, not fatal — the config text is
/// still in memory at this point and the fresh write below is what matters.
fn backup_current(paths: &Paths, text: &str) {
    let now_unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    if let Err(e) = std::fs::write(paths.dir.join(sm_core::backup_filename(now_unix)), text) {
        tracing::warn!("config backup write failed: {e}");
    }
}

/// Validates then atomically writes `config.toml` (`config.toml.tmp` → rename).
pub fn save_config(paths: &Paths, cfg: &Config) -> Result<(), String> {
    sm_core::validate(cfg).map_err(|e| e.to_string())?;
    let tmp = paths.config.with_extension("toml.tmp");
    std::fs::write(&tmp, sm_core::serialize_config(cfg)).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &paths.config).map_err(|e| e.to_string())?;
    Ok(())
}
