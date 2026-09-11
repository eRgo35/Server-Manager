//! `<sm-app>` — the application shell.
//!
//! Vertical stack per spec §9.1: machine selector, status line, action
//! buttons, stats panel, slide-out settings (chevron tab), toasts. Holds the
//! `AppSnapshot` from `get_state` plus the latest status/stats for the
//! active machine, subscribes to the poller's `status://update` /
//! `stats://update` events, and routes every child event to `api.js`
//! (components themselves never invoke IPC).
//!
//! Structured-error retry flow (spec §3, commands.rs contract):
//! - `HOSTKEY_UNTRUSTED` / `HOSTKEY_CHANGED` → modal (Trust / Cancel) →
//!   `trustHost(id)` → retry the failed action. NOTE: the captured host-key
//!   fingerprint is not exposed over IPC in M1 (commands.rs `err_string`
//!   drops it), so the modal cannot show the fingerprint yet.
//! - `SECRET_REQUIRED:<kind>` → modal password prompt → `provideSecret(id,
//!   kind, value, remember)` → retry the failed action.

import {
  getMachine,
  getState,
  onStatus,
  onStats,
  openFiles,
  power,
  provideSecret,
  saveSettings,
  setActive,
  trustHost,
  upsertMachine,
  deleteMachine,
  wake,
} from "../api.js";
import "./machine-selector.js";
import "./status-line.js";
import "./action-buttons.js";
import "./stats-panel.js";
import "./settings-panel.js";
import "./toast-host.js";

const SECRET_LABELS = {
  SshPassword: "SSH password",
  KeyPassphrase: "key passphrase",
  SudoPassword: "sudo password",
};

const tpl = document.createElement("template");
tpl.innerHTML = `
  <style>
    :host {
      display: block;
      height: 100%;
    }
    .col {
      display: flex;
      flex-direction: column;
      gap: 8px;
      height: 100%;
    }
  </style>
  <div class="col">
    <sm-machine-selector></sm-machine-selector>
    <sm-status-line></sm-status-line>
    <sm-action-buttons></sm-action-buttons>
    <sm-stats-panel></sm-stats-panel>
  </div>
  <sm-settings-panel></sm-settings-panel>
  <sm-toast-host></sm-toast-host>
`;

export class SmApp extends HTMLElement {
  // AppSnapshot from get_state: {settings, machines, active, notice}
  #snapshot = null;
  #status = "unknown";
  #platform = "desktop-linux";
  #lastNotice = null;
  #unlisteners = [];

  constructor() {
    super();
    this.attachShadow({ mode: "open" }).appendChild(tpl.content.cloneNode(true));
    const $ = (sel) => this.shadowRoot.querySelector(sel);
    this.$selector = $("sm-machine-selector");
    this.$status = $("sm-status-line");
    this.$actions = $("sm-action-buttons");
    this.$stats = $("sm-stats-panel");
    this.$settings = $("sm-settings-panel");
    this.$toast = $("sm-toast-host");
    this.#wire();
  }

  #wire() {
    this.$selector.addEventListener("machine-change", (e) => {
      void this.#selectMachine(e.detail.id);
    });

    this.$actions.addEventListener("action", (e) => {
      void this.#runAction(e.detail.name);
    });

