//! `<sm-action-buttons status platform busy>` — Wake / Shutdown / Reboot /
//! Open Files / Map Drive.
//!
//! State via attributes or properties:
//! - `status`: "online" | "offline" | "unknown" — drives enable rules.
//! - `platform`: "desktop-linux" | "windows" | "android" — drives visibility
//!   (Open Files hidden on android; Map Drive shown only on windows).
//! - `busy`: any value — disables everything while an action is in flight.
//!
//! Emits `action {name}` with name one of
//! `wake | shutdown | reboot | open-files | map-drive`.

const tpl = document.createElement("template");
tpl.innerHTML = `
  <style>
    .row {
      display: flex;
      flex-wrap: wrap;
      gap: 6px;
    }
    button {
      font: inherit;
      color: var(--fg);
      background: var(--surface);
      border: 1px solid var(--border);
      border-radius: 4px;
      padding: 4px 10px;
      cursor: pointer;
    }
    button:hover:not(:disabled) {
      border-color: var(--accent);
    }
    button:disabled {
      opacity: 0.45;
      cursor: default;
    }
    button.primary {
      background: var(--accent);
      border-color: var(--accent);
      color: #fff;
    }
  </style>
  <div class="row">
    <button data-action="wake" class="primary">Wake</button>
    <button data-action="shutdown">Shutdown</button>
    <button data-action="reboot">Reboot</button>
    <button data-action="open-files">Open Files</button>
    <button data-action="map-drive">Map Drive</button>
  </div>
`;


export class ActionButtons extends HTMLElement {
  #status = "unknown";
  #platform = "desktop-linux";
  #busy = false;

  constructor() {
    super();
    this.attachShadow({ mode: "open" }).appendChild(tpl.content.cloneNode(true));
    for (const b of this.shadowRoot.querySelectorAll("button[data-action]")) {
      b.addEventListener("click", () => {
        if (b.disabled) return;
        this.dispatchEvent(
          new CustomEvent("action", {
            detail: { name: b.dataset.action },
            bubbles: true,
            composed: true,
          }),
        );
      });
    }
  }

  static get observedAttributes() {
    return ["status", "platform", "busy"];
  }

  attributeChangedCallback() {
    this.status = this.getAttribute("status") || "unknown";
    this.platform = this.getAttribute("platform") || "desktop-linux";
    this.busy = this.getAttribute("busy") !== null;
  }

  /** @returns {"online"|"offline"|"unknown"} */
  get status() {
    return this.#status;
  }
  /** @param {"online"|"offline"|"unknown"} v */
  set status(v) {
    this.#status = v === "online" || v === "offline" ? v : "unknown";
    this.#render();
  }

  /** @returns {"desktop-linux"|"windows"|"android"} */
  get platform() {
    return this.#platform;
  }
  /** @param {"desktop-linux"|"windows"|"android"} v */
  set platform(v) {
    this.#platform = v === "android" || v === "windows" ? v : "desktop-linux";
    this.#render();
  }

  get busy() {
    return this.#busy;
  }
  set busy(v) {
    this.#busy = Boolean(v);
    this.#render();
  }

  #render() {
    const online = this.#status === "online";
    const offline = this.#status === "offline";
    const android = this.#platform === "android";
    for (const b of this.shadowRoot.querySelectorAll("button[data-action]")) {
      const name = b.dataset.action;
      b.disabled = this.#busy;
      switch (name) {
        case "wake":
          b.disabled ||= !offline;
          break;
        // Shutdown/Reboot need a live SSH session.
        case "shutdown":
        case "reboot":
          b.disabled ||= !online;
          break;
        // The share is only reachable while the machine is up.
        case "open-files":
        case "map-drive":
          b.disabled ||= !online;
          break;
      }
      b.hidden =
        name === "map-drive"
          ? this.#platform !== "windows"
          : name === "open-files" && android;
    }
  }
}

customElements.define("sm-action-buttons", ActionButtons);