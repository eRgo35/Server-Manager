# Server Manager v2 — M1 (Linux) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a working Linux desktop build of Server Manager v2 — pick a
machine, see if it's up, Wake-on-LAN it, shut it down / reboot it and read
CPU/mem/disk/uptime over SSH, all from a small VSCode-styled window.

**Architecture:** Cargo workspace. `core` holds pure domain logic (types,
config schema + migration, WOL packet, stats parser, backoff, power-command
resolution) with zero I/O. `services` defines the traits the app depends
on. `infra` implements them (UDP, async TCP, `russh`, keyring, TOFU
known_hosts, file-manager launch). `src-tauri` is the Tauri v2 shell:
commands, background polling tasks that emit events, config/logging wiring.
`ui/` is a Vite project — plain-JS custom elements, no framework — that
renders the last event it received and calls commands.

**Tech Stack:** Rust 1.98 (workspace), Tauri v2, `tokio`, `russh`,
`keyring`, `directories`, `serde`/`toml`, `tracing`; Vite + vanilla JS
(ESM, custom elements, JSDoc); `just` task runner; Docker (integration
tests only); GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-09-10-server-manager-v2-design.md`
— read it alongside this plan; every task argues from it.

## Global Constraints

- **Language floors:** Rust edition 2021, toolchain ≥ 1.98. Node ≥ 24 for the `ui/` build.
- **`core` crate purity:** no `tokio`, no `russh`, no `tauri`, no filesystem, no network. Only `serde`, `toml`, `thiserror`, std. Enforced by a test in Task 2.
- **No new runtime dependency without it being named in this plan or the spec.** Dev/test deps are freer.
- **Frontend:** plain JavaScript only — no TypeScript, no UI framework. Type hints via JSDoc. Vite is allowed (dev server + minified static build).
- **Config location (via `directories::ProjectDirs`, qualifier `""`, org `""`, app `"server-manager"`):** Linux `~/.config/server-manager/`. Files: `config.toml`, `known_hosts`, `latest.log`.
- **Config schema:** `schema_version` integer, current value **2**. No v1→v2 migration path exists.
- **App identity:** product name `Server Manager`, bundle identifier `dev.czyz.servermanager`, license `MIT`.
- **Naming:** binary/crate root `server-manager`; workspace member crates `sm-core`, `sm-services`, `sm-infra`.
- **Status semantics:** "online" = a TCP connection to `os_host:ssh_port` succeeds within the timeout. No ICMP anywhere.
- **WOL:** magic packet = 6×`0xFF` + 16×MAC, sent UDP to `broadcast_addr` (default `255.255.255.255`) on ports **9 and 7**. No SecureOn, no IPv6.
- **Backoff:** base = `settings.poll_base_secs` (default 5s), factor ×2, cap 300s, reset to base on first success.
- **Every task ends on a green test run and a commit.** Conventional Commits. Commit message trailers on every commit:
  ```
  Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>
  Claude-Session: https://claude.ai/code/session_01K4bJ5wqXdS5rpJAPh8CYgc
  ```
- **Branch:** all work on `v2`.

---

## File Structure

```
server-manager/
├── Cargo.toml                     # workspace manifest
├── justfile                       # task runner
├── rust-toolchain.toml            # pin 1.98
├── .github/workflows/ci.yml       # lint + unit tests + integration + linux release
├── crates/
│   ├── sm-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs             # re-exports
│   │       ├── model.rs           # Machine, Settings, enums, Status, Stats
│   │       ├── config.rs          # Config, parse/serialize, defaults, validate
│   │       ├── migrate.rs         # schema_version handling + migration chain
│   │       ├── backoff.rs         # BackoffState
│   │       ├── wol.rs             # magic_packet()
│   │       ├── stats.rs           # parse_proc_stats()
│   │       └── power.rs           # resolve_power_command()
│   ├── sm-services/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs             # trait defs + shared error enum
│   └── sm-infra/
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── wol.rs             # UdpWaker
│           ├── probe.rs           # TcpStatusProbe
│           ├── known_hosts.rs     # FileHostKeyStore (TOFU)
│           ├── secrets.rs         # KeyringSecretStore + InMemory
│           ├── ssh.rs             # RusshRunner
│           ├── stats.rs           # SshStatsProbe
│           └── files.rs           # LinuxFileOpener
├── src-tauri/
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs
│       ├── state.rs               # AppState, config load/save
│       ├── logging.rs             # latest.log init
│       ├── commands.rs            # #[tauri::command] fns
│       └── poller.rs              # background status/stats tasks → events
├── ui/
│   ├── package.json
│   ├── vite.config.js
│   ├── index.html
│   └── src/
│       ├── main.js
│       ├── api.js                 # invoke/listen wrappers
│       ├── i18n/
│       │   ├── index.js
│       │   ├── en.json
│       │   └── pl.json
│       ├── components/
│       │   ├── machine-selector.js
│       │   ├── status-line.js
│       │   ├── action-buttons.js
│       │   ├── stats-panel.js
│       │   ├── settings-panel.js
│       │   └── toast-host.js
│       └── styles/
│           ├── theme.css
│           └── app.css
├── tests/
│   └── integration/
│       ├── docker-compose.yml     # sshd container
│       └── ssh_it.rs              # #[ignore]-gated integration tests
└── docs/
    └── visual-checklist.md
```

---

## Task 1: Workspace scaffold

**Files:**
- Create: `Cargo.toml`, `rust-toolchain.toml`, `justfile`, `crates/sm-core/Cargo.toml`, `crates/sm-core/src/lib.rs`, `crates/sm-services/Cargo.toml`, `crates/sm-services/src/lib.rs`, `crates/sm-infra/Cargo.toml`, `crates/sm-infra/src/lib.rs`

**Interfaces:**
- Consumes: nothing.
- Produces: a compiling workspace with three library crates `sm-core`, `sm-services`, `sm-infra`.

- [ ] **Step 1: Create the workspace manifest**

`Cargo.toml`:
```toml
[workspace]
resolver = "2"
members = ["crates/sm-core", "crates/sm-services", "crates/sm-infra", "src-tauri"]

[workspace.package]
edition = "2021"
license = "MIT"
rust-version = "1.98"

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
toml = "0.8"
thiserror = "1"
tokio = { version = "1", features = ["rt-multi-thread", "net", "time", "macros", "sync"] }
tracing = "0.1"
```
Note: `src-tauri` is a member now but is created in Task 17; until then, comment it out of `members` if `cargo` complains — re-add in Task 17.

- [ ] **Step 2: Pin the toolchain**

`rust-toolchain.toml`:
```toml
[toolchain]
channel = "1.98"
components = ["rustfmt", "clippy"]
```

- [ ] **Step 3: Create the three crate manifests**

`crates/sm-core/Cargo.toml`:
```toml
[package]
name = "sm-core"
version = "0.1.0"
edition.workspace = true
license.workspace = true

[dependencies]
serde = { workspace = true }
toml = { workspace = true }
thiserror = { workspace = true }
```
`crates/sm-services/Cargo.toml`:
```toml
[package]
name = "sm-services"
version = "0.1.0"
edition.workspace = true
license.workspace = true

[dependencies]
sm-core = { path = "../sm-core" }
thiserror = { workspace = true }
```
`crates/sm-infra/Cargo.toml`:
```toml
[package]
name = "sm-infra"
version = "0.1.0"
edition.workspace = true
license.workspace = true

[dependencies]
sm-core = { path = "../sm-core" }
sm-services = { path = "../sm-services" }
tokio = { workspace = true }
tracing = { workspace = true }
```

- [ ] **Step 4: Create empty lib roots**

Each of `crates/sm-core/src/lib.rs`, `crates/sm-services/src/lib.rs`, `crates/sm-infra/src/lib.rs`:
```rust
//! placeholder — populated by later tasks
```

- [ ] **Step 5: Create the justfile**

`justfile`:
```just
default:
    @just --list

# install the extra tooling this repo needs
setup:
    cargo install just tauri-cli --locked || true
    rustup component add rustfmt clippy

fmt:
    cargo fmt --all

lint:
    cargo fmt --all --check
    cargo clippy --workspace --all-targets -- -D warnings

test:
    cargo test --workspace

# docker-backed SSH tests, opt-in
test-integration:
    docker compose -f tests/integration/docker-compose.yml up -d --wait
    -cargo test --package sm-infra --test ssh_it -- --ignored --test-threads 1
    docker compose -f tests/integration/docker-compose.yml down

build-linux:
    cargo tauri build --bundles appimage,rpm
    bash packaging/build-pkgbuild.sh

build-android:
    cargo tauri android build --apk
```

- [ ] **Step 6: Verify it compiles**

Run: `cargo build --workspace`
Expected: builds `sm-core`, `sm-services`, `sm-infra` with no errors (warnings about empty crates are fine).

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml rust-toolchain.toml justfile crates/
git commit -m "chore: scaffold cargo workspace (sm-core/services/infra)"
```

---

## Task 2: `sm-core` — domain model

**Files:**
- Create: `crates/sm-core/src/model.rs`
- Modify: `crates/sm-core/src/lib.rs`
- Test: inline `#[cfg(test)]` in `model.rs`

