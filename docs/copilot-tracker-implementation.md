# Copilot Tracker - Cross-Platform Implementation Plan

> A modern, cross-platform GitHub Copilot usage monitoring application with system tray support for macOS, Windows, and Linux.

**Target Folder**: `/Users/muhammadfaiz/Custom APP/copilot-tracker`

## Table of Contents

1. [Overview](#overview)
2. [Technology Stack](#technology-stack)
3. [Architecture](#architecture)
4. [Project Structure](#project-structure)
5. [Implementation Phases](#implementation-phases)
6. [Core Logic Ported from Original](#core-logic-ported-from-original)
7. [UI/UX Improvements](#uiux-improvements)
8. [Getting Started](#getting-started)

---

## Overview

### Goals

- **Cross-platform**: Works on macOS, Windows, and Linux
- **Same functionality**: All features from the original macOS app
- **Better UI/UX**: Modern design with dark/light theme support
- **Usage visualization**: Trend charts for usage over time
- **Smart notifications**: Alerts when approaching usage limits

### Original App Analysis

The original app (`copilot-usage-monitor`) is a macOS-only Swift/SwiftUI application that:

- Runs as a menu bar app (no dock icon)
- Uses WebView-based GitHub OAuth (no password storage)
- Fetches usage data via JavaScript injection into authenticated WebView
- Shows current usage, predictions, and history in a dropdown menu
- Uses Sparkle for auto-updates

### Key Challenge

The app relies on `WKWebView` with persistent cookies for authentication. Our cross-platform solution will use Electron's `BrowserView` which provides equivalent functionality via Chromium's WebView.

---

## Technology Stack

| Layer | Technology | Reason |
|-------|-----------|--------|
| **Framework** | Electron 33+ | Native system tray, Chromium WebView, mature ecosystem |
| **Frontend** | React 18 | Best Electron integration, largest ecosystem |
| **Language** | TypeScript | Type safety, better maintainability |
| **Styling** | Tailwind CSS + shadcn/ui | Modern, beautiful, customizable components |
| **State** | Zustand | Lightweight, simple, works great with React |
| **Charts** | Recharts | Best React charting library |
| **Build** | electron-vite | Modern, fast HMR, native ESM |
| **Persistence** | electron-store | Simple key-value storage for cache & settings |
| **Updates** | electron-updater | Auto-updates for all platforms |
| **Animations** | Framer Motion | Smooth, performant animations |

### Key Dependencies

```json
{
  "dependencies": {
    "react": "^18.3.0",
    "react-dom": "^18.3.0",
    "zustand": "^4.5.0",
    "recharts": "^2.12.0",
    "framer-motion": "^11.0.0",
    "electron-store": "^8.2.0"
  },
  "devDependencies": {
    "electron": "^33.0.0",
    "electron-vite": "^2.3.0",
    "electron-builder": "^24.13.0",
    "electron-updater": "^6.1.0",
    "typescript": "^5.5.0",
    "tailwindcss": "^3.4.0",
    "@types/react": "^18.3.0"
  }
}
```

---

## Architecture

```
+--------------------------------------------------------------+
|                        ELECTRON                               |
+-------------------------+------------------------------------+
|     Main Process        |         Renderer Process           |
|  +-----------------+    |    +----------------------------+  |
|  |  System Tray    |<---+--->|  React Dashboard           |  |
|  |  (all platforms)|    |    |  - Usage Card              |  |
|  +--------+--------+    |    |  - Trend Chart             |  |
|           |             |    |  - Settings                |  |
|  +--------v--------+    |    +-------------+--------------+  |
|  |  Window Manager |    |                  |                  |
|  +--------+--------+    |    +-------------v--------------+  |
|           |             |    |  BrowserView (GitHub Auth) |  |
|  +--------v--------+    |    |  - OAuth Login             |  |
|  |  IPC Handlers   |<---+--->|  - API Calls (JS inject)   |  |
|  +--------+--------+    |    +----------------------------+  |
|           |             |                                    |
|  +--------v--------+    |                                    |
|  |  Notifications  |    |                                    |
|  |  electron-store |    |                                    |
|  +-----------------+    |                                    |
+-------------------------+------------------------------------+
```

### Data Flow

```
1. App Startup
   └─> Main Process initializes
       ├─> Create System Tray
       ├─> Create hidden BrowserView (for API)
       └─> Load cached data (if available)

2. Authentication Flow
   └─> BrowserView loads billing page
       ├─> If redirected to /login → Show login window
       └─> If billing page loads → Ready to fetch

3. Data Fetching (via IPC)
   └─> Main Process: Execute JS in BrowserView
       ├─> Get customer ID (3 fallback methods)
       ├─> Fetch current usage (usage_card API)
       ├─> Fetch history (usage_table API)
       └─> Send to Renderer via IPC

4. UI Update
   └─> Renderer receives data
       ├─> Update Zustand store
       ├─> React re-renders components
       └─> Main Process updates tray
```

---

## Project Structure

```
copilot-tracker/
├── electron/
│   ├── main/
│   │   ├── index.ts              # Main process entry point
│   │   ├── tray.ts               # System tray controller
│   │   ├── windows.ts            # Window management (main, login)
│   │   ├── auth-view.ts          # BrowserView for GitHub OAuth
│   │   └── ipc/
│   │       ├── index.ts          # IPC handler registration
│   │       ├── auth.ts           # Auth-related IPC
│   │       └── usage.ts          # Usage data IPC
│   └── preload/
│       └── index.ts              # Secure IPC bridge
├── src/
│   ├── main.tsx                  # Renderer entry point
│   ├── App.tsx                   # Root component with routing
│   ├── components/
│   │   ├── ui/                   # shadcn/ui components
│   │   │   ├── button.tsx
│   │   │   ├── card.tsx
│   │   │   ├── progress.tsx
│   │   │   ├── select.tsx
│   │   │   └── tooltip.tsx
│   │   ├── layout/
│   │   │   ├── Header.tsx        # App header with theme toggle
│   │   │   └── Layout.tsx        # Main layout wrapper
│   │   ├── dashboard/
│   │   │   ├── Dashboard.tsx     # Main dashboard view
│   │   │   ├── UsageCard.tsx     # Current usage display
│   │   │   ├── UsageChart.tsx    # Usage trend visualization
│   │   │   ├── PredictionCard.tsx # EOM prediction
│   │   │   └── HistoryTable.tsx  # Daily usage table
│   │   ├── settings/
│   │   │   ├── Settings.tsx      # Settings panel
│   │   │   └── ThemeToggle.tsx   # Dark/light theme
│   │   └── auth/
│   │       └── LoginPrompt.tsx   # Login prompt overlay
│   ├── hooks/
│   │   ├── useUsage.ts           # Usage data hook
│   │   ├── useAuth.ts            # Auth state hook
│   │   └── useTheme.ts           # Theme hook
│   ├── services/
│   │   ├── api.ts                # API client (JS injection scripts)
│   │   ├── auth.ts               # Auth manager
│   │   └── predictor.ts          # Usage prediction algorithm
│   ├── types/
│   │   ├── usage.ts              # Usage data types
│   │   ├── settings.ts           # Settings types
│   │   └── electron.d.ts         # Electron API types
│   ├── stores/
│   │   ├── usageStore.ts         # Usage state (zustand)
│   │   └── settingsStore.ts      # Settings state
│   └── styles/
│       └── globals.css           # Tailwind + custom styles
├── resources/
│   ├── icons/                    # App icons (all sizes)
│   │   ├── icon.icns             # macOS
│   │   ├── icon.ico              # Windows
│   │   └── icon.png              # Linux
│   └── tray/                     # Tray icons
│       ├── trayTemplate.png      # macOS (template)
│       ├── trayTemplate@2x.png   # macOS retina
│       ├── tray.png              # Windows/Linux
│       └── tray@2x.png           # Windows HiDPI
├── electron-builder.yml          # Build configuration
├── package.json
├── tsconfig.json
├── tsconfig.node.json
├── tailwind.config.js
├── postcss.config.js
├── vite.config.ts
└── README.md
```

---

## Implementation Phases

### Phase 1: Project Setup & Scaffolding

**Duration**: ~1.25 hours | **Tasks**: 7

| # | Task | Description | Est. |
|---|------|-------------|------|
| 1 | Create project folder | Initialize at target location | 5 min |
| 2 | Initialize electron-vite | `npm create @electron-vite/electron-vite@latest` | 10 min |
| 3 | Configure Tailwind CSS | Install & configure Tailwind + PostCSS | 10 min |
| 4 | Install shadcn/ui | Add base components (button, card, progress) | 15 min |
| 5 | Setup folder structure | Create all directories as per structure | 10 min |
| 6 | Configure TypeScript | Strict mode, path aliases, electron types | 10 min |
| 7 | Basic main process | Minimal Electron app that opens a window | 15 min |

**Deliverable**: Running Electron app with React + Tailwind + TypeScript

**Commands**:
```bash
# Initialize project
npm create @electron-vite/electron-vite@latest copilot-tracker -- --template react-ts

# Install Tailwind
npm install -D tailwindcss postcss autoprefixer
npx tailwindcss init -p

# Install shadcn/ui
npx shadcn-ui@latest init
```

---

### Phase 2: Core Data Types & Services

**Duration**: ~2.25 hours | **Tasks**: 6

| # | Task | Description | Est. |
|---|------|-------------|------|
| 8 | Port CopilotUsage type | TypeScript interface from Swift model | 15 min |
| 9 | Port UsageHistory types | DailyUsage, UsageHistory interfaces | 15 min |
| 10 | Port UsagePrediction types | Prediction result interface | 10 min |
| 11 | Implement UsagePredictor | Port prediction algorithm from Swift | 45 min |
| 12 | Create API JavaScript | Port JS injection scripts for GitHub API | 30 min |
| 13 | Setup electron-store | Persistent storage for cache & settings | 15 min |

**Deliverable**: All TypeScript types and core prediction logic

**Key Types to Port**:
```typescript
// From Swift: CopilotUsage.swift
interface CopilotUsage {
  netBilledAmount: number;      // Add-on cost (dollars)
  netQuantity: number;          // Net requests used
  discountQuantity: number;     // Requests within limit
  userPremiumRequestEntitlement: number;  // Monthly limit
}

// From Swift: UsageHistory.swift
interface DailyUsage {
  date: Date;
  includedRequests: number;
  billedRequests: number;
  grossAmount: number;
  billedAmount: number;
}

interface UsageHistory {
  fetchedAt: Date;
  days: DailyUsage[];
}
```

---

### Phase 3: Authentication System

**Duration**: ~3 hours | **Tasks**: 7

| # | Task | Description | Est. |
|---|------|-------------|------|
| 14 | Create BrowserView wrapper | Hidden WebView for API calls | 30 min |
| 15 | Implement cookie persistence | Session persistence using partition | 20 min |
| 16 | Navigation monitoring | Detect login redirects, billing page loads | 30 min |
| 17 | Create login window | Visible window for GitHub OAuth | 30 min |
| 18 | IPC for auth state | Main<->Renderer communication for auth | 30 min |
| 19 | Session expired detection | Monitor for logout/session timeout | 20 min |
| 20 | Reset session functionality | Clear all cookies/data on logout | 15 min |

**Deliverable**: Complete GitHub OAuth flow working

**Key Implementation Details**:
```typescript
// electron/main/auth-view.ts
const authView = new BrowserView({
  webPreferences: {
    partition: 'persist:github',  // Persistent cookies
    nodeIntegration: false,
    contextIsolation: true,
  }
});

// Navigate to billing page
authView.webContents.loadURL(
  'https://github.com/settings/billing/premium_requests_usage'
);

// Monitor navigation
authView.webContents.on('did-navigate', (event, url) => {
  if (url.includes('/login') || url.includes('/session')) {
    mainWindow.webContents.send('auth:session-expired');
  }
  if (url.includes('/settings/billing')) {
    mainWindow.webContents.send('auth:ready');
  }
});
```

---

### Phase 4: Data Fetching & Caching

**Duration**: ~3.25 hours | **Tasks**: 8

| # | Task | Description | Est. |
|---|------|-------------|------|
| 21 | Get customer ID | 3 fallback methods (API, DOM, regex) | 30 min |
| 22 | Fetch current usage | JS injection for usage_card API | 30 min |
| 23 | Parse usage response | Convert JSON to CopilotUsage | 20 min |
| 24 | Fetch usage history | JS injection for usage_table API | 30 min |
| 25 | Parse history response | Convert to UsageHistory with dates | 25 min |
| 26 | Implement caching | Cache usage & history in electron-store | 20 min |
| 27 | Auto-refresh timer | Configurable refresh intervals | 20 min |
| 28 | IPC for usage data | Send usage data to renderer | 20 min |

**Deliverable**: All data fetching and caching working

**API JavaScript (port from Swift)**:
```javascript
// Get Customer ID - Method 1: API
const userApiJS = `
  return await (async function() {
    const response = await fetch('/api/v3/user', {
      headers: { 'Accept': 'application/json' }
    });
    const data = await response.json();
    return JSON.stringify(data);
  })()
`;

// Get Current Usage
const usageCardJS = (customerId) => `
  return await (async function() {
    const res = await fetch('/settings/billing/copilot_usage_card?customer_id=${customerId}&period=3', {
      headers: { 'Accept': 'application/json', 'x-requested-with': 'XMLHttpRequest' }
    });
    return await res.json();
  })()
`;

// Get Usage History
const usageTableJS = (customerId) => `
  return await (async function() {
    const res = await fetch('/settings/billing/copilot_usage_table?customer_id=${customerId}&group=0&period=3&query=&page=1', {
      headers: { 'Accept': 'application/json', 'x-requested-with': 'XMLHttpRequest' }
    });
    return await res.json();
  })()
`;
```

---

### Phase 5: UI - Dashboard

**Duration**: ~5.5 hours | **Tasks**: 10

| # | Task | Description | Est. |
|---|------|-------------|------|
| 29 | Create Layout component | Header, main content area | 20 min |
| 30 | Implement theme system | Dark/light mode with system detection | 30 min |
| 31 | Create UsageCard | Current usage with progress bar | 45 min |
| 32 | Style progress indicator | Color-coded (green/yellow/orange/red) | 20 min |
| 33 | Create PredictionCard | End of month prediction display | 30 min |
| 34 | Create UsageChart | Line chart with Recharts | 60 min |
| 35 | Create HistoryTable | Daily usage breakdown | 45 min |
| 36 | Dashboard layout | Grid layout with responsive design | 30 min |
| 37 | Loading states | Skeleton loaders during data fetch | 20 min |
| 38 | Error states | Error display with retry option | 20 min |

**Deliverable**: Beautiful, functional dashboard UI

**UI Components**:
```tsx
// UsageCard - Color-coded progress
function getProgressColor(percentage: number): string {
  if (percentage < 50) return 'bg-green-500';
  if (percentage < 75) return 'bg-yellow-500';
  if (percentage < 90) return 'bg-orange-500';
  return 'bg-red-500';
}

// UsageChart - Recharts configuration
<LineChart data={history}>
  <XAxis dataKey="date" />
  <YAxis />
  <Tooltip />
  <Line type="monotone" dataKey="totalRequests" stroke="#8884d8" />
  <Line type="monotone" dataKey="predicted" stroke="#82ca9d" strokeDasharray="5 5" />
</LineChart>
```

---

### Phase 6: System Tray

**Duration**: ~4 hours | **Tasks**: 8

| # | Task | Description | Est. |
|---|------|-------------|------|
| 39 | Create base Tray class | Cross-platform tray setup | 30 min |
| 40 | Platform-specific icons | macOS template, Windows/Linux standard | 30 min |
| 41 | Tray context menu | Usage, refresh, settings, quit items | 30 min |
| 42 | Update tray tooltip | Show current usage in tooltip | 15 min |
| 43 | macOS menu bar style | Match native macOS appearance | 30 min |
| 44 | Click to show dashboard | Toggle dashboard window visibility | 15 min |
| 45 | Tray icon with badge | Show usage percentage visually | 45 min |
| 46 | Real-time tray updates | Update tray when data changes | 20 min |

**Deliverable**: Native system tray on all platforms

**Tray Implementation**:
```typescript
// electron/main/tray.ts
import { Tray, Menu, nativeImage } from 'electron';
import path from 'path';

class TrayController {
  private tray: Tray | null = null;

  create() {
    // Platform-specific icons
    const iconPath = process.platform === 'darwin'
      ? path.join(__dirname, '../../resources/tray/trayTemplate.png')
      : path.join(__dirname, '../../resources/tray/tray.png');

    this.tray = new Tray(nativeImage.createFromPath(iconPath));
    
    // macOS-specific: set as template image
    if (process.platform === 'darwin') {
      this.tray.setImage(nativeImage.createFromPath(iconPath));
    }

    this.updateMenu();
  }

  updateMenu(usage?: CopilotUsage) {
    const menu = Menu.buildFromTemplate([
      {
        label: usage 
          ? `Used: ${usage.usedRequests} / ${usage.limitRequests} (${usage.usagePercentage.toFixed(1)}%)`
          : 'Loading...',
        enabled: false
      },
      { type: 'separator' },
      { label: 'Open Dashboard', click: () => showDashboard() },
      { label: 'Refresh', click: () => fetchUsage() },
      { type: 'separator' },
      { label: 'Settings', click: () => showSettings() },
      { label: 'Quit', click: () => app.quit() }
    ]);

    this.tray?.setContextMenu(menu);
    this.tray?.setToolTip(
      usage ? `Copilot: ${usage.usedRequests}/${usage.limitRequests}` : 'Copilot Tracker'
    );
  }
}
```

---

### Phase 7: Settings & Preferences

**Duration**: ~2.5 hours | **Tasks**: 6

| # | Task | Description | Est. |
|---|------|-------------|------|
| 47 | Create Settings panel | Settings page/modal component | 30 min |
| 48 | Refresh interval setting | 10s, 30s, 1m, 5m, 30m options | 20 min |
| 49 | Prediction period setting | 7, 14, 21 day options | 15 min |
| 50 | Launch at login | Platform-specific autostart | 45 min |
| 51 | Notification preferences | Enable/disable, threshold setting | 30 min |
| 52 | Theme preference | Light, dark, system options | 15 min |

**Deliverable**: Complete settings system

**Settings Interface**:
```typescript
interface Settings {
  refreshInterval: 10 | 30 | 60 | 300 | 1800;  // seconds
  predictionPeriod: 7 | 14 | 21;  // days
  launchAtLogin: boolean;
  notifications: {
    enabled: boolean;
    thresholds: number[];  // e.g., [50, 75, 90, 100]
  };
  theme: 'light' | 'dark' | 'system';
}
```

**Launch at Login (cross-platform)**:
```typescript
// electron/main/autostart.ts
import { app } from 'electron';

function setAutoLaunch(enabled: boolean) {
  app.setLoginItemSettings({
    openAtLogin: enabled,
    // macOS-specific
    openAsHidden: true,
    // Windows-specific args
    args: ['--hidden']
  });
}
```

---

### Phase 8: Notifications

**Duration**: ~1.5 hours | **Tasks**: 4

| # | Task | Description | Est. |
|---|------|-------------|------|
| 53 | Native notifications | Electron Notification API | 20 min |
| 54 | Threshold alerts | Alert at 50%, 75%, 90%, 100% | 30 min |
| 55 | Add-on cost alerts | Alert when exceeding included quota | 20 min |
| 56 | Notification preferences | Respect user settings | 15 min |

**Deliverable**: Smart notification system

**Notification Implementation**:
```typescript
import { Notification } from 'electron';

class NotificationManager {
  private notifiedThresholds = new Set<number>();

  check(usage: CopilotUsage, settings: Settings) {
    if (!settings.notifications.enabled) return;

    const percentage = usage.usagePercentage;
    
    for (const threshold of settings.notifications.thresholds) {
      if (percentage >= threshold && !this.notifiedThresholds.has(threshold)) {
        this.show(
          `Usage Alert: ${threshold}%`,
          `You've used ${usage.usedRequests} of ${usage.limitRequests} requests.`
        );
        this.notifiedThresholds.add(threshold);
      }
    }
  }

  private show(title: string, body: string) {
    new Notification({ title, body }).show();
  }

  resetMonthly() {
    this.notifiedThresholds.clear();
  }
}
```

---

### Phase 9: Packaging & Distribution

**Duration**: ~4.75 hours | **Tasks**: 8

| # | Task | Description | Est. |
|---|------|-------------|------|
| 57 | Configure electron-builder | Build settings for all platforms | 30 min |
| 58 | macOS DMG | Create DMG with drag-to-Applications | 30 min |
| 59 | Windows NSIS installer | Create .exe installer | 30 min |
| 60 | Linux AppImage | Create portable AppImage | 20 min |
| 61 | App icons | Create icons for all platforms | 30 min |
| 62 | Auto-updater setup | electron-updater with GitHub releases | 45 min |
| 63 | Code signing (macOS) | Sign for Gatekeeper (optional) | 30 min |
| 64 | GitHub Actions CI/CD | Automated builds on release | 60 min |

**Deliverable**: Distributable packages for all platforms

**electron-builder.yml**:
```yaml
appId: com.copilot-tracker.app
productName: Copilot Tracker
copyright: Copyright 2024

directories:
  output: dist

files:
  - "!**/.vscode/*"
  - "!src/*"
  - "!electron/*"
  - "!node_modules/*/{CHANGELOG.md,README.md,readme.md}"

mac:
  category: public.app-category.developer-tools
  target:
    - target: dmg
      arch: [x64, arm64]
  icon: resources/icons/icon.icns
  hardenedRuntime: true
  gatekeeperAssess: false

win:
  target:
    - target: nsis
      arch: [x64]
  icon: resources/icons/icon.ico

linux:
  target:
    - target: AppImage
      arch: [x64]
  icon: resources/icons
  category: Development

nsis:
  oneClick: false
  allowToChangeInstallationDirectory: true

publish:
  provider: github
  owner: your-username
  repo: copilot-tracker
```

---

## Core Logic Ported from Original

### Usage Prediction Algorithm

Port from `UsagePredictor.swift`:

```typescript
// src/services/predictor.ts

interface PredictionWeights {
  period: number;
  weights: number[];
}

const PREDICTION_PERIODS: Record<string, PredictionWeights> = {
  '7': { period: 7, weights: [1.5, 1.5, 1.2, 1.2, 1.2, 1.0, 1.0] },
  '14': { period: 14, weights: [2.0, 1.8, 1.6, 1.4, 1.2, 1.2, 1.0, 1.0, 1.0, 1.0, 0.8, 0.8, 0.6, 0.6] },
  '21': { period: 21, weights: [/* ... */] }
};

const COST_PER_REQUEST = 0.04;  // $0.04 per add-on request

interface UsagePrediction {
  predictedMonthlyRequests: number;
  predictedBilledAmount: number;
  confidenceLevel: 'low' | 'medium' | 'high';
  daysUsedForPrediction: number;
}

export function predictUsage(
  history: UsageHistory,
  currentUsage: CopilotUsage,
  predictionPeriod: number
): UsagePrediction {
  const config = PREDICTION_PERIODS[predictionPeriod.toString()];
  const dailyData = history.days.slice(0, predictionPeriod);
  
  // 1. Calculate weighted average daily usage
  let weightedSum = 0;
  let totalWeight = 0;
  
  dailyData.forEach((day, index) => {
    const weight = config.weights[index] || 1.0;
    weightedSum += day.totalRequests * weight;
    totalWeight += weight;
  });
  
  const weightedAvgDaily = totalWeight > 0 ? weightedSum / totalWeight : 0;
  
  // 2. Calculate weekend/weekday ratio
  const weekendDays = dailyData.filter(d => isWeekend(d.date));
  const weekdayDays = dailyData.filter(d => !isWeekend(d.date));
  
  const avgWeekend = average(weekendDays.map(d => d.totalRequests));
  const avgWeekday = average(weekdayDays.map(d => d.totalRequests));
  
  const weekendRatio = avgWeekday > 0 ? avgWeekend / avgWeekday : 1.0;
  
  // 3. Calculate remaining days
  const today = new Date();
  const daysInMonth = getDaysInMonth(today);
  const currentDay = today.getDate();
  const remainingDays = daysInMonth - currentDay;
  
  const { remainingWeekdays, remainingWeekends } = countRemainingDays(today, remainingDays);
  
  // 4. Predict total monthly usage
  const currentTotal = currentUsage.usedRequests;
  const predictedRemaining = 
    (weightedAvgDaily * remainingWeekdays) +
    (weightedAvgDaily * weekendRatio * remainingWeekends);
  
  const predictedMonthlyTotal = currentTotal + predictedRemaining;
  
  // 5. Calculate predicted add-on cost
  const limit = currentUsage.limitRequests;
  const excessRequests = Math.max(0, predictedMonthlyTotal - limit);
  const predictedBilledAmount = excessRequests * COST_PER_REQUEST;
  
  // 6. Determine confidence level
  let confidenceLevel: 'low' | 'medium' | 'high';
  if (dailyData.length < 3) {
    confidenceLevel = 'low';
  } else if (dailyData.length < 7) {
    confidenceLevel = 'medium';
  } else {
    confidenceLevel = 'high';
  }
  
  return {
    predictedMonthlyRequests: Math.round(predictedMonthlyTotal),
    predictedBilledAmount: Math.round(predictedBilledAmount * 100) / 100,
    confidenceLevel,
    daysUsedForPrediction: dailyData.length
  };
}

// Helper functions
function isWeekend(date: Date): boolean {
  const day = date.getDay();
  return day === 0 || day === 6;
}

function average(nums: number[]): number {
  return nums.length > 0 ? nums.reduce((a, b) => a + b, 0) / nums.length : 0;
}

function getDaysInMonth(date: Date): number {
  return new Date(date.getFullYear(), date.getMonth() + 1, 0).getDate();
}

function countRemainingDays(today: Date, remaining: number): { remainingWeekdays: number; remainingWeekends: number } {
  let weekdays = 0;
  let weekends = 0;
  
  for (let i = 1; i <= remaining; i++) {
    const date = new Date(today);
    date.setDate(today.getDate() + i);
    if (isWeekend(date)) {
      weekends++;
    } else {
      weekdays++;
    }
  }
  
  return { remainingWeekdays: weekdays, remainingWeekends: weekends };
}
```

---

## UI/UX Improvements

### 1. Modern Visual Design

- **shadcn/ui components** with smooth Framer Motion animations
- **Consistent spacing** using Tailwind's spacing scale
- **Subtle shadows and gradients** for depth
- **Smooth transitions** for state changes

### 2. Dark/Light Theme

```typescript
// src/hooks/useTheme.ts
import { useEffect } from 'react';
import { useSettingsStore } from '../stores/settingsStore';

export function useTheme() {
  const theme = useSettingsStore((s) => s.theme);

  useEffect(() => {
    const root = document.documentElement;
    
    if (theme === 'system') {
      const isDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      root.classList.toggle('dark', isDark);
    } else {
      root.classList.toggle('dark', theme === 'dark');
    }
  }, [theme]);
}
```

### 3. Usage Trend Visualization

```tsx
// src/components/dashboard/UsageChart.tsx
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer, Area } from 'recharts';

export function UsageChart({ history, prediction }: Props) {
  const chartData = history.days.map((day, i) => ({
    date: format(day.date, 'MMM d'),
    usage: day.totalRequests,
    predicted: i >= history.days.length - 7 ? prediction.dailyAvg : undefined,
    isWeekend: day.isWeekend
  }));

  return (
    <ResponsiveContainer width="100%" height={300}>
      <LineChart data={chartData}>
        <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
        <XAxis dataKey="date" className="text-xs" />
        <YAxis className="text-xs" />
        <Tooltip 
          contentStyle={{ 
            backgroundColor: 'hsl(var(--card))',
            border: '1px solid hsl(var(--border))'
          }}
        />
        <Line 
          type="monotone" 
          dataKey="usage" 
          stroke="hsl(var(--primary))" 
          strokeWidth={2}
          dot={{ fill: 'hsl(var(--primary))' }}
        />
        <Line 
          type="monotone" 
          dataKey="predicted" 
          stroke="hsl(var(--muted-foreground))" 
          strokeDasharray="5 5"
          strokeWidth={2}
        />
      </LineChart>
    </ResponsiveContainer>
  );
}
```

### 4. Desktop Notifications

- **Smart thresholds**: 50%, 75%, 90%, 100%
- **Non-intrusive**: Native OS notifications
- **Configurable**: Enable/disable per threshold
- **No spam**: Only triggers once per threshold per month

### 5. Same macOS Menu Bar Experience

- Native system tray integration
- Quick-glance usage info in tooltip
- Keyboard shortcuts (Cmd+R to refresh)
- Plus a rich dashboard window for detailed view

---

## Getting Started

### Prerequisites

- Node.js 18+ (LTS recommended)
- npm or pnpm
- Git

### Development Setup

```bash
# Clone the repository
git clone https://github.com/your-username/copilot-tracker.git
cd copilot-tracker

# Install dependencies
npm install

# Start development server
npm run dev
```

### Build for Production

```bash
# Build for current platform
npm run build

# Build for specific platforms
npm run build:mac
npm run build:win
npm run build:linux

# Build for all platforms
npm run build:all
```

### Project Commands

| Command | Description |
|---------|-------------|
| `npm run dev` | Start development with hot reload |
| `npm run build` | Build for production (current platform) |
| `npm run build:mac` | Build macOS DMG |
| `npm run build:win` | Build Windows installer |
| `npm run build:linux` | Build Linux AppImage |
| `npm run lint` | Run ESLint |
| `npm run typecheck` | TypeScript type checking |

---

## Summary

| Metric | Value |
|--------|-------|
| **Total Phases** | 9 |
| **Total Tasks** | 64 |
| **Estimated Time** | ~28 hours |
| **Platforms** | macOS, Windows, Linux |
| **Tech Stack** | Electron + React + TypeScript + Tailwind |

This plan provides a clear roadmap from setup to distribution, with detailed tasks for each phase. The implementation preserves all core functionality from the original macOS app while adding modern UI/UX improvements and cross-platform support.

---

## Next Steps

When ready to implement:

1. Copy this plan to `/Users/muhammadfaiz/Custom APP/copilot-tracker/IMPLEMENTATION_PLAN.md`
2. Start with Phase 1: Project Setup
3. Follow each phase sequentially
4. Use the task checklist to track progress

The plan is designed to be executed incrementally, with each phase delivering a working milestone.
