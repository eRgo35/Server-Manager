//! `<sm-settings-panel open settings machines machine>` — slide-out settings
//! panel toggled by a chevron tab (spec §9.1 item 6).
//!
//! State: `open` attribute (the tab button flips it; a CSS transform slides
//! the panel in from the right), `settings` property (global settings
//! object), `machines` property (array of `{id, name}` summaries) and
//! `machine` property (full machine with secrets blanked, pushed by
//! `<sm-app>` when the user picks one from the list).
//!
//! Emits (bubbles, composed):
//! - `settings-save {settings}`
//! - `machine-save {machine}` — plaintext secrets are only included when
//!   re-supplied in the inputs (see the T18 note in `#readMachine`)
//! - `machine-delete {id}`
//! - `machine-open {id}` — machine picked in the list; `<sm-app>` loads it
//!   via `getMachine` and pushes it back as the `machine` property
//! - `close {}` — panel toggled closed (informational)

import { t } from "../i18n/index.js";

const tpl = document.createElement("template");
tpl.innerHTML = `
  <style>
    :host {
      position: fixed;
      inset: 0;
      pointer-events: none;
      z-index: 50;
    }
    #tab {
      pointer-events: auto;
      position: absolute;
      top: 8px;
      right: 8px;
      font: inherit;
      font-size: 16px;
      line-height: 1;
      border: 1px solid var(--border);
      border-radius: 4px;
      background: var(--surface);
      color: var(--fg);
      cursor: pointer;
      z-index: 2;
    }
    #tab:hover { border-color: var(--accent); }
    #panel {
      pointer-events: auto;
      position: absolute;
      top: 0;
      right: 0;
      height: 100%;
      width: min(330px, 100vw);
      overflow-y: auto;
      box-sizing: border-box;
      background: var(--bg);
      border-left: 1px solid var(--border);
      padding: 10px 12px;
      transform: translateX(105%);
      transition: transform 0.18s ease;
      box-shadow: -4px 0 12px rgba(0, 0, 0, 0.25);
    }
    :host([open]) #panel { transform: translateX(0); }
    h2 {
      font-size: 13px;
      margin: 0 0 8px;
      text-transform: uppercase;
      letter-spacing: 0.05em;
      color: var(--muted);
    }
    section + section { margin-top: 16px; }
    .field {
      display: flex;
      align-items: center;
      gap: 8px;
      margin-bottom: 6px;
    }
    .field label {
      flex: 0 0 118px;
      color: var(--muted);
    }
    .field input, .field select {
      flex: 1;
      min-width: 0;
      font: inherit;
      color: var(--fg);
      background: var(--surface);
      border: 1px solid var(--border);
      border-radius: 4px;
      padding: 3px 6px;
    }
    .field input[readonly] {
      color: var(--muted);
      background: transparent;
    }
    button {
      font: inherit;
      padding: 3px 10px;
      border-radius: 4px;
      border: 1px solid var(--border);
      background: var(--surface);
      color: var(--fg);
      cursor: pointer;
    }
    button.primary {
      background: var(--accent);
      border-color: var(--accent);
      color: #fff;
    }
    .row {
      display: flex;
      gap: 6px;
      align-items: center;
    }
    .row select { flex: 1; min-width: 0; }
    .secret-note {
      color: var(--muted);
      font-size: 11px;
      margin: 6px 0;
    }
    .secret-note[hidden] { display: none; }
  </style>
  <button id="tab" aria-label="Toggle settings">‹</button>
  <aside id="panel">
    <section>
      <h2 id="settings-title">Settings</h2>
      <div class="field"><label for="s-language">Language</label>
        <select id="s-language"><option value="en">English</option><option value="pl">Polish</option></select></div>
      <div class="field"><label for="s-theme">Theme</label>
        <select id="s-theme"><option value="system">System</option><option value="light">Light</option><option value="dark">Dark</option></select></div>
      <div class="field"><label for="s-stats-display">Stats display</label>
        <select id="s-stats-display"><option value="numbers">Numbers</option><option value="graph">Graph</option></select></div>
      <div class="field"><label for="s-poll">Poll interval (s)</label>
        <input id="s-poll" type="number" min="1" step="1" /></div>
      <div class="field"><label for="s-secret-mode">Secret mode (default)</label>
        <select id="s-secret-mode"><option value="plaintext">Plaintext (config)</option><option value="prompt">Prompt (session only)</option></select></div>
      <div class="field"><label for="s-mount">Mount protocol</label>
        <select id="s-mount"><option value="smb">SMB</option><option value="sshfs">SSHFS</option></select></div>
      <div class="row"><button id="save-settings" class="primary">Save Settings</button></div>
      <p class="secret-note">M1 secret modes are plaintext or prompt — the OS keyring mode is deferred.</p>
    </section>

    <section>
      <h2 id="machines-title">Machines</h2>
      <div class="row">
        <select id="machine-list" aria-label="Edit machine"></select>
        <button id="new-machine" title="Add a new machine">＋</button>
        <button id="delete-machine" title="Delete the selected machine">−</button>
      </div>
      <form id="machine-form" onsubmit="return false">
        <div class="field"><label for="m-id">ID</label><input id="m-id" /></div>
        <div class="field"><label for="m-name">Name</label><input id="m-name" /></div>
        <div class="field"><label for="m-mac">MAC</label><input id="m-mac" placeholder="aa:bb:cc:dd:ee:ff" /></div>
        <div class="field"><label for="m-broadcast">Broadcast</label><input id="m-broadcast" placeholder="255.255.255.255" /></div>
        <div class="field"><label for="m-os-host">Host</label><input id="m-os-host" /></div>
        <div class="field"><label for="m-ssh-port">SSH port</label><input id="m-ssh-port" type="number" min="1" max="65535" /></div>
        <div class="field"><label for="m-ssh-user">SSH user</label><input id="m-ssh-user" /></div>
        <div class="field"><label for="m-key-path">Key file</label><input id="m-key-path" placeholder="/home/me/.ssh/id_ed25519" /></div>
        <div class="field"><label for="m-secret-mode">Secret mode</label>
          <select id="m-secret-mode"><option value="">Default</option><option value="plaintext">Plaintext (config)</option><option value="prompt">Prompt (session only)</option></select></div>
        <div class="field"><label for="m-shutdown">Shutdown cmd</label><input id="m-shutdown" placeholder="shutdown -h now" /></div>
        <div class="field"><label for="m-reboot">Reboot cmd</label><input id="m-reboot" placeholder="shutdown -r now" /></div>
        <div class="field"><label for="m-stats">Stats cmd</label><input id="m-stats" placeholder="(default: read /proc over SSH)" /></div>
        <div class="field"><label for="m-share">Share path</label><input id="m-share" /></div>
        <p class="secret-note" id="secret-note">
          Plaintext secrets are stored unencrypted in config.toml.
        </p>
        <div class="field secret-field"><label for="m-ssh-password">SSH password</label><input id="m-ssh-password" type="password" autocomplete="off" /></div>
        <div class="field secret-field"><label for="m-key-passphrase">Key passphrase</label><input id="m-key-passphrase" type="password" autocomplete="off" /></div>
        <div class="field secret-field"><label for="m-sudo-password">Sudo password</label><input id="m-sudo-password" type="password" autocomplete="off" /></div>
        <div class="row">
          <button id="save-machine" type="button" class="primary">Save Machine</button>
        </div>
      </form>
    </section>
  </aside>
`;
export class SettingsPanel extends HTMLElement {
  #settings = null;
  #machines = [];
  #machine = null; // machine being edited (null = new-machine form)