**Interfaces:**
- Consumes: nothing.
- Produces:
  - `pub struct MachineId(pub String)` — newtype, `Display`, `From<&str>`.
  - `pub enum SecretMode { Plaintext, Keyring, Prompt }` — serde `rename_all = "lowercase"`.
  - `pub enum Theme { System, Light, Dark }` — serde lowercase.
  - `pub enum StatsDisplay { Graph, Numbers }` — serde lowercase.
  - `pub enum MountProtocol { Smb, Sshfs }` — serde lowercase.
  - `pub enum PowerAction { Shutdown, Reboot }`
  - `pub enum Status { Online, Offline, Unknown }`
  - `pub struct Stats { pub cpu_pct: f32, pub mem_used: u64, pub mem_total: u64, pub disk_used: u64, pub disk_total: u64, pub uptime_secs: u64 }` — `Serialize`, `Clone`, `PartialEq`.
  - `pub struct Machine { pub id: MachineId, pub name: String, pub mac: String, pub broadcast_addr: Option<String>, pub os_host: String, pub ssh_port: u16, pub ssh_user: String, pub secret_mode: Option<SecretMode>, pub shutdown_cmd: Option<String>, pub reboot_cmd: Option<String>, pub stats_cmd: Option<String>, pub share_path: Option<String>, pub ssh_password: Option<String>, pub key_passphrase: Option<String>, pub sudo_password: Option<String> }` — `Serialize`, `Deserialize`, `Clone`.
  - `pub struct Settings { pub language: String, pub theme: Theme, pub stats_display: StatsDisplay, pub poll_base_secs: u64, pub default_secret_mode: SecretMode, pub mount_protocol: MountProtocol }` — `Serialize`, `Deserialize`, `Clone`; `impl Default`.

- [ ] **Step 1: Write the failing test**

In `crates/sm-core/src/model.rs`:
```rust
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

    #[test]
    fn secret_mode_serde_is_lowercase() {
        let t = toml::to_string(&SecretMode::Keyring).unwrap();
        assert!(t.contains("keyring"));
        let back: SecretMode = toml::from_str("v = \"prompt\"")
            .map(|w: Wrap| w.v).unwrap();
        assert_eq!(back, SecretMode::Prompt);
    }

    #[derive(serde::Deserialize)]
    struct Wrap { v: SecretMode }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p sm-core`
Expected: FAIL — `Settings`, `Theme`, etc. not defined.

- [ ] **Step 3: Write the implementation**

Fill `crates/sm-core/src/model.rs` with the types listed in **Interfaces** above. Details:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MachineId(pub String);

impl std::fmt::Display for MachineId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}
impl From<&str> for MachineId {
    fn from(s: &str) -> Self { MachineId(s.to_string()) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SecretMode { Plaintext, Keyring, Prompt }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme { System, Light, Dark }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StatsDisplay { Graph, Numbers }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MountProtocol { Smb, Sshfs }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerAction { Shutdown, Reboot }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Status { Online, Offline, Unknown }

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
    #[serde(default)]
    pub broadcast_addr: Option<String>,
    pub os_host: String,
    #[serde(default = "default_ssh_port")]
    pub ssh_port: u16,
    pub ssh_user: String,
    #[serde(default)]
    pub secret_mode: Option<SecretMode>,
    #[serde(default)]
    pub shutdown_cmd: Option<String>,
    #[serde(default)]
    pub reboot_cmd: Option<String>,
    #[serde(default)]
    pub stats_cmd: Option<String>,
    #[serde(default)]
    pub share_path: Option<String>,
    #[serde(default)]
    pub ssh_password: Option<String>,
    #[serde(default)]
    pub key_passphrase: Option<String>,
    #[serde(default)]
    pub sudo_password: Option<String>,
}

fn default_ssh_port() -> u16 { 22 }

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

fn def_lang() -> String { "en".into() }
fn def_poll() -> u64 { 5 }
fn def_secret() -> SecretMode { SecretMode::Keyring }

impl Default for Theme { fn default() -> Self { Theme::System } }
impl Default for StatsDisplay { fn default() -> Self { StatsDisplay::Graph } }
impl Default for MountProtocol { fn default() -> Self { MountProtocol::Smb } }
impl Default for Settings {
    fn default() -> Self {
        Settings {
            language: def_lang(), theme: Theme::System,
            stats_display: StatsDisplay::Graph, poll_base_secs: def_poll(),
            default_secret_mode: def_secret(), mount_protocol: MountProtocol::Smb,
        }
    }
}
```

- [ ] **Step 4: Wire the module and add the purity guard**

`crates/sm-core/src/lib.rs`:
```rust
//! Pure domain logic for Server Manager. No I/O, no async, no framework types.
pub mod model;
pub use model::*;
```
Add `crates/sm-core/tests/purity.rs`:
```rust
//! Fails the build if sm-core grows a forbidden dependency.
#[test]
fn no_forbidden_deps() {
    let manifest = include_str!("../Cargo.toml");
    for bad in ["tokio", "russh", "tauri", "reqwest", "keyring"] {
        assert!(!manifest.contains(bad), "sm-core must not depend on {bad}");
    }
}
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cargo test -p sm-core`
Expected: PASS (3 tests).

- [ ] **Step 6: Commit**

```bash
git add crates/sm-core
git commit -m "feat(core): domain model types and Settings defaults"
```

---

## Task 3: `sm-core` — config parse, serialize, validate

**Files:**
- Create: `crates/sm-core/src/config.rs`
- Modify: `crates/sm-core/src/lib.rs`
- Test: inline in `config.rs`

**Interfaces:**
- Consumes: `model::{Machine, Settings, MachineId}`.
- Produces:
  - `pub const CURRENT_SCHEMA: u32 = 2;`
  - `pub struct Config { pub schema_version: u32, pub settings: Settings, pub machines: Vec<Machine> }` — `Serialize`, `Deserialize` (`#[serde(default)]` on machines), `Default` (schema 2, default settings, empty machines).
  - `pub enum ConfigError { Parse(String), Validation(String) }` — `thiserror`, `Display`.
  - `pub fn parse_config(text: &str) -> Result<Config, ConfigError>` — parses TOML, then runs `validate`.
  - `pub fn serialize_config(cfg: &Config) -> String` — pretty TOML.
  - `pub fn validate(cfg: &Config) -> Result<(), ConfigError>` — rules below.
  - `impl Config { pub fn machine(&self, id: &MachineId) -> Option<&Machine> }`

Validation rules:
1. machine ids unique → else `Validation("duplicate machine id: <id>")`.
2. every `mac` matches `^([0-9A-Fa-f]{2}[:-]){5}[0-9A-Fa-f]{2}$` → else `Validation("invalid mac for <id>")`.
3. `ssh_port != 0`.
4. `os_host` non-empty, `ssh_user` non-empty.
5. `poll_base_secs >= 1`.

- [ ] **Step 1: Write the failing tests**

```rust
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
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p sm-core config`
Expected: FAIL — `parse_config` undefined.

- [ ] **Step 3: Implement `config.rs`**

```rust
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
```
`lib.rs`: add `pub mod config; pub use config::*;`

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p sm-core`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/sm-core
git commit -m "feat(core): config schema parse/serialize/validate"
```

---

## Task 4: `sm-core` — schema migration

**Files:**
- Create: `crates/sm-core/src/migrate.rs`
- Modify: `crates/sm-core/src/lib.rs`
- Test: inline

**Interfaces:**
- Consumes: `config::CURRENT_SCHEMA`.
- Produces:
  - `pub enum LoadOutcome { Loaded(Config), MigratedFrom { from: u32, config: Config }, BackedUpAndReset { reason: String, config: Config }, TooNew { found: u32 } }`
  - `pub fn load_from_text(text: &str) -> LoadOutcome` — the whole decision tree from spec §4.3, minus the actual file writes (caller does those):
    - unparseable → `BackedUpAndReset { reason, Config::default() }`
    - `schema_version > CURRENT_SCHEMA` → `TooNew { found }`
    - `schema_version < CURRENT_SCHEMA` → run `migrate_chain`, return `MigratedFrom`
    - equal → `Loaded`
  - `pub fn backup_filename(now_unix: i64) -> String` → `format!("config.toml.bak-{now_unix}")`
  - `fn migrate_chain(mut value: toml::Value, from: u32) -> Result<Config, ConfigError>` — applies `migrate_v{n}_to_v{n+1}` steps in order. For M1 there are no older versions, so the chain is empty and this is only reachable if a future v3 downgrades; keep the structure, return `Loaded` behavior. Add a `#[test]` that documents "no migrations registered yet".

- [ ] **Step 1: Failing tests**

```rust
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
```

- [ ] **Step 2: Run — expect FAIL** (`load_from_text` undefined). `cargo test -p sm-core migrate`

- [ ] **Step 3: Implement `migrate.rs`**

```rust
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
```
`lib.rs`: `pub mod migrate; pub use migrate::*;`

- [ ] **Step 4: Run — expect PASS.** `cargo test -p sm-core`

- [ ] **Step 5: Commit**

```bash
git add crates/sm-core
git commit -m "feat(core): schema-version load outcomes and migration scaffold"
```

---

## Task 5: `sm-core` — backoff state machine

**Files:**
- Create: `crates/sm-core/src/backoff.rs`; Modify: `lib.rs`; Test: inline.

**Interfaces:**
- Produces:
  - `pub struct BackoffState { base_secs: u64, cap_secs: u64, failures: u32 }`
  - `pub fn new(base_secs: u64) -> BackoffState` — cap fixed at 300.
  - `pub fn on_success(&mut self)` — `failures = 0`.
  - `pub fn on_failure(&mut self)` — `failures = failures.saturating_add(1)`.
  - `pub fn current_delay_secs(&self) -> u64` — `min(cap, base * 2^(failures.saturating_sub? ))`: 0 failures → base; 1 → base*2; capped at 300. Use `base.saturating_mul(2u64.saturating_pow(failures))` then `.min(cap)`.

- [ ] **Step 1: Failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ramps_and_caps_and_resets() {
        let mut b = BackoffState::new(5);
        assert_eq!(b.current_delay_secs(), 5);
        b.on_failure(); assert_eq!(b.current_delay_secs(), 10);
        b.on_failure(); assert_eq!(b.current_delay_secs(), 20);
        for _ in 0..20 { b.on_failure(); }
        assert_eq!(b.current_delay_secs(), 300); // capped
        b.on_success();
        assert_eq!(b.current_delay_secs(), 5);
    }
}
```

- [ ] **Step 2: Run — FAIL.** `cargo test -p sm-core backoff`

- [ ] **Step 3: Implement**

```rust
#[derive(Debug, Clone)]
pub struct BackoffState { base_secs: u64, cap_secs: u64, failures: u32 }

