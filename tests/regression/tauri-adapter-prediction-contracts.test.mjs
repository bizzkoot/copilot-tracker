import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

const adapterPath = path.resolve("src/renderer/src/tauri-adapter.ts");
const adapterSource = fs.readFileSync(adapterPath, "utf8");
const appTypesPath = path.resolve("src/renderer/src/types/app.ts");
const appTypesSource = fs.readFileSync(appTypesPath, "utf8");

test("UsageFetchResult allows prediction to be cleared explicitly", () => {
  assert.match(
    appTypesSource,
    /prediction\?:\s*UsagePrediction\s*\|\s*null;/,
    "UsageFetchResult.prediction should accept null so logout/reset payloads can clear stale forecasts",
  );
});

test("usage:data listener maps missing prediction to null instead of undefined", () => {
  assert.match(
    adapterSource,
    /prediction:\s*payload\.prediction[\s\S]*:\s*null,/,
    "usage:data listener should map missing prediction to null so Zustand clears stale forecast state",
  );
});

test("getCachedUsage maps missing prediction to null instead of undefined", () => {
  assert.match(
    adapterSource,
    /prediction:\s*payload\.prediction[\s\S]*:\s*null,\s*\n\s*};\s*\n\s*return result;/,
    "getCachedUsage should map missing prediction to null so cached empty payloads clear stale forecast state",
  );
});
