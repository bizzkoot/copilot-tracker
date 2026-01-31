# Copilot Tracker - Research Document

> Technical research and analysis for the cross-platform implementation

## Table of Contents

1. [Original App Analysis](#original-app-analysis)
2. [Technology Evaluation](#technology-evaluation)
3. [Cross-Platform Framework Comparison](#cross-platform-framework-comparison)
4. [Authentication Research](#authentication-research)
5. [API Analysis](#api-analysis)
6. [UI/UX Research](#uiux-research)
7. [Platform-Specific Considerations](#platform-specific-considerations)
8. [Technical Decisions](#technical-decisions)

---

## Original App Analysis

### Architecture Overview

The original `copilot-usage-monitor` is a **macOS-only** application built with:

| Component | Technology |
|-----------|------------|
| Language | Swift 5.9 |
| UI Framework | SwiftUI + AppKit hybrid |
| WebView | WKWebView |
| Auto-updates | Sparkle 2.8.1 |
| Persistence | UserDefaults |
| Target OS | macOS 13.0+ |

### File Structure Analysis

```
CopilotMonitor/
├── CopilotMonitorApp.swift      # Entry point
├── App/
│   ├── AppDelegate.swift        # App lifecycle
│   └── StatusBarController.swift # Main logic (1269 lines!)
├── Services/
│   ├── AuthManager.swift        # OAuth handling
│   ├── UsageFetcher.swift       # Alternative fetcher
│   └── UsagePredictor.swift     # Prediction algorithm
├── Views/
│   └── LoginView.swift          # WebView wrapper
└── Models/
    ├── CopilotUsage.swift       # Usage data model
    └── UsageHistory.swift       # History data model
```

### Key Findings

1. **StatusBarController is the heart**: Contains most logic (1269 lines)
   - Menu construction
   - Data fetching via JS injection
   - UI updates
   - Caching
   - Timer management

2. **Authentication approach**: WebView-based OAuth
   - No API tokens stored
   - Session persisted in WKWebsiteDataStore
   - Navigation monitored for login redirects

3. **Data fetching**: JavaScript injection into authenticated WebView
   - Customer ID fetched via 3 fallback methods
   - Usage data fetched from internal billing APIs
   - JSON responses parsed into Swift models

4. **Prediction algorithm**: Weighted average with patterns
   - Uses configurable period (7/14/21 days)
   - Accounts for weekend/weekday patterns
   - Calculates confidence level based on data availability

### Code Patterns Worth Preserving

```swift
// Customer ID retrieval - 3 fallback methods
// Method 1: GitHub API
fetch('/api/v3/user')

// Method 2: DOM extraction
document.querySelector('script[data-target="react-app.embeddedData"]')

// Method 3: Regex patterns on HTML
customerId":(\d+)
customerId&quot;:(\d+)
customer_id=(\d+)
```

```swift
// Usage API endpoints
/settings/billing/copilot_usage_card?customer_id=X&period=3
/settings/billing/copilot_usage_table?customer_id=X&group=0&period=3&query=&page=1
```

---

## Technology Evaluation

### Why Electron?

| Criterion | Electron | Tauri | Flutter | React Native |
|-----------|----------|-------|---------|--------------|
| Cross-platform | Excellent | Excellent | Good | Limited |
| WebView support | Chromium (consistent) | Native (varies) | Limited | Limited |
| System tray | Excellent | Good | Poor | Poor |
| Cookie persistence | Easy | Complex | Complex | Complex |
| Ecosystem | Massive | Growing | Large | Large |
| Bundle size | Large (~100MB) | Small (~10MB) | Medium | Medium |
| Learning curve | Low (if know web) | Medium | Medium | Medium |

**Decision: Electron**

Reasons:
1. **Chromium WebView**: Consistent behavior across platforms for GitHub OAuth
2. **Cookie persistence**: Built-in support via `partition` option
3. **System tray**: Mature, well-documented Tray API
4. **Ecosystem**: Established patterns, lots of examples
5. **Bundle size acceptable**: Users install once, not a deal-breaker

### Why React?

| Criterion | React | Vue | Svelte | Solid |
|-----------|-------|-----|--------|-------|
| Ecosystem | Huge | Large | Growing | Small |
| Electron integration | Best | Good | Good | Limited |
| Component libraries | Many | Many | Growing | Few |
| TypeScript support | Excellent | Excellent | Good | Excellent |
| Learning curve | Medium | Low | Low | Medium |

**Decision: React**

Reasons:
1. **Best Electron integration**: Most Electron apps use React
2. **shadcn/ui**: Beautiful component library for React + Tailwind
3. **Recharts**: Best React charting library
4. **Zustand**: Simple state management

### Why Tailwind + shadcn/ui?

| Option | Pros | Cons |
|--------|------|------|
| Tailwind + shadcn/ui | Beautiful, customizable, modern | Requires Tailwind knowledge |
| Material UI | Comprehensive, well-documented | Can look generic |
| Chakra UI | Accessible, easy to use | Less customizable |
| Ant Design | Enterprise-ready | Complex, opinionated |

**Decision: Tailwind + shadcn/ui**

Reasons:
1. **Modern aesthetics**: Looks great out of the box
2. **Customizable**: Easy to adjust for dark/light themes
3. **Not a dependency**: Components copied into project, full control
4. **Performance**: No runtime CSS-in-JS overhead

---

## Cross-Platform Framework Comparison

### Detailed Electron Analysis

#### Pros
- Chromium-based: Consistent WebView behavior
- Mature: 10+ years of development
- Large community: Extensive documentation
- Native APIs: Tray, notifications, auto-updates
- Security: Context isolation, sandbox by default

#### Cons
- Bundle size: ~100MB minimum
- Memory usage: ~100-150MB typical
- Startup time: ~2-3 seconds
- Needs native builds per platform

#### Mitigation Strategies
- Use `electron-vite` for faster builds and HMR
- Use `contextBridge` for secure IPC
- Minimize renderer process complexity
- Use `electron-builder` for optimized packaging

### Tauri Evaluation (Alternative)

Considered but rejected because:
1. **Native WebView inconsistency**: Different engines on different platforms
2. **Cookie handling complexity**: More manual work required
3. **Less mature**: Fewer production examples
4. **Rust learning curve**: Team would need to learn Rust

### Flutter Desktop Evaluation (Alternative)

Considered but rejected because:
1. **WebView plugin issues**: Incomplete cross-platform support
2. **System tray plugins**: Less mature than Electron
3. **Cookie persistence**: Requires custom implementation
4. **Desktop focus**: Flutter's strength is mobile

---

## Authentication Research

### GitHub OAuth Flow Analysis

The original app uses a **WebView-based session authentication**:

```
1. User opens app
2. App shows WKWebView pointed to GitHub billing page
3. If not logged in, GitHub redirects to /login
4. App detects /login redirect, shows login window
5. User logs in via WebView
6. Session cookies stored in WKWebsiteDataStore
7. App navigates back to billing page
8. App executes JS to fetch data using session cookies
```

### Electron Implementation

```typescript
// Create BrowserView with persistent partition
const authView = new BrowserView({
  webPreferences: {
    partition: 'persist:github',  // Persistent cookies
    nodeIntegration: false,
    contextIsolation: true,
  }
});

// Monitor navigation
authView.webContents.on('did-navigate', (event, url) => {
  if (url.includes('/login') || url.includes('/session')) {
    // Show login window
    showLoginWindow();
  }
  if (url.includes('/settings/billing')) {
    // Ready to fetch data
    fetchUsageData();
  }
});
```

### Session Management

| Aspect | Original (macOS) | Electron |
|--------|------------------|----------|
| Cookie storage | WKWebsiteDataStore | Chromium partition |
| Persistence | Automatic | Automatic with `persist:` prefix |
| Clear session | removeData(ofTypes:) | session.clearStorageData() |
| Cross-platform | N/A | Same API everywhere |

### Security Considerations

1. **No API tokens**: Session-based, no secrets stored
2. **Partition isolation**: GitHub cookies isolated from app
3. **Context isolation**: Renderer can't access main process
4. **CSP headers**: Respect GitHub's security headers

---

## API Analysis

### Endpoints Used

| Endpoint | Method | Purpose | Response |
|----------|--------|---------|----------|
| `/api/v3/user` | GET | Get user ID | User JSON |
| `/settings/billing/copilot_usage_card` | GET | Current usage | Usage JSON |
| `/settings/billing/copilot_usage_table` | GET | Usage history | Table JSON |

### Request Headers

```javascript
{
  'Accept': 'application/json',
  'x-requested-with': 'XMLHttpRequest'  // Required for billing APIs
}
```

### Customer ID Retrieval

Three fallback methods (in order):

```javascript
// Method 1: API call
const response = await fetch('/api/v3/user');
const data = await response.json();
return data.id;

// Method 2: DOM extraction
const el = document.querySelector('script[data-target="react-app.embeddedData"]');
const data = JSON.parse(el.textContent);
return data.payload.customer.customerId;

// Method 3: Regex on HTML
const patterns = [
  /customerId":(\d+)/,
  /customerId&quot;:(\d+)/,
  /customer_id=(\d+)/
];
const match = document.body.innerHTML.match(patterns[0]);
return match[1];
```

### Response Parsing

**Usage Card Response:**
```json
{
  "net_billed_amount": 0.00,
  "net_quantity": 150,
  "discount_quantity": 150,
  "user_premium_request_entitlement": 500,
  "filtered_user_premium_request_entitlement": 500
}
```

**Usage Table Response:**
```json
{
  "rows": [
    {
      "date": "2024-01-30",
      "included_requests": 50,
      "billed_requests": 0,
      "gross_amount": 0,
      "billed_amount": 0
    }
  ]
}
```

### Rate Limiting

- No documented rate limits for internal APIs
- Original app uses 10s-30min refresh intervals
- Recommendation: Default to 1 minute, respect user settings

---

## UI/UX Research

### Current App UI Analysis

**Strengths:**
- Simple, focused interface
- Quick access via menu bar
- Color-coded progress indicator

**Weaknesses:**
- Basic visual design
- No usage trends visualization
- Limited customization
- Menu-only interface (no dashboard)

### Design Inspiration

**Reference Applications:**
1. **Raycast**: Clean, modern macOS utility design
2. **Linear**: Beautiful dark/light themes
3. **Figma Desktop**: Electron app with great UX
4. **Discord**: Cross-platform Electron with system tray

### Component Library Selection

**shadcn/ui chosen for:**
- Radix primitives (accessible)
- Tailwind-based (consistent styling)
- Copy-paste model (full control)
- Great dark mode support

### Theme System

```css
/* CSS variables for theme */
:root {
  --background: 0 0% 100%;
  --foreground: 222.2 84% 4.9%;
  --primary: 222.2 47.4% 11.2%;
  /* ... */
}

.dark {
  --background: 222.2 84% 4.9%;
  --foreground: 210 40% 98%;
  --primary: 210 40% 98%;
  /* ... */
}
```

### Dashboard Layout

```
+--------------------------------------------------+
| [Logo] Copilot Tracker          [Theme] [Settings]|
+--------------------------------------------------+
|                                                   |
|  +-------------------+  +----------------------+  |
|  |   USAGE CARD      |  |   PREDICTION CARD    |  |
|  |   ███████░░░      |  |   Est. End of Month  |  |
|  |   350 / 500 (70%) |  |   480 requests       |  |
|  |   Add-on: $0.00   |  |   Confidence: High   |  |
|  +-------------------+  +----------------------+  |
|                                                   |
|  +----------------------------------------------+ |
|  |            USAGE TREND CHART                 | |
|  |   📈 Line chart with daily usage            | |
|  |      + Prediction overlay                    | |
|  +----------------------------------------------+ |
|                                                   |
|  +----------------------------------------------+ |
|  |            USAGE HISTORY TABLE               | |
|  |   Date     | Included | Billed | Cost       | |
|  |   Jan 30   | 50       | 0      | $0.00      | |
|  |   Jan 29   | 45       | 0      | $0.00      | |
|  +----------------------------------------------+ |
+--------------------------------------------------+
```

---

## Platform-Specific Considerations

### macOS

| Feature | Implementation |
|---------|----------------|
| System tray | Tray with template image |
| Menu bar style | Use `Tray` with context menu |
| Launch at login | `app.setLoginItemSettings()` |
| App signature | Developer ID certificate |
| Notarization | Required for distribution |

### Windows

| Feature | Implementation |
|---------|----------------|
| System tray | Tray with icon |
| HiDPI support | Use @2x icons |
| Launch at login | `app.setLoginItemSettings()` |
| Installer | NSIS via electron-builder |

### Linux

| Feature | Implementation |
|---------|----------------|
| System tray | Tray (AppIndicator on Ubuntu) |
| Launch at login | .desktop file in autostart |
| Package format | AppImage (portable) |
| Icon handling | Multiple PNG sizes |

### Cross-Platform Icon Requirements

```
resources/
├── icons/
│   ├── icon.icns       # macOS (multiple sizes bundled)
│   ├── icon.ico        # Windows (multiple sizes bundled)
│   ├── 16x16.png       # Linux
│   ├── 32x32.png
│   ├── 48x48.png
│   ├── 64x64.png
│   ├── 128x128.png
│   ├── 256x256.png
│   └── 512x512.png
└── tray/
    ├── trayTemplate.png      # macOS (must end in Template)
    ├── trayTemplate@2x.png   # macOS Retina
    ├── tray.png              # Windows/Linux
    └── tray@2x.png           # Windows HiDPI
```

---

## Technical Decisions

### Decision Log

| # | Decision | Options Considered | Choice | Rationale |
|---|----------|-------------------|--------|-----------|
| 1 | Framework | Electron, Tauri, Flutter | Electron | Best WebView support |
| 2 | Frontend | React, Vue, Svelte | React | Best ecosystem |
| 3 | Language | TypeScript, JavaScript | TypeScript | Type safety |
| 4 | Styling | Tailwind, MUI, Chakra | Tailwind + shadcn/ui | Modern, beautiful |
| 5 | State | Redux, Zustand, Jotai | Zustand | Simple, lightweight |
| 6 | Charts | Recharts, Chart.js, D3 | Recharts | Best React integration |
| 7 | Build | electron-builder, electron-forge | electron-builder | More features |
| 8 | Bundler | Vite, Webpack | electron-vite | Fast, modern |

### Architecture Decisions

#### AD-1: Use BrowserView for API Calls

**Context**: Need to make authenticated API calls to GitHub

**Decision**: Use hidden BrowserView instead of BrowserWindow

**Rationale**:
- BrowserView can be attached to main window
- Shares session with login window
- Can execute JavaScript for API calls
- Lower memory than separate window

#### AD-2: Centralized IPC Handlers

**Context**: Need communication between main and renderer

**Decision**: Create dedicated IPC handler modules

**Rationale**:
- Clean separation of concerns
- Easy to test
- Type-safe with TypeScript
- Follows Electron best practices

#### AD-3: Zustand for State Management

**Context**: Need to manage usage data and settings state

**Decision**: Use Zustand instead of Redux or Context

**Rationale**:
- Minimal boilerplate
- Great TypeScript support
- Works well with React
- Persist middleware for settings

#### AD-4: electron-store for Persistence

**Context**: Need to persist settings and cache

**Decision**: Use electron-store instead of localStorage

**Rationale**:
- Works in main process
- Encrypted option available
- Schema validation
- Automatic migrations

---

## References

### Documentation
- [Electron Documentation](https://www.electronjs.org/docs)
- [electron-vite](https://electron-vite.org/)
- [shadcn/ui](https://ui.shadcn.com/)
- [Recharts](https://recharts.org/)
- [Zustand](https://zustand-demo.pmnd.rs/)

### Original App
- [copilot-usage-monitor](https://github.com/hyp3rflow/copilot-usage-monitor)
- StatusBarController.swift (main implementation reference)
- UsagePredictor.swift (prediction algorithm reference)

### Similar Projects
- [Raycast](https://www.raycast.com/) - UI inspiration
- [Linear](https://linear.app/) - Theme inspiration