impl BackoffState {
    pub fn new(base_secs: u64) -> Self {
        Self { base_secs: base_secs.max(1), cap_secs: 300, failures: 0 }
    }
    pub fn on_success(&mut self) { self.failures = 0; }
    pub fn on_failure(&mut self) { self.failures = self.failures.saturating_add(1); }
    pub fn current_delay_secs(&self) -> u64 {
        let factor = 2u64.saturating_pow(self.failures);
        self.base_secs.saturating_mul(factor).min(self.cap_secs)
    }
}
```
`lib.rs`: `pub mod backoff; pub use backoff::*;`

- [ ] **Step 4: Run — PASS.**

- [ ] **Step 5: Commit** — `git commit -m "feat(core): exponential backoff state machine"`

---

## Task 6: `sm-core` — Wake-on-LAN magic packet

**Files:** Create `crates/sm-core/src/wol.rs`; Modify `lib.rs`; Test inline.

**Interfaces:**
- Produces:
  - `pub enum WolError { BadMac(String) }` — `thiserror`.
  - `pub fn parse_mac(s: &str) -> Result<[u8; 6], WolError>` — accepts `:` or `-` separators.
  - `pub fn magic_packet(mac: [u8; 6]) -> [u8; 102]` — 6×0xFF then MAC ×16.
  - `pub const WOL_PORTS: [u16; 2] = [9, 7];`

- [ ] **Step 1: Failing test**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn packet_layout() {
        let mac = parse_mac("01:02:03:04:05:06").unwrap();
        let p = magic_packet(mac);
        assert_eq!(&p[0..6], &[0xFF; 6]);
        assert_eq!(&p[6..12], &mac);
        assert_eq!(&p[96..102], &mac);
        assert_eq!(p.len(), 102);
    }
    #[test]
    fn rejects_bad_mac() {
        assert!(parse_mac("zz:zz").is_err());
    }
    #[test]
    fn accepts_dash_form() {
        assert!(parse_mac("AA-BB-CC-DD-EE-FF").is_ok());
    }
}
```

- [ ] **Step 2: Run — FAIL.**

- [ ] **Step 3: Implement**

```rust
#[derive(Debug, thiserror::Error)]
pub enum WolError {
    #[error("invalid MAC address: {0}")]
    BadMac(String),
}

pub const WOL_PORTS: [u16; 2] = [9, 7];

pub fn parse_mac(s: &str) -> Result<[u8; 6], WolError> {
    let parts: Vec<&str> = s.split([':', '-']).collect();
    if parts.len() != 6 { return Err(WolError::BadMac(s.into())); }
    let mut out = [0u8; 6];
    for (i, p) in parts.iter().enumerate() {
        out[i] = u8::from_str_radix(p, 16).map_err(|_| WolError::BadMac(s.into()))?;
    }
    Ok(out)
}

pub fn magic_packet(mac: [u8; 6]) -> [u8; 102] {
    let mut p = [0u8; 102];
    for b in p.iter_mut().take(6) { *b = 0xFF; }
    for chunk in 0..16 {
        p[6 + chunk * 6..6 + chunk * 6 + 6].copy_from_slice(&mac);
    }
    p
}
```
`lib.rs`: `pub mod wol; pub use wol::*;`

- [ ] **Step 4: Run — PASS.**
- [ ] **Step 5: Commit** — `git commit -m "feat(core): wake-on-lan magic packet builder"`

---

## Task 7: `sm-core` — `/proc` stats parser

**Files:** Create `crates/sm-core/src/stats.rs`; Modify `lib.rs`; Test inline with fixtures.

**Interfaces:**
- Consumes: `model::Stats`.
- Produces:
  - `pub enum StatsError { Malformed(String) }`
  - `pub fn parse_proc_stats(raw: &str) -> Result<Stats, StatsError>` — parses the concatenated output of the batched command from spec §7.4. Input format (in order): `/proc/uptime` line, `/proc/meminfo` block, `df -B1 /` output, first `/proc/stat`, then (after 0.2s) second `/proc/stat`. Delimit the two `/proc/stat` snapshots with a sentinel: the batched command is redefined here as
    ```
    echo '---SM-UPTIME'; cat /proc/uptime;
    echo '---SM-MEM'; cat /proc/meminfo;
    echo '---SM-DISK'; df -B1 --output=size,used / | tail -1;
    echo '---SM-CPU1'; head -1 /proc/stat;
    sleep 0.2;
    echo '---SM-CPU2'; head -1 /proc/stat
    ```
  - CPU%: from two `cpu ...` lines, `busy = total - idle`, `pct = 100 * dbusy / dtotal`.
  - mem: `MemTotal`, `MemAvailable` → `mem_used = total - available`.
  - disk: the two integers from the `df` tail line (bytes).
  - uptime: first float of `/proc/uptime`, truncated to u64.

- [ ] **Step 1: Failing test with a realistic fixture**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    const RAW: &str = "\
---SM-UPTIME
12345.67 98765.43
---SM-MEM
MemTotal:       16384000 kB
MemFree:         1000000 kB
MemAvailable:    8192000 kB
Buffers:          200000 kB
---SM-DISK
   500107862016   123456789012
---SM-CPU1
cpu  100 0 100 800 0 0 0 0 0 0
---SM-CPU2
cpu  110 0 110 860 0 0 0 0 0 0
";

    #[test]
    fn parses_all_fields() {
        let s = parse_proc_stats(RAW).unwrap();
        assert_eq!(s.uptime_secs, 12345);
        assert_eq!(s.mem_total, 16_384_000 * 1024);
        assert_eq!(s.mem_used, (16_384_000 - 8_192_000) * 1024);
        assert_eq!(s.disk_total, 500_107_862_016);
        assert_eq!(s.disk_used, 123_456_789_012);
        // delta busy = (110+110)-(100+100)=40 ... wait: busy1=100+0+100=200, busy2=110+0+110=220 -> dbusy=20
        // total1=1000, total2=1190 -> dtotal=190 ; pct = 100*20/190 ≈ 10.53
        assert!((s.cpu_pct - 10.526).abs() < 0.1);
    }

    #[test]
    fn malformed_errs() {
        assert!(parse_proc_stats("nonsense").is_err());
    }
}
```

- [ ] **Step 2: Run — FAIL.**

- [ ] **Step 3: Implement `stats.rs`**

Write a parser that splits on the `---SM-*` sentinels into a map, then:
```rust
use crate::model::Stats;

#[derive(Debug, thiserror::Error)]
pub enum StatsError {
    #[error("malformed stats output: {0}")]
    Malformed(String),
}

fn section<'a>(raw: &'a str, name: &str) -> Result<&'a str, StatsError> {
    let start = raw.find(&format!("---SM-{name}\n"))
        .ok_or_else(|| StatsError::Malformed(format!("missing section {name}")))?;
    let rest = &raw[start + name.len() + 7..];
    let end = rest.find("---SM-").unwrap_or(rest.len());
    Ok(rest[..end].trim())
}

fn cpu_busy_total(line: &str) -> Result<(u64, u64), StatsError> {
    let nums: Vec<u64> = line.split_whitespace().skip(1)
        .filter_map(|t| t.parse().ok()).collect();
    if nums.len() < 4 { return Err(StatsError::Malformed("cpu line".into())); }
    let total: u64 = nums.iter().sum();
    let idle = nums[3];
    Ok((total - idle, total))
}

