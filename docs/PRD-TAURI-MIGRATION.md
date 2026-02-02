# PRD: Tauri Migration for Copilot Tracker

## Document Information

| Field       | Value                  |
| ----------- | ---------------------- |
| **Version** | 1.0                    |
| **Date**    | 2026-02-02             |
| **Status**  | Draft / Research Phase |
| **Author**  | Development Team       |

---

## Executive Summary

This document provides a comprehensive analysis of migrating Copilot Tracker from Electron to Tauri v2. The analysis covers technical feasibility, risk assessment, migration strategy, and recommendations for a phased approach with a "beta" release strategy.

### Key Recommendation

**Implement a dual-packaging strategy**: Ship Tauri as a beta alongside the stable Electron version to minimize risk while gaining real-world validation.

---

## 1. Current Architecture Analysis

### 1.1 Application Overview

Copilot Tracker is a desktop application that:

- Monitors GitHub Copilot usage via WebView-based authentication
- Displays real-time usage statistics in a dashboard
- Provides system tray integration with live usage data
- Offers prediction algorithms for end-of-month usage
- Supports auto-updates via GitHub releases

### 1.2 Current Technology Stack

| Component        | Technology                |
| ---------------- | ------------------------- |
| Framework        | Electron 31.0.2           |
| Build Tool       | electron-vite 2.3.0       |
| Frontend         | React 18.3.1 + TypeScript |
| Styling          | Tailwind CSS 3.4.4        |
| State Management | Zustand 4.5.2             |
| Charts           | Recharts 2.12.7           |
| Local Storage    | electron-store 8.2.0      |
| Native Graphics  | canvas (node module)      |
| Packaging        | electron-builder          |

### 1.3 Electron-Specific Features Used

| Feature                      | Usage                                    | Migration Complexity             |
| ---------------------------- | ---------------------------------------- | -------------------------------- |
| **WebContentsView**          | GitHub OAuth via embedded WebView        | **HIGH** - Core auth mechanism   |
| **System Tray**              | Custom icon with canvas-rendered numbers | **MEDIUM** - Different API       |
| **IPC (Main↔Renderer)**      | 15+ channels for auth, usage, settings   | **MEDIUM** - Different patterns  |
| **electron-store**           | Settings & cache persistence             | **LOW** - Plugin available       |
| **BrowserWindow**            | Main window + Auth window                | **LOW** - Standard feature       |
| **shell.openExternal**       | External URL opening                     | **LOW** - Plugin available       |
| **Notification**             | Update alerts                            | **LOW** - Plugin available       |
| **app.setLoginItemSettings** | Launch at login                          | **LOW** - Plugin available       |
| **canvas (node)**            | Dynamic tray icon rendering              | **HIGH** - Needs native solution |

---

## 2. Tauri v2 Capability Analysis

### 2.1 Feature Mapping

| Electron Feature | Tauri v2 Equivalent            | Status          |
| ---------------- | ------------------------------ | --------------- |
| BrowserWindow    | WebviewWindow                  | Fully supported |
| System Tray      | `tauri::tray::TrayIconBuilder` | Fully supported |
| IPC              | Tauri Commands + Events        | Fully supported |
| electron-store   | `tauri-plugin-store`           | Fully supported |
| Notifications    | `tauri-plugin-notification`    | Fully supported |
| Auto-updater     | `tauri-plugin-updater`         | Fully supported |
| Shell operations | `tauri-plugin-shell`           | Fully supported |
| Auto-start       | `tauri-plugin-autostart`       | Fully supported |
| HTTP Client      | `tauri-plugin-http`            | Fully supported |

### 2.2 Critical Gap: WebContentsView for Authentication

Reference: A reliability-first checklist for webview auth and session extraction is maintained in docs/tauri-migration/tauri-v2-auth-webview-checklist.md.

Summary: Embedded webview auth reliability is most sensitive to redirect handling, HttpOnly cookie access limits, and cross-platform persistence differences. The checklist captures required safeguards (navigation stabilization, persistence verification, Rust-side cookie access when available, and recovery paths) to reduce auth failure rates during the Tauri v2 POC.

**Current Electron Implementation:**

```typescript
// Creates an embedded WebView with GitHub session
authView = new WebContentsView({
  webPreferences: {
    partition: "persist:github", // Persistent session
    nodeIntegration: false,
    contextIsolation: true,
  },
});
authView.webContents.loadURL("https://github.com/settings/billing");

// Execute JS in context of authenticated page
const result = await authView.webContents.executeJavaScript(`
  // Extract data from GitHub DOM
`);
```

**Tauri Approach Options:**

| Option                         | Description                                    | Feasibility                                    |
| ------------------------------ | ---------------------------------------------- | ---------------------------------------------- |
| **1. External Browser OAuth**  | Open system browser, use deep-linking callback | Not applicable - no GitHub OAuth API           |
| **2. tauri-plugin-http**       | Direct HTTP requests with stored cookies       | **Requires GitHub API access** (not available) |
| **3. Hidden WebviewWindow**    | Use a secondary hidden window for auth         | **Promising** - Can inject JS                  |
| **4. Multiwebview (unstable)** | Tauri v2's experimental multiwebview           | **Experimental** - Not production-ready        |

**Analysis of Option 3 (Most Viable):**

```rust
// Tauri v2: Use a hidden WebviewWindow for auth
use tauri::WebviewWindowBuilder;

let auth_window = WebviewWindowBuilder::new(&app, "auth",
    tauri::WebviewUrl::External("https://github.com/settings/billing".parse()?))
    .visible(false)
    .build()?;

// Execute JavaScript in the webview context
auth_window.eval("window.extractCustomerId();")?;
```

**Challenges:**

1. JavaScript injection is limited compared to Electron's `executeJavaScript`
2. Cookie/session persistence needs investigation
3. No direct equivalent to `webContents.on('did-navigate')`

### 2.3 Critical Gap: Dynamic Tray Icon with Canvas

**Current Implementation:**

```typescript
// Uses node-canvas to render numbers on tray icon
const canvasModule = require("canvas");
const canvas = createCanvas(32, 16);
const ctx = canvas.getContext("2d");
ctx.fillText(usageCount.toString(), 16, 12);
const dataUrl = canvas.toDataURL();
tray.setImage(nativeImage.createFromDataURL(dataUrl));
```

**Tauri Approach Options:**

| Option                         | Description                               | Feasibility                               |
| ------------------------------ | ----------------------------------------- | ----------------------------------------- |
| **1. Pre-rendered Icons**      | Generate all possible icons at build time | **Not practical** - Too many combinations |
| **2. Rust Image Manipulation** | Use `image` crate to generate icons       | **Viable** - Native Rust solution         |
| **3. Headless Canvas (Skia)**  | Use `tiny-skia` for 2D rendering          | **Viable** - Full canvas-like API         |
| **4. Text-only Tray**          | Simplify to text tooltip only             | **Compromise** - Reduced functionality    |

**Recommended: Option 3 (tiny-skia)**

```rust
use tiny_skia::*;
use tauri::image::Image;

fn create_tray_icon(used: u32, limit: u32) -> Image {
    let mut pixmap = Pixmap::new(32, 16).unwrap();
    // Draw text and progress indicator
    let mut paint = Paint::default();
    paint.set_color_rgba8(255, 255, 255, 255);
    // ... rendering logic
    Image::new_owned(pixmap.data().to_vec(), 32, 16)
}
```

---

## 3. Migration Impact Analysis

### 3.1 Bundle Size Comparison

| Metric           | Electron    | Tauri (Expected) | Improvement |
| ---------------- | ----------- | ---------------- | ----------- |
| DMG (macOS)      | ~180 MB     | ~15-25 MB        | **~85-90%** |
| NSIS (Windows)   | ~150 MB     | ~8-15 MB         | **~90%**    |
| AppImage (Linux) | ~200 MB     | ~10-20 MB        | **~90%**    |
| Memory Usage     | ~150-300 MB | ~30-50 MB        | **~80%**    |

### 3.2 Performance Expectations

| Aspect       | Impact         | Notes                              |
| ------------ | -------------- | ---------------------------------- |
| Startup Time | **Faster**     | No Chromium cold start             |
| Memory Usage | **Much Lower** | System WebView vs bundled Chromium |
| CPU Usage    | **Lower**      | More efficient process model       |
| Disk I/O     | **Lower**      | Smaller binaries, less caching     |

### 3.3 Platform Considerations

| Platform | WebView Engine  | Considerations                                  |
| -------- | --------------- | ----------------------------------------------- |
| macOS    | WKWebView       | Excellent support, consistent behavior          |
| Windows  | WebView2 (Edge) | Requires WebView2 runtime (often pre-installed) |
| Linux    | WebKitGTK       | May require installation on some distros        |

**Windows WebView2 Consideration:**

- Windows 11: Pre-installed (Evergreen runtime)
- Windows 10 (recent): Usually installed via updates, but not guaranteed
- Windows 10 (older/managed): May be missing; runtime detection required
- Production should use the WebView2 Runtime (not Edge Stable)
- Recommended flow: detect at install or first launch; if missing, run the Evergreen bootstrapper silently and provide a retry/manual install path
- Runtime updates apply on next start; prompt for restart if a new runtime version is detected

---

## 4. Risk Assessment

### 4.1 High-Risk Items

| Risk                                 | Impact   | Probability | Mitigation                            |
| ------------------------------------ | -------- | ----------- | ------------------------------------- |
| **Authentication Flow Breaking**     | Critical | Medium      | Extensive testing, fallback mechanism |
| **Session Persistence Issues**       | High     | Medium      | Cookie management research            |
| **JavaScript Injection Limitations** | High     | Medium      | Alternative data extraction methods   |
| **WebView Inconsistencies**          | Medium   | Medium      | Platform-specific testing             |

### 4.2 Medium-Risk Items

| Risk                         | Impact | Probability | Mitigation                     |
| ---------------------------- | ------ | ----------- | ------------------------------ |
| Tray Icon Rendering Quality  | Medium | Low         | Use high-quality Rust graphics |
| Auto-update Compatibility    | Medium | Low         | Thorough update flow testing   |
| Linux WebKitGTK Availability | Medium | Low         | Document requirements          |

### 4.3 Low-Risk Items

| Risk                     | Impact | Probability | Mitigation               |
| ------------------------ | ------ | ----------- | ------------------------ |
| Settings Migration       | Low    | Low         | Provide migration script |
| UI Rendering Differences | Low    | Low         | Minor CSS adjustments    |

---

## 5. Recommended Migration Strategy

### 5.1 Dual-Package "Beta" Approach

**Rationale:** Given the critical authentication mechanism's uncertainty, ship Tauri as beta alongside stable Electron.

```
copilot-tracker-v1.3.0.dmg          (Electron - Stable)
copilot-tracker-v1.3.0-tauri-beta.dmg  (Tauri - Beta)
```

### 5.2 Implementation Phases

#### Phase 1: Proof of Concept (2-3 weeks)

**Goal:** Validate authentication flow feasibility

- [ ] Initialize Tauri v2 project structure
- [ ] Implement basic window with React frontend
- [ ] Test WebviewWindow for GitHub authentication
- [ ] Verify JavaScript injection capabilities
- [ ] Test session/cookie persistence

**Exit Criteria:**

- [ ] Successfully authenticate and fetch usage data
- [ ] Session persists across app restart on macOS/Windows/Linux
- [ ] Navigation handling is stable across multi-step redirects and 2FA
- [ ] Documented, working path for session/cookie access (no JS-only extraction)

#### Phase 2: Core Features (3-4 weeks)

**Goal:** Implement all core functionality

- [ ] Migrate IPC handlers to Tauri commands
- [ ] Implement `tauri-plugin-store` for settings
- [ ] Create system tray with menu
- [ ] Implement dynamic tray icon (tiny-skia)
- [ ] Add notifications support
- [ ] Implement auto-start functionality

**Exit Criteria:** Feature parity with Electron version (except edge cases)

#### Phase 3: Polish & Testing (2-3 weeks)

**Goal:** Production-ready beta

- [ ] Cross-platform testing (macOS, Windows, Linux)
- [ ] Performance benchmarking
- [ ] Auto-update implementation
- [ ] Settings migration from Electron version
- [ ] Edge case handling
- [ ] Beta documentation

**Exit Criteria:** Stable beta release

#### Phase 4: Beta Release & Feedback (4-8 weeks)

**Goal:** Gather real-world feedback

- [ ] Release as beta channel
- [ ] Monitor crash reports
- [ ] Gather user feedback
- [ ] Iterate on issues
- [ ] Document platform-specific quirks

**Exit Criteria:** No critical issues for 2+ weeks

#### Phase 5: Transition (2-4 weeks)

**Goal:** Full migration or informed decision

- [ ] Evaluate beta feedback
- [ ] Decide: Full transition, continue dual-packaging, or abandon
- [ ] If transitioning: Deprecation notice for Electron version
- [ ] Update release pipeline

---

## 6. Technical Implementation Details

### 6.1 Project Structure

```
copilot-tracker/
├── src/                    # Shared frontend code
│   └── renderer/          # React frontend (reused)
├── src-electron/          # Electron main process (existing)
│   └── main/
│       └── index.ts
├── src-tauri/             # NEW: Tauri backend
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities/
│   │   └── default.json
│   └── src/
│       ├── lib.rs
│       ├── commands/      # IPC command handlers
│       ├── auth.rs        # Authentication logic
│       ├── tray.rs        # System tray logic
│       └── store.rs       # Settings/cache
└── package.json           # Updated scripts
```

### 6.2 Tauri Configuration

```json
// src-tauri/tauri.conf.json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Copilot Tracker",
  "version": "1.3.0-beta",
  "identifier": "com.copilot-tracker.app",
  "build": {
    "frontendDist": "../dist-renderer",
    "devUrl": "http://localhost:5173"
  },
  "app": {
    "withGlobalTauri": true,
    "trayIcon": {
      "iconPath": "icons/tray.png",
      "iconAsTemplate": true
    },
    "windows": [
      {
        "title": "Copilot Tracker",
        "width": 900,
        "height": 700,
        "minWidth": 600,
        "minHeight": 500,
        "visible": false,
        "decorations": true
      }
    ],
    "security": {
      "csp": null
    }
  },
  "plugins": {
    "updater": {
      "pubkey": "YOUR_PUBLIC_KEY",
      "endpoints": [
        "https://api.github.com/repos/bizzkoot/copilot-tracker/releases/latest"
      ]
    }
  },
  "bundle": {
    "active": true,
    "createUpdaterArtifacts": "v1Compatible",
    "icon": ["icons/icon.png", "icons/icon.icns"],
    "macOS": {
      "dmg": {
        "appPosition": { "x": 180, "y": 170 },
        "applicationFolderPosition": { "x": 480, "y": 170 }
      }
    },
    "windows": {
      "nsis": {
        "shortcutName": "Copilot Tracker"
      }
    }
  }
}
```

### 6.3 IPC Migration Example

**Electron (current):**

```typescript
// Main process
ipcMain.on("usage:fetch", () => fetchUsageData());
ipcMain.handle("settings:get", () => store.get("settings"));

// Preload
ipcRenderer.send("usage:fetch");
const settings = await ipcRenderer.invoke("settings:get");
```

**Tauri (new):**

```rust
// src-tauri/src/commands/usage.rs
#[tauri::command]
async fn fetch_usage(state: tauri::State<'_, AppState>) -> Result<UsageData, String> {
    // Fetch logic
    Ok(usage_data)
}

#[tauri::command]
async fn get_settings(state: tauri::State<'_, AppState>) -> Result<Settings, String> {
    state.store.get("settings").map_err(|e| e.to_string())
}
```

```typescript
// Frontend
import { invoke } from "@tauri-apps/api/core";

const usage = await invoke<UsageData>("fetch_usage");
const settings = await invoke<Settings>("get_settings");
```

### 6.4 Authentication Flow (Proposed)

```rust
// src-tauri/src/auth.rs
use tauri::{WebviewWindow, WebviewWindowBuilder, Manager};

pub struct AuthManager {
    auth_window: Option<WebviewWindow>,
}

impl AuthManager {
    pub async fn start_auth(&mut self, app: &tauri::AppHandle) -> Result<(), String> {
        // Create hidden auth window
        let auth_window = WebviewWindowBuilder::new(
            app,
            "github-auth",
            tauri::WebviewUrl::External(
                "https://github.com/settings/billing/premium_requests_usage".parse().unwrap()
            )
        )
        .visible(false)
        .build()
        .map_err(|e| e.to_string())?;

        // Listen for navigation
        auth_window.on_navigation(|url| {
            if url.path().contains("/login") {
                // User needs to authenticate
                // Show the window
            }
            true
        });

        self.auth_window = Some(auth_window);
        Ok(())
    }

    pub async fn extract_customer_id(&self) -> Result<u64, String> {
        if let Some(window) = &self.auth_window {
            // Use window.eval() or listen for custom events from injected JS
            // This is the uncertain part that needs POC validation
        }
        Err("No auth window".to_string())
    }
}
```

---

## 7. Dependencies for Tauri Build

### 7.1 Cargo Dependencies

```toml
# src-tauri/Cargo.toml
[dependencies]
tauri = { version = "2", features = ["tray-icon", "image-png"] }
tauri-plugin-store = "2"
tauri-plugin-notification = "2"
tauri-plugin-shell = "2"
tauri-plugin-autostart = "2"
tauri-plugin-updater = "2"
tauri-plugin-http = "2"
tauri-plugin-process = "2"
tiny-skia = "0.11"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.12", features = ["json"] }
tokio = { version = "1", features = ["full"] }
semver = "1"

[target."cfg(target_os = \"macos\")".dependencies]
cocoa = "0.25"
```

### 7.2 NPM Scripts Update

```json
{
  "scripts": {
    "dev": "electron-vite dev",
    "dev:tauri": "tauri dev",
    "build": "electron-vite build",
    "build:tauri": "tauri build",
    "build:tauri:mac": "tauri build --target universal-apple-darwin",
    "build:tauri:win": "tauri build --target x86_64-pc-windows-msvc",
    "build:tauri:linux": "tauri build --target x86_64-unknown-linux-gnu"
  }
}
```

---

## 8. Success Metrics

### 8.1 Technical Metrics

| Metric            | Target      | Measurement    |
| ----------------- | ----------- | -------------- |
| Auth Success Rate | 99%+        | Analytics      |
| App Startup Time  | <2 seconds  | Benchmarks     |
| Memory Usage      | <50 MB idle | Profiling      |
| Bundle Size       | <25 MB      | Build output   |
| Crash Rate        | <0.1%       | Error tracking |

### 8.2 User Experience Metrics

| Metric            | Target        | Measurement   |
| ----------------- | ------------- | ------------- |
| Beta Adoption     | 20%+ of users | Downloads     |
| User Satisfaction | 4+ stars      | Feedback      |
| Bug Reports       | <5 critical   | GitHub issues |
| Feature Requests  | Document gaps | GitHub issues |

---

## 9. Rollback Plan

If Tauri migration fails or is deemed too risky:

1. **Immediate:** Continue Electron-only releases
2. **Short-term:** Archive Tauri branch for future evaluation
3. **Long-term:** Re-evaluate when:
   - Tauri's WebView APIs mature
   - GitHub provides official OAuth API
   - Community solutions emerge

---

## 10. Appendices

### A. Electron vs Tauri Feature Comparison

| Feature          | Electron            | Tauri v2               |
| ---------------- | ------------------- | ---------------------- |
| WebView Engine   | Chromium (bundled)  | System WebView         |
| Backend Language | Node.js             | Rust                   |
| IPC Model        | Main/Renderer       | Commands + Events      |
| Bundle Size      | Large (~150-200MB)  | Small (~10-25MB)       |
| Memory Usage     | High (~150-300MB)   | Low (~30-50MB)         |
| Startup Time     | Slow (2-5s)         | Fast (<1s)             |
| Security         | Depends on config   | Secure by default      |
| Mobile Support   | No                  | Yes (iOS/Android)      |
| Learning Curve   | Low (JS everywhere) | Medium (Rust required) |

### B. Tauri Plugin Ecosystem

Needed plugins for feature parity:

- `tauri-plugin-store` - Settings persistence
- `tauri-plugin-notification` - System notifications
- `tauri-plugin-shell` - Open external URLs
- `tauri-plugin-autostart` - Launch at login
- `tauri-plugin-updater` - Auto-updates
- `tauri-plugin-http` - HTTP requests
- `tauri-plugin-process` - App lifecycle

### C. References

- [Tauri v2 Documentation](https://v2.tauri.app/)
- [Tauri Migration Guide](https://v2.tauri.app/start/migrate/from-tauri-1/)
- [Tauri System Tray](https://v2.tauri.app/learn/system-tray/)
- [Tauri Store Plugin](https://v2.tauri.app/plugin/store/)
- [tiny-skia Crate](https://github.com/ArtskydJ/tiny-skia)

---

## 11. Decision Required

Before proceeding, stakeholder input needed on:

1. **Resource Allocation:** Dedicate 1-2 developers for 8-12 weeks?
2. **Risk Tolerance:** Accept potential authentication flow uncertainty?
3. **Beta Strategy:** Ship as optional beta or hold for full replacement?
4. **Timeline:** Target release for beta?

---

_This document will be updated as the POC phase reveals new information._
