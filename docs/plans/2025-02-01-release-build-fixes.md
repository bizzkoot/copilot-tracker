# Release Build Fixes Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix critical release build issues to ensure GitHub releases work reliably

**Architecture:**

- Add security warnings with bypass instructions to README
- Move hardcoded values to environment variables using dotenv
- Add asset validation with pre-build checks
- Configure canvas as optional dependency for GitHub Actions
- Wrap debug code to only show in development mode

**Tech Stack:** Electron, electron-builder, GitHub Actions, canvas, dotenv

---

## Summary of Issues

This plan addresses 5 critical issues:

1. **Code Signing** - No free options exist, will add warnings + bypass instructions to README
2. **Hardcoded Values** - `costPerRequest`, GitHub URLs, app ID duplicated across files
3. **Asset Validation** - No checks if resources exist before build
4. **Canvas Dependency** - Native module that may fail in CI
5. **Debug Code** - Console.log statements in production

---

## Task 1: Add Security Warnings to README

**Files:**

- Modify: `README.md`

**Step 1: Add security warning section to README**

Insert this section after the "Installation" section (after line 57):

````markdown
## Security Warnings (Unsigned Builds)

⚠️ **This application is distributed without code signing.** You may see security warnings when first running the app. This is expected for open-source projects without paid developer certificates.

### macOS

When you first try to open the app, you may see:

> "App cannot be opened because it was not downloaded from the App Store"

**To bypass:**

**Option 1: Right-click method (GUI)**

1. Right-click (or Control-click) on the app
2. Select "Open"
3. Click "Open" in the confirmation dialog

**Option 2: Terminal method (remove quarantine)**

```bash
# After copying the app to Applications folder
xattr -cr /Applications/copilot-tracker.app
```
````

**Option 3: System Settings**

1. Open System Settings → Privacy & Security
2. Find the message about the app being blocked
3. Click "Open Anyway"

### Windows

When you first run the installer or app, you may see:

> "Microsoft Defender SmartScreen prevented an unrecognized app from starting"

**To bypass:**

1. Click "More info"
2. Click "Run anyway"

### Linux

No warnings - Linux apps run without restrictions.

### Why Unsigned?

Code signing requires paid certificates:

- **macOS**: Apple Developer Program ($99/year)
- **Windows**: Code signing certificate ($100-300/year)

As an open-source project, we distribute unsigned builds. The source code is publicly available for review if you wish to verify the app's safety.

````

**Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add security warnings for unsigned builds"
````

---

## Task 2: Add Environment Variables Configuration

**Files:**

- Create: `.env.example`
- Create: `src/main/config.ts`
- Modify: `package.json`
- Modify: `src/main/index.ts`

**Step 1: Install dotenv**

Run: `npm install --save dotenv`

Expected: dotenv added to dependencies

**Step 2: Create .env.example**

Create `.env.example`:

```env
# App Configuration
APP_ID=com.electron.copilot-tracker
PRODUCT_NAME=copilot-tracker

# GitHub URLs
GITHUB_BILLING_URL=https://github.com/settings/billing/premium_requests_usage
GITHUB_LOGIN_URL=https://github.com/login

# Cost Configuration
COST_PER_REQUEST=0.04

# Development (do not commit actual .env file)
NODE_ENV=development
```

**Step 3: Create config module**

Create `src/main/config.ts`:

```typescript
import { app } from "electron";

// Check if running in development mode
export const isDevelopment = !app.isPackaged;

// Load configuration from environment variables with defaults
export const config = {
  appId: process.env.APP_ID || "com.electron.copilot-tracker",
  productName: process.env.PRODUCT_NAME || "copilot-tracker",
  githubBillingUrl:
    process.env.GITHUB_BILLING_URL ||
    "https://github.com/settings/billing/premium_requests_usage",
  githubLoginUrl: process.env.GITHUB_LOGIN_URL || "https://github.com/login",
  costPerRequest: parseFloat(process.env.COST_PER_REQUEST || "0.04"),
} as const;

// Development logger - only logs in development
export const devLog = {
  log: (...args: unknown[]) => {
    if (isDevelopment) {
      console.log("[Dev]", ...args);
    }
  },
  error: (...args: unknown[]) => {
    if (isDevelopment) {
      console.error("[Dev]", ...args);
    }
  },
};
```