pub fn parse_proc_stats(raw: &str) -> Result<Stats, StatsError> {
    let uptime_secs = section(raw, "UPTIME")?
        .split_whitespace().next()
        .and_then(|f| f.parse::<f64>().ok())
        .ok_or_else(|| StatsError::Malformed("uptime".into()))? as u64;

    let mem = section(raw, "MEM")?;
    let kb = |key: &str| mem.lines()
        .find(|l| l.starts_with(key))
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|v| v.parse::<u64>().ok())
        .map(|v| v * 1024);
    let mem_total = kb("MemTotal:").ok_or_else(|| StatsError::Malformed("MemTotal".into()))?;
    let mem_avail = kb("MemAvailable:").ok_or_else(|| StatsError::Malformed("MemAvailable".into()))?;
    let mem_used = mem_total.saturating_sub(mem_avail);

    let disk_line = section(raw, "DISK")?;
    let mut dit = disk_line.split_whitespace();
    let disk_total = dit.next().and_then(|v| v.parse().ok())
        .ok_or_else(|| StatsError::Malformed("disk size".into()))?;
    let disk_used = dit.next().and_then(|v| v.parse().ok())
        .ok_or_else(|| StatsError::Malformed("disk used".into()))?;

    let (b1, t1) = cpu_busy_total(section(raw, "CPU1")?)?;
    let (b2, t2) = cpu_busy_total(section(raw, "CPU2")?)?;
    let dtotal = t2.saturating_sub(t1).max(1);
    let dbusy = b2.saturating_sub(b1);
    let cpu_pct = (100.0 * dbusy as f64 / dtotal as f64) as f32;

    Ok(Stats { cpu_pct, mem_used, mem_total, disk_used, disk_total, uptime_secs })
}

/// The remote command whose output `parse_proc_stats` consumes.
pub const PROC_STATS_CMD: &str = "echo '---SM-UPTIME'; cat /proc/uptime; echo '---SM-MEM'; cat /proc/meminfo; echo '---SM-DISK'; df -B1 --output=size,used / | tail -1; echo '---SM-CPU1'; head -1 /proc/stat; sleep 0.2; echo '---SM-CPU2'; head -1 /proc/stat";
```
`lib.rs`: `pub mod stats; pub use stats::*;`

- [ ] **Step 4: Run — PASS.** Adjust the CPU assertion tolerance if the arithmetic in the fixture comment differs; recompute from the fixture and lock the expected value.
- [ ] **Step 5: Commit** — `git commit -m "feat(core): /proc stats parser + batched command string"`

---

## Task 8: `sm-core` — power command resolution

**Files:** Create `crates/sm-core/src/power.rs`; Modify `lib.rs`; Test inline.

**Interfaces:**
- Consumes: `model::{Machine, PowerAction}`.
- Produces:
  - `pub const DEFAULT_SHUTDOWN: &str = "shutdown -h now";`
  - `pub const DEFAULT_REBOOT: &str = "shutdown -r now";`
  - `pub struct PowerPlan { pub primary: String, pub sudo_fallback: Option<String> }`
  - `pub fn resolve_power(machine: &Machine, action: PowerAction) -> PowerPlan` —
    - `primary` = machine override (`shutdown_cmd`/`reboot_cmd`) if `Some`, else the default.
    - `sudo_fallback` = `Some(format!("sudo -S -p '' {primary}"))` **iff** `machine.sudo_password.is_some()`, else `None`.
  - `pub fn needs_sudo(exit_code: i32, stderr: &str) -> bool` — true if exit != 0 and stderr matches any of `["permission denied", "must be root", "Operation not permitted", "not authorized"]` case-insensitive.

- [ ] **Step 1: Failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::*;

    fn m() -> Machine {
        Machine { id: "x".into(), name: "x".into(), mac: "AA:BB:CC:DD:EE:FF".into(),
            broadcast_addr: None, os_host: "h".into(), ssh_port: 22, ssh_user: "u".into(),
            secret_mode: None, shutdown_cmd: None, reboot_cmd: None, stats_cmd: None,
            share_path: None, ssh_password: None, key_passphrase: None, sudo_password: None }
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
        assert_eq!(p.sudo_fallback.as_deref(), Some("sudo -S -p '' systemctl reboot"));
    }

    #[test]
    fn detects_permission_failure() {
        assert!(needs_sudo(1, "shutdown: Permission denied"));
        assert!(!needs_sudo(0, ""));
        assert!(!needs_sudo(1, "command not found"));
    }
}
```

- [ ] **Step 2: Run — FAIL.**
- [ ] **Step 3: Implement per Interfaces.**
- [ ] **Step 4: Run — PASS.**
- [ ] **Step 5: Commit** — `git commit -m "feat(core): power command resolution and sudo detection"`

---

## Task 9: `sm-services` — trait definitions

**Files:** Modify `crates/sm-services/src/lib.rs`; Test: a `tests/compiles.rs` that implements each trait with a dummy to lock the signatures.

**Interfaces:**
- Consumes: `sm-core::{Machine, MachineId, Stats, PowerAction}`.
- Produces (all `async` traits via `async fn` in traits — stable in 1.98; add `Send` bounds where needed for tokio):
  ```rust
  #[derive(Debug, thiserror::Error)]
  pub enum ServiceError {
      #[error("network: {0}")] Network(String),
      #[error("auth failed")] Auth,
      #[error("host key changed for {0}")] HostKeyChanged(String),
      #[error("host key not trusted")] HostKeyUntrusted,
      #[error("timeout")] Timeout,
      #[error("remote command failed ({code}): {stderr}")] Remote { code: i32, stderr: String },
      #[error("{0}")] Other(String),
  }

  pub trait Waker: Send + Sync {
      fn wake(&self, mac: [u8; 6], broadcast_addr: &str) -> impl std::future::Future<Output = Result<(), ServiceError>> + Send;
  }

  pub trait StatusProbe: Send + Sync {
      fn is_up(&self, host: &str, port: u16, timeout: std::time::Duration)
          -> impl std::future::Future<Output = bool> + Send;
  }

  pub struct CommandOutput { pub code: i32, pub stdout: String, pub stderr: String }

  pub trait SshRunner: Send + Sync {
      fn run(&self, machine: &Machine, command: &str)
          -> impl std::future::Future<Output = Result<CommandOutput, ServiceError>> + Send;
  }

  pub trait StatsProbe: Send + Sync {
      fn sample(&self, machine: &Machine)
          -> impl std::future::Future<Output = Result<Stats, ServiceError>> + Send;
  }

  #[derive(Debug, Clone, Copy, PartialEq)]
  pub enum SecretKind { SshPassword, KeyPassphrase, SudoPassword }

  pub trait SecretStore: Send + Sync {
      fn get(&self, machine_id: &MachineId, kind: SecretKind) -> Option<String>;
      fn set(&self, machine_id: &MachineId, kind: SecretKind, value: &str) -> Result<(), ServiceError>;
      fn clear(&self, machine_id: &MachineId, kind: SecretKind) -> Result<(), ServiceError>;
  }

  #[derive(Debug, PartialEq)]
  pub enum HostKeyVerdict { TrustedMatch, Unknown, Changed { stored_fp: String } }

  pub trait HostKeyStore: Send + Sync {
      fn verify(&self, host: &str, port: u16, fingerprint: &str) -> HostKeyVerdict;
      fn trust(&self, host: &str, port: u16, fingerprint: &str) -> Result<(), ServiceError>;
  }

  pub trait FileOpener: Send + Sync {
      fn open_files(&self, machine: &Machine) -> Result<(), ServiceError>;
      fn map_drive(&self, machine: &Machine) -> Result<(), ServiceError>;
  }
  ```

- [ ] **Step 1: Write the compile-lock test**

`crates/sm-services/tests/compiles.rs`: a zero-behavior struct implementing every trait (returning `Default`/`false`/`None`/errors), asserting the crate's contracts compile. Include one `#[tokio::test]`-free sync assertion (`SecretStore`, `HostKeyStore`, `FileOpener` are sync).

Add `tokio` as a `[dev-dependencies]` of `sm-services` for this test only.

- [ ] **Step 2: Run — FAIL** (`ServiceError` etc. undefined). `cargo test -p sm-services`
- [ ] **Step 3: Implement `lib.rs` with the block above.**
- [ ] **Step 4: Run — PASS.**
- [ ] **Step 5: Commit** — `git commit -m "feat(services): trait contracts for infra layer"`

---

## Task 10: `sm-infra` — `UdpWaker`

**Files:** Create `crates/sm-infra/src/wol.rs`; Modify `lib.rs`; Test inline (`#[tokio::test]`).

Add to `sm-infra/Cargo.toml` deps: nothing new (tokio already there). Dev-deps: none.

**Interfaces:**
- Consumes: `sm-core::wol::{magic_packet, WOL_PORTS}`, `sm-services::{Waker, ServiceError}`.
- Produces: `pub struct UdpWaker;` implementing `Waker`. `wake` binds `0.0.0.0:0`, sets broadcast, sends the packet to `broadcast_addr:9` and `:7`.