  constructor() {
    super();
    this.attachShadow({ mode: "open" }).appendChild(tpl.content.cloneNode(true));
    const $ = (id) => this.shadowRoot.getElementById(id);

    $("tab").addEventListener("click", () => {
      this.toggleAttribute("open");
      if (!this.hasAttribute("open")) {
        this.dispatchEvent(new CustomEvent("close", { bubbles: true, composed: true }));
      }
    });
    $("save-settings").addEventListener("click", () => this.#saveGlobal());
    $("save-machine").addEventListener("click", () => this.#saveMachine());
    $("new-machine").addEventListener("click", () => {
      this.machine = null;
    });
    $("delete-machine").addEventListener("click", () => this.#deleteMachine());
    $("machine-list").addEventListener("change", () => {
      const id = $("machine-list").value;
      if (id) {
        this.dispatchEvent(
          new CustomEvent("machine-open", { detail: { id }, bubbles: true, composed: true }),
        );
      }
    });
    this.#localize();
  }

  /**
   * Localizes all static template text (labels, headings, option labels,
   * notes, buttons, titles). Called on construction (en) and again from
   * `#fillGlobal` whenever a settings snapshot is pushed, so a language
   * change re-renders everything.
   */
  #localize() {
    const root = this.shadowRoot;
    const label = (forId, key) => {
      root.querySelector(`label[for="${forId}"]`).textContent = t(key);
    };
    root.getElementById("settings-title").textContent = t("settings.title");
    root.getElementById("machines-title").textContent = t("settings.machines");
    root.getElementById("tab").setAttribute("aria-label", t("toast.toggleSettings"));
    root.getElementById("machine-list").setAttribute("aria-label", t("machine.editList"));
    root.getElementById("new-machine").title = t("machine.newTitle");
    root.getElementById("delete-machine").title = t("machine.deleteTitle");
    root.getElementById("save-settings").textContent = t("settings.save");
    root.getElementById("save-machine").textContent = t("machine.save");

    for (const [id, key] of [
      ["s-language", "settings.language"],
      ["s-theme", "settings.theme"],
      ["s-stats-display", "settings.statsDisplay"],
      ["s-poll", "settings.pollBaseSecs"],
      ["s-secret-mode", "settings.secretMode"],
      ["s-mount", "settings.mountProtocol"],
      ["m-id", "machine.id"],
      ["m-name", "machine.name"],
      ["m-mac", "machine.mac"],
      ["m-broadcast", "machine.broadcast"],
      ["m-os-host", "machine.host"],
      ["m-ssh-port", "machine.sshPort"],
      ["m-ssh-user", "machine.sshUser"],
      ["m-key-path", "machine.keyFile"],
      ["m-secret-mode", "machine.secretMode"],
      ["m-shutdown", "machine.shutdownCmd"],
      ["m-reboot", "machine.rebootCmd"],
      ["m-stats", "machine.statsCmd"],
      ["m-share", "machine.sharePath"],
      ["m-ssh-password", "machine.sshPassword"],
      ["m-key-passphrase", "machine.keyPassphrase"],
      ["m-sudo-password", "machine.sudoPassword"],
    ]) {
      label(id, key);
    }

    // Option labels (the language options stay native: English / Polski).
    for (const o of root.querySelectorAll("#s-theme option")) {
      o.textContent = t(`settings.theme${o.value[0].toUpperCase()}${o.value.slice(1)}`);
    }
    for (const o of root.querySelectorAll("#s-stats-display option")) {
      o.textContent = t(o.value === "numbers" ? "settings.statsNumbers" : "settings.statsGraph");
    }
    for (const selId of ["s-secret-mode", "m-secret-mode"]) {
      for (const o of root.querySelectorAll(`#${selId} option`)) {
        o.textContent = o.value === ""
          ? t("machine.secretDefault")
          : t(o.value === "plaintext" ? "settings.secretPlaintext" : "settings.secretPrompt");
      }
    }

    root.querySelector("section .secret-note").textContent = t("settings.secretNote");
    root.getElementById("secret-note").textContent = t("machine.plaintextNote");
  }

  // ----- state -----

  /** @param {object|null} v global settings object */
  set settings(v) {
    this.#settings = v ?? null;
    this.#fillGlobal();
    this.#updateSecretVisibility();
  }
  get settings() {
    return this.#settings;
  }

  /** @param {{id: string, name: string}[]} v */
  set machines(v) {
    this.#machines = Array.isArray(v) ? v : [];
    this.#fillMachineList();
  }
  get machines() {
    return this.#machines;
  }

  /**
   * Machine being edited (secrets blanked, from `get_machine`).
   * `null` switches the form to "new machine" mode (editable id).
   * @param {object|null} m
   */
  set machine(m) {
    this.#machine = m ?? null;
    this.#fillMachine();
  }
  get machine() {
    return this.#machine;
  }

  static get observedAttributes() {
    return ["open"];
  }

  attributeChangedCallback() {
    this.shadowRoot.getElementById("tab").textContent = this.hasAttribute("open")
      ? "›"
      : "‹";
  }

  // ----- events -----

  #saveGlobal() {
    const $ = (id) => this.shadowRoot.getElementById(id);
    this.dispatchEvent(
      new CustomEvent("settings-save", {
        detail: {
          settings: {
            language: $("s-language").value,
            theme: $("s-theme").value,
            stats_display: $("s-stats-display").value,
            poll_base_secs: Math.max(1, Math.trunc(Number($("s-poll").value)) || 5),
            default_secret_mode: $("s-secret-mode").value,
            mount_protocol: $("s-mount").value,
          },
        },
        bubbles: true,
        composed: true,
      }),
    );
  }

  #saveMachine() {
    this.dispatchEvent(
      new CustomEvent("machine-save", {
        detail: { machine: this.#readMachine() },
        bubbles: true,
        composed: true,
      }),
    );
  }

  #deleteMachine() {
    const id =
      this.#machine?.id ?? this.shadowRoot.getElementById("machine-list").value;
    if (!id) return;
    this.dispatchEvent(
      new CustomEvent("machine-delete", { detail: { id }, bubbles: true, composed: true }),
    );
  }