**Step 4: Update package.json scripts**

Modify `package.json` scripts section:

```json
"scripts": {
  "format": "prettier --write .",
  "lint": "eslint . --ext .js,.jsx,.cjs,.mjs,.ts,.tsx,.cts,.mts --fix",
  "typecheck:node": "tsc --noEmit -p tsconfig.node.json --composite false",
  "typecheck:web": "tsc --noEmit -p tsconfig.web.json --composite false",
  "typecheck": "npm run typecheck:node && npm run typecheck:web",
  "start": "electron-vite preview",
  "dev": "cross-env-shell NODE_ENV=development electron-vite dev",
  "build": "npm run typecheck && electron-vite build",
  "postinstall": "electron-builder install-app-deps",
  "build:unpack": "npm run build && electron-builder --dir",
  "build:win": "npm run build && electron-builder --win",
  "build:mac": "npm run build && electron-builder --mac",
  "build:linux": "npm run build && electron-builder --linux"
},
```

**Step 5: Install cross-env**

Run: `npm install --save-dev cross-env`

Expected: cross-env added to devDependencies (for Windows compatibility)

**Step 6: Update src/main/index.ts - import config**

Modify `src/main/index.ts` line 12-15 (after imports):

```typescript
import { join } from "path";
import Store from "electron-store";
import { config, devLog } from "./config";

const icon = join(__dirname, "../../resources/icon.png");
```

Remove lines 58-60 (old constants):

```typescript
// OLD - Remove these lines:
// const GITHUB_BILLING_URL =
//   "https://github.com/settings/billing/premium_requests_usage";
// const GITHUB_LOGIN_URL = "https://github.com/login";
```

**Step 7: Update src/main/index.ts - replace GITHUB\_\* URLs**

Find and replace all occurrences of `GITHUB_BILLING_URL` with `config.githubBillingUrl`:

- Line 288: `shell.openExternal(config.githubBillingUrl);`
- Line 699: `authView.webContents.loadURL(config.githubBillingUrl);`
- Line 805: `authView.webContents.loadURL(config.githubBillingUrl);`
- Line 1304: `shell.openExternal(config.githubBillingUrl);`

Find and replace all occurrences of `GITHUB_LOGIN_URL` with `config.githubLoginUrl`:

- Line 757: `authWindow.loadURL(config.githubLoginUrl);`
- Line 812: `authWindow.loadURL(config.githubLoginUrl);`

**Step 8: Update src/main/index.ts - replace costPerRequest**

Modify `src/main/index.ts` line 505:

```typescript
// OLD: const costPerRequest = 0.04;
const costPerRequest = config.costPerRequest;
```

**Step 9: Update src/main/index.ts - replace console.log with devLog**

Replace all debug console.log statements with devLog:

- Line 387: `devLog.error("[Tray] Failed to create custom icon:", e);`
- Line 579: `devLog.log("[TrayIcon] Loading icon from:", basePath);`
- Line 592: `devLog.log("[TrayIcon] Failed to load base icon:", e);`
- Line 590: `devLog.log("[TrayIcon] Base icon loaded successfully");` (currently commented out)
- Line 611: `devLog.log("[TrayIcon] Drawing cost text:", costText);`
- Line 618: `devLog.log("[TrayIcon] Drawing count text:", countText);`
- Line 655: `devLog.log("[TrayIcon] Created native image, size:", nativeImg.getSize());`
- Line 658: `devLog.error("[TrayIcon] Failed to create custom icon:", e);`

Replace authentication console.log statements (keep critical errors):

