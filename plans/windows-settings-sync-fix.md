# Fix Plan: Windows Settings File Sync Error (OS Error 5)

## Problem Summary

**Error:** `Failed to sync settings file: Access is denied. (os error 5)`

**Affected Version:** 2.4.1 (Windows builds)

**Root Cause:** The [`save_settings_to_disk`](src-tauri/src/store.rs:224) function uses `std::fs::write` followed by reopening the file for `sync_all()`. On Windows, this creates a race condition where the file may still be locked when attempting to reopen it.

## Current Implementation Problem

```rust
// src-tauri/src/store.rs:224-238
fn save_settings_to_disk(path: &PathBuf, settings: &AppSettings) -> Result<(), String> {
    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    std::fs::write(path, content)  // Opens, writes, closes file
        .map_err(|e| format!("Failed to write settings file: {}", e))?;

    // PROBLEM: Reopening immediately after write
    let file = std::fs::File::open(path)  // Can fail with "Access denied" on Windows
        .map_err(|e| format!("Failed to open settings file for sync: {}", e))?;
    file.sync_all()
        .map_err(|e| format!("Failed to sync settings file: {}", e))?;

    Ok(())
}
```

## Why This Affects Windows But Not macOS/Linux

| Factor        | Windows                 | macOS/Linux             |
| ------------- | ----------------------- | ----------------------- |
| File Locking  | Strict exclusive locks  | Permissive shared locks |
| Lock Duration | Longer (OS + antivirus) | Shorter                 |
| Antivirus     | Common (Defender, etc.) | Rare                    |
| Race Window   | 10-100ms                | <1ms                    |

## Cross-Platform Solution

### Option A: Single File Handle (Recommended)

Use a single file handle for both write and sync operations:

```rust
fn save_settings_to_disk(path: &PathBuf, settings: &AppSettings) -> Result<(), String> {
    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    // Create parent directory if needed
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create settings directory: {}", e))?;
        }
    }

    // Use single file handle for write and sync
    use std::io::Write;
    let mut file = std::fs::File::create(path)
        .map_err(|e| format!("Failed to create settings file: {}", e))?;

    file.write_all(content.as_bytes())
        .map_err(|e| format!("Failed to write settings file: {}", e))?;

    file.sync_all()
        .map_err(|e| format!("Failed to sync settings file: {}", e))?;

    Ok(())
}
```

**Benefits:**

- Single file handle eliminates race condition
- Works on all platforms
- Maintains sync guarantee for crash safety
- Simpler, more idiomatic Rust

### Option B: Atomic Write with Temp File

For maximum safety, write to a temp file then rename:

```rust
fn save_settings_to_disk(path: &PathBuf, settings: &AppSettings) -> Result<(), String> {
    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    // Write to temp file first
    let temp_path = path.with_extension("json.tmp");

    use std::io::Write;
    let mut file = std::fs::File::create(&temp_path)
        .map_err(|e| format!("Failed to create temp settings file: {}", e))?;

    file.write_all(content.as_bytes())
        .map_err(|e| format!("Failed to write settings file: {}", e))?;

    file.sync_all()
        .map_err(|e| format!("Failed to sync settings file: {}", e))?;

    // Atomic rename
    std::fs::rename(&temp_path, path)
        .map_err(|e| format!("Failed to rename settings file: {}", e))?;

    Ok(())
}
```

**Benefits:**

- Atomic operation - no partial writes
- Works on all platforms
- Maximum crash safety
- Standard pattern for config files

**Drawbacks:**

- Slightly more complex
- Leaves temp files on failure (cleanup needed)

## Recommended Implementation

**Use Option A (Single File Handle)** as the primary fix because:

1. Simpler implementation
2. Solves the immediate Windows issue
3. Maintains existing behavior
4. No temp file cleanup needed

## Files to Modify

1. **[`src-tauri/src/store.rs`](src-tauri/src/store.rs)**
   - Update `save_settings_to_disk` function (lines 224-238)
   - Consider updating `save_history_to_disk` for consistency (lines 252-260)

## Testing Checklist

- [ ] Windows: Theme toggle works without errors
- [ ] Windows: Settings persist after app restart
- [ ] Windows: Settings persist after crash/force-close
- [ ] macOS: No regression in settings behavior
- [ ] Linux: No regression in settings behavior
- [ ] All platforms: Rapid settings changes work correctly

## Additional Considerations

### Why `sync_all` Was Added

The `sync_all()` call ensures data is flushed to disk, which is important for:

- Crash recovery
- Power failure protection
- Ensuring settings persist after unexpected shutdown

**Do NOT remove `sync_all()`** - it's a safety feature.

### History File Consistency

The `save_history_to_disk` function doesn't have `sync_all()`. Consider adding it for consistency:

```rust
fn save_history_to_disk(path: &PathBuf, history: &Vec<UsageEntry>) -> Result<(), String> {
    let content = serde_json::to_string_pretty(history)
        .map_err(|e| format!("Failed to serialize history: {}", e))?;

    use std::io::Write;
    let mut file = std::fs::File::create(path)
        .map_err(|e| format!("Failed to create history file: {}", e))?;

    file.write_all(content.as_bytes())
        .map_err(|e| format!("Failed to write history file: {}", e))?;

    file.sync_all()
        .map_err(|e| format!("Failed to sync history file: {}", e))?;

    Ok(())
}
```

## Implementation Steps

1. Modify `save_settings_to_disk` to use single file handle
2. Modify `save_history_to_disk` for consistency
3. Test on Windows, macOS, and Linux
4. Verify settings persistence after crashes
