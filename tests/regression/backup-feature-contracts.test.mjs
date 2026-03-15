import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

const settingsPath = path.resolve("src/renderer/src/types/settings.ts");
const adapterPath = path.resolve("src/renderer/src/tauri-adapter.ts");
const appTypesPath = path.resolve("src/renderer/src/types/app.ts");
const settingsComponentPath = path.resolve(
  "src/renderer/src/components/settings/Settings.tsx",
);
const storeRsPath = path.resolve("src-tauri/src/store.rs");
const mainRsPath = path.resolve("src-tauri/src/main.rs");

const settingsSource = fs.readFileSync(settingsPath, "utf8");
const adapterSource = fs.readFileSync(adapterPath, "utf8");
const appTypesSource = fs.readFileSync(appTypesPath, "utf8");
const settingsComponentSource = fs.readFileSync(settingsComponentPath, "utf8");
const storeRsSource = fs.readFileSync(storeRsPath, "utf8");
const mainRsSource = fs.readFileSync(mainRsPath, "utf8");

// ─── settings.ts: BackupInfo shape ─────────────────────────────────────────

test("BackupInfo has required fields: backupId, createdAt, files, sizeBytes", () => {
  assert.match(
    settingsSource,
    /interface BackupInfo \{/,
    "BackupInfo interface must be declared in settings.ts",
  );
  const requiredFields = ["backupId", "createdAt", "files", "sizeBytes"];
  for (const field of requiredFields) {
    assert.ok(
      settingsSource.includes(field),
      `BackupInfo must contain field: ${field}`,
    );
  }
});

// ─── settings.ts: BackupFrequency type ─────────────────────────────────────

test("BackupFrequency type includes all four frequency values", () => {
  const requiredValues = [
    '"everyRefresh"',
    '"daily"',
    '"every3Days"',
    '"weekly"',
  ];
  for (const value of requiredValues) {
    assert.ok(
      settingsSource.includes(value),
      `BackupFrequency must include value: ${value}`,
    );
  }
});

// ─── settings.ts: DEFAULT_SETTINGS backup fields ───────────────────────────

test("DEFAULT_SETTINGS has autoBackupEnabled defaulting to false", () => {
  assert.match(
    settingsSource,
    /autoBackupEnabled:\s*false/,
    "DEFAULT_SETTINGS.autoBackupEnabled must default to false",
  );
});

test("DEFAULT_SETTINGS has backupFrequency defaulting to daily", () => {
  assert.match(
    settingsSource,
    /backupFrequency:\s*"daily"/,
    "DEFAULT_SETTINGS.backupFrequency must default to 'daily'",
  );
});

test("DEFAULT_SETTINGS has backupRetentionCount defaulting to 10", () => {
  assert.match(
    settingsSource,
    /backupRetentionCount:\s*10/,
    "DEFAULT_SETTINGS.backupRetentionCount must default to 10",
  );
});

test("DEFAULT_SETTINGS has backupDirectory defaulting to null", () => {
  assert.match(
    settingsSource,
    /backupDirectory:\s*null/,
    "DEFAULT_SETTINGS.backupDirectory must default to null",
  );
});

// ─── settings.ts: BACKUP_FREQUENCY_OPTIONS array ───────────────────────────

test("BACKUP_FREQUENCY_OPTIONS is exported and contains all four entries", () => {
  assert.match(
    settingsSource,
    /export const BACKUP_FREQUENCY_OPTIONS/,
    "BACKUP_FREQUENCY_OPTIONS must be exported from settings.ts",
  );
  const frequencyValues = ["everyRefresh", "daily", "every3Days", "weekly"];
  for (const v of frequencyValues) {
    assert.ok(
      settingsSource.includes(`"${v}"`),
      `BACKUP_FREQUENCY_OPTIONS must include value: ${v}`,
    );
  }
});

// ─── settings.ts: BACKUP_RETENTION_OPTIONS array ───────────────────────────

test("BACKUP_RETENTION_OPTIONS is exported and includes unlimited (0) option", () => {
  assert.match(
    settingsSource,
    /export const BACKUP_RETENTION_OPTIONS/,
    "BACKUP_RETENTION_OPTIONS must be exported from settings.ts",
  );
  assert.match(
    settingsSource,
    /\{ value: 0, label: "Unlimited" \}/,
    "BACKUP_RETENTION_OPTIONS must include an Unlimited (0) option",
  );
});

// ─── settings.ts: Settings interface has backup fields ─────────────────────

test("Settings interface includes all backup-related fields", () => {
  const requiredFields = [
    "autoBackupEnabled",
    "backupFrequency",
    "backupRetentionCount",
    "backupDirectory",
  ];
  for (const field of requiredFields) {
    assert.ok(
      settingsSource.includes(field),
      `Settings interface must include backup field: ${field}`,
    );
  }
});

// ─── tauri-adapter.ts: RustAppSettings includes backup fields ──────────────

test("RustAppSettings interface has all required backup fields", () => {
  const requiredFields = [
    "autoBackupEnabled",
    "backupFrequency",
    "backupRetentionCount",
    "lastAutoBackupAt",
    "backupDirectory",
  ];
  for (const field of requiredFields) {
    assert.ok(
      adapterSource.includes(field),
      `RustAppSettings must include backup field: ${field}`,
    );
  }
});

// ─── tauri-adapter.ts: setSettings merges backup fields ────────────────────

test("setSettings merge preserves all backup fields from current settings", () => {
  // Each field uses null-coalescing to prefer the new value over the current one.
  // The source may format each assignment across two lines, so we check for each
  // field name alongside its null-coalescing pattern independently.
  const fieldPatterns = [
    {
      key: "autoBackupEnabled",
      pattern: /autoBackupEnabled:[\s\S]*?newSettings\.autoBackupEnabled \?\? current\.autoBackupEnabled/,
    },
    {
      key: "backupFrequency",
      pattern: /backupFrequency:[\s\S]*?newSettings\.backupFrequency \?\? current\.backupFrequency/,
    },
    {
      key: "backupRetentionCount",
      pattern:
        /backupRetentionCount:[\s\S]*?newSettings\.backupRetentionCount \?\? current\.backupRetentionCount/,
    },
    {
      key: "backupDirectory",
      pattern: /backupDirectory:[\s\S]*?newSettings\.backupDirectory \?\? current\.backupDirectory/,
    },
  ];
  for (const { key, pattern } of fieldPatterns) {
    assert.match(
      adapterSource,
      pattern,
      `setSettings merge must preserve backup field: ${key}`,
    );
  }
});

// ─── tauri-adapter.ts: backup commands invoke correct backend commands ───────

test("createBackup invokes the create_backup Tauri command", () => {
  assert.match(
    adapterSource,
    /invoke<string>\("create_backup"\)/,
    "createBackup must invoke create_backup command returning string",
  );
});

test("restoreBackup invokes restore_backup with backupId parameter", () => {
  assert.match(
    adapterSource,
    /invoke\("restore_backup",\s*\{ backupId \}\)/,
    "restoreBackup must invoke restore_backup with backupId argument",
  );
});

test("listBackups invokes list_backups command returning BackupInfo array", () => {
  assert.match(
    adapterSource,
    /invoke<BackupInfo\[\]>\("list_backups"\)/,
    "listBackups must invoke list_backups command returning BackupInfo[]",
  );
});

test("deleteBackup invokes delete_backup with backupId parameter", () => {
  assert.match(
    adapterSource,
    /invoke\("delete_backup",\s*\{ backupId \}\)/,
    "deleteBackup must invoke delete_backup with backupId argument",
  );
});

// ─── tauri-adapter.ts: backup methods present in mock adapter ──────────────

test("mock adapter implements createBackup returning mock-backup-id", () => {
  assert.match(
    adapterSource,
    /createBackup:\s*async \(\) => "mock-backup-id"/,
    "mock adapter must implement createBackup",
  );
});

test("mock adapter implements all backup methods", () => {
  const mockMethods = [
    "restoreBackup: async () => {}",
    "listBackups: async () => []",
    "deleteBackup: async () => {}",
  ];
  for (const method of mockMethods) {
    assert.ok(
      adapterSource.includes(method),
      `mock adapter must implement: ${method}`,
    );
  }
});

// ─── app.ts: AppAPI interface has backup methods ────────────────────────────

test("AppAPI interface declares all four backup methods", () => {
  const backupMethods = [
    "createBackup: () => Promise<string>",
    "restoreBackup: (backupId: string) => Promise<void>",
    "listBackups: () => Promise<BackupInfo[]>",
    "deleteBackup: (backupId: string) => Promise<void>",
  ];
  for (const method of backupMethods) {
    assert.ok(
      appTypesSource.includes(method),
      `AppAPI must declare method: ${method}`,
    );
  }
});

test("AppAPI imports BackupInfo from settings types", () => {
  assert.match(
    appTypesSource,
    /import type \{[^}]*BackupInfo[^}]*\} from ["']\.\/settings["']/,
    "app.ts must import BackupInfo from settings types",
  );
});