- Line 703: `devLog.log("[Auth] Navigated to:", url);`
- Line 705: `devLog.log("[Auth] Detected login page");`
- Line 710: `devLog.log("[Auth] Detected billing page - user is authenticated");`
- Line 728: `devLog.log("[Auth] Page finished loading:", url);`
- Line 730: `devLog.log("[Auth] Login page loaded");`
- Line 734: `devLog.log("[Auth] Billing page loaded - authenticated");`
- Line 793: `devLog.log("[Auth] AuthWindow navigation:", url);`
- Line 804: `devLog.log("[Auth] User logged in, loading billing page");`
- Line 830: `devLog.log("[Auth] Getting customer ID from URL:", url);`
- Line 834: `devLog.log("[Auth] Found customer ID in URL:", customerId);`
- Line 868: `devLog.log("[Auth] Trying Method 2: DOM extraction from script tag");`
- Line 886: `devLog.log("[Auth] Method 2 result:", result);`
- Line 891: `devLog.log("[Auth] Method 2 success - Customer ID:", customerId);`
- Line 894: `devLog.log("[Auth] Method 2 failed:", result);`
- Line 897: `devLog.error("[Auth] Method 2 exception:", e);`

Replace usage console.log statements (keep critical errors):

- Line 932: `devLog.log("[Usage] Starting fetchUsageData");`
- Line 937: `devLog.log("[Usage] Got customer ID:", id);`
- Line 947: `devLog.log("[Usage] Fetching usage card for customer:", id);`
- Line 951-962: Keep existing verbose logs (API debugging)
- Line 967-971: Keep existing logs
- Line 981: `devLog.log("[Usage] Fetching usage history");`
- Line 988-995: Keep existing logs (API debugging)
- Line 1000-1004: Keep existing logs
- Line 1006-1046: Keep existing verbose logs (debugging history structure)
- Line 1050-1063: Keep existing logs (data parsing debugging)

Replace shortcut console.log statements:

- Line 1297: `devLog.log("[Shortcuts] Refresh triggered");`
- Line 1303: `devLog.log("[Shortcuts] Open Billing triggered");`
- Line 1307: `devLog.log("[Shortcuts] Registered keyboard shortcuts");`
- Line 1315: `devLog.log("[Shortcuts] Unregistered all keyboard shortcuts");`

**Step 10: Update src/renderer/src/types/usage.ts**

Modify `src/renderer/src/types/usage.ts` line 72:

```typescript
// OLD: export const COST_PER_REQUEST = 0.04;
export const COST_PER_REQUEST = parseFloat(
  import.meta.env.VITE_COST_PER_REQUEST || "0.04",
);
```

**Step 11: Update .gitignore**

Modify `.gitignore` to add:

```gitignore
# Environment variables
.env
.env.local
.env.production
```

**Step 12: Test in development**

Run: `npm run dev`

Expected: App starts without errors, config values loaded correctly

**Step 13: Commit**

```bash
git add .env.example src/main/config.ts package.json package-lock.json src/main/index.ts src/renderer/src/types/usage.ts .gitignore
git commit -m "feat: add environment variables configuration and dev-only logging"
```

---

## Task 3: Add Asset Validation

**Files:**

- Create: `scripts/validate-assets.ts`
- Modify: `package.json`

**Step 1: Install ts-node for running TypeScript scripts**

Run: `npm install --save-dev ts-node`

Expected: ts-node added to devDependencies

**Step 2: Create asset validation script**

Create `scripts/validate-assets.ts`:

```typescript
import { readFileSync, existsSync } from "fs";
import { join } from "path";
import { exit } from "process";

const requiredAssets = [
  "resources/icon.png",
  "resources/icon.icns",
  "resources/tray/tray.png",
  "resources/tray/trayTemplate.png",
];

let hasErrors = false;

console.log("🔍 Validating required assets...\n");

for (const asset of requiredAssets) {
  const assetPath = join(__dirname, "..", asset);

  if (!existsSync(assetPath)) {
    console.error(`❌ Missing: ${asset}`);
    hasErrors = true;
  } else {
    // Check file size
    const stats = readFileSync(assetPath);
    if (stats.length === 0) {
      console.error(`❌ Empty file: ${asset}`);
      hasErrors = true;
    } else {
      console.log(`✅ Found: ${asset} (${stats.length} bytes)`);
    }
  }
}

if (hasErrors) {
  console.error("\n❌ Asset validation failed!");
  console.error("Please ensure all required assets exist before building.\n");
  exit(1);
}

console.log("\n✅ All assets validated successfully!\n");
exit(0);
```