- [ ] **Step 1: Failing test** — bind a UDP socket on `127.0.0.1:0`, call `UdpWaker.wake(mac, "127.0.0.1")` (override ports in the test via a `wake_to(addr, ports)` internal helper pointed at the bound port), assert 102 bytes received and first 6 are `0xFF`.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::UdpSocket;

    #[tokio::test]
    async fn sends_magic_packet() {
        let listener = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let w = UdpWaker;
        w.wake_to([1,2,3,4,5,6], "127.0.0.1", &[port]).await.unwrap();
        let mut buf = [0u8; 200];
        let n = listener.recv(&mut buf).await.unwrap();
        assert_eq!(n, 102);
        assert_eq!(&buf[0..6], &[0xFF; 6]);
    }
}
```

- [ ] **Step 2: Run — FAIL.**
- [ ] **Step 3: Implement**

```rust
use sm_core::wol::{magic_packet, WOL_PORTS};
use sm_services::{ServiceError, Waker};
use tokio::net::UdpSocket;

pub struct UdpWaker;

impl UdpWaker {
    pub(crate) async fn wake_to(&self, mac: [u8; 6], addr: &str, ports: &[u16]) -> Result<(), ServiceError> {
        let sock = UdpSocket::bind("0.0.0.0:0").await.map_err(|e| ServiceError::Network(e.to_string()))?;
        sock.set_broadcast(true).map_err(|e| ServiceError::Network(e.to_string()))?;
        let packet = magic_packet(mac);
        for p in ports {
            sock.send_to(&packet, (addr, *p)).await.map_err(|e| ServiceError::Network(e.to_string()))?;
        }
        Ok(())
    }
}

impl Waker for UdpWaker {
    async fn wake(&self, mac: [u8; 6], broadcast_addr: &str) -> Result<(), ServiceError> {
        self.wake_to(mac, broadcast_addr, &WOL_PORTS).await
    }
}
```
`lib.rs`: `pub mod wol; pub use wol::UdpWaker;`

- [ ] **Step 4: Run — PASS.**
- [ ] **Step 5: Commit** — `git commit -m "feat(infra): UDP wake-on-lan sender"`

---

## Task 11: `sm-infra` — `TcpStatusProbe`

**Files:** Create `crates/sm-infra/src/probe.rs`; Modify `lib.rs`; Test inline.

**Interfaces:**
- Produces: `pub struct TcpStatusProbe;` implementing `StatusProbe`. `is_up` = `tokio::time::timeout(timeout, TcpStream::connect((host, port))).await` → `matches!(_, Ok(Ok(_)))`. Never panics, never blocks beyond `timeout`.

- [ ] **Step 1: Failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn up_when_listener_present() {
        let l = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = l.local_addr().unwrap().port();
        assert!(TcpStatusProbe.is_up("127.0.0.1", port, Duration::from_secs(1)).await);
    }

    #[tokio::test]
    async fn down_on_closed_port() {
        assert!(!TcpStatusProbe.is_up("127.0.0.1", 1, Duration::from_millis(200)).await);
    }

    #[tokio::test]
    async fn down_on_unroutable_within_timeout() {
        let start = std::time::Instant::now();
        let up = TcpStatusProbe.is_up("10.255.255.1", 22, Duration::from_millis(300)).await;
        assert!(!up);
        assert!(start.elapsed() < Duration::from_secs(2));
    }
}
```

- [ ] **Step 2: Run — FAIL.**
- [ ] **Step 3: Implement** the probe as described.
- [ ] **Step 4: Run — PASS.**
- [ ] **Step 5: Commit** — `git commit -m "feat(infra): non-blocking TCP status probe"`

---

## Task 12: `sm-infra` — `FileHostKeyStore` (TOFU)

**Files:** Create `crates/sm-infra/src/known_hosts.rs`; Modify `lib.rs`; Test inline with `tempfile`.

Add `sm-infra` dev-dep: `tempfile = "3"`.

**Interfaces:**
- Produces:
  - `pub struct FileHostKeyStore { path: PathBuf }`
  - `pub fn new(path: impl Into<PathBuf>) -> Self`
  - implements `HostKeyStore`. File format: one `host:port SHA256:base64fp` per line. `verify` reads the file fresh each call (cheap; small file). `trust` appends (or replaces the line if host:port already present with a different fp — but `verify` would have returned `Changed` first; `trust` overwrites deliberately on user confirmation).

- [ ] **Step 1: Failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sm_services::{HostKeyStore, HostKeyVerdict};

    #[test]
    fn unknown_then_trusted_then_changed() {
        let dir = tempfile::tempdir().unwrap();
        let store = FileHostKeyStore::new(dir.path().join("known_hosts"));
        assert_eq!(store.verify("nas", 22, "SHA256:aaa"), HostKeyVerdict::Unknown);
        store.trust("nas", 22, "SHA256:aaa").unwrap();
        assert_eq!(store.verify("nas", 22, "SHA256:aaa"), HostKeyVerdict::TrustedMatch);
        match store.verify("nas", 22, "SHA256:bbb") {
            HostKeyVerdict::Changed { stored_fp } => assert_eq!(stored_fp, "SHA256:aaa"),
            v => panic!("{v:?}"),
        }
    }
}
```

- [ ] **Step 2: Run — FAIL.**
- [ ] **Step 3: Implement.** Create parent dir on `trust` if missing.
- [ ] **Step 4: Run — PASS.**
- [ ] **Step 5: Commit** — `git commit -m "feat(infra): file-backed TOFU host key store"`

---

## Task 13: `sm-infra` — secret stores

**Files:** Create `crates/sm-infra/src/secrets.rs`; Modify `lib.rs`; Test inline.

Add `sm-infra` dep: `keyring = "3"`.

**Interfaces:**
- Produces:
  - `pub struct InMemorySecretStore { map: Mutex<HashMap<(String, SecretKind-as-u8), String>> }` implementing `SecretStore` — used for `prompt` mode (session-lifetime) and in tests.
  - `pub struct KeyringSecretStore { service: String }` implementing `SecretStore` via `keyring::Entry::new(&self.service, &format!("{machine_id}:{kind:?}"))`. `service` = `"server-manager"`. On keyring backend failure, return `ServiceError::Other`.
  - `pub fn secret_key(kind: SecretKind) -> &'static str` helper for stable naming.

- [ ] **Step 1: Failing tests** (InMemory only — keyring needs a session bus, gate that test `#[ignore]`):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use sm_services::{SecretStore, SecretKind};

    #[test]
    fn in_memory_roundtrip() {
        let s = InMemorySecretStore::default();
        let id = sm_core::MachineId::from("nas");
        assert!(s.get(&id, SecretKind::SshPassword).is_none());
        s.set(&id, SecretKind::SshPassword, "hunter2").unwrap();
        assert_eq!(s.get(&id, SecretKind::SshPassword).as_deref(), Some("hunter2"));
        s.clear(&id, SecretKind::SshPassword).unwrap();
        assert!(s.get(&id, SecretKind::SshPassword).is_none());
    }
}
```

- [ ] **Step 2: Run — FAIL.**
- [ ] **Step 3: Implement both stores.**
- [ ] **Step 4: Run — PASS** (`cargo test -p sm-infra secrets`).
- [ ] **Step 5: Commit** — `git commit -m "feat(infra): in-memory and OS-keyring secret stores"`

---

## Task 14: `sm-infra` — `RusshRunner`

**Files:** Create `crates/sm-infra/src/ssh.rs`; Modify `lib.rs`. Unit test: one offline test for auth-method selection logic (pure). Integration test lives in Task 16.

Add `sm-infra` deps: `russh = "0.45"`, `russh-keys = "0.45"`, `async-trait = "0.1"` (only if russh handler needs it), `base64 = "0.22"`, `sha2 = "0.10"`.

**Interfaces:**
- Consumes: `sm-core::{Machine, SecretMode}`, `sm-services::{SshRunner, CommandOutput, ServiceError, SecretStore, HostKeyStore, HostKeyVerdict, SecretKind}`.
- Produces:
  - `pub struct RusshRunner<S: SecretStore, H: HostKeyStore> { secrets: Arc<S>, host_keys: Arc<H>, default_secret_mode: SecretMode, connect_timeout: Duration }`
  - `pub fn new(secrets: Arc<S>, host_keys: Arc<H>, default_secret_mode: SecretMode) -> Self`
  - implements `SshRunner`. `run`:
    1. resolve effective secret_mode (`machine.secret_mode` or default).
    2. TCP+SSH handshake to `os_host:ssh_port` with `connect_timeout`.
    3. in the russh `check_server_key` callback compute `SHA256:` fingerprint, ask `host_keys.verify`; `Unknown` → return `ServiceError::HostKeyUntrusted` (UI will prompt then call a separate `trust` + retry), `Changed` → `ServiceError::HostKeyChanged(host)`, `TrustedMatch` → proceed.
    4. auth: try in order that applies — agent (if `SSH_AUTH_SOCK` set and no explicit key), key file + passphrase (passphrase via secret store when mode≠prompt; caller supplies when prompt), password (from machine/secret store).
    5. open a channel, `exec` the command, collect stdout/stderr/exit code → `CommandOutput`.
  - `pub async fn trust_pending(&self, host: &str, port: u16) -> Result<(), ServiceError>` — re-handshake, capture fp, `host_keys.trust`. (Used by the "trust this host" UI action.)

- [ ] **Step 1: Failing unit test — auth plan selection**

Factor the pure decision into `pub(crate) fn auth_plan(machine: &Machine, mode: SecretMode, agent_available: bool) -> Vec<AuthStep>` where `AuthStep` ∈ `{Agent, Key{path,has_passphrase}, Password}`. Test:
```rust
#[test]
fn password_only_when_no_key_configured() {
    let mut m = test_machine();
    m.ssh_password = Some("p".into());
    let plan = auth_plan(&m, SecretMode::Plaintext, false);
    assert_eq!(plan, vec![AuthStep::Password]);
}
#[test]
fn agent_first_when_available_and_no_explicit_key() {
    let plan = auth_plan(&test_machine(), SecretMode::Keyring, true);
    assert_eq!(plan.first(), Some(&AuthStep::Agent));
}
```
(Represent the key path via a new optional `Machine` field? No — spec keeps schema tight. Decision: key path is `~/.ssh/id_*` autodiscovered OR a `key_path: Option<String>` machine field. **This is a spec gap — add `key_path: Option<String>` to `Machine` in Task 2's struct and the schema doc.** Update Task 2 interface + spec §4.2 before implementing.)

- [ ] **Step 2: Run — FAIL.**
- [ ] **Step 3: Implement `ssh.rs`.** Keep the russh handler minimal; log via `tracing`.
- [ ] **Step 4: Run unit test — PASS.** (`cargo test -p sm-infra ssh`)
- [ ] **Step 5: Commit** — `git commit -m "feat(infra): russh-based SSH runner with TOFU + auth plan"`

---

## Task 15: `sm-infra` — `SshStatsProbe` and `LinuxFileOpener`

**Files:** Create `crates/sm-infra/src/stats.rs`, `crates/sm-infra/src/files.rs`; Modify `lib.rs`.

**Interfaces:**
- `pub struct SshStatsProbe<R: SshRunner> { runner: Arc<R> }` implementing `StatsProbe`:
  - if `machine.stats_cmd` is `Some(c)` → run `c`, parse as `key=value` lines into `Stats` (`cpu_pct`, `mem_used`, `mem_total`, `disk_used`, `disk_total`, `uptime_secs`); missing keys → `ServiceError::Other`.
  - else → run `sm_core::stats::PROC_STATS_CMD`, feed stdout to `sm_core::parse_proc_stats`.
- `pub struct LinuxFileOpener;` implementing `FileOpener`:
  - `open_files`: spawn `xdg-open` with `smb://<share_or_host>`; `share_path` (strip leading `\\`, convert `\` → `/`) if set, else `os_host`.
  - `map_drive`: on Linux this is a no-op returning `ServiceError::Other("map drive is Windows-only")` — the UI hides the button on Linux, this is just a guard.
  - Factor the command construction into `pub(crate) fn smb_url(machine: &Machine) -> String` and unit-test that.