// ─── Settings.tsx: backup UI calls correct backend commands ─────────────────

test("Settings.tsx calls window.electron.listBackups on mount", () => {
  assert.match(
    settingsComponentSource,
    /window\.electron\.listBackups\(\)/,
    "Settings must call window.electron.listBackups to load backup list",
  );
});

test("Settings.tsx calls window.electron.createBackup for manual backup", () => {
  assert.match(
    settingsComponentSource,
    /window\.electron\.createBackup\(\)/,
    "Settings must call window.electron.createBackup for backup creation",
  );
});

test("Settings.tsx calls window.electron.restoreBackup with backupId", () => {
  assert.match(
    settingsComponentSource,
    /window\.electron\.restoreBackup\(backupId\)/,
    "Settings must call window.electron.restoreBackup with backupId",
  );
});

test("Settings.tsx calls window.electron.deleteBackup with backupId", () => {
  assert.match(
    settingsComponentSource,
    /window\.electron\.deleteBackup\(backupId\)/,
    "Settings must call window.electron.deleteBackup with backupId",
  );
});

test("Settings.tsx imports BACKUP_FREQUENCY_OPTIONS and BACKUP_RETENTION_OPTIONS", () => {
  assert.match(
    settingsComponentSource,
    /BACKUP_FREQUENCY_OPTIONS/,
    "Settings must import BACKUP_FREQUENCY_OPTIONS",
  );
  assert.match(
    settingsComponentSource,
    /BACKUP_RETENTION_OPTIONS/,
    "Settings must import BACKUP_RETENTION_OPTIONS",
  );
});

