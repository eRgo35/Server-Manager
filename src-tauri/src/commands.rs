//! Tauri invoke commands (Task 18).
//!
//! Every command is a thin `#[tauri::command]` wrapper over an `*_inner`
//! async fn taking `&AppState`, keeping the behaviour testable without the
//! tauri runtime. All commands return `Result<T, String>`; structured error
//! strings the UI recognises:
//!
//! - `"HOSTKEY_UNTRUSTED"` / `"HOSTKEY_CHANGED"` (from `ServiceError`)
//! - `"SECRET_REQUIRED:<kind>"`, e.g. `SECRET_REQUIRED:SshPassword`, when a
//!   prompt-mode lookup is needed but neither the machine nor the session
//!   store holds the secret.

use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sm_core::{
    Config, Machine, MachineId, PowerAction, SecretMode, Settings, Stats, needs_sudo, parse_mac,
    resolve_power,
};
use sm_services::{
    FileOpener, SecretKind, SecretStore, ServiceError, SshRunner, StatsProbe, StatusProbe, Waker,
};

use crate::state::{self, AppState};

/// Timeout of the one-shot `refresh_now` TCP probe.
const PROBE_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Deserialize, Serialize)]
pub struct MachineSummary {
    pub id: String,
    pub name: String,
}

/// Also `Deserialize` so tests can decode an IPC response payload.
#[derive(Debug, Deserialize, Serialize)]
pub struct AppSnapshot {
    pub settings: Settings,
    pub machines: Vec<MachineSummary>,
    pub active: Option<String>,
    pub notice: Option<String>,
}

// ---------- helpers ----------

/// Maps a `ServiceError` to the structured error-string contract.
fn err_string(e: ServiceError) -> String {
    match e {
        ServiceError::HostKeyUntrusted => "HOSTKEY_UNTRUSTED".into(),
        ServiceError::HostKeyChanged(_) => "HOSTKEY_CHANGED".into(),
        e => e.to_string(),
    }
}

fn find_machine(state: &AppState, id: &str) -> Result<Machine, String> {
    state
        .cfg
        .read()
        .unwrap()
        .machine(&MachineId(id.to_owned()))
        .cloned()
        .ok_or_else(|| format!("unknown machine id: {id}"))
}

fn effective_mode(state: &AppState, machine: &Machine) -> SecretMode {
    machine
        .secret_mode
        .unwrap_or(state.cfg.read().unwrap().settings.default_secret_mode)
}

/// The `SECRET_REQUIRED:SshPassword` check: password auth is planned only
/// when the machine holds a plaintext password or the effective mode allows
/// the store (prompt; keyring defers to prompt in M1). If neither has one,
/// the UI must prompt before the SSH call is attempted.
fn require_ssh_password(state: &AppState, machine: &Machine) -> Result<(), String> {
    if machine.key_path.is_some() || machine.ssh_password.is_some() {
        return Ok(());
    }
    let mode = effective_mode(state, machine);
    if matches!(mode, SecretMode::Keyring | SecretMode::Prompt)
        && state
            .secrets_prompt
            .get(&machine.id, SecretKind::SshPassword)
            .is_none()
    {
        return Err("SECRET_REQUIRED:SshPassword".into());
    }
    Ok(())
}

fn blank_secrets(mut machine: Machine) -> Machine {
    machine.ssh_password = None;
    machine.key_passphrase = None;
    machine.sudo_password = None;
    machine
}

fn parse_secret_kind(s: &str) -> Option<SecretKind> {
    match s.to_ascii_lowercase().as_str() {
        "sshpassword" | "ssh_password" => Some(SecretKind::SshPassword),
        "keypassphrase" | "key_passphrase" => Some(SecretKind::KeyPassphrase),
        "sudopassword" | "sudo_password" => Some(SecretKind::SudoPassword),
        _ => None,
    }
}

/// Validates then persists `cfg`, committing it to the shared state only on
/// success (so a rejected upsert leaves the in-memory config untouched).
fn commit(state: &AppState, cfg: Config) -> Result<(), String> {
    state::save_config(&state.paths, &cfg)?;
    *state.cfg.write().unwrap() = cfg;
    Ok(())
}

// ---------- command implementations ----------

