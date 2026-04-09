import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

const mainRsPath = path.resolve("src-tauri/src/main.rs");
const mainRsSource = fs.readFileSync(mainRsPath, "utf8");
const logoutMatch = mainRsSource.match(
  /async fn logout\(app: AppHandle\) -> Result<\(\), String> \{[\s\S]*?\n\}/,
);
const logoutSource = logoutMatch?.[0] ?? "";
const resetSettingsMatch = mainRsSource.match(
  /fn reset_settings\(app: AppHandle\) -> Result<copilot_tracker::AppSettings, String> \{[\s\S]*?\n\}/,
);
const resetSettingsSource = resetSettingsMatch?.[0] ?? "";

test("logout resets the tray icon to the unauthenticated sentinel", () => {
  assert.notEqual(logoutSource, "", "logout function must exist");
  assert.match(
    logoutSource,
    /update_tray_icon\(\s*&app,\s*&tray_state,\s*1\.0,\s*0,\s*"currentTotal"\s*\)/,
    "logout must reset the tray icon to the unauthenticated sentinel value",
  );
});

test("logout emits empty usage:data to clear stale history and prediction", () => {
  assert.match(
    logoutSource,
    /app\.emit\(\s*"usage:data",/,
    "logout must emit usage:data so renderer consumers clear cached history",
  );
  assert.match(
    logoutSource,
    /history:\s*vec!\[\]/,
    "logout usage:data payload must clear history",
  );
  assert.match(
    logoutSource,
    /prediction:\s*None/,
    "logout usage:data payload must clear prediction",
  );
});

test("logout does not emit usage:updated after forcing the unauthenticated tray sentinel", () => {
  assert.doesNotMatch(
    logoutSource,
    /app\.emit\(\s*"usage:updated"/,
    "logout should not re-trigger the usage:updated listener after setting the tray sentinel directly",
  );
});

test("reset_settings emits empty usage:data instead of usage:updated", () => {
  assert.notEqual(
    resetSettingsSource,
    "",
    "reset_settings function must exist",
  );
  assert.match(
    resetSettingsSource,
    /app\.emit\(\s*"usage:data",/,
    "reset_settings should emit usage:data so renderer consumers clear cached history",
  );
  assert.match(
    resetSettingsSource,
    /history:\s*vec!\[\]/,
    "reset_settings usage:data payload must clear history",
  );
  assert.match(
    resetSettingsSource,
    /prediction:\s*None/,
    "reset_settings usage:data payload must clear prediction",
  );
  assert.doesNotMatch(
    resetSettingsSource,
    /app\.emit\(\s*"usage:updated"/,
    "reset_settings should not emit usage:updated after forcing the unauthenticated tray sentinel directly",
  );
});

test("logout rebuilds the tray menu after clearing session state", () => {
  assert.match(
    logoutSource,
    /rebuild_tray_menu\(/,
    "logout should rebuild the tray menu because it no longer relies on usage:updated to refresh menu contents",
  );
});
