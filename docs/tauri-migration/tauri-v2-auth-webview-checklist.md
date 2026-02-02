# Tauri v2 GitHub Auth Webview Checklist

Purpose: reliability-first checklist for GitHub session-based extraction in Tauri v2.

## Decision Gate
- [ ] Confirm whether embedded webview auth is required.
- [ ] If not required, prefer system-browser OAuth + deep-link callback.
- [ ] Verify GitHub OAuth policy constraints for embedded webviews.

## Webview Configuration
- [ ] Use documented webview initialization script only for UI helpers.
- [ ] Do not rely on injected JS to read cookies (HttpOnly/SameSite).
- [ ] Configure a persistent webview data store and verify its location per OS.

## Navigation Handling
- [ ] Register navigation handler to detect final redirect target.
- [ ] Implement URL stabilization (same URL for N ms) before completing auth.
- [ ] Handle multi-step redirects across GitHub domains.

## Session Extraction
- [ ] Use Rust-side cookie/session APIs if available.
- [ ] Validate which cookies are accessible and required.
- [ ] Store session data securely (OS keychain/secure storage).

## Reliability & Recovery
- [ ] Add retry + backoff for navigation failures.
- [ ] Provide a forced re-auth path (clear session + reopen login).
- [ ] Track and surface user-facing errors without leaking sensitive data.

## Observability
- [ ] Log navigation sequence with redaction.
- [ ] Log webview lifecycle events and error states.
- [ ] Capture OS and webview version info in diagnostics.

## Testing
- [ ] Test across macOS/Windows/Linux.
- [ ] Test with 2FA and account recovery challenges.
- [ ] Verify session persistence across app restarts and updates.

## Open Documentation Items
- [ ] Exact API signatures for initialization_script, on_navigation, on_webview_event.
- [ ] Documented persistence configuration details per OS.
- [ ] Availability and constraints of cookie/session APIs.