- [ ] **Step 1: Failing tests**

```rust
#[test]
fn smb_url_prefers_share_path() {
    let mut m = test_machine();
    m.share_path = Some(r"\\192.168.1.10\media".into());
    assert_eq!(smb_url(&m), "smb://192.168.1.10/media");
}
#[test]
fn smb_url_falls_back_to_host() {
    assert_eq!(smb_url(&test_machine()), "smb://host.example");
}
```
For `SshStatsProbe`, unit-test the `key=value` custom-command parser with a fake `SshRunner` returning a canned `CommandOutput`.

- [ ] **Step 2: Run — FAIL.**
- [ ] **Step 3: Implement both.**
- [ ] **Step 4: Run — PASS.**
- [ ] **Step 5: Commit** — `git commit -m "feat(infra): SSH stats probe and Linux file-manager opener"`

---

## Task 16: SSH integration tests (Docker)

**Files:** Create `tests/integration/docker-compose.yml`, `crates/sm-infra/tests/ssh_it.rs`. Modify `.github/workflows/ci.yml` (Task 25 adds the job; this task just makes the tests runnable locally).

**Interfaces:**
- Consumes: everything in `sm-infra`.
- Produces: `#[ignore]`-gated tests exercised via `just test-integration`.

- [ ] **Step 1: docker-compose with an sshd container**

`tests/integration/docker-compose.yml`:
```yaml
services:
  sshd:
    image: lscr.io/linuxserver/openssh-server:latest
    environment:
      - USER_NAME=tester
      - USER_PASSWORD=testpass
      - PASSWORD_ACCESS=true
      - SUDO_ACCESS=true
    ports:
      - "2222:2222"
```

- [ ] **Step 2: Write integration tests**

`crates/sm-infra/tests/ssh_it.rs`:
```rust
#![cfg(test)]
use std::sync::Arc;
use sm_core::{Machine, SecretMode};
use sm_infra::{RusshRunner, InMemorySecretStore, FileHostKeyStore, SshStatsProbe};
use sm_services::{SshRunner, StatsProbe, HostKeyStore};

fn machine() -> Machine { /* points at 127.0.0.1:2222, user tester, ssh_password testpass */ }

#[tokio::test]
#[ignore = "needs docker sshd from just test-integration"]
async fn runs_a_command_after_trusting_host() {
    let dir = tempfile::tempdir().unwrap();
    let hk = Arc::new(FileHostKeyStore::new(dir.path().join("known_hosts")));
    let secrets = Arc::new(InMemorySecretStore::default());
    let runner = RusshRunner::new(secrets, hk.clone(), SecretMode::Plaintext);
    // first attempt: untrusted
    let err = runner.run(&machine(), "echo hi").await.unwrap_err();
    assert!(matches!(err, sm_services::ServiceError::HostKeyUntrusted));
    runner.trust_pending("127.0.0.1", 2222).await.unwrap();
    let out = runner.run(&machine(), "echo hi").await.unwrap();
    assert_eq!(out.stdout.trim(), "hi");
    assert_eq!(out.code, 0);
}

#[tokio::test]
#[ignore]
async fn samples_proc_stats() {
    // trust, then SshStatsProbe.sample -> Stats with mem_total > 0, uptime_secs > 0
}
```

- [ ] **Step 3: Run** `just test-integration`
Expected: container starts, both tests PASS, container stops.

- [ ] **Step 4: Document** in `docs/visual-checklist.md`'s sibling — add a short `tests/integration/README.md` explaining `just test-integration`, the fixed port 2222, and that these never run in `just test`.

- [ ] **Step 5: Commit**

```bash
git add tests/integration crates/sm-infra/tests
git commit -m "test(infra): docker-backed SSH integration tests"
```

---

## Task 17: `src-tauri` — app skeleton, state, config & logging wiring

**Files:** Create `src-tauri/Cargo.toml`, `src-tauri/build.rs`, `src-tauri/tauri.conf.json`, `src-tauri/src/main.rs`, `src-tauri/src/state.rs`, `src-tauri/src/logging.rs`. Modify workspace `Cargo.toml` (re-add `src-tauri` to members).

Deps: `tauri = { version = "2", features = [] }`, `tauri-build = "2"` (build-dep), `sm-core`, `sm-services`, `sm-infra`, `tokio`, `serde`, `serde_json`, `directories = "5"`, `tracing`, `tracing-subscriber = { version = "0.3", features = ["fmt"] }`, `tracing-appender = "0.2"`.

**Interfaces:**
- Produces:
  - `pub struct Paths { pub dir: PathBuf, pub config: PathBuf, pub known_hosts: PathBuf, pub log: PathBuf }` + `pub fn resolve_paths() -> Paths` (via `ProjectDirs::from("", "", "server-manager")`, creating `dir`).
  - `pub struct AppState { pub cfg: RwLock<Config>, pub secrets_prompt: Arc<InMemorySecretStore>, pub paths: Paths, pub active: RwLock<Option<MachineId>>, /* handles set by poller */ }`
  - `pub fn load_config(paths: &Paths) -> (Config, Option<String>)` — reads file (or `Config::default()` if absent), routes through `sm_core::load_from_text`, performs the side effects: on `BackedUpAndReset`/`MigratedFrom` write the `.bak-<ts>` copy and the fresh/upgraded file; returns `(config, user_message)`.
  - `pub fn save_config(paths: &Paths, cfg: &Config) -> Result<(), String>` — validates then atomically writes (`config.toml.tmp` → rename).
  - `pub fn init_logging(paths: &Paths)` — `tracing_subscriber` writing to a non-rolling `latest.log` (truncate on start) plus stderr; level from `SM_LOG` env (default `info`).

- [ ] **Step 1: Failing test — config load side effects**

`src-tauri/tests/config_io.rs`:
```rust
#[test]
fn garbage_config_is_backed_up_and_reset() {
    let dir = tempfile::tempdir().unwrap();
    let paths = server_manager::state::Paths {
        dir: dir.path().into(),
        config: dir.path().join("config.toml"),
        known_hosts: dir.path().join("known_hosts"),
        log: dir.path().join("latest.log"),
    };
    std::fs::write(&paths.config, "== broken ==").unwrap();
    let (cfg, msg) = server_manager::state::load_config(&paths);
    assert_eq!(cfg.schema_version, 2);
    assert!(msg.unwrap().contains("valid TOML"));
    assert!(std::fs::read_dir(dir.path()).unwrap()
        .any(|e| e.unwrap().file_name().to_string_lossy().starts_with("config.toml.bak-")));
}
```
Expose a `lib.rs` from `src-tauri` (`server_manager`) so tests can call in; `main.rs` stays a thin `fn main`.