pub async fn get_state_inner(state: &AppState) -> Result<AppSnapshot, String> {
    let cfg = state.cfg.read().unwrap();
    Ok(AppSnapshot {
        settings: cfg.settings.clone(),
        machines: cfg
            .machines
            .iter()
            .map(|m| MachineSummary {
                id: m.id.0.clone(),
                name: m.name.clone(),
            })
            .collect(),
        active: state.active.read().unwrap().as_ref().map(|id| id.0.clone()),
        notice: state.notice.read().unwrap().clone(),
    })
}

pub async fn set_active_inner(state: &AppState, id: String) -> Result<(), String> {
    find_machine(state, &id)?;
    *state.active.write().unwrap() = Some(MachineId(id));
    Ok(())
}

/// Returns the machine with all secret fields blanked (the UI never reads
/// secrets back; provide them via `provide_secret`).
pub async fn get_machine_inner(state: &AppState, id: String) -> Result<Machine, String> {
    Ok(blank_secrets(find_machine(state, &id)?))
}

pub async fn upsert_machine_inner(state: &AppState, machine: Machine) -> Result<(), String> {
    let mut cfg = state.cfg.read().unwrap().clone();
    match cfg.machines.iter_mut().find(|m| m.id == machine.id) {
        Some(existing) => *existing = machine,
        None => cfg.machines.push(machine),
    }
    commit(state, cfg)
}

pub async fn delete_machine_inner(state: &AppState, id: String) -> Result<(), String> {
    let mid = MachineId(id.clone());
    let mut cfg = state.cfg.read().unwrap().clone();
    let before = cfg.machines.len();
    cfg.machines.retain(|m| m.id != mid);
    if cfg.machines.len() == before {
        return Err(format!("unknown machine id: {id}"));
    }
    commit(state, cfg)?;
    for kind in [
        SecretKind::SshPassword,
        SecretKind::KeyPassphrase,
        SecretKind::SudoPassword,
    ] {
        let _ = state.secrets_prompt.clear(&mid, kind);
    }
    Ok(())
}

pub async fn save_settings_inner(state: &AppState, settings: Settings) -> Result<(), String> {
    let mut cfg = state.cfg.read().unwrap().clone();
    cfg.settings = settings;
    commit(state, cfg)
}

pub async fn wake_inner(state: &AppState, id: String) -> Result<(), String> {
    let machine = find_machine(state, &id)?;
    let mac = parse_mac(&machine.mac).map_err(|e| e.to_string())?;
    let broadcast = machine
        .broadcast_addr
        .as_deref()
        .unwrap_or("255.255.255.255");
    state.waker.wake(mac, broadcast).await.map_err(err_string)
}

/// One immediate reachability probe. T19 (poller): must also reset the
/// machine's poller backoff; the poller does not exist yet.
pub async fn refresh_now_inner(state: &AppState, id: String) -> Result<bool, String> {
    let machine = find_machine(state, &id)?;
    Ok(state
        .probe
        .is_up(&machine.os_host, machine.ssh_port, PROBE_TIMEOUT)
        .await)
}

pub async fn power_inner(state: &AppState, id: String, action: String) -> Result<(), String> {
    let machine = find_machine(state, &id)?;
    let action = match action.as_str() {
        "shutdown" => PowerAction::Shutdown,
        "reboot" => PowerAction::Reboot,
        other => return Err(format!("unknown power action: {other}")),
    };
    require_ssh_password(state, &machine)?;
    let plan = resolve_power(&machine, action);
    match state.runner.run(&machine, &plan.primary).await {
        Ok(_) => Ok(()),
        Err(e) => {
            let needs = match &e {
                ServiceError::Remote { code, stderr } => needs_sudo(*code, stderr),
                _ => false,
            };
            // The runner cannot feed stdin, so `sudo -S` gets its password
            // piped through the remote shell (single-quote escaped).
            match (needs, plan.sudo_fallback) {
                (true, Some(fallback)) => {
                    let pw = machine.sudo_password.clone().unwrap_or_default();
                    let quoted = pw.replace('\'', "'\\''");
                    let cmd = format!("printf '%s\\n' '{quoted}' | {fallback}");
                    state
                        .runner
                        .run(&machine, &cmd)
                        .await
                        .map(|_| ())
                        .map_err(err_string)
                }
                _ => Err(err_string(e)),
            }
        }
    }
}

