//! Thin wrappers over the Tauri IPC commands (src-tauri/src/commands.rs) and
//! the poller events (src-tauri/src/poller.rs).
//!
//! Invoke arg keys match the Rust command parameters exactly (all single
//! words: `id`, `machine`, `settings`, `action`, `kind`, `value`, `remember`).
//! Errors are plain strings; the structured ones the UI must recognise are
//! `HOSTKEY_UNTRUSTED`, `HOSTKEY_CHANGED` and `SECRET_REQUIRED:<kind>`.

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

/**
 * Full app snapshot: { settings, machines: [{id, name}], active, notice }.
 * @returns {Promise<object>}
 */
export const getState = () => invoke("get_state");

/** @param {string} id */
export const setActive = (id) => invoke("set_active", { id });

/** @param {string} id */
export const wake = (id) => invoke("wake", { id });

/**
 * @param {string} id
 * @param {"shutdown"|"reboot"} action
 */
export const power = (id, action) => invoke("power", { id, action });

/** Manual reachability probe; resolves true when online.
 * @param {string} id @returns {Promise<boolean>}
 */
export const refreshNow = (id) => invoke("refresh_now", { id });

/**
 * Store a secret in the session store (and machine config when plaintext
 * mode + `remember`).
 * @param {string} id
 * @param {"SshPassword"|"KeyPassphrase"|"SudoPassword"} kind
 * @param {string} value
 * @param {boolean} remember
 */
export const provideSecret = (id, kind, value, remember) =>
  invoke("provide_secret", { id, kind, value, remember });

/** Open the file manager at the machine's share.
 * @param {string} id
 */
export const openFiles = (id) => invoke("open_files", { id });

/** Trust the machine's pending SSH host key.
 * @param {string} id
 */
export const trustHost = (id) => invoke("trust_host", { id });

/** Fingerprint the server presented on the last host-key rejection.
 * @param {string} id @returns {Promise<string|null>}
 */
export const pendingHostKey = (id) => invoke("pending_host_key", { id });

/** Machine with all secret fields blanked.
 * @param {string} id @returns {Promise<object>}
 */
export const getMachine = (id) => invoke("get_machine", { id });

/** Insert or replace a machine (matched by `machine.id`).
 * @param {object} machine
 */
export const upsertMachine = (machine) => invoke("upsert_machine", { machine });

/** @param {string} id */
export const deleteMachine = (id) => invoke("delete_machine", { id });

/** @param {object} settings */
export const saveSettings = (settings) => invoke("save_settings", { settings });

/** One-shot stats sample (CPU/mem/disk/uptime).
 * @param {string} id @returns {Promise<object>}
 */
export const sampleStats = (id) => invoke("sample_stats", { id });

/**
 * Subscribe to the poller's status event
 * (`{id, status: "online"|"offline", next_poll_secs}`). Unlisten is returned.
 * @param {(payload: object) => void} cb
 * @returns {Promise<() => void>} unlisten
 */
export const onStatus = (cb) =>
  listen("status://update", (e) => cb(e.payload));

/**
 * Subscribe to the poller's stats event
 * (`{id, stats}` on success, `{id, error}` on failure). Unlisten is returned.
 * @param {(payload: object) => void} cb
 * @returns {Promise<() => void>} unlisten
 */
export const onStats = (cb) => listen("stats://update", (e) => cb(e.payload));