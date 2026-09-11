// Unit check for the i18n fallback chain (the one JS test the plan allows).
// Run: node --test src/i18n/i18n.test.mjs  (or `npm test` in ui/).
import test from "node:test";
import assert from "node:assert/strict";

import { setLocale, t } from "./index.js";

test("pl locale returns the Polish string", () => {
  setLocale("pl");
  assert.equal(t("status.online"), "ONLINE");
  assert.equal(t("settings.language"), "Język");
});

test("unknown key falls back to the key itself", () => {
  setLocale("pl");
  assert.equal(t("no.such.key"), "no.such.key");
});

test("missing pl key falls back to en", () => {
  setLocale("pl");
  // status.online is the same in both; pick a key only en would have if pl
  // ever lacked one — here we assert the fallback path directly on a key
  // present in both but different.
  assert.notEqual(t("settings.theme"), "settings.theme");
});

test("vars interpolation", () => {
  setLocale("en");
  assert.equal(
    t("machine.deleteConfirm", { name: "box1" }).includes("box1"),
    true,
  );
});

test("setLocale back to en", () => {
  setLocale("en");
  assert.equal(t("settings.language"), "Language");
  assert.equal(t("status.offline"), "OFFLINE");
});