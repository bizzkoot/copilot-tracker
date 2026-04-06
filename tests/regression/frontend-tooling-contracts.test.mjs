import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

const appTypesSource = fs.readFileSync(
  path.resolve("src/renderer/src/types/app.ts"),
  "utf8",
);
const useAuthSource = fs.readFileSync(
  path.resolve("src/renderer/src/hooks/useAuth.ts"),
  "utf8",
);
const useUsageSource = fs.readFileSync(
  path.resolve("src/renderer/src/hooks/useUsage.ts"),
  "utf8",
);
const adapterSource = fs.readFileSync(
  path.resolve("src/renderer/src/tauri-adapter.ts"),
  "utf8",
);

function getSection(source, startMarker, endMarker) {
  const start = source.indexOf(startMarker);
  assert.notEqual(start, -1, `Missing section start: ${startMarker}`);
  const end = endMarker ? source.indexOf(endMarker, start) : source.length;
  assert.notEqual(end, -1, `Missing section end: ${endMarker}`);
  return source.slice(start, end);
}

test("AppAPI marks auth and usage bridge commands as async promises", () => {
  for (const signature of [
    /login:\s*\(\)\s*=>\s*Promise<void>;/,
    /logout:\s*\(\)\s*=>\s*Promise<void>;/,
    /checkAuth:\s*\(\)\s*=>\s*Promise<void>;/,
    /fetchUsage:\s*\(\)\s*=>\s*Promise<void>;/,
    /refreshUsage:\s*\(\)\s*=>\s*Promise<void>;/,
    /forceRefreshUsage:\s*\(\)\s*=>\s*Promise<void>;/,
  ]) {
    assert.match(
      appTypesSource,
      signature,
      `Expected async AppAPI signature ${signature}`,
    );
  }
});

test("useAuth awaits async auth bridge commands and preserves backend logout state", () => {
  assert.match(
    useAuthSource,
    /const login = useCallback\(async \(\) => \{[\s\S]*await window\.electron\.login\(\);[\s\S]*\}, \[[^\]]*setError[^\]]*\]\);/,
    "login should await the async bridge call",
  );
  assert.match(
    useAuthSource,
    /const checkAuth = useCallback\(async \(\) => \{[\s\S]*await window\.electron\.checkAuth\(\);[\s\S]*\}, \[[^\]]*setAuthState[^\]]*setError[^\]]*\]\);/,
    "checkAuth should await the async bridge call",
  );
  assert.match(
    useAuthSource,
    /const logout = useCallback\(async \(\) => \{[\s\S]*await window\.electron\.logout\(\);[\s\S]*setAuthState\("unauthenticated"\);[\s\S]*setError\(null\);[\s\S]*catch \(err\)[\s\S]*setError\(/,
    "logout should await backend logout, preserve the unauthenticated auth state, and surface failures",
  );
  assert.doesNotMatch(
    useAuthSource,
    /await window\.electron\.logout\(\);[\s\S]*reset\(\);/,
    "logout should not reset auth state back to unknown after backend logout succeeds",
  );
});

test("useUsage awaits async bridge fetch commands", () => {
  for (const pattern of [
    /await window\.electron\.fetchUsage\(\);/,
    /await window\.electron\.refreshUsage\(\);/,
    /await window\.electron\.forceRefreshUsage\(\);/,
  ]) {
    assert.match(
      useUsageSource,
      pattern,
      `Expected useUsage to await ${pattern}`,
    );
  }
});

test("tauri adapter fetch helpers rely on usage:data events instead of partial success payloads", () => {
  const fetchUsageSection = getSection(
    adapterSource,
    "fetchUsage: async () => {",
    "refreshUsage: async () => {",
  );
  const refreshUsageSection = getSection(
    adapterSource,
    "refreshUsage: async () => {",
    "forceRefreshUsage: async () => {",
  );
  const forceRefreshUsageSection = getSection(
    adapterSource,
    "forceRefreshUsage: async () => {",
    "captureExtractionDebug: async () => {",
  );

  for (const [name, section, command] of [
    ["fetchUsage", fetchUsageSection, '"fetch_usage"'],
    ["refreshUsage", refreshUsageSection, '"fetch_usage"'],
    ["forceRefreshUsage", forceRefreshUsageSection, '"force_fetch_usage"'],
  ]) {
    assert.match(
      section,
      new RegExp(`await invoke<[^>]+>\\(${command}\\);`),
      `${name} should await the backend command`,
    );
    assert.doesNotMatch(
      section,
      /convertUsageData|notifyUsageListeners\(result\)/,
      `${name} should not emit partial success payloads from the adapter`,
    );
    assert.match(
      section,
      /notifyUsageListeners\(\{\s*success:\s*false,/,
      `${name} should still surface explicit failures`,
    );
  }
});

test("tauri adapter rehydrates usage:updated events from cached full payloads", () => {
  const usageUpdatedSection = getSection(
    adapterSource,
    'listen<RustUsageSummary>("usage:updated", () => {',
    'listen<RustUsagePayload>("usage:data", (event) => {',
  );

  assert.match(
    adapterSource,
    /async function getCachedUsageResult\([\s\S]*invoke<RustUsagePayload \| null>\("get_cached_usage_data"\)/,
    "adapter should expose a shared cached usage helper for full payload hydration",
  );
  assert.match(
    usageUpdatedSection,
    /getCachedUsageResult\(invoke\)/,
    "usage:updated should refresh listeners from the cached full payload",
  );
  assert.doesNotMatch(
    usageUpdatedSection,
    /convertUsageData/,
    "usage:updated should not emit a summary-only partial success payload",
  );
});
