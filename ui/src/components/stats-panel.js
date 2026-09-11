//! `<sm-stats-panel stats display>` — CPU/memory/disk/uptime as numbers or a
//! CPU sparkline, per the `display` attribute/property
//! ("numbers" | "graph"; anything else falls back to "graph").
//!
//! The sparkline draws from an in-memory ring buffer kept inside the element
//! (last 60 samples, session-only per spec §7.4). A `stats` property of
//! `null` clears the buffer (used when the active machine changes); a stats
//! object with an `error` field shows "unavailable" but keeps the buffer.

const tpl = document.createElement("template");
tpl.innerHTML = `
  <style>
    :host {
      display: block;
      min-height: 22px;
    }
    .nums {
      display: flex;
      flex-wrap: wrap;
      gap: 4px 14px;
      font-variant-numeric: tabular-nums;
    }
    .nums div {
      min-width: 0;
    }
    .label {
      color: var(--muted);
      margin-right: 4px;
    }
    svg {
      display: block;
      width: 100%;
      height: 36px;
      max-width: 100%;
    }
    polyline {
      fill: none;
      stroke: var(--accent);
      stroke-width: 1.5;
    }
    .unavail {
      color: var(--muted);
      font-style: italic;
    }
    .hidden { display: none; }
  </style>
  <div class="unavail">unavailable</div>
  <div class="nums hidden"></div>
  <svg class="hidden" viewBox="0 0 60 24" preserveAspectRatio="none" aria-hidden="true">
    <polyline points=""></polyline>
    <line x1="0" y1="23.5" x2="60" y2="23.5" stroke="var(--border)" stroke-width="0.5"></line>
  </svg>
`;

/// Renders "12.3 GB" style byte counts.
function humanBytes(n) {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let v = n;
  let u = 0;
  while (v >= 1024 && u < units.length - 1) {
    v /= 1024;
    u += 1;
  }
  return `${v.toFixed(v >= 100 || u === 0 ? 0 : 1)} ${units[u]}`;
}

/// Renders "1d 3h 12m" style uptime.
function humanUptime(secs) {
  const d = Math.floor(secs / 86400);
  const h = Math.floor((secs % 86400) / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const parts = [];
  if (d) parts.push(`${d}d`);
  if (h || d) parts.push(`${h}h`);
  parts.push(`${m}m`);
  return parts.join(" ");
}

export class StatsPanel extends HTMLElement {
  #stats = null;
  #display = "graph";
  #samples = []; // ring buffer of cpu_pct, newest last

  constructor() {
    super();
    this.attachShadow({ mode: "open" }).appendChild(tpl.content.cloneNode(true));
  }

  static get observedAttributes() {
    return ["display"];
  }

  attributeChangedCallback() {
    this.display = this.getAttribute("display") || "graph";
  }

  /** @returns {"numbers"|"graph"} */
  get display() {
    return this.#display;
  }
  /** @param {"numbers"|"graph"} v */
  set display(v) {
    this.#display = v === "numbers" ? "numbers" : "graph";
    this.#render();
  }

  /**
   * Latest stats payload: a `Stats` object (cpu_pct, mem_used, mem_total,
   * disk_used, disk_total, uptime_secs), `{error: string}`, or `null`.
   * `null` also clears the sparkline buffer (machine switch / offline).
   * @param {object|null} v
   */
  set stats(v) {
    if (v == null) {
      this.#samples = [];
    } else if (v.error == null && typeof v.cpu_pct === "number") {
      this.#samples.push(Math.max(0, Math.min(100, v.cpu_pct)));
      if (this.#samples.length > 60) this.#samples.shift();
    }
    this.#stats = v;
    this.#render();
  }
  /** @returns {object|null} */
  get stats() {
    return this.#stats;
  }

  #render() {
    const unavail = this.#stats == null || this.#stats.error != null;
    const unavailEl = this.shadowRoot.querySelector(".unavail");
    const nums = this.shadowRoot.querySelector(".nums");
    const svg = this.shadowRoot.querySelector("svg");
    const polyline = this.shadowRoot.querySelector("polyline");

    unavailEl.classList.toggle("hidden", !unavail);
    if (unavail) {
      nums.classList.add("hidden");
      svg.classList.add("hidden");
      if (this.#stats && this.#stats.error) {
        unavailEl.title = this.#stats.error;
        unavailEl.textContent = `unavailable (${this.#stats.error})`;
      } else {
        unavailEl.title = "";
        unavailEl.textContent = "unavailable";
      }
      return;
    }

    const s = this.#stats;
    if (this.#display === "numbers") {
      nums.classList.remove("hidden");
      svg.classList.add("hidden");
      nums.innerHTML = `
        <div><span class="label">CPU</span>${s.cpu_pct.toFixed(1)}%</div>
        <div><span class="label">Mem</span>${humanBytes(s.mem_used)} / ${humanBytes(s.mem_total)}</div>
        <div><span class="label">Disk</span>${humanBytes(s.disk_used)} / ${humanBytes(s.disk_total)}</div>
        <div><span class="label">Up</span>${humanUptime(s.uptime_secs)}</div>`;
      return;
    }

    // Graph mode: sparkline + a compact numbers row beneath it.
    svg.classList.remove("hidden");
    nums.classList.remove("hidden");
    // 60-slot viewBox, one x-unit per buffer slot so the line grows in from
    // the left until the buffer fills.
    const points = this.#samples
      .map((v, i) => `${i},${(23 - (v / 100) * 22).toFixed(2)}`)
      .join(" ");
    polyline.setAttribute("points", points);

    const last = this.#samples.at(-1);
    nums.innerHTML = `
      <div><span class="label">CPU</span>${last == null ? "—" : `${last.toFixed(1)}%`}</div>
      <div><span class="label">Mem</span>${humanBytes(s.mem_used)} / ${humanBytes(s.mem_total)}</div>
      <div><span class="label">Disk</span>${humanBytes(s.disk_used)} / ${humanBytes(s.disk_total)}</div>
      <div><span class="label">Up</span>${humanUptime(s.uptime_secs)}</div>`;
  }
}

customElements.define("sm-stats-panel", StatsPanel);