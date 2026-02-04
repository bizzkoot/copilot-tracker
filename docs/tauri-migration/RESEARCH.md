# Tauri Migration Research

> Deep-dive research for a reliable, small-bundle Tauri v2 release on macOS, Windows, and Linux

## Goals

- Match current Electron feature set with minimal regressions
- Keep bundle size small (target <25 MB) without sacrificing reliability
- Ensure authentication and usage data extraction are stable across platforms
- Ship a beta alongside Electron until confidence is high

## Scope

- Authentication via webview (GitHub session-based)
- Usage extraction from GitHub billing endpoints
- System tray with dynamic usage indicator
- Auto-updater and launch-at-login
- Cross-platform packaging with smallest practical size

## Key Risks and Reliable Solution Options

### 1) Authentication via webview (highest risk)

**Why risky**

- Electron uses `WebContentsView.executeJavaScript` and persistent sessions via partitions
- Tauri uses native webview engines with different behavior per OS
- JS injection limitations and timing issues can break data extraction

**Options**

1. **Hidden WebviewWindow + initialization script (recommended)**
   - Use a dedicated hidden webview for auth and extraction
   - Inject a minimal script with `initialization_script` and guard by origin
   - Use `on_navigation` to detect login redirects and show the auth window only when needed
   - Emit extracted data to Rust via events or `invoke`

2. **Visible auth window only**
   - Always show the auth window and allow user to login
   - Extract data after page load; lower automation, higher user friction

3. **External browser OAuth**
   - Not viable due to lack of official GitHub OAuth for these endpoints

**Reliability notes**

- Tauri supports `on_navigation` for webview navigation allowlisting
- `initialization_script` runs before page scripts; guard by `window.location.origin`
- Add a watchdog loop to re-run extraction if DOM is not ready
- Add a fallback path that shows the auth window if extraction fails

**Recommended approach**

- Hidden auth webview + visible fallback
- Event-based data exfiltration from injected script
- Allowlist GitHub domains via `on_navigation`

**Open questions**

- Exact DOM selectors and stability across GitHub UI changes
- Best signal for "page ready" in each platform webview

---

### 2) Session persistence and shared cookies

**Why risky**

- Electron uses `partition: persist:github` to guarantee persistence
- Tauri uses native webviews with different profile handling

**Options**

1. **Default persistent storage (recommended)**
   - Tauri default is persistent; incognito is opt-in
   - Use the same app data directory for all windows

2. **Explicit shared WebView2 environment (Windows)**
   - Create and reuse a single WebView2 environment to share cookies

**Reliability notes**

- Windows is the most sensitive due to WebView2 runtime profile handling
- macOS WKWebView default data store is persistent

**Recommended approach**

- Use default persistent storage everywhere
- On Windows, explicitly reuse the WebView2 environment across windows

---

### 3) JavaScript injection and CSP

**Why risky**

- External pages often use strict CSP
- Injection timing can fail on native webviews

**Options**

1. **Initialization script (recommended)**
   - Use `initialization_script` to inject before page scripts
   - Guard with origin checks to avoid unintended pages

2. **Post-load eval (lower reliability)**
   - More likely to be blocked or too late

**Reliability notes**

- Keep injected script minimal and defensive
- Use retries, timeouts, and structured errors

---

### 4) Dynamic tray icon rendering

**Why risky**

- Current Electron version uses node-canvas for text drawing
- Tauri has no built-in canvas; tiny-skia has no text layout or font rasterization
- Glyph metrics and baseline positioning must be handled manually for tiny icons

**Options matrix**

| Option                     | What it does                                                             | Text quality                | Complexity | Dependency risk | Notes                                                         |
| -------------------------- | ------------------------------------------------------------------------ | --------------------------- | ---------- | --------------- | ------------------------------------------------------------- |
| fontdue                    | Rasterize digits from TTF at startup, cache glyph alpha in a digit atlas | Good for digits; no shaping | Low        | Low             | Fast, small API; matches current approach.                    |
| ab_glyph                   | Rasterize digits from TTF via ab_glyph and cache                         | Good for digits; no shaping | Low        | Low             | Lightweight; similar flow to fontdue.                         |
| rusttype                   | Rasterize digits from TTF via rusttype and cache                         | OK for digits; no shaping   | Medium     | Medium          | Heavier and less active; not ideal unless already used.       |
| Digit atlas (pre-rendered) | Pre-bake 0-9 bitmap glyphs at build time; blit only                      | Stable, predictable         | Lowest     | Lowest          | Best runtime stability; requires build-time asset generation. |
| Text-less tray icon        | Remove text rendering and show a static icon                             | N/A                         | Lowest     | Lowest          | Lowest risk but degrades UX.                                  |

**Reliability notes**

- tiny-skia provides direct RGBA pixel buffers for tray icons
- No shaping or kerning; plan for digits-only and manual baselines
- Tauri tray APIs support setting an icon dynamically

**Recommended approach**

- Use tiny-skia for composition and a cached digit atlas generated at startup (fontdue or ab_glyph)
- If you want maximum runtime determinism, switch to a pre-rendered digit atlas asset
- Keep sizes aligned with the Electron-equivalent tray size; limit to 2-3 digits

---

### 5) Webview inconsistencies across OS

**Why risky**

- WKWebView, WebView2, and WebKitGTK do not behave identically

**Options**

1. **Cross-platform POC harness (recommended)**
   - Build a minimal harness that runs the auth flow and extractor
   - Validate on all three platforms before full migration

2. **Platform-specific extraction variants**
   - Tailor selectors and timing if inconsistencies are found

**Reliability notes**

- Use `on_navigation` allowlist to reduce unexpected redirects
- Use conservative navigation guards and error reporting

---

### 6) WebView2 runtime on Windows

**Why risky**

- WebView2 runtime may be missing or outdated on some systems

**Options**

1. **Detect and install Evergreen runtime (recommended)**
   - Small installer; run bootstrapper if missing

2. **Bundle Evergreen offline installer**
   - Larger installer; works for offline or managed environments

3. **Bundle Fixed Version runtime**
   - Largest size; strict version control, higher maintenance

**Recommended approach**

- Detect and install Evergreen runtime via bootstrapper on first launch or during install

---

### 7) Auto-updater reliability

**Why risky**

- Migration changes packaging and signing requirements

**Options**

1. **tauri-plugin-updater with signed updates (recommended)**
   - Separate beta channel feed
   - Keep Electron stable channel unchanged

2. **Manual update flow**
   - Lowest dependency risk but poor UX

---

## Implementation Notes from Tauri v2 Research

- `WebviewWindowBuilder::on_navigation` can allowlist URLs
- `initialization_script` runs before page scripts; guard by origin
- `on_webview_event` can be used to track load progress
- Tray supports event handlers and dynamic icon updates

## Cross-Platform Reliability Checklist (POC)

- macOS: login, session persistence, JS extraction, tray icon updates
- Windows: WebView2 runtime availability, shared profile, JS extraction
- Linux: WebKitGTK availability, extraction success, tray icon rendering

## Known Gaps

- Confirm exact cookie/profile persistence behavior across platforms
- Choose a final glyph rendering approach for tray icons
- Validate GitHub DOM selectors for long-term stability

## Recommended Next Steps

1. Build a minimal Tauri POC with the hidden auth webview and extraction script
2. Add a tray icon renderer using a digit atlas (or fontdue + tiny-skia)
3. Validate the flow on macOS, Windows, and Linux
4. Update the risk matrix and confidence score based on POC results

## Confidence Level

- Current confidence: Medium (68/100)
- Main blockers: cookie/profile persistence specifics and text rendering path
- Confidence target after POC: 85/100