pub async fn trust_host_inner(state: &AppState, id: String) -> Result<(), String> {
    let machine = find_machine(state, &id)?;
    state
        .runner
        .trust_pending(&machine.os_host, machine.ssh_port)
        .await
        .map_err(err_string)
}

/// Always stores in the session store. With `remember`, plaintext mode
/// persists to the machine's config fields; keyring mode is treated as
/// prompt (M1 ruling 2026-09-11: keyring is deferred), so the value stays
/// in-memory for the session only.
pub async fn provide_secret_inner(
    state: &AppState,
    id: String,
    kind: String,
    value: String,
    remember: bool,
) -> Result<(), String> {
    let machine = find_machine(state, &id)?;
    let kind = parse_secret_kind(&kind).ok_or_else(|| format!("unknown secret kind: {kind}"))?;
    state
        .secrets_prompt
        .set(&machine.id, kind, &value)
        .map_err(err_string)?;

    if remember && effective_mode(state, &machine) == SecretMode::Plaintext {
        let mut cfg = state.cfg.read().unwrap().clone();
        if let Some(m) = cfg.machines.iter_mut().find(|m| m.id == machine.id) {
            match kind {
                SecretKind::SshPassword => m.ssh_password = Some(value),
                SecretKind::KeyPassphrase => m.key_passphrase = Some(value),
                SecretKind::SudoPassword => m.sudo_password = Some(value),
            }
        }
        commit(state, cfg)?;
    }
    Ok(())
}

pub async fn open_files_inner(state: &AppState, id: String) -> Result<(), String> {
    let machine = find_machine(state, &id)?;
    state.opener.open_files(&machine).map_err(err_string)
}

pub async fn sample_stats_inner(state: &AppState, id: String) -> Result<Stats, String> {
    let machine = find_machine(state, &id)?;
    require_ssh_password(state, &machine)?;
    state.stats.sample(&machine).await.map_err(err_string)
}

// ---------- tauri command wrappers ----------

#[tauri::command]
pub async fn get_state(state: tauri::State<'_, Arc<AppState>>) -> Result<AppSnapshot, String> {
    get_state_inner(&state).await
}

#[tauri::command]
pub async fn set_active(state: tauri::State<'_, Arc<AppState>>, id: String) -> Result<(), String> {
    set_active_inner(&state, id).await
}

#[tauri::command]
pub async fn get_machine(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<Machine, String> {
    get_machine_inner(&state, id).await
}

#[tauri::command]
pub async fn upsert_machine(
    state: tauri::State<'_, Arc<AppState>>,
    machine: Machine,
) -> Result<(), String> {
    upsert_machine_inner(&state, machine).await
}

#[tauri::command]
pub async fn delete_machine(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<(), String> {
    delete_machine_inner(&state, id).await
}

#[tauri::command]
pub async fn save_settings(
    state: tauri::State<'_, Arc<AppState>>,
    settings: Settings,
) -> Result<(), String> {
    save_settings_inner(&state, settings).await
}

#[tauri::command]
pub async fn wake(state: tauri::State<'_, Arc<AppState>>, id: String) -> Result<(), String> {
    wake_inner(&state, id).await
}

#[tauri::command]
pub async fn refresh_now(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<bool, String> {
    refresh_now_inner(&state, id).await
}

#[tauri::command]
pub async fn power(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
    action: String,
) -> Result<(), String> {
    power_inner(&state, id, action).await
}

#[tauri::command]
pub async fn trust_host(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<(), String> {
    trust_host_inner(&state, id).await
}

#[tauri::command]
pub async fn provide_secret(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
    kind: String,
    value: String,
    remember: bool,
) -> Result<(), String> {
    provide_secret_inner(&state, id, kind, value, remember).await
}

#[tauri::command]
pub async fn open_files(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<(), String> {
    open_files_inner(&state, id).await
}

#[tauri::command]
pub async fn sample_stats(
    state: tauri::State<'_, Arc<AppState>>,
    id: String,
) -> Result<Stats, String> {
    sample_stats_inner(&state, id).await
}