- [ ] **Step 2: Run — FAIL.** `cargo test -p server-manager`
- [ ] **Step 3: Implement `state.rs`, `logging.rs`, minimal `tauri.conf.json`** (product name, identifier `dev.czyz.servermanager`, `build.frontendDist = "../ui/dist"`, `build.devUrl = "http://localhost:5173"`, window 360×280 min 320×240, `title = "Server Manager"`).
- [ ] **Step 4: Run — PASS.**
- [ ] **Step 5: Commit** — `git commit -m "feat(tauri): app state, config load/save side effects, latest.log"`

---

## Task 18: `src-tauri` — commands

**Files:** Create `src-tauri/src/commands.rs`; Modify `main.rs` (register handler).

**Interfaces:**
- Consumes: `AppState`, all infra impls.
- Produces `#[tauri::command] async fn`s, all returning `Result<T, String>`:
  - `get_state() -> AppSnapshot` where `AppSnapshot { settings: Settings, machines: Vec<MachineSummary>, active: Option<String>, notice: Option<String> }`, `MachineSummary { id, name }`.
  - `set_active(id: String)`.
  - `get_machine(id: String) -> Machine` (secrets blanked out).
  - `upsert_machine(machine: Machine) -> Result<(), String>` — validates whole config, persists.
  - `delete_machine(id: String)`.
  - `save_settings(settings: Settings)`.
  - `wake(id: String)` — parse mac, resolve `broadcast_addr` (machine or `255.255.255.255`), `UdpWaker.wake`.
  - `refresh_now(id: String) -> bool` — one immediate `TcpStatusProbe.is_up`, also resets that machine's poller backoff.
  - `power(id: String, action: String)` — `resolve_power`, `SshRunner.run(primary)`; if `needs_sudo` and `sudo_fallback` present, feed the sudo password to stdin and retry; map `HostKeyUntrusted`/`HostKeyChanged` to structured error strings the UI recognises (`"HOSTKEY_UNTRUSTED"`, `"HOSTKEY_CHANGED"`).
  - `trust_host(id: String)` — `RusshRunner.trust_pending`.
  - `provide_secret(id: String, kind: String, value: String, remember: bool)` — writes to `InMemorySecretStore` always; if `remember` and effective mode is `keyring`, also `KeyringSecretStore.set`.
  - `open_files(id: String)` — `LinuxFileOpener.open_files`.
  - `sample_stats(id: String) -> Stats` — on-demand stats (poller also pushes them; this backs a manual refresh).

- [ ] **Step 1: Failing test** — `src-tauri/tests/commands.rs` using `tauri::test::mock_builder` + `mock_context`. Test `get_state` returns defaults on a fresh temp config; test `upsert_machine` then `get_machine` returns it with `ssh_password == None`.

- [ ] **Step 2: Run — FAIL.**
- [ ] **Step 3: Implement `commands.rs`; register all in `main.rs` `invoke_handler`.**
- [ ] **Step 4: Run — PASS.**
- [ ] **Step 5: Commit** — `git commit -m "feat(tauri): invoke commands for config, wake, power, stats, secrets"`

---

## Task 19: `src-tauri` — background poller

**Files:** Create `src-tauri/src/poller.rs`; Modify `main.rs` (spawn on setup).

**Interfaces:**
- Produces:
  - `pub fn spawn(app: AppHandle, state: Arc<AppState>)` — starts one tokio task that, every tick, for the **active** machine only (spec: one machine at a time): runs `TcpStatusProbe.is_up`; updates a per-machine `BackoffState`; emits Tauri event `status://update` with `{ id, status: "online"|"offline", next_poll_secs }`. When online, also every Nth tick sample stats and emit `stats://update` with `{ id, stats }` (or `{ id, error }`).
  - Reacts to `set_active`/`refresh_now` via a `tokio::sync::Notify` or an mpsc command channel so switching machines re-polls immediately.
- Event names are also declared in `ui/src/api.js` (Task 21).

- [ ] **Step 1: Failing test** — extract the tick decision into pure `fn plan_tick(online: bool, tick: u64, backoff: &mut BackoffState) -> TickPlan { emit_status: bool, sample_stats: bool, sleep_secs: u64 }` and unit-test: offline ⇒ `sleep_secs` follows backoff; online tick%6==0 ⇒ `sample_stats`.
- [ ] **Step 2: Run — FAIL.**
- [ ] **Step 3: Implement `poller.rs`.**
- [ ] **Step 4: Run — PASS.**
- [ ] **Step 5: Commit** — `git commit -m "feat(tauri): active-machine status/stats poller with backoff"`

---

## Task 20: `ui` — Vite project + Tauri dev integration

**Files:** Create `ui/package.json`, `ui/vite.config.js`, `ui/index.html`, `ui/src/main.js`, `ui/src/api.js`, `ui/src/styles/theme.css`, `ui/src/styles/app.css`.

**Interfaces:**
- Produces:
  - `ui/src/api.js`: `export async function getState()`, `setActive(id)`, `wake(id)`, `power(id, action)`, `refreshNow(id)`, `provideSecret(...)`, `openFiles(id)`, `trustHost(id)`, `getMachine(id)`, `upsertMachine(m)`, `deleteMachine(id)`, `saveSettings(s)`, `sampleStats(id)` — thin wrappers over `@tauri-apps/api/core` `invoke`.
  - `export function onStatus(cb)`, `onStats(cb)` — wrappers over `@tauri-apps/api/event` `listen` for `status://update` / `stats://update`.
- `package.json` deps: `@tauri-apps/api@^2`; devDeps: `vite@^5`, `@tauri-apps/cli@^2`.
- `vite.config.js`: `server.port = 5173`, `server.strictPort = true`, `build.outDir = "dist"`, `build.target = "es2022"`.

- [ ] **Step 1: Scaffold the files.** `index.html` loads `/src/main.js` as a module and contains `<sm-app></sm-app>`.
- [ ] **Step 2: `main.js`** registers components (Task 21) and mounts nothing else — the `<sm-app>` element owns layout.
- [ ] **Step 3: Verify dev server** — `cd ui && npm install && npm run build`
Expected: `ui/dist/index.html` produced, no errors.
- [ ] **Step 4: Verify the app launches** — from repo root `cargo tauri dev` (needs `just setup` first for `tauri-cli`); a 360×280 window opens showing an empty shell.
- [ ] **Step 5: Commit** — `git commit -m "feat(ui): vite project, tauri api wrappers, app shell"`

---

## Task 21: `ui` — components

**Files:** Create `ui/src/components/{machine-selector,status-line,action-buttons,stats-panel,settings-panel,toast-host}.js` and an `sm-app.js` container.

Each component is one `class extends HTMLElement`, uses a `<template>` cloned into a shadow root, exposes state via attributes/properties, emits `CustomEvent`s upward. No shared framework.

**Interfaces (per component):**
- `<sm-app>` — owns layout (vertical stack from spec §9.1), holds current `AppSnapshot` + latest status/stats, subscribes via `onStatus`/`onStats`, routes child events to `api.js`. Applies theme (Task 22).
- `<machine-selector machines active>` — `<select>`; emits `machine-change {id}`.
- `<status-line status>` — shows `ONLINE`/`OFFLINE`/`—` with the spec's green/red; `aria-live="polite"`.
- `<action-buttons status platform>` — Wake / Shutdown / Reboot / Open Files / Map Drive; disables per rules (Wake enabled when offline; Shutdown/Reboot when online; Open Files/Map Drive hidden unless `platform === "linux"|"windows"`, Map Drive only `windows`). Emits `action {name}`.
- `<stats-panel stats display>` — numbers or sparkline (`display` from settings); sparkline is a tiny inline `<svg>` polyline from an in-memory ring buffer (last 60 samples). "unavailable" state when `stats` carries an error.
- `<settings-panel open settings machines>` — slide-out (CSS transform, chevron toggle button); forms for global settings + machine CRUD + per-machine secret mode; emits `settings-save`, `machine-save`, `machine-delete`, `close`.
- `<toast-host>` — `show(message, kind)`; used for the migration notice, host-key prompts, "share path copied", errors.

- [ ] **Step 1: Build `sm-app` + `machine-selector` + `status-line`** end to end; `cargo tauri dev`, confirm selecting a machine emits the change and the status line updates from a live `status://update` event (add a machine by hand to `config.toml` first, or via settings once Task 21 step 3 lands).
- [ ] **Step 2: Build `action-buttons`**; wire Wake and Open Files; manually verify Wake sends a packet (`sudo tcpdump -ni any udp port 9` on the host).
- [ ] **Step 3: Build `settings-panel`**; machine CRUD round-trips through `upsert_machine`/`delete_machine`; slide-out animates.
- [ ] **Step 4: Build `stats-panel` + `toast-host`**; stats render both as numbers and as sparkline; toast shows on a forced error.
- [ ] **Step 5: Host-key + secret-prompt flow** — when `power`/`sample_stats` returns `HOSTKEY_UNTRUSTED`, `<toast-host>` (or a small modal) shows the fingerprint with Trust/Cancel → `trustHost(id)` then retry; on `SECRET_REQUIRED:<kind>` show a password field → `provideSecret`.
- [ ] **Step 6: Commit** — `git commit -m "feat(ui): machine selector, status, actions, stats, settings, toasts"`

