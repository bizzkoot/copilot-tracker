# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

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
