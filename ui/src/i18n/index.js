//! i18n — English/Polish dictionaries with a module-level active locale.
//!
//! `sm-app` calls `setLocale(code)` from the settings snapshot; components
//! resolve every user-facing string through `t(key, vars)`. Resolution:
//! active locale → `en` → the key itself. Static imports keep both
//! dictionaries in the bundle (M1 is small; no async loading).

import en from "./en.json" with { type: "json" };
import pl from "./pl.json" with { type: "json" };

const DICTS = { en, pl };

/** @type {"en"|"pl"} */
let locale = "en";

/** @param {string} code — unknown codes are ignored (locale stays as-is). */
export function setLocale(code) {
  if (DICTS[code]) locale = code;
}

/**
 * Resolve `key` in the active locale, falling back to `en`, then to the key
 * itself. `{name}`-style `vars` are interpolated into the resolved string.
 * @param {string} key
 * @param {Record<string, string|number>} [vars]
 * @returns {string}
 */
export function t(key, vars) {
  let s = DICTS[locale][key] ?? DICTS.en[key] ?? key;
  if (vars) {
    for (const [k, v] of Object.entries(vars)) {
      s = s.replaceAll(`{${k}}`, String(v));
    }
  }
  return s;
}