(Split into 2 commits if the reviewer prefers — components 1–3 then 4–6.)

---

## Task 22: `ui` — i18n + theming

**Files:** Create `ui/src/i18n/{index.js,en.json,pl.json}`; Modify `ui/src/styles/theme.css`, `sm-app.js`, `settings-panel.js`.

**Interfaces:**
- `i18n/index.js`: `export function setLocale(code)`, `export function t(key, vars)` — loads `en.json`/`pl.json` (static imports), falls back to `en` then to the key itself. Every user-facing string in components goes through `t()`.
- `en.json`/`pl.json`: flat key→string maps. Keys: `status.online`, `status.offline`, `action.wake`, `action.shutdown`, `action.reboot`, `action.openFiles`, `action.mapDrive`, `settings.title`, `settings.language`, `settings.theme`, `settings.statsDisplay`, `settings.pollBaseSecs`, `settings.secretMode`, `settings.mountProtocol`, `machine.name`, `machine.host`, `machine.mac`, `machine.sshUser`, `machine.sshPort`, `toast.shareCopied`, `toast.migrated`, `hostkey.prompt`, `hostkey.changed`, `secret.required`, `stats.unavailable`, `stats.cpu`, `stats.mem`, `stats.disk`, `stats.uptime`, plus each `ServiceError` variant as `error.*`.
- `theme.css`: `:root` light vars + `@media (prefers-color-scheme: dark)` + `[data-theme="dark"]`/`[data-theme="light"]` overrides. `sm-app` sets `data-theme` on `document.documentElement` from `settings.theme` (`system` → remove attribute).

- [ ] **Step 1: Write `i18n/index.js` + a unit check** — a plain `ui/src/i18n/i18n.test.mjs` run with `node --test`: `t("status.online")` in `pl` returns the Polish string; unknown key returns the key. Add `"test": "node --test src/**/*.test.mjs"` to `package.json` (this is the one JS test the plan allows — the i18n fallback is real logic).
- [ ] **Step 2: Run — FAIL, then implement, then PASS.**
- [ ] **Step 3: Route all component strings through `t()`; wire theme switching.** Manually verify PL + dark.
- [ ] **Step 4: Commit** — `git commit -m "feat(ui): English/Polish i18n and light/dark theming"`

---

## Task 23: Visual checklist + manual QA pass

**Files:** Create `docs/visual-checklist.md`.

- [ ] **Step 1: Write the checklist** — a table the releaser walks per release. Rows: window opens at 360×280 and won't shrink below 320×240; machine dropdown lists all config machines; status flips ONLINE/OFFLINE within `poll_base_secs` of the box going up/down; offline machine's poll interval visibly backs off; Wake sends packets (tcpdump); Shutdown then Reboot work on a real Linux box; sudo-password path works when passwordless sudo is absent; host-key prompt appears on first connect and trust persists across restart; host-key-changed warning blocks; stats show sane CPU/mem/disk/uptime, both display modes; Open Files opens the file manager at `smb://`; settings slide-out toggles; language switch to PL translates everything; dark/light both legible; malformed `config.toml` → backup + toast + app still starts.
- [ ] **Step 2: Do one full pass** against a real machine; file bugs as follow-up tasks; fix anything trivial inline.
- [ ] **Step 3: Commit** — `git commit -m "docs: manual visual QA checklist for releases"`

---

## Task 24: Linux packaging

**Files:** Create `packaging/build-pkgbuild.sh`, `packaging/PKGBUILD.template`, `packaging/server-manager.desktop`; Modify `src-tauri/tauri.conf.json` (bundle config), `justfile`, `README.md`.

**Interfaces:**
- `tauri.conf.json` `bundle`: `active = true`, `targets = ["appimage", "rpm"]`, `category = "Utility"`, icons, `linux.rpm.depends` as needed (webkit2gtk).
- `packaging/build-pkgbuild.sh`: after `cargo tauri build`, generate a `PKGBUILD` from the template with the version + the built binary, run `makepkg -f` if available else just emit the `PKGBUILD` + tarball and print a note.

- [ ] **Step 1: Configure the AppImage + rpm bundle**; `cargo tauri build --bundles appimage,rpm`
Expected: `src-tauri/target/release/bundle/appimage/*.AppImage` and `.../rpm/*.rpm` produced.
- [ ] **Step 2: AppImage smoke test** — run the AppImage on the dev machine; window opens, a wake works.
- [ ] **Step 3: rpm install test** — `sudo dnf install ./*.rpm` in a Fedora container or the host; launches; `sudo dnf remove` clean.
- [ ] **Step 4: PKGBUILD** — `packaging/build-pkgbuild.sh` produces a valid `PKGBUILD`; if `makepkg` present, `namcap` it.
- [ ] **Step 5: README build section** — exact commands for all three Linux artifacts + the Android note (deferred to M2) + Windows note (deferred to M3).
- [ ] **Step 6: Commit** — `git commit -m "build: AppImage, rpm, and PKGBUILD packaging for Linux"`

---

## Task 25: CI

**Files:** Create `.github/workflows/ci.yml`.

**Interfaces:**
- Jobs:
  - `lint-test` (ubuntu): `just lint`, `just test`. Caches cargo + `ui/node_modules`. Installs system deps for tauri (`libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, etc.).
  - `integration` (ubuntu, `needs: lint-test`): `docker compose … up --wait`, `cargo test -p sm-infra --test ssh_it -- --ignored --test-threads 1`.
  - `release-linux` (ubuntu, on tag `v2.*`): `just build-linux`, upload AppImage + rpm + PKGBUILD tarball to a GitHub Release.

- [ ] **Step 1: Write `ci.yml`** with the three jobs, pinned action SHAs or `@vN`.
- [ ] **Step 2: Push the branch, open a draft PR from `v2`** (base `master`), confirm `lint-test` + `integration` go green.
- [ ] **Step 3: Tag a `v2.0.0-rc.1` prerelease** on a scratch push to confirm `release-linux` uploads artifacts; delete the test release after.
- [ ] **Step 4: Commit** — `git commit -m "ci: lint/test, docker integration, and linux release workflow"`

---

## Self-Review

**Spec coverage:**

| Spec section | Task(s) |
|---|---|
| §2 stack (Tauri/Rust/plain-JS/russh/tokio) | 1, 14, 17, 20 |
| §3 architecture (core/services/infra/tauri/ui) | 1–22 |
| §4.1 config location | 17 |
| §4.2 schema (incl. new `key_path`) | 2, 3, 14 |
| §4.3 malformed/old config | 4, 17 |
| §5 Wake-on-LAN | 6, 10, 18 |
| §6 status detection + smart backoff | 5, 11, 19 |
| §7.1 auth + secret modes | 9, 13, 14, 18, 21 |
| §7.2 TOFU host keys | 12, 14, 21 |
| §7.3 power + sudo fallback | 8, 18 |
| §7.4 stats (`/proc` + `stats_cmd`) | 7, 15 |
| §8 Open Files (Linux); Map Drive (guarded, M3) | 15, 21 |
| §9.1 layout + resizable window + slide-out settings | 17, 21 |
| §9.2 theming | 22 |
| §9.3 EN/PL i18n | 22 |
| §9.4 latest.log | 17 |
| §10 build/packaging + justfile | 1, 24, 25 |
| §11 testing (unit/integration/visual checklist) | every task; 16, 23 |
| §12 M1 scope | whole plan |

**Gaps found & resolved during review:**
- SSH key file path had nowhere to live → added `key_path: Option<String>` to `Machine` (Task 2 interface + Task 14 note) and it must be added to spec §4.2.
- The batched stats command needed explicit delimiters for reliable parsing → defined `PROC_STATS_CMD` with `---SM-*` sentinels in Task 7 (a refinement of spec §7.4's illustrative command; update the spec to match).
- Structured error strings between backend and UI (`HOSTKEY_UNTRUSTED`, `SECRET_REQUIRED:<kind>`) weren't in the spec → defined in Tasks 18 & 21.

**Placeholder scan:** no `TODO`/`TBD`/"handle errors appropriately" remain; every code step has real code or a real test.

**Type consistency:** `MachineId`, `Machine`, `Stats`, `ServiceError`, `CommandOutput`, `HostKeyVerdict`, `BackoffState`, `PowerPlan` names are used identically across Tasks 2–22. Event names `status://update` / `stats://update` match between Task 19 and Tasks 20–21.

**Spec edits required before execution (small):**
1. §4.2 — add `key_path` (optional) to the machine schema + example.
2. §7.4 — replace the illustrative command with `PROC_STATS_CMD` and note the `---SM-*` sentinel contract.
3. §7.1 — mention the backend↔frontend structured error codes.
