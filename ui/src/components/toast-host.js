//! `<sm-toast-host>` — overlay toasts + modal dialogs, used by `<sm-app>`
//! for notices, errors, host-key prompts and secret prompts.
//!
//! Methods (no events upward):
//! - `show(message, kind = "info", timeoutMs?)` — transient toast,
//!   auto-dismisses (errors stay longer); click to dismiss early.
//! - `confirm(message, {kind, okLabel, cancelLabel})` → `Promise<boolean>`.
//! - `promptSecret(message, {kind})` → `Promise<{value, remember}|null>`
//!   (`null` when the user cancels or leaves the field empty).
//!
import { t } from "../i18n/index.js";
//! `kind`: "info" | "success" | "error" | "danger" (danger styles the modal
//! OK button red — used for the loud host-key-changed warning).

const tpl = document.createElement("template");
tpl.innerHTML = `
  <style>
    :host {
      position: fixed;
      inset: 0;
      pointer-events: none;
      z-index: 100;
    }
    #toasts {
      position: absolute;
      right: 8px;
      bottom: 8px;
      display: flex;
      flex-direction: column;
      gap: 6px;
      max-width: min(320px, calc(100vw - 16px));
    }
    .toast {
      pointer-events: auto;
      padding: 6px 10px;
      border-radius: 4px;
      border: 1px solid var(--border);
      background: var(--surface);
      color: var(--fg);
      font-size: 12px;
      cursor: pointer;
      word-break: break-word;
    }
    .toast.error {
      border-color: var(--err);
      color: var(--err);
    }
    .toast.success {
      border-color: var(--ok);
    }
    .toast.info {
      border-color: var(--accent);
    }
    #modal {
      pointer-events: auto;
      position: absolute;
      inset: 0;
      display: grid;
      place-items: center;
      background: rgba(0, 0, 0, 0.4);
    }
    #modal[hidden] { display: none; }
    .card {
      background: var(--bg);
      border: 1px solid var(--border);
      border-radius: 6px;
      padding: 12px;
      max-width: min(300px, calc(100vw - 24px));
      display: flex;
      flex-direction: column;
      gap: 10px;
    }
    #modal-msg {
      word-break: break-word;
    }
    #secret-row {
      display: flex;
      flex-direction: column;
      gap: 6px;
    }
    #secret-row[hidden] { display: none; }
    input[type="password"] {
      font: inherit;
      color: var(--fg);
      background: var(--bg);
      border: 1px solid var(--border);
      border-radius: 4px;
      padding: 4px 6px;
    }
    label.remember {
      display: flex;
      align-items: center;
      gap: 6px;
      color: var(--muted);
      font-size: 12px;
    }
    .buttons {
      display: flex;
      justify-content: flex-end;
      gap: 6px;
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
    button:hover { border-color: var(--accent); }
    #ok.danger {
      background: var(--err);
      border-color: var(--err);
      color: #fff;
    }
  </style>
  <div id="toasts"></div>
  <div id="modal" hidden role="dialog" aria-modal="true">
    <div class="card">
      <div id="modal-msg"></div>
      <div id="secret-row" hidden>
        <input id="secret-input" type="password" autocomplete="off" />
        <label class="remember">
          <input id="remember" type="checkbox" />
          <span id="remember-text">Remember (stored in config in plaintext mode)</span>
        </label>
      </div>
      <div class="buttons">
        <button id="cancel">Cancel</button>
        <button id="ok">OK</button>
      </div>
    </div>
  </div>
`;

export class ToastHost extends HTMLElement {
  constructor() {
    super();
    this.attachShadow({ mode: "open" }).appendChild(tpl.content.cloneNode(true));
    // Block background clicks while a modal is up.
    this.shadowRoot.getElementById("modal").addEventListener(
      "click",
      (e) => {
        if (e.target === e.currentTarget) this.#resolvePending?.(null);
      },
    );
  }

  /**
   * Show a transient toast.
   * @param {string} message
   * @param {"info"|"success"|"error"|"danger"} [kind]
   * @param {number} [timeoutMs] default 4 s (8 s for errors)
   */
  show(message, kind = "info", timeoutMs) {
    const host = this.shadowRoot.getElementById("toasts");
    const el = document.createElement("div");
    el.className = `toast ${kind}`;
    // Plain text only — toasts carry backend error strings.
    el.textContent = message;
    host.append(el);
    while (host.children.length > 4) host.firstElementChild.remove();
    const t = setTimeout(
      () => el.remove(),
      timeoutMs ?? (kind === "error" ? 8000 : 4000),
    );
    el.addEventListener("click", () => {
      clearTimeout(t);
      el.remove();
    });
  }

  /**
   * Modal confirmation. Resolves `true` on OK, `false` otherwise.
   * @param {string} message
   * @param {{kind?: "info"|"danger", okLabel?: string, cancelLabel?: string}} [opts]
   * @returns {Promise<boolean>}
   */
  confirm(message, opts = {}) {
    return this.#modal({ message, kind: opts.kind, okLabel: opts.okLabel, cancelLabel: opts.cancelLabel })
      .then((v) => v === true);
  }

  /**
   * Modal secret prompt.
   * @param {string} message
   * @param {{kind?: "info"|"danger"}} [opts]
   * @returns {Promise<{value: string, remember: boolean}|null>}
   */
  promptSecret(message, opts = {}) {
    return this.#modal({ message, kind: opts.kind, withSecret: true });
  }

  #resolvePending = null;

  /**
   * Opens the shared modal; resolves with `true`, `null` (cancel/Escape/
   * backdrop) or, with `withSecret`, `{value, remember}`.
   */
  #modal({ message, kind = "info", okLabel = t("toast.ok"), cancelLabel = t("toast.cancel"), withSecret = false }) {
    return new Promise((resolve) => {
      // A previous modal should have resolved already; be defensive anyway.
      this.#resolvePending?.(null);
      this.#resolvePending = resolve;

      const overlay = this.shadowRoot.getElementById("modal");
      const ok = this.shadowRoot.getElementById("ok");
      const cancel = this.shadowRoot.getElementById("cancel");
      const input = this.shadowRoot.getElementById("secret-input");
      const remember = this.shadowRoot.getElementById("remember");
      const secretRow = this.shadowRoot.getElementById("secret-row");

      this.shadowRoot.getElementById("modal-msg").textContent = message;
      overlay.className = kind;
      ok.textContent = okLabel;
      cancel.textContent = cancelLabel;
      ok.classList.toggle("danger", kind === "danger");
      secretRow.hidden = !withSecret;
      this.shadowRoot.getElementById("remember-text").textContent =
        t("toast.remember");
      input.value = "";
      remember.checked = false;
      overlay.hidden = false;
      if (withSecret) input.focus();
      else ok.focus();

      const done = (v) => {
        this.#resolvePending = null;
        overlay.hidden = true;
        resolve(v);
      };
      ok.onclick = () => {
        if (withSecret) {
          // Empty secret: no-op — the user must type one or Cancel.
          if (!input.value) return;
          done({ value: input.value, remember: remember.checked });
        } else {
          done(true);
        }
      };
      cancel.onclick = () => done(null);
    });
  }
}

customElements.define("sm-toast-host", ToastHost);