**Step 3: Add validation script to package.json**

Modify `package.json` scripts section:

```json
"scripts": {
  "validate:assets": "ts-node scripts/validate-assets.ts",
  "format": "prettier --write .",
  "lint": "eslint . --ext .js,.jsx,.cjs,.mjs,.ts,.tsx,.cts,.mts --fix",
  "typecheck:node": "tsc --noEmit -p tsconfig.node.json --composite false",
  "typecheck:web": "tsc --noEmit -p tsconfig.web.json --composite false",
  "typecheck": "npm run typecheck:node && npm run typecheck:web",
  "start": "electron-vite preview",
  "dev": "cross-env-shell NODE_ENV=development electron-vite dev",
  "build": "npm run validate:assets && npm run typecheck && electron-vite build",
  "postinstall": "electron-builder install-app-deps",
  "build:unpack": "npm run build && electron-builder --dir",
  "build:win": "npm run build && electron-builder --win",
  "build:mac": "npm run build && electron-builder --mac",
  "build:linux": "npm run build && electron-builder --linux"
}
```

**Step 4: Test asset validation**

Run: `npm run validate:assets`

Expected: `✅ All assets validated successfully!`

**Step 5: Test with missing asset (simulate failure)**

Run: `mv resources/icon.png resources/icon.png.bak && npm run validate:assets`

Expected: `❌ Missing: resources/icon.png` and exit code 1

Then restore: `mv resources/icon.png.bak resources/icon.png`

**Step 6: Commit**

```bash
git add scripts/validate-assets.ts package.json package-lock.json
git commit -m "feat: add asset validation script"
```

---

## Task 4: Configure Canvas Locally (Do NOT push yet!)

**Files:**

- Modify: `electron-builder.yml`
- Modify: `src/main/index.ts`

**Step 1: Update electron-builder.yml to handle canvas**

Modify `electron-builder.yml` line 12-13:

```yaml
asarUnpack:
  - resources/**
  - node_modules/canvas/** # Unpack canvas native module
```

**Step 2: Add fallback for canvas in main process**

Modify `src/main/index.ts` line 557-573 (createTrayIconWithNumbers function):

```typescript
function createTrayIconWithNumbers(
  used: number,
  limit: number,
  addOnCost: number = 0,
): Electron.NativeImage {
  try {
    const size = 16;
    // Try to import canvas, with fallback if not available
    let createCanvas: any;
    try {
      const canvasModule = require("canvas");
      createCanvas = canvasModule.createCanvas;
    } catch (e) {
      devLog.error("[TrayIcon] Canvas module not available:", e);
      // Return a simple fallback icon
      return nativeImage.createFromPath(icon);
    }
```

**Step 3: Test build locally**

Run: `npm run build:unpack`

Expected: Build succeeds, tray icon renders correctly

**Step 4: Commit (local only, do NOT push yet!)**

```bash
git add electron-builder.yml src/main/index.ts
git commit -m "fix: configure canvas dependency with fallback"
```

⚠️ **DO NOT PUSH TO GITHUB YET!** The GitHub Actions workflow update (Task 6) must be done last.

---

## Task 5: Update README with Build Information

**Files:**

- Modify: `README.md`

**Step 1: Add build instructions to README**

Add section to `README.md` (after the "Development" section, before "Tech Stack"):

````markdown
## Building for Production

### Prerequisites

For Linux builds, install canvas dependencies:

```bash
sudo apt-get install build-essential libcairo2-dev libpango1.0-dev \
  libjpeg-dev libgif-dev librsvg2-dev libxi-dev
```
````

For macOS builds, install:

```bash
brew install cairo pango libjpeg giflib librsvg
```

### Build Commands

```bash
# Build for current platform (unsigned)
npm run build

# Build for specific platforms
npm run build:mac      # macOS DMG
npm run build:win      # Windows installer
npm run build:linux    # Linux AppImage, snap, deb

# Build without packaging (for testing)
npm run build:unpack
```

### Environment Variables

Copy `.env.example` to `.env` and configure:

```bash
cp .env.example .env
```

Available variables:

- `APP_ID` - Application identifier
- `PRODUCT_NAME` - Product name
- `GITHUB_BILLING_URL` - GitHub billing page URL
- `GITHUB_LOGIN_URL` - GitHub login page URL
- `COST_PER_REQUEST` - Cost per Copilot request (default: 0.04)

````

**Step 2: Commit**

```bash
git add README.md
git commit -m "docs: add build instructions to README"
````

---

## Task 6: Update GitHub Actions Workflow ⚠️ DO LAST!

⚠️ **IMPORTANT:** This task modifies `.github/workflows/release.yml` which will trigger the GitHub Actions workflow when pushed. Only do this task after ALL other changes are complete and tested.

**Files:**

- Modify: `.github/workflows/release.yml`

**Step 1: Update GitHub Actions to handle canvas dependency**

Modify `.github/workflows/release.yml` lines 46-50:

```yaml
- name: Install Dependencies (Linux)
  if: runner.os == 'Linux'
  run: |
    sudo apt-get update
    sudo apt-get install -y build-essential libcairo2-dev libpango1.0-dev libjpeg-dev libgif-dev librsvg2-dev libxi-dev

- name: Install Dependencies (macOS)
  if: runner.os == 'macOS'
  run: |
    brew install cairo pango libjpeg giflib librsvg

- name: Install Dependencies (Windows)
  if: runner.os == 'Windows'
  run: |
    choco install -y gtk-runtime
```

**Step 2: Add optional canvas rebuild**

Modify `.github/workflows/release.yml` after dependencies (after line 54):

```yaml
- name: Rebuild Canvas (if needed)
  run: |
    if [ "$RUNNER_OS" == "Windows" ]; then
      # Windows: Try to rebuild canvas, continue on failure
      npm rebuild canvas --no-optional || echo "Canvas rebuild skipped"
    else
      # Unix: Try to rebuild canvas, continue on failure
      npm rebuild canvas --no-optional || echo "Canvas rebuild skipped"
    fi
  shell: bash
```

**Step 3: Verify workflow changes**

Check that the workflow file looks correct:

```bash
cat .github/workflows/release.yml
```

**Step 4: Commit and push (THIS WILL TRIGGER GITHUB ACTIONS)**

```bash
git add .github/workflows/release.yml
git commit -m "ci: configure canvas dependencies for GitHub Actions"
git push
```

🚀 **After pushing:** Monitor the GitHub Actions run to ensure the build succeeds.

---

## Task 7: Final Testing

**Step 1: Test development mode**

Run: `npm run dev`

Expected checks:

- App starts successfully
- Tray icon appears
- Debug logs appear in console
- Environment variables loaded correctly

**Step 2: Test production build**

Run: `npm run build:unpack`

Expected checks:

- Asset validation passes
- Build completes without errors
- App runs from `dist/` folder
- No debug logs in production console

**Step 3: Test asset validation**

Run: `npm run validate:assets`

Expected: All assets validated successfully

**Step 4: Test with missing asset**

Run: `mv resources/icon.png resources/icon.png.bak && npm run build`

Expected: Build fails with asset validation error

Restore: `mv resources/icon.png.bak resources/icon.png`

**Step 5: Final commit**

```bash
git add .
git commit -m "chore: final cleanup after release build fixes"
```

---

## Summary

After completing this plan:

1. ✅ Security warnings added to README with bypass instructions
2. ✅ All hardcoded values moved to environment variables
3. ✅ Asset validation prevents build failures
4. ✅ Canvas dependency configured for GitHub Actions
5. ✅ Debug code only shows in development mode
6. ✅ README updated with build instructions

### Files Created

- `.env.example` - Environment variable template
- `src/main/config.ts` - Configuration module with devLog
- `scripts/validate-assets.ts` - Asset validation script

### Files Modified

- `README.md` - Security warnings, build instructions
- `package.json` - Added dotenv, cross-env, ts-node, new scripts
- `src/main/index.ts` - Use config, devLog for logging
- `src/renderer/src/types/usage.ts` - Use env for COST_PER_REQUEST
- `.github/workflows/release.yml` - Canvas dependencies
- `electron-builder.yml` - Unpack canvas module
- `.gitignore` - Ignore .env files
