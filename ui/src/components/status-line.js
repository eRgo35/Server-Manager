//! `<sm-status-line status>` — ONLINE / OFFLINE / — with the theme's
//! green/red. State via the `status` attribute or property
//! (`"online" | "offline" | "unknown"`); announces via `aria-live="polite"`.

const tpl = document.createElement("template");
tpl.innerHTML = `
  <style>
    .line {
      display: flex;
      align-items: center;
      gap: 6px;
      font-weight: 600;
      letter-spacing: 0.04em;
      min-height: 20px;
    }
    .dot {
      width: 8px;
      height: 8px;
      border-radius: 50%;
      background: var(--muted);
    }
    :host([status="online"]) .dot { background: var(--ok); }
    :host([status="offline"]) .dot { background: var(--err); }
    :host([status="online"]) .text { color: var(--ok); }
    :host([status="offline"]) .text { color: var(--err); }
    :host(:not([status="online"]):not([status="offline"])) .text {
      color: var(--muted);
    }
  </style>
  <div class="line" aria-live="polite">
    <span class="dot" aria-hidden="true"></span>
    <span class="text">—</span>
  </div>
`;

const LABELS = { online: "ONLINE", offline: "OFFLINE" };

export class StatusLine extends HTMLElement {
  #status = "unknown";

  constructor() {
    super();
    this.attachShadow({ mode: "open" }).appendChild(tpl.content.cloneNode(true));
  }

  static get observedAttributes() {
    return ["status"];
  }

  attributeChangedCallback(_name, _old, value) {
    this.status = value || "unknown";
  }

  /** @returns {"online"|"offline"|"unknown"} */
  get status() {
    return this.#status;
  }
  /** @param {"online"|"offline"|"unknown"} v */
  set status(v) {
    this.#status = v === "online" || v === "offline" ? v : "unknown";
    if (this.getAttribute("status") !== this.#status) {
      // Reflected so the theme-colour selectors above work.
      this.setAttribute("status", this.#status);
      return; // attributeChangedCallback re-enters with the same value.
    }
    this.shadowRoot.querySelector(".text").textContent =
      LABELS[this.#status] ?? "—";
  }
}

customElements.define("sm-status-line", StatusLine);