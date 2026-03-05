import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

const widgetPath = path.resolve(
  "src/renderer/src/components/widget/Widget.tsx",
);
const adapterPath = path.resolve("src/renderer/src/tauri-adapter.ts");

const widgetSource = fs.readFileSync(widgetPath, "utf8");
const adapterSource = fs.readFileSync(adapterPath, "utf8");

test("widget pin toggle persists through backend command", () => {
  assert.match(
    widgetSource,
    /tauriInvoke\("set_widget_pinned", \{ pinned: newPinned \}\)/,
    "pin toggle must call set_widget_pinned backend command",
  );
});

test("widget startup reads persisted pin state from backend", () => {
  assert.match(
    widgetSource,
    /tauriInvoke<boolean>\("is_widget_pinned"\)/,
    "widget init must read is_widget_pinned for persisted state",
  );
});

test("settings merge preserves widget fields to avoid reset", () => {
  const requiredFields = [
    "widgetEnabled: current.widgetEnabled",
    "widgetPosition: current.widgetPosition",
    "widgetPinned: current.widgetPinned",
    "widgetVisible: current.widgetVisible",
  ];

  for (const field of requiredFields) {
    assert.ok(
      adapterSource.includes(field),
      `missing merge field in tauri adapter: ${field}`,
    );
  }
});