test("Settings.tsx persists autoBackupEnabled via setSettings call", () => {
  assert.match(
    settingsComponentSource,
    /autoBackupEnabled:\s*newValue/,
    "Settings must persist autoBackupEnabled toggle via setSettings",
  );
});

test("Settings.tsx persists backupFrequency via setSettings call", () => {
  assert.match(
    settingsComponentSource,
    /backupFrequency:\s*option\.value/,
    "Settings must persist backupFrequency selection via setSettings",
  );
});

test("Settings.tsx persists backupRetentionCount via setSettings call", () => {
  assert.match(
    settingsComponentSource,
    /backupRetentionCount:\s*option\.value/,
    "Settings must persist backupRetentionCount selection via setSettings",
  );
});

// ─── store.rs: BackupFrequency enum ────────────────────────────────────────

test("store.rs declares BackupFrequency enum with all four variants", () => {
  const variants = ["EveryRefresh", "Daily", "Every3Days", "Weekly"];
  for (const variant of variants) {
    assert.ok(
      storeRsSource.includes(variant),
      `BackupFrequency enum must have variant: ${variant}`,
    );
  }
});

test("store.rs BackupFrequency uses camelCase serde serialization", () => {
  assert.match(
    storeRsSource,
    /pub enum BackupFrequency/,
    "BackupFrequency enum must be public",
  );
  assert.ok(
    storeRsSource.includes('rename_all = "camelCase"'),
    "BackupFrequency must use camelCase serde rename",
  );
});

test("store.rs BackupFrequency Daily variant is the default", () => {
  assert.match(
    storeRsSource,
    /#\[default\]\s+Daily/,
    "BackupFrequency::Daily must be the default variant",
  );
});

// ─── store.rs: AppSettings backup fields ───────────────────────────────────

test("store.rs AppSettings has all required backup fields", () => {
  const requiredFields = [
    "pub auto_backup_enabled: bool",
    "pub backup_frequency: BackupFrequency",
    "pub backup_retention_count: u32",
    "pub last_auto_backup_at: Option<String>",
    "pub backup_directory: Option<String>",
  ];
  for (const field of requiredFields) {
    assert.ok(
      storeRsSource.includes(field),
      `AppSettings must have field: ${field}`,
    );
  }
});

// ─── store.rs: backup default functions ────────────────────────────────────

test("store.rs default_auto_backup_enabled returns false", () => {
  assert.match(
    storeRsSource,
    /fn default_auto_backup_enabled\(\) -> bool \{\s*false\s*\}/,
    "default_auto_backup_enabled must return false",
  );
});

test("store.rs default_backup_frequency returns Daily", () => {
  assert.match(
    storeRsSource,
    /fn default_backup_frequency\(\) -> BackupFrequency \{\s*BackupFrequency::Daily\s*\}/,
    "default_backup_frequency must return BackupFrequency::Daily",
  );
});

test("store.rs default_backup_retention_count returns 10", () => {
  assert.match(
    storeRsSource,
    /fn default_backup_retention_count\(\) -> u32 \{\s*10\s*\}/,
    "default_backup_retention_count must return 10",
  );
});