    this.$settings.addEventListener("settings-save", (e) => {
      void this.#saveGlobalSettings(e.detail.settings);
    });
    this.$settings.addEventListener("machine-save", (e) => {
      void this.#saveMachine(e.detail.machine);
    });
    this.$settings.addEventListener("machine-delete", (e) => {
      void this.#deleteMachine(e.detail.id);
    });
    this.$settings.addEventListener("machine-open", (e) => {
      void this.#openMachine(e.detail.id);
    });
    // The panel manages its own open state; `close` is informational.
    this.$settings.addEventListener("close", () => {});
  }

  async #selectMachine(id) {
    if (!id) return;
    try {
      await setActive(id);
    } catch (e) {
      this.#toast(String(e), "error");
      return;
    }
    if (this.#snapshot) this.#snapshot.active = id;
    this.$selector.active = id;
    this.#resetMachineState();
  }

  /** Clears per-machine state when the active machine changes. */
  #resetMachineState() {
    this.#status = "unknown";
    this.$status.status = "unknown";
    this.$actions.status = "unknown";
    this.$stats.stats = null; // also clears the sparkline ring buffer
  }

  async #runAction(name) {
    const id = this.#snapshot?.active;
    if (!id) return;
    const op = this.#opFor(name, id);
    if (!op) return;
    this.$actions.busy = true;
    try {
      await op();
      this.#toast(this.#doneMsg(name), "success");
    } catch (e) {
      await this.#handleActionError(name, id, op, e);
    } finally {
      this.$actions.busy = false;
    }
  }

  #opFor(name, id) {
    switch (name) {
      case "wake":
        return () => wake(id);
      case "shutdown":
      case "reboot":
        return () => power(id, name);
      case "open-files":
        return () => openFiles(id);
      case "map-drive":
        // Map Drive is Windows-only (spec §8) and hidden elsewhere; the
        // wizard-prefill/clipboard flow needs a backend command that M1
        // does not ship, so surface it explicitly if it is ever reached.
        return () => Promise.reject("Map Drive requires Windows (later milestone)");
      default:
        return null;
    }
  }

  #doneMsg(name) {
    switch (name) {
      case "wake":
        return "Wake packet sent";
      case "shutdown":
        return "Shutdown command sent";
      case "reboot":
        return "Reboot command sent";
      case "open-files":
        return "File manager opened";
      default:
        return "Done";
    }
  }

  /**
   * The structured-error retry flow: host-key prompts re-arm and retry the
   * same operation; secret prompts store the secret then retry.
   */
  async #handleActionError(name, id, op, e) {
    const msg = String(e);
    if (msg === "HOSTKEY_UNTRUSTED" || msg === "HOSTKEY_CHANGED") {
      await this.#hostKeyFlow(name, id, msg === "HOSTKEY_CHANGED", op);
      return;
    }
    if (msg.startsWith("SECRET_REQUIRED:")) {
      const kind = msg.slice("SECRET_REQUIRED:".length);
      await this.#secretFlow(name, id, kind, op);
      return;
    }
    this.#toast(msg, "error");
  }

  /** Host-key prompt → trustHost → retry the same operation. */
  async #hostKeyFlow(name, id, changed, op) {
    const message = changed
      ? "This server's SSH host key has CHANGED since it was last trusted. " +
        "This can indicate a man-in-the-middle attack. " +
        "Trust the new key and continue?"
      : "This server's SSH host key is not trusted yet. Trust it and continue?";
    const ok = await this.$toast.confirm(message, {
      kind: changed ? "danger" : "info",
    });
    if (!ok) {
      this.#toast("Action cancelled — host key not trusted.", "info");
      return;
    }
    try {
      await trustHost(id);
    } catch (e) {
      this.#toast(String(e), "error");
      return;
    }
    try {
      await op();
      this.#toast(this.#doneMsg(name), "success");
    } catch (e) {
      this.#toast(String(e), "error");
    }
  }

  /** Secret prompt → provideSecret → retry the same operation. */
  async #secretFlow(name, id, kind, op) {
    const res = await this.$toast.promptSecret(
      `Enter the ${SECRET_LABELS[kind] ?? kind} for this machine:`,
      { kind: "info" },
    );
    if (!res) {
      this.#toast("Action cancelled — no secret provided.", "info");
      return;
    }
    try {
      await provideSecret(id, kind, res.value, res.remember);
    } catch (e) {
      this.#toast(String(e), "error");
      return;
    }
    try {
      await op();
      this.#toast(this.#doneMsg(name), "success");
    } catch (e) {
      this.#toast(String(e), "error");
    }
  }

  connectedCallback() {
    void this.#init();
  }

  disconnectedCallback() {
    for (const un of this.#unlisteners) un();
    this.#unlisteners = [];
  }

  async #init() {
    try {
      const [unStatus, unStats] = await Promise.all([
        onStatus((p) => this.#onStatusEvent(p)),
        onStats((p) => this.#onStatsEvent(p)),
      ]);
      this.#unlisteners.push(unStatus, unStats);
    } catch (e) {
      this.#toast(String(e), "error");
    }
    await this.#loadState();
    this.#applyPlatform();
  }

  /** Loads the snapshot and pushes it into the child components. */
  async #loadState() {
    try {
      this.#snapshot = await getState();
    } catch (e) {
      this.#toast(String(e), "error");
      return;
    }
    this.$selector.machines = this.#snapshot.machines;
    this.$selector.active = this.#snapshot.active;
    this.$settings.settings = this.#snapshot.settings;
    this.$settings.machines = this.#snapshot.machines;
    this.$stats.display = this.#snapshot.settings.stats_display === "numbers"
      ? "numbers"
      : "graph";
    this.$actions.status = this.#status;
    if (
      this.#snapshot.notice &&
      this.#snapshot.notice !== this.#lastNotice
    ) {
      this.#lastNotice = this.#snapshot.notice;
      this.#toast(this.#snapshot.notice, "info");
    }
  }

  /** M1 platform detection: Android webview vs desktop (linux/windows). */
  #applyPlatform() {
    this.#platform = /Android/i.test(navigator.userAgent)
      ? "android"
      : "desktop-linux";
    this.$actions.platform = this.#platform;
  }

  #onStatusEvent(payload) {
    if (!this.#snapshot || payload.id !== this.#snapshot.active) return;
    const s = payload.status === "online" ? "online" : "offline";
    this.#status = s;
    this.$status.status = s;
    this.$actions.status = s;
    if (s === "offline") this.$stats.stats = null;
  }

  #onStatsEvent(payload) {
    if (!this.#snapshot || payload.id !== this.#snapshot.active) return;
    // Poller sends {id, stats} on success, {id, error} on failure.
    this.$stats.stats = payload.error != null
      ? { error: payload.error }
      : payload.stats;
  }

  async #saveGlobalSettings(settings) {
    try {
      await saveSettings(settings);
      await this.#loadState();
      this.$stats.display = settings.stats_display === "numbers"
        ? "numbers"
        : "graph";
      this.#toast("Settings saved", "success");
    } catch (e) {
      this.#toast(String(e), "error");
    }
  }

  async #saveMachine(machine) {
    try {
      await upsertMachine(machine);
      await this.#loadState();
      this.#toast(`Machine "${machine.name || machine.id}" saved`, "success");
    } catch (e) {
      this.#toast(String(e), "error");
    }
  }

  async #deleteMachine(id) {
    const machine = this.#snapshot?.machines.find((m) => m.id === id);
    const label = machine ? machine.name || machine.id : id;
    const ok = await this.$toast.confirm(
      `Delete machine "${label}"? Its stored secrets are removed too.`,
      { kind: "danger" },
    );
    if (!ok) return;
    try {
      await deleteMachine(id);
      if (this.#snapshot?.active === id) {
        this.#snapshot.active = null;
        this.#resetMachineState();
      }
      await this.#loadState();
      this.#toast(`Machine "${label}" deleted`, "success");
    } catch (e) {
      this.#toast(String(e), "error");
    }
  }

  /** Loads a machine (secrets blanked) into the settings form. */
  async #openMachine(id) {
    try {
      this.$settings.machine = await getMachine(id);
    } catch (e) {
      this.#toast(String(e), "error");
    }
  }

  #toast(message, kind) {
    this.$toast?.show(message, kind);
  }
}

customElements.define("sm-app", SmApp);