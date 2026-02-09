# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Codebase Structure Index

The file map below provides instant orientation. For detailed export signatures and dependencies, read the relevant `.claude/structure/*.yaml` file for the directory you're working in.

After adding, removing, or renaming source files or public classes/functions, update both the file map below and the relevant structure YAML file.

### File Map

## src-tauri/src/ (Rust backend)

src-tauri/src/main.rs - Application entry point with tray icon management
src-tauri/src/auth.rs - GitHub authentication and data extraction
src-tauri/src/usage.rs - Usage data fetching and prediction calculations
src-tauri/src/store.rs - Persistent settings and data storage
src-tauri/src/tray_icon_renderer.rs - Custom tray icon rendering with fontdue
src-tauri/src/lib.rs - Module exports and common types

## src/renderer/src/ (React TypeScript frontend)

### components/

src/renderer/src/components/auth/LoginPrompt.tsx - GitHub login interface
src/renderer/src/components/dashboard/Dashboard.tsx - Main usage dashboard
src/renderer/src/components/dashboard/UsageCard.tsx - Usage quota display
src/renderer/src/components/dashboard/UsageChart.tsx - Usage trend visualization
src/renderer/src/components/dashboard/PredictionCard.tsx - Usage predictions
src/renderer/src/components/dashboard/HistoryTable.tsx - Historical usage data
src/renderer/src/components/settings/Settings.tsx - App preferences
src/renderer/src/components/layout/Layout.tsx - App layout wrapper
src/renderer/src/components/layout/Header.tsx - Navigation header
src/renderer/src/components/ui/button.tsx - Reusable button component
src/renderer/src/components/ui/card.tsx - Reusable card component
src/renderer/src/components/ui/progress.tsx - Progress bar display
src/renderer/src/components/ui/skeleton.tsx - Loading placeholder
src/renderer/src/components/ui/tooltip.tsx - Tooltip component
src/renderer/src/components/ui/gauge.tsx - Circular gauge display
src/renderer/src/components/ui/UpdateBanner.tsx - Update notification

### hooks/

src/renderer/src/hooks/useAuth.ts - Authentication management hook
src/renderer/src/hooks/useUsage.ts - Usage data fetching hook
src/renderer/src/hooks/useSettingsSync.ts - Settings synchronization hook
src/renderer/src/hooks/useTheme.ts - Theme management hook
src/renderer/src/hooks/index.ts - Hook exports

### services/

src/renderer/src/services/api.ts - Tauri/Electron API adapter
src/renderer/src/services/predictor.ts - Usage prediction calculations
src/renderer/src/services/index.ts - Service exports

### stores/

src/renderer/src/stores/usageStore.ts - Zustand usage state management
src/renderer/src/stores/settingsStore.ts - Zustand settings state management
src/renderer/src/stores/index.ts - Store exports

### types/

src/renderer/src/types/index.ts - Type definitions
src/renderer/src/types/usage.ts - Usage-related types
src/renderer/src/types/settings.ts - Settings-related types
src/renderer/src/types/electron.ts - Electron API types

### lib/

src/renderer/src/lib/utils.ts - Utility functions

### Root files

src/renderer/src/tauri-adapter.ts - Tauri/Electron API bridge
src/renderer/src/App.tsx - App component entry point
src/renderer/src/main.tsx - React app bootstrap

## src/main/ (Electron main process)

src/main/index.ts - Electron main process entry point
src/main/config.ts - Electron configuration and settings

## src/preload/ (Preload scripts)

src/preload/index.ts - Preload script for main process
src/preload/index.d.ts - Type definitions for preload script

---

## Project Overview

**Copilot Tracker** is a cross-platform desktop application built with Tauri v2 (Rust backend) and React + TypeScript (frontend). It tracks GitHub Copilot usage by extracting data from GitHub's billing page and displays it in a system tray app with usage predictions.

**Key Technologies:**

- **Backend:** Rust with Tauri 2.x
- **Frontend:** React 18, TypeScript, Tailwind CSS, Zustand for state management
- **Build:** electron-vite for development, Tauri CLI for production builds
- **Charts:** Recharts for data visualization

## Development Commands

### Tauri (Primary - Production)

```bash
# Development (Tauri)
npm run tauri:dev

# Build for current platform
npm run tauri:build

# Platform-specific builds
npm run tauri:build:win    # Windows x64
npm run tauri:build:mac    # macOS Universal
npm run tauri:build:linux  # Linux x64

# Lint Rust code
npm run tauri:lint
```

### Electron (Legacy - Still Supported)

```bash
# Development
npm run dev

# Build
npm run build              # Full build with typecheck
npm run build:win          # Windows
npm run build:mac          # macOS
npm run build:linux        # Linux
```

### Common Commands

```bash
# Type checking
npm run typecheck          # Both node and web
npm run typecheck:node
npm run typecheck:web

# Linting/Formatting
npm run lint               # ESLint with auto-fix
npm run format             # Prettier
```

## Architecture

### Backend (Rust - `src-tauri/src/`)

The Rust backend is organized into modules with clear separation of concerns:

- **`main.rs`** - Application entry point, tray icon management, menu building, event handling
- **`auth.rs`** - GitHub authentication via hidden webview with JavaScript injection for data extraction
- **`usage.rs`** - Usage data fetching, history management, and prediction calculations
- **`store.rs`** - Persistent settings storage (JSON files in app data directory)
- **`tray_icon_renderer.rs`** - Custom tray icon rendering using fontdue/tiny-skia

**Key Patterns:**

- Tauri commands are defined with `#[tauri::command]` and registered in `invoke_handler!`
- State is managed using Tauri's `.manage()` API with `Arc<Mutex<T>>` for shared state
- Events are emitted via `app.emit()` for frontend communication
- Hidden webview extraction uses tokio channels for async event handling

### Frontend (React - `src/renderer/src/`)

The frontend uses a feature-based folder structure:

- **`components/`** - UI components organized by feature (dashboard, settings, auth, layout, ui)
- **`hooks/`** - Custom React hooks (useAuth, useUsage, useTheme, useSettingsSync)
- **`stores/`** - Zustand state management (usageStore, settingsStore)
- **`services/`** - API layer and prediction logic
- **`types/`** - TypeScript type definitions

**Key Files:**

- **`tauri-adapter.ts`** - Critical bridge layer that normalizes Tauri/Electron APIs. Uses `window.__TAURI__` when available, falls back to mock. Converts Rust naming conventions (snake_case) to JS conventions (camelCase).

**State Management Pattern:**

- Zustand stores hold the canonical state
- Hooks connect stores to components
- Tauri adapter emits events that update stores
- Components re-render based on store changes

### Authentication Flow

The app uses a sophisticated hidden webview extraction method:

1. User clicks "Login" → `show_auth_window` creates a visible webview to GitHub
2. User logs in normally in the webview
3. `on_navigation` callback intercepts URL for `copilot-auth-success.local`
4. URL hash contains JSON payload with customer_id, usage data, and history
5. Payload is parsed and stored, authentication state emitted
6. For refreshes, `perform_extraction` creates a hidden webview that runs JavaScript to fetch data from GitHub's API endpoints

**Important:** The hidden webview approach is used because GitHub's billing data is only available via authenticated API calls that require session cookies from a logged-in browser session.

### Tray Icon System

The tray icon is dynamically rendered to show usage (e.g., "450/1200"):

- `TrayIconRenderer` in `tray_icon_renderer.rs` renders text as PNG using fontdue
- `update_tray_icon()` updates the tray icon when usage changes
- `build_tray_menu()` creates the full context menu with prediction, history, and settings
- Menu is debounced (max once per second) to prevent performance issues
- macOS uses `ActivationPolicy::Accessory` to hide dock icon when window is closed

### Data Flow

```
User Action → Frontend Hook → Tauri Adapter → Rust Command
                                           ↓
                                    State Update
                                           ↓
                                    Event Emission
                                           ↓
                                    Frontend Listener → Zustand Store → Component Re-render
```

**Events:**

- `auth:state-changed` - Authentication state changes
- `usage:data` - Full usage payload (summary, history, prediction)
- `usage:updated` - Usage summary only (tray icon update)
- `settings:changed` - Settings updates
- `navigate` - Navigation events (dashboard ↔ settings)

## Important Notes

### Platform Differences

- **macOS:** Uses `ActivationPolicy` to control dock icon visibility. Private API enabled in `tauri.conf.json`.
- **Windows:** Uses 1x1 transparent window for hidden webview (off-screen positioning may not work).
- **Linux:** Similar to macOS for tray icon behavior.

### Version Management

- `package.json` contains the Electron version (currently 1.5.1)
- `src-tauri/Cargo.toml` contains the Tauri version (currently 1.5.0)
- Use release-please for automated version bumps (see `.github/`)

### Common Issues

- **Tray menu rebuild spam:** The `build_tray_menu()` function is expensive. A 1-second debounce is implemented in `main.rs`.
- **Auth state sync:** When resetting settings, auth events must be emitted BEFORE settings events to ensure proper frontend state cleanup.
- **Window close behavior:** Window close is intercepted - window hides instead of closing. On macOS, this also hides the dock icon via `ActivationPolicy::Accessory`.

### File Locations

- Settings: `$APPDATA/com.copilottracker/settings.json`
- History: `$APPDATA/com.copilottracker/usage_history.json`
- Font: `src-tauri/assets/fonts/RobotoMono-Medium.ttf`

## Testing

No automated tests are currently present in the codebase. Manual testing is required for:

- Authentication flow (GitHub login)
- Usage data extraction and display
- Tray icon updates
- Settings persistence
- Platform-specific behaviors (dock/taskbar, window management)
