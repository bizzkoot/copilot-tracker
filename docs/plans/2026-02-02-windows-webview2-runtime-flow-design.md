# Windows WebView2 Runtime Handling (Evergreen Bootstrapper)

## Purpose

Ensure Copilot Tracker launches reliably on Windows by detecting and installing the Evergreen WebView2 runtime when missing, while preserving shared session behavior across app windows and managing persistence safely.

## Goals

- Prevent blank window startup failures due to missing WebView2 runtime.
- Keep installer size small and rely on the shared Evergreen runtime.
- Provide clear user recovery paths and safe persistence defaults.

## Non-Goals

- Shipping Fixed Version runtime in the default release.
- Supporting air-gapped installs in the base installer.

## Runtime Detection and Install Flow

1. On first launch or post-install, check for WebView2 runtime presence via registry or API.
2. If missing, show a blocking dialog with a one-click install action.
3. Run the Evergreen bootstrapper silently, then relaunch the app on success.
4. On failure or offline state, show retry plus a manual download link.
5. Do not uninstall the runtime when the app is removed.

## Implementation Notes

- Missing runtime cannot render a webview window, so use a native dialog (tauri-plugin-dialog) for the install/retry/manual flow.
- Gate window creation by setting startup windows to create=false in config and creating windows only after runtime validation.
- Registry-based detection (Evergreen Runtime): read pv (REG_SZ) from:
  - HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}
  - HKCU\Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}
- Bootstrapper command: MicrosoftEdgeWebview2Setup.exe /silent /install
- Prefer a shared WebView2 user data folder across windows by setting a single absolute data_directory via WebviewWindowBuilder in Rust.

## Persistence and Multi-Window Sharing

- Use one shared user data folder (UDF) per OS user to keep cookies and storage consistent across windows.
- Store the UDF in the app data directory to guarantee write permissions.
- If isolated sessions are required later, use a separate UDF or profile and make it explicit to the user.

## Updates and Restart Handling

- Evergreen updates apply on new WebView2 environments; long-running sessions should prompt for restart when a new runtime version is detected.
- If direct runtime events are not exposed, compare runtime versions across launches and show a soft restart prompt.

## Error Handling

- Distinguish missing runtime vs UDF permission errors and provide targeted recovery guidance.
- Do not delete UDF while a WebView2 session is active; close all windows first.

## UI Copy

### Missing runtime dialog

- Title: "WebView2 runtime required"
- Body: "Copilot Tracker needs Microsoft WebView2 to display the app UI. It is a small, one-time install shared with other apps."
- Primary: "Install WebView2"
- Secondary: "Try again"
- Link: "Install manually from Microsoft's download page."

### Restart prompt

- Title: "A WebView2 update is ready"
- Body: "Restart Copilot Tracker to apply the latest security updates."
- Primary: "Restart now"
- Secondary: "Later"

### Sign-out confirmation

- Title: "Sign out and clear web data?"
- Body: "This will remove stored cookies and site data used by Copilot Tracker. You will need to sign in again."
- Primary: "Sign out"
- Secondary: "Cancel"
