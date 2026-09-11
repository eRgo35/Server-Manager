//! `<sm-machine-selector machines active>` — dropdown of configured machines.
//!
//! State: `machines` property (array of `{id, name}`) and `active` property
//! (or `active` attribute). Emits `machine-change {id}` (bubbles, composed)
//! when the user picks a machine; `id` is `null` when nothing is selected.

const tpl = document.createElement("template");
tpl.innerHTML = `
  <style>
    select {
      font: inherit;
      color: var(--fg);
      background: var(--surface);
      border: 1px solid var(--border);
      border-radius: 4px;
      padding: 4px 6px;
      width: 100%;
    }
    select:focus {
      outline: 1px solid var(--accent);
      outline-offset: 0;
    }
  </style>
  <select id="sel" aria-label="Machine"></select>
`;

export class MachineSelector extends HTMLElement {
  #machines = [];
  #active = null;

  constructor() {
    super();
    this.attachShadow({ mode: "open" }).appendChild(tpl.content.cloneNode(true));
    const sel = this.shadowRoot.getElementById("sel");
    sel.addEventListener("change", () => {
      this.#active = sel.value || null;
      this.dispatchEvent(
        new CustomEvent("machine-change", {
          detail: { id: this.#active },
          bubbles: true,
          composed: true,
        }),
      );
    });
  }

  static get observedAttributes() {
    return ["active"];
  }

  attributeChangedCallback() {
    this.active = this.getAttribute("active") || null;
  }

  /** @returns {{id: string, name: string}[]} */
  get machines() {
    return this.#machines;
  }
  /** @param {{id: string, name: string}[]} v */
  set machines(v) {
    this.#machines = Array.isArray(v) ? v : [];
    this.#render();
  }

  /** @returns {string|null} */
  get active() {
    return this.#active;
  }
  /** @param {string|null} v */
  set active(v) {
    this.#active = v ?? null;
    this.shadowRoot.getElementById("sel").value = this.#active ?? "";
  }

  #render() {
    const sel = this.shadowRoot.getElementById("sel");
    sel.textContent = "";
    if (this.#machines.length === 0) {
      const o = document.createElement("option");
      o.value = "";
      o.textContent = "No machines configured";
      sel.append(o);
    } else {
      for (const m of this.#machines) {
        const o = document.createElement("option");
        o.value = m.id;
        o.textContent = m.name || m.id;
        sel.append(o);
      }
    }
    sel.value = this.#active ?? "";
  }
}

customElements.define("sm-machine-selector", MachineSelector);