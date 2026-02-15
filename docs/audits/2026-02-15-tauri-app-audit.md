# Tauri App Audit - 2026-02-15

App Type: Open-source desktop app (Tauri + React + Rust)
Scope: Security, Performance, Code Quality, UX/Accessibility

## Summary

- Security posture is mostly solid, but a few permissions and data handling choices can be tightened.
- Performance is acceptable, with clear opportunities to reduce repeated work and heavy log output.
- Code quality is generally good; there are a few inconsistencies and duplication points worth refactoring.
- UX is strong, but some accessibility and interaction improvements are needed.

## Findings

### 🚨 CRITICAL: Shell execute permission enabled without usage (DONE)
Where: `src-tauri/capabilities/default.json`
Why: `shell:allow-execute` is enabled but no Rust or frontend code uses shell execution. This expands attack surface unnecessarily.
Evidence: No usage found in `src-tauri/src/*.rs` for shell execution; only plugin init is present.
How: Remove `shell:allow-execute` from `src-tauri/capabilities/default.json`. Keep `shell:allow-open` for URL opens (used in `src-tauri/src/main.rs`).

### ⚠️ MAJOR: Electron dependencies still installed in a Tauri-only app (DONE)
Where: `package.json`
Why: `electron`, `electron-vite`, `electron-builder`, `@electron-toolkit/*`, and `electron-store` remain in dependencies/devDependencies. This increases install size, security exposure, and developer confusion.
How: Remove Electron packages and any remaining Electron-only tooling references from `package.json` and `package-lock.json` after you confirm no code paths import them. Keep a dedicated legacy archive under `temp/electron/` as you already started.

### ⚠️ MAJOR: Excessive logging in production paths (DONE)
Where: `src/renderer/src/hooks/useUsage.ts`, `src/renderer/src/hooks/useAuth.ts`, `src/renderer/src/tauri-adapter.ts`
Why: Production logging includes usage payloads, history rows, and raw API data. This risks leaking sensitive usage data into logs and can degrade performance.
How: Gate these logs behind a dev flag (e.g., `import.meta.env.DEV`) or wrap with a debug helper. Remove raw usage dumps in production builds.

### ⚠️ MAJOR: Hidden webview auth extraction relies on DOM scraping without versioning guards
Where: `src-tauri/src/auth.rs` (inline JS injection) and `src/renderer/src/services/api.ts`
Why: The GitHub billing DOM structure is not stable. A minor change can silently break auth extraction or usage parsing, yielding empty data without clear user feedback.
How: Add a server-side schema/version check or explicit extraction failure state surfaced to UI. Consider a retry strategy with user-visible error when extraction fails.

### ⚠️ MAJOR: Shell and HTTP permissions are broad (DONE)
Where: `src-tauri/capabilities/default.json`, `src-tauri/capabilities/remote-github.json`
Why: `http:default` and broad window permissions are enabled globally. For an app that only calls GitHub, least-privilege scoping should be used.
How: Replace `http:default` with a restricted allowlist (GitHub domains only). Narrow window permissions to the minimum used by each window.

### 🔧 MINOR: Prediction logic duplicated between Rust and TS (DONE)
Where: `src-tauri/src/usage.rs` and `src/renderer/src/services/predictor.ts`
Why: Two prediction implementations can diverge and produce inconsistent forecasts between tray and dashboard.
How: Choose a single prediction source of truth (prefer Rust for consistency) and expose results to the renderer only.

### 🔧 MINOR: Inconsistent type usage for AuthState (DONE)
Where: `src/renderer/src/hooks/useAuth.ts`, `src/renderer/src/stores/usageStore.ts`
Why: Both import `AuthState` from `../types/electron`, which is a legacy name in a Tauri-only app. This is semantically confusing for contributors.
How: Rename the type file or export it from a neutral path (e.g., `types/app.ts`) and update imports.

### 🔧 MINOR: Widget drag UX can create unintended drags
Where: `src/renderer/src/components/widget/WidgetHeader.tsx`
Why: Drag starts on any mousedown except buttons. Clicking text or empty space triggers drag, which may feel jumpy.
How: Add a small drag threshold (e.g., 4-6px) before treating as a drag, or require grab only from a dedicated drag handle region.

### 🧹 NITPICK: README and package metadata mismatch
Where: `package.json`, `README.md`
Why: `description` still says Electron, while README is Tauri-only.
How: Update `description` in `package.json` to remove Electron mention.

## Recommended Action Plan

P0 (Immediate)
- Remove `shell:allow-execute` permission.
- Gate or remove verbose logs that include usage payloads.

P1 (Soon)
- Remove Electron dependencies and update package metadata.
- Restrict HTTP capability to GitHub domains only.
- Add user-visible failure states for auth extraction and usage parsing.

P2 (Backlog)
- Consolidate prediction logic into a single source of truth.
- Improve widget drag UX with a threshold or handle.
- Rename `electron` types to neutral app types.

## Evidence References

- `src-tauri/capabilities/default.json`
- `src-tauri/capabilities/remote-github.json`
- `src-tauri/src/main.rs`
- `src-tauri/src/auth.rs`
- `src-tauri/src/usage.rs`
- `src/renderer/src/hooks/useUsage.ts`
- `src/renderer/src/hooks/useAuth.ts`
- `src/renderer/src/services/api.ts`
- `src/renderer/src/components/widget/WidgetHeader.tsx`
- `package.json`