  // ----- population -----

  #fillGlobal() {
    const $ = (id) => this.shadowRoot.getElementById(id);
    const s = this.#settings;
    if (!s) return;
    // A pushed snapshot may carry a new language — re-localize first so the
    // form shows the fresh strings, then fill the values.
    this.#localize();
    $("s-language").value = s.language ?? "en";
    $("s-theme").value = s.theme ?? "system";
    $("s-stats-display").value = s.stats_display ?? "graph";
    $("s-poll").value = s.poll_base_secs ?? 5;
    $("s-secret-mode").value = s.default_secret_mode ?? "prompt";
    $("s-mount").value = s.mount_protocol ?? "smb";
  }

  #fillMachineList() {
    const sel = this.shadowRoot.getElementById("machine-list");
    const current = sel.value;
    sel.textContent = "";
    if (this.#machines.length === 0) {
      const o = document.createElement("option");
      o.value = "";
      o.textContent = t("machine.none");
      sel.append(o);
    }
    for (const m of this.#machines) {
      const o = document.createElement("option");
      o.value = m.id;
      o.textContent = m.name || m.id;
      sel.append(o);
    }
    sel.value = current;
  }

  #fillMachine() {
    const $ = (id) => this.shadowRoot.getElementById(id);
    const m = this.#machine;
    const f = (v) => v ?? "";
    $("m-id").value = f(m?.id);
    $("m-id").readOnly = Boolean(m);
    $("m-id").disabled = Boolean(m);
    $("m-name").value = f(m?.name);
    $("m-mac").value = f(m?.mac);
    $("m-broadcast").value = f(m?.broadcast_addr);
    $("m-os-host").value = f(m?.os_host);
    $("m-ssh-port").value = m?.ssh_port ?? 22;
    $("m-ssh-user").value = f(m?.ssh_user);
    $("m-key-path").value = f(m?.key_path);
    $("m-secret-mode").value = m?.secret_mode ?? "";
    $("m-shutdown").value = f(m?.shutdown_cmd);
    $("m-reboot").value = f(m?.reboot_cmd);
    $("m-stats").value = f(m?.stats_cmd);
    $("m-share").value = f(m?.share_path);
    // Secrets always blank here — get_machine never returns them.
    $("m-ssh-password").value = "";
    $("m-key-passphrase").value = "";
    $("m-sudo-password").value = "";
    this.#updateSecretVisibility();
  }

  #updateSecretVisibility() {
    const m = this.#machine;
    const effective =
      m?.secret_mode ?? this.#settings?.default_secret_mode ?? "prompt";
    const plaintext = effective === "plaintext";
    for (const el of this.shadowRoot.querySelectorAll(".secret-field")) {
      el.hidden = !plaintext;
    }
    this.shadowRoot.getElementById("secret-note").hidden = !plaintext;
  }

  #readMachine() {
    const $ = (id) => this.shadowRoot.getElementById(id);
    const opt = (id) => {
      const v = $(id).value.trim();
      return v === "" ? undefined : v;
    };
    const required = (id) => $(id).value.trim();
    const secret = (id) => {
      const v = $(id).value.trim();
      // NOTE (T18 deferred round-trip item): `get_machine` returns secrets
      // blanked and `upsert_machine` REPLACES the machine wholesale, so
      // saving an existing machine WIPES its stored plaintext secrets
      // unless the user re-supplies them in these inputs. A proper fix
      // needs a "keep existing secrets" flag on the backend's
      // upsert_machine — deferred together with the T18 review item.
      return v === "" ? undefined : v;
    };
    return {
      id: required("m-id"),
      name: required("m-name"),
      mac: required("m-mac"),
      broadcast_addr: opt("m-broadcast"),
      os_host: required("m-os-host"),
      ssh_port: Math.max(1, Math.trunc(Number($("m-ssh-port").value)) || 22),
      ssh_user: required("m-ssh-user"),
      key_path: opt("m-key-path"),
      secret_mode: opt("m-secret-mode"),
      shutdown_cmd: opt("m-shutdown"),
      reboot_cmd: opt("m-reboot"),
      stats_cmd: opt("m-stats"),
      share_path: opt("m-share"),
      ssh_password: secret("m-ssh-password"),
      key_passphrase: secret("m-key-passphrase"),
      sudo_password: secret("m-sudo-password"),
    };
  }
}

customElements.define("sm-settings-panel", SettingsPanel);