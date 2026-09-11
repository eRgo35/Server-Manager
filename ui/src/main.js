// Entry point: import styles, then register all custom elements.
// Every component module self-registers on import; sm-app also imports its
// children (module singletons — customElements.define runs once either way).
import "./styles/theme.css";
import "./styles/app.css";

import "./components/sm-app.js";
import "./components/machine-selector.js";
import "./components/status-line.js";
import "./components/action-buttons.js";
import "./components/stats-panel.js";
import "./components/settings-panel.js";
import "./components/toast-host.js";