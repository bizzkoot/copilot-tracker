# Copilot Tracker - Implementation Plan

> Detailed task breakdown for building the cross-platform application

**Estimated Total Time**: ~28 hours across 9 phases

---

## Phase 1: Project Setup & Scaffolding

**Duration**: ~1.25 hours  
**Dependencies**: None  
**Deliverable**: Running Electron app with React + Tailwind + TypeScript

### Tasks

#### Task 1.1: Create Project Folder

**Time**: 5 minutes  
**Owner**: Developer

```bash
cd "/Users/muhammadfaiz/Custom APP"
mkdir copilot-tracker
cd copilot-tracker
```

#### Task 1.2: Initialize electron-vite Project

**Time**: 10 minutes  
**Owner**: Developer

```bash
npm create @quick-start/electron@latest . -- --template react-ts
# Or manually:
npm init -y
npm install electron electron-vite vite react react-dom
npm install -D @types/react @types/react-dom @types/node typescript
```

#### Task 1.3: Configure Tailwind CSS

**Time**: 10 minutes  
**Owner**: Developer

```bash
npm install -D tailwindcss postcss autoprefixer
npx tailwindcss init -p
```

**tailwind.config.js**:

```js
/** @type {import('tailwindcss').Config} */
export default {
  darkMode: ["class"],
  content: ["./src/**/*.{js,jsx,ts,tsx}"],
  theme: {
    extend: {},
  },
  plugins: [],
};
```

**src/styles/globals.css**:

```css
@tailwind base;
@tailwind components;
@tailwind utilities;
```

#### Task 1.4: Install shadcn/ui

**Time**: 15 minutes  
**Owner**: Developer

```bash
npx shadcn-ui@latest init
# Choose: TypeScript, Tailwind, React
npx shadcn-ui@latest add button card progress select tooltip
```

#### Task 1.5: Setup Folder Structure

**Time**: 10 minutes  
**Owner**: Developer

```bash
mkdir -p electron/main electron/preload
mkdir -p src/components/{ui,layout,dashboard,settings,auth}
mkdir -p src/{hooks,services,types,stores,styles}
mkdir -p resources/{icons,tray}
```

#### Task 1.6: Configure TypeScript

**Time**: 10 minutes  
**Owner**: Developer

**tsconfig.json**:

```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ESNext",
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "jsx": "react-jsx",
    "strict": true,
    "moduleResolution": "bundler",
    "resolveJsonModule": true,
    "isolatedModules": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "allowSyntheticDefaultImports": true,
    "forceConsistentCasingInFileNames": true,
    "baseUrl": ".",
    "paths": {
      "@/*": ["./src/*"],
      "@electron/*": ["./electron/*"]
    }
  },
  "include": ["src"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
```

#### Task 1.7: Basic Main Process

**Time**: 15 minutes  
**Owner**: Developer

**electron/main/index.ts**:

```typescript
import { app, BrowserWindow } from "electron";
import path from "path";

let mainWindow: BrowserWindow | null = null;

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 900,
    height: 700,
    webPreferences: {
      preload: path.join(__dirname, "../preload/index.js"),
      nodeIntegration: false,
      contextIsolation: true,
    },
  });

  if (process.env.NODE_ENV === "development") {
    mainWindow.loadURL("http://localhost:5173");
  } else {
    mainWindow.loadFile(path.join(__dirname, "../renderer/index.html"));
  }
}

app.whenReady().then(createWindow);

app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    app.quit();
  }
});

app.on("activate", () => {
  if (BrowserWindow.getAllWindows().length === 0) {
    createWindow();
  }
});
```

**Verification**:

```bash
npm run dev
# App should open with empty window
```

---

## Phase 2: Core Data Types & Services

**Duration**: ~2.25 hours  
**Dependencies**: Phase 1 complete  
**Deliverable**: All TypeScript types and core prediction logic

### Tasks

#### Task 2.1: Port CopilotUsage Type

**Time**: 15 minutes  
**Owner**: Developer

**src/types/usage.ts**:

```typescript
export interface CopilotUsage {
  netBilledAmount: number; // Add-on cost in dollars
  netQuantity: number; // Net requests used
  discountQuantity: number; // Requests within included quota
  userPremiumRequestEntitlement: number; // Monthly limit
  filteredUserPremiumRequestEntitlement: number;
}

export interface CachedUsage {
  usage: CopilotUsage;
  timestamp: Date;
}

// Computed properties
export function getUsedRequests(usage: CopilotUsage): number {
  return Math.round(usage.discountQuantity);
}

export function getLimitRequests(usage: CopilotUsage): number {
  return usage.userPremiumRequestEntitlement;
}

export function getUsagePercentage(usage: CopilotUsage): number {
  const limit = getLimitRequests(usage);
  if (limit === 0) return 0;
  return (getUsedRequests(usage) / limit) * 100;
}
```

#### Task 2.2: Port UsageHistory Types

**Time**: 15 minutes  
**Owner**: Developer

**src/types/usage.ts** (continued):

```typescript
export interface DailyUsage {
  date: Date; // UTC date
  includedRequests: number; // Requests within quota
  billedRequests: number; // Add-on billed requests
  grossAmount: number; // Gross amount
  billedAmount: number; // Add-on cost
}

export interface UsageHistory {
  fetchedAt: Date;
  days: DailyUsage[];
}

// Computed properties
export function getTotalRequests(day: DailyUsage): number {
  return day.includedRequests + day.billedRequests;
}

export function isWeekend(date: Date): boolean {
  const day = date.getDay();
  return day === 0 || day === 6;
}

export function getDayOfWeek(date: Date): number {
  return date.getDay();
}
```

#### Task 2.3: Port UsagePrediction Types

**Time**: 10 minutes  
**Owner**: Developer

**src/types/usage.ts** (continued):

```typescript
export interface UsagePrediction {
  predictedMonthlyRequests: number;
  predictedBilledAmount: number;
  confidenceLevel: "low" | "medium" | "high";
  daysUsedForPrediction: number;
}

export interface PredictionWeights {
  period: number;
  weights: number[];
}

export const PREDICTION_PERIODS: Record<number, PredictionWeights> = {
  7: {
    period: 7,
    weights: [1.5, 1.5, 1.2, 1.2, 1.2, 1.0, 1.0],
  },
  14: {
    period: 14,
    weights: [
      2.0, 1.8, 1.6, 1.4, 1.2, 1.2, 1.0, 1.0, 1.0, 1.0, 0.8, 0.8, 0.6, 0.6,
    ],
  },
  21: {
    period: 21,
    weights: [
      2.5, 2.3, 2.1, 1.9, 1.7, 1.5, 1.3, 1.3, 1.2, 1.2, 1.1, 1.1, 1.0, 1.0, 0.9,
      0.9, 0.8, 0.8, 0.7, 0.7, 0.6,
    ],
  },
};
```

#### Task 2.4: Implement UsagePredictor

**Time**: 45 minutes  
**Owner**: Developer

**src/services/predictor.ts**:

```typescript
import {
  CopilotUsage,
  UsageHistory,
  UsagePrediction,
  DailyUsage,
  PREDICTION_PERIODS,
  getTotalRequests,
  isWeekend,
  getUsedRequests,
  getLimitRequests,
} from "@/types/usage";

const COST_PER_REQUEST = 0.04;

export function predictUsage(
  history: UsageHistory,
  currentUsage: CopilotUsage,
  predictionPeriod: number,
): UsagePrediction {
  const config = PREDICTION_PERIODS[predictionPeriod];
  if (!config) {
    throw new Error(`Invalid prediction period: ${predictionPeriod}`);
  }

  const dailyData = history.days.slice(0, predictionPeriod);

  // 1. Calculate weighted average daily usage
  const weightedAvgDaily = calculateWeightedAverage(dailyData, config.weights);

  // 2. Calculate weekend/weekday ratio
  const weekendRatio = calculateWeekendRatio(dailyData);

  // 3. Calculate remaining days
  const today = new Date();
  const daysInMonth = getDaysInMonth(today);
  const currentDay = today.getDate();
  const remainingDays = daysInMonth - currentDay;
  const { remainingWeekdays, remainingWeekends } = countRemainingDays(
    today,
    remainingDays,
  );

  // 4. Predict total monthly usage
  const currentTotal = getUsedRequests(currentUsage);
  const predictedRemaining =
    weightedAvgDaily * remainingWeekdays +
    weightedAvgDaily * weekendRatio * remainingWeekends;
  const predictedMonthlyTotal = currentTotal + predictedRemaining;

  // 5. Calculate predicted add-on cost
  const limit = getLimitRequests(currentUsage);
  const excessRequests = Math.max(0, predictedMonthlyTotal - limit);
  const predictedBilledAmount = excessRequests * COST_PER_REQUEST;

  // 6. Determine confidence level
  const confidenceLevel = getConfidenceLevel(dailyData.length);

  return {
    predictedMonthlyRequests: Math.round(predictedMonthlyTotal),
    predictedBilledAmount: Math.round(predictedBilledAmount * 100) / 100,
    confidenceLevel,
    daysUsedForPrediction: dailyData.length,
  };
}

function calculateWeightedAverage(
  dailyData: DailyUsage[],
  weights: number[],
): number {
  let weightedSum = 0;
  let totalWeight = 0;

  dailyData.forEach((day, index) => {
    const weight = weights[index] || 1.0;
    weightedSum += getTotalRequests(day) * weight;
    totalWeight += weight;
  });

  return totalWeight > 0 ? weightedSum / totalWeight : 0;
}

function calculateWeekendRatio(dailyData: DailyUsage[]): number {
  const weekendDays = dailyData.filter((d) => isWeekend(d.date));
  const weekdayDays = dailyData.filter((d) => !isWeekend(d.date));

  if (weekdayDays.length === 0) return 1.0;

  const avgWeekend = average(weekendDays.map((d) => getTotalRequests(d)));
  const avgWeekday = average(weekdayDays.map((d) => getTotalRequests(d)));

  return avgWeekday > 0 ? avgWeekend / avgWeekday : 1.0;
}

function average(nums: number[]): number {
  return nums.length > 0 ? nums.reduce((a, b) => a + b, 0) / nums.length : 0;
}

function getDaysInMonth(date: Date): number {
  return new Date(date.getFullYear(), date.getMonth() + 1, 0).getDate();
}

function countRemainingDays(
  today: Date,
  remaining: number,
): { remainingWeekdays: number; remainingWeekends: number } {
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

function getConfidenceLevel(daysCount: number): "low" | "medium" | "high" {
  if (daysCount < 3) return "low";
  if (daysCount < 7) return "medium";
  return "high";
}
```

#### Task 2.5: Create API JavaScript

**Time**: 30 minutes  
**Owner**: Developer

**src/services/api.ts**:

```typescript
export const API_SCRIPTS = {
  // Method 1: Get user ID from GitHub API
  getUserId: `
    return await (async function() {
      try {
        const response = await fetch('/api/v3/user', {
          headers: { 'Accept': 'application/json' }
        });
        const data = await response.json();
        return JSON.stringify({ success: true, id: data.id });
      } catch (error) {
        return JSON.stringify({ success: false, error: error.message });
      }
    })()
  `,

  // Method 2: Extract customer ID from embedded data
  getCustomerIdFromDOM: `
    return (function() {
      try {
        const el = document.querySelector('script[data-target="react-app.embeddedData"]');
        if (!el) return JSON.stringify({ success: false, error: 'Element not found' });
        const data = JSON.parse(el.textContent);
        return JSON.stringify({ success: true, id: data.payload.customer.customerId });
      } catch (error) {
        return JSON.stringify({ success: false, error: error.message });
      }
    })()
  `,

  // Method 3: Extract from HTML via regex
  getCustomerIdFromHTML: `
    return (function() {
      try {
        const html = document.body.innerHTML;
        const patterns = [
          /customerId":(\\d+)/,
          /customerId&quot;:(\\d+)/,
          /customer_id=(\\d+)/
        ];
        for (const pattern of patterns) {
          const match = html.match(pattern);
          if (match) {
            return JSON.stringify({ success: true, id: parseInt(match[1]) });
          }
        }
        return JSON.stringify({ success: false, error: 'No match found' });
      } catch (error) {
        return JSON.stringify({ success: false, error: error.message });
      }
    })()
  `,

  // Fetch current usage
  getUsageCard: (customerId: number) => `
    return await (async function() {
      try {
        const res = await fetch('/settings/billing/copilot_usage_card?customer_id=${customerId}&period=3', {
          headers: {
            'Accept': 'application/json',
            'x-requested-with': 'XMLHttpRequest'
          }
        });
        const data = await res.json();
        return JSON.stringify({ success: true, data });
      } catch (error) {
        return JSON.stringify({ success: false, error: error.message });
      }
    })()
  `,

  // Fetch usage history
  getUsageTable: (customerId: number) => `
    return await (async function() {
      try {
        const res = await fetch('/settings/billing/copilot_usage_table?customer_id=${customerId}&group=0&period=3&query=&page=1', {
          headers: {
            'Accept': 'application/json',
            'x-requested-with': 'XMLHttpRequest'
          }
        });
        const data = await res.json();
        return JSON.stringify({ success: true, data });
      } catch (error) {
        return JSON.stringify({ success: false, error: error.message });
      }
    })()
  `,
};
```

#### Task 2.6: Setup electron-store

**Time**: 15 minutes  
**Owner**: Developer

```bash
npm install electron-store
```

**src/types/settings.ts**:

```typescript
export interface Settings {
  refreshInterval: 10 | 30 | 60 | 300 | 1800; // seconds
  predictionPeriod: 7 | 14 | 21; // days
  launchAtLogin: boolean;
  notifications: {
    enabled: boolean;
    thresholds: number[]; // e.g., [50, 75, 90, 100]
  };
  theme: "light" | "dark" | "system";
}

export const DEFAULT_SETTINGS: Settings = {
  refreshInterval: 60,
  predictionPeriod: 7,
  launchAtLogin: false,
  notifications: {
    enabled: true,
    thresholds: [75, 90, 100],
  },
  theme: "system",
};
```

**Verification**:

```bash
npm run build
# Should compile without errors
```

---

## Phase 3-9 Summary

Due to length constraints, I'll create a comprehensive checklist document that covers all remaining phases (3-9) with similar detail.

**Phases covered**:

- Phase 3: Authentication System (7 tasks, ~3 hours)
- Phase 4: Data Fetching & Caching (8 tasks, ~3.25 hours)
- Phase 5: UI - Dashboard (10 tasks, ~5.5 hours)
- Phase 6: System Tray (8 tasks, ~4 hours)
- Phase 7: Settings & Preferences (6 tasks, ~2.5 hours)
- Phase 8: Notifications (4 tasks, ~1.5 hours)
- Phase 9: Packaging & Distribution (8 tasks, ~4.75 hours)

---

## Quick Start Commands

```bash
# Development
npm run dev

# Build
npm run build

# Build for specific platforms
npm run build:mac
npm run build:win
npm run build:linux

# Lint
npm run lint

# Type check
npm run typecheck
```

---

## Testing Checklist

- [ ] Authentication flow works
- [ ] Data fetching retrieves accurate usage
- [ ] Prediction calculations match original app
- [ ] System tray shows on all platforms
- [ ] Dark/light theme switches correctly
- [ ] Notifications trigger at thresholds
- [ ] Settings persist across restarts
- [ ] Auto-updater downloads and installs updates
- [ ] App works offline with cached data
- [ ] macOS: Menu bar icon matches native style
- [ ] Windows: Tray icon appears correctly
- [ ] Linux: AppIndicator works on Ubuntu

---

## Progress Tracking

Use this table to track implementation progress:

| Phase                  | Status      | Start Date | End Date | Notes |
| ---------------------- | ----------- | ---------- | -------- | ----- |
| Phase 1: Setup         | Not Started | -          | -        | -     |
| Phase 2: Core Types    | Not Started | -          | -        | -     |
| Phase 3: Auth          | Not Started | -          | -        | -     |
| Phase 4: Data          | Not Started | -          | -        | -     |
| Phase 5: UI            | Not Started | -          | -        | -     |
| Phase 6: Tray          | Not Started | -          | -        | -     |
| Phase 7: Settings      | Not Started | -          | -        | -     |
| Phase 8: Notifications | Not Started | -          | -        | -     |
| Phase 9: Packaging     | Not Started | -          | -        | -     |
