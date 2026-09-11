//! App state: filesystem paths, shared config, and config load/save I/O.

use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use sm_core::{BackoffState, Config, LoadOutcome, MachineId};
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
    /// Signals the background poller (Task 19) to re-poll immediately.
    pub notify: Arc<tokio::sync::Notify>,
    /// Backoff of the active machine's poll loop; reset on
    /// `set_active`/`refresh_now`, ramped by consecutive offline polls.
    pub backoff: RwLock<BackoffState>,
}

impl AppState {
    /// Wires the infra implementations: the TOFU store points at
    /// `paths.known_hosts`, and the runner/probe share the prompt secret store.
    pub fn new(paths: Paths, cfg: Config, notice: Option<String>) -> Self {
        let secrets_prompt = Arc::new(InMemorySecretStore::default());
        let host_keys = Arc::new(FileHostKeyStore::new(paths.known_hosts.clone()));
        // The runner captures `default_secret_mode` at construction; a later
        // `save_settings` changing it takes effect on restart. Keyring is
        // deferred beyond M1, where keyring and prompt behave identically.
        let runner = Arc::new(RusshRunner::new(
            secrets_prompt.clone(),
            host_keys,
            cfg.settings.default_secret_mode,
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
            probe: TcpStatusProbe,
            opener: LinuxFileOpener,
            runner,
            stats,
        }
    }
}

/// Loads `config.toml` through `sm_core::load_from_text`, applying the spec §4.3
/// side effects: on garbage/migration the original is copied to
/// `config.toml.bak-<unix-ts>` and the fresh/upgraded file is written.
/// Returns `(config, user_message)`; message is `None` when the file loaded clean.
pub fn load_config(paths: &Paths) -> (Config, Option<String>) {
    let text = match std::fs::read_to_string(&paths.config) {
        Ok(text) => text,
        Err(_) => return (Config::default(), None),
    };
    match sm_core::load_from_text(&text) {
        LoadOutcome::Loaded(config) => (config, None),
        LoadOutcome::MigratedFrom { from, config } => {
            backup_current(paths, &text);
            let _ = std::fs::write(&paths.config, sm_core::serialize_config(&config));
            (
                config,
                Some(format!(
                    "config.toml was upgraded from schema v{from} to v{}; \
                     the old file was kept as config.toml.bak-*",
                    sm_core::CURRENT_SCHEMA
                )),
            )
        }
        LoadOutcome::BackedUpAndReset { reason, config } => {
            backup_current(paths, &text);
            let _ = std::fs::write(&paths.config, sm_core::serialize_config(&config));
            (
                config,
                Some(format!(
                    "config.toml is not valid TOML — it was backed up as \
                     config.toml.bak-* and reset to defaults ({reason})"
                )),
            )
        }
        LoadOutcome::TooNew { found } => (
            Config::default(),
            Some(format!(
                "config.toml uses schema v{found}, newer than this app \
                 supports (v{}); running with defaults and refusing to write \
                 until the app is upgraded",
                sm_core::CURRENT_SCHEMA
            )),
        ),
    }
}

/// Copies the current `config.toml` text to `config.toml.bak-<unix-ts>`.
fn backup_current(paths: &Paths, text: &str) {
    let now_unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let _ = std::fs::write(paths.dir.join(sm_core::backup_filename(now_unix)), text);
}

/// Validates then atomically writes `config.toml` (`config.toml.tmp` → rename).
pub fn save_config(paths: &Paths, cfg: &Config) -> Result<(), String> {
    sm_core::validate(cfg).map_err(|e| e.to_string())?;
    let tmp = paths.config.with_extension("toml.tmp");
    std::fs::write(&tmp, sm_core::serialize_config(cfg)).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &paths.config).map_err(|e| e.to_string())?;
    Ok(())
}