// ─── store.rs: should_auto_backup logic ────────────────────────────────────

test("store.rs should_auto_backup returns false when auto_backup_enabled is false", () => {
  assert.match(
    storeRsSource,
    /if !settings\.auto_backup_enabled \{\s*return false;\s*\}/,
    "should_auto_backup must return false when auto_backup_enabled is false",
  );
});

test("store.rs should_auto_backup uses correct threshold hours for each frequency", () => {
  assert.match(
    storeRsSource,
    /BackupFrequency::EveryRefresh => 0/,
    "should_auto_backup must use 0 hours for EveryRefresh",
  );
  assert.match(
    storeRsSource,
    /BackupFrequency::Daily => 24/,
    "should_auto_backup must use 24 hours for Daily",
  );
  assert.match(
    storeRsSource,
    /BackupFrequency::Every3Days => 72/,
    "should_auto_backup must use 72 hours for Every3Days",
  );
  assert.match(
    storeRsSource,
    /BackupFrequency::Weekly => 168/,
    "should_auto_backup must use 168 hours for Weekly",
  );
});

test("store.rs should_auto_backup returns true when no previous backup recorded", () => {
  assert.match(
    storeRsSource,
    /true \/\/ no previous backup recorded, should backup/,
    "should_auto_backup must return true when no previous backup timestamp exists",
  );
});

// ─── store.rs: BackupInfo struct ───────────────────────────────────────────

test("store.rs BackupInfo struct has all required fields with correct types", () => {
  assert.match(
    storeRsSource,
    /pub struct BackupInfo/,
    "BackupInfo struct must be declared in store.rs",
  );
  const fields = [
    "pub backup_id: String",
    "pub created_at: String",
    "pub files: Vec<String>",
    "pub size_bytes: u64",
  ];
  for (const field of fields) {
    assert.ok(
      storeRsSource.includes(field),
      `BackupInfo must have field: ${field}`,
    );
  }
});

test("store.rs BackupInfo uses camelCase serde serialization", () => {
  // Find BackupInfo struct and verify its serde attribute
  const backupInfoIdx = storeRsSource.indexOf("pub struct BackupInfo");
  assert.notEqual(backupInfoIdx, -1, "BackupInfo struct must exist");

  // The camelCase serde rename must appear before the struct
  const nearbyContent = storeRsSource.substring(
    backupInfoIdx - 200,
    backupInfoIdx,
  );
  assert.match(
    nearbyContent,
    /rename_all = "camelCase"/,
    "BackupInfo must have camelCase serde rename_all",
  );
});

// ─── store.rs: StoreManager backup methods exist ───────────────────────────

test("store.rs StoreManager has create_backup, restore_backup, list_backups, delete_backup, prune_backups methods", () => {
  const methods = [
    "pub fn create_backup(",
    "pub fn restore_backup(",
    "pub fn list_backups(",
    "pub fn delete_backup(",
    "pub fn prune_backups(",
  ];
  for (const method of methods) {
    assert.ok(
      storeRsSource.includes(method),
      `StoreManager must implement method: ${method}`,
    );
  }
});

test("store.rs lib.rs exports BackupFrequency and BackupInfo", () => {
  const libRsPath = path.resolve("src-tauri/src/lib.rs");
  const libRsSource = fs.readFileSync(libRsPath, "utf8");
  assert.match(
    libRsSource,
    /BackupFrequency/,
    "lib.rs must export BackupFrequency",
  );
  assert.match(libRsSource, /BackupInfo/, "lib.rs must export BackupInfo");
});

// ─── main.rs: backup Tauri commands are registered ─────────────────────────

test("main.rs registers all four backup Tauri commands", () => {
  const commands = [
    "create_backup",
    "restore_backup",
    "list_backups",
    "delete_backup",
  ];
  for (const cmd of commands) {
    assert.ok(
      mainRsSource.includes(cmd),
      `main.rs must register Tauri command: ${cmd}`,
    );
  }
});

test("main.rs auto-backup logic runs after each usage fetch", () => {
  assert.match(
    mainRsSource,
    /store\.should_auto_backup\(\)/,
    "main.rs must call store.should_auto_backup() after fetching usage",
  );
  assert.match(
    mainRsSource,
    /store\.create_backup\(\)/,
    "main.rs must call store.create_backup() when auto-backup is due",
  );
  assert.match(
    mainRsSource,
    /store\.record_auto_backup_time\(\)/,
    "main.rs must call store.record_auto_backup_time() after successful backup",
  );
});