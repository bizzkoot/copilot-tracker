import {
  app,
  shell,
  BrowserWindow,
  ipcMain,
  Tray,
  Menu,
  nativeImage,
  WebContentsView,
} from "electron";
import { join } from "path";
import Store from "electron-store";
import dotenv from "dotenv";
import { config, devLog } from "./config";

// Load environment variables from .env file
dotenv.config();

const icon = join(__dirname, "../../resources/icon.png");

// Types
interface Settings {
  refreshInterval: number;
  predictionPeriod: number;
  launchAtLogin: boolean;
  notifications: {
    enabled: boolean;
    thresholds: number[];
  };
  theme: "light" | "dark" | "system";
}

interface CopilotUsage {
  netBilledAmount: number;
  netQuantity: number;
  discountQuantity: number;
  userPremiumRequestEntitlement: number;
  filteredUserPremiumRequestEntitlement: number;
}

interface DailyUsage {
  date: string;
  includedRequests: number;
  billedRequests: number;
  grossAmount: number;
  billedAmount: number;
}

interface UsageHistory {
  fetchedAt: string;
  days: DailyUsage[];
}

interface UsagePrediction {
  predictedMonthlyRequests: number;
  predictedBilledAmount: number;
  confidenceLevel: "low" | "medium" | "high";
  daysUsedForPrediction: number;
}

// Constants
const DEFAULT_SETTINGS: Settings = {
  refreshInterval: 60,
  predictionPeriod: 7,
  launchAtLogin: false,
  notifications: {
    enabled: true,
    thresholds: [75, 90, 100],
  },
  theme: "system",
};

// Store for settings and cache
const store = new Store({
  defaults: {
    settings: DEFAULT_SETTINGS,
    cache: {
      usage: null,
      history: null,
      lastFetched: null,
    },
  },
});

// Global references
let mainWindow: BrowserWindow | null = null;
let authWindow: BrowserWindow | null = null;
let tray: Tray | null = null;
let authView: WebContentsView | null = null;
let customerId: number | null = null;
let refreshTimer: NodeJS.Timeout | null = null;
let usageHistory: UsageHistory | null = null;
let usagePrediction: UsagePrediction | null = null;
let trayBaseIconBuffer: Buffer | null = null;

// ============= Window Management =============

function createWindow(): void {
  mainWindow = new BrowserWindow({
    width: 900,
    height: 700,
    minWidth: 600,
    minHeight: 500,
    show: false,
    autoHideMenuBar: true,
    titleBarStyle: process.platform === "darwin" ? "hiddenInset" : "default",
    ...(process.platform === "linux" ? { icon } : {}),
    webPreferences: {
      preload: join(__dirname, "../preload/index.js"),
      sandbox: false,
      contextIsolation: true,
      nodeIntegration: false,
    },
  });

  mainWindow.on("ready-to-show", () => {
    mainWindow?.show();
  });

  mainWindow.on("close", (event) => {
    // On macOS, hide window and app from dock instead of closing
    if (process.platform === "darwin") {
      event.preventDefault();
      mainWindow?.hide();
      app.hide(); // Also hide from dock so only tray icon remains visible
    }
  });

  mainWindow.webContents.setWindowOpenHandler((details) => {
    shell.openExternal(details.url);
    return { action: "deny" };
  });

  // HMR for renderer
  if (!app.isPackaged && process.env["ELECTRON_RENDERER_URL"]) {
    mainWindow.loadURL(process.env["ELECTRON_RENDERER_URL"]);
  } else {
    mainWindow.loadFile(join(__dirname, "../renderer/index.html"));
  }
}

// ============= System Tray =============

function createTray(): void {
  const trayIconPath =
    process.platform === "darwin"
      ? join(__dirname, "../../resources/tray/trayTemplate.png")
      : join(__dirname, "../../resources/tray/tray.png");

  // Use a fallback icon if tray icon doesn't exist
  let trayIcon: Electron.NativeImage;
  try {
    trayIcon = nativeImage.createFromPath(trayIconPath);
    if (trayIcon.isEmpty()) {
      trayIcon = nativeImage.createFromPath(icon);
    }
  } catch {
    trayIcon = nativeImage.createFromPath(icon);
  }

  if (process.platform === "darwin") {
    trayIcon = trayIcon.resize({ width: 16, height: 16 });
  }

  tray = new Tray(trayIcon);
  tray.setToolTip("Copilot Tracker");

  updateTrayMenu();

  tray.on("click", () => {
    if (process.platform === "darwin") {
      // On macOS, clicking the tray icon shows the menu
    } else {
      // On Windows/Linux, toggle window visibility
      if (mainWindow?.isVisible()) {
        mainWindow.hide();
      } else {
        mainWindow?.show();
      }
    }
  });
}

function updateTrayMenu(usage?: CopilotUsage | null): void {
  if (!tray) return;

  const settings = store.get("settings") as Settings;
  const version = app.getVersion() || "0.0.1";

  // Usage label with progress
  const usageLabel = usage
    ? `Used: ${Math.round(usage.discountQuantity)} / ${usage.userPremiumRequestEntitlement} (${((usage.discountQuantity / usage.userPremiumRequestEntitlement) * 100).toFixed(1)}%)`
    : "Loading...";

  // Build menu items
  const menuItems: Electron.MenuItemConstructorOptions[] = [
    {
      label: usageLabel,
      enabled: false,
    },
    { type: "separator" },
  ];

  // Add Usage History submenu if we have history data
  if (usageHistory && usageHistory.days.length > 0) {
    const historySubmenu: Electron.MenuItemConstructorOptions[] = [];

    // Add prediction if available
    if (usagePrediction) {
      historySubmenu.push({
        label: `Predicted EOM: ${Math.round(usagePrediction.predictedMonthlyRequests)} requests`,
        enabled: false,
      });

      if (usagePrediction.predictedBilledAmount > 0) {
        historySubmenu.push({
          label: `Predicted Add-on: $${usagePrediction.predictedBilledAmount.toFixed(2)}`,
          enabled: false,
        });
      }

      // Confidence level
      const confidenceText =
        usagePrediction.confidenceLevel === "high"
          ? "High prediction accuracy"
          : usagePrediction.confidenceLevel === "medium"
            ? "Medium prediction accuracy"
            : "Low prediction accuracy";

      historySubmenu.push({
        label: confidenceText,
        enabled: false,
      });

      historySubmenu.push({ type: "separator" });
    }

    // Add last 7 days of history
    const recentDays = usageHistory.days.slice(0, 7);
    const dateFormatter = new Intl.DateTimeFormat("en-US", {
      month: "short",
      day: "numeric",
    });

    for (const day of recentDays) {
      const totalRequests = day.includedRequests + day.billedRequests;
      const dateStr = dateFormatter.format(new Date(day.date));
      historySubmenu.push({
        label: `${dateStr}: ${Math.round(totalRequests)} req`,
        enabled: false,
      });
    }

    menuItems.push({
      label: "Usage History",
      submenu: historySubmenu,
    });
  }

  // Add Prediction Period submenu
  const predictionPeriods = [
    { label: "7 days", value: 7 },
    { label: "14 days", value: 14 },
    { label: "21 days", value: 21 },
  ];

  const predictionSubmenu = predictionPeriods.map((period) => ({
    label: period.label,
    type: "radio" as const,
    checked: settings.predictionPeriod === period.value,
    click: (): void => {
      mainWindow?.webContents.send("settings:changed", {
        predictionPeriod: period.value,
      });
      store.set("settings.predictionPeriod", period.value);
    },
  }));

  menuItems.push({
    label: "Prediction Period",
    submenu: predictionSubmenu,
  });

  menuItems.push({ type: "separator" });

  // Add Open Billing action
  menuItems.push({
    label: "Open Billing",
    click: (): void => {
      shell.openExternal(config.githubBillingUrl);
    },
  });

  // Add Refresh
  menuItems.push({
    label: "Refresh",
    click: (): void => {
      fetchUsageData();
    },
  });

  // Add Auto-Refresh interval submenu
  const refreshIntervals = [
    { label: "10 seconds", value: 10 },
    { label: "30 seconds", value: 30 },
    { label: "1 minute", value: 60 },
    { label: "5 minutes", value: 300 },
    { label: "30 minutes", value: 1800 },
  ];

  const refreshSubmenu = refreshIntervals.map((interval) => ({
    label: interval.label,
    type: "radio" as const,
    checked: settings.refreshInterval === interval.value,
    click: (): void => {
      store.set("settings.refreshInterval", interval.value);
      startRefreshTimer();
      mainWindow?.webContents.send("settings:changed", {
        refreshInterval: interval.value,
      });
    },
  }));

  menuItems.push({
    label: "Auto Refresh",
    submenu: refreshSubmenu,
  });

  menuItems.push({ type: "separator" });

  // Add Settings
  menuItems.push({
    label: "Settings",
    click: (): void => {
      mainWindow?.show();
      mainWindow?.focus();
      mainWindow?.webContents.send("navigate", "settings");
    },
  });

  // Add Launch at Login toggle
  menuItems.push({
    label: "Launch at Login",
    type: "checkbox" as const,
    checked: settings.launchAtLogin,
    click: (menuItem): void => {
      const enabled = menuItem.checked;
      store.set("settings.launchAtLogin", enabled);
      app.setLoginItemSettings({ openAtLogin: enabled });
      mainWindow?.webContents.send("settings:changed", {
        launchAtLogin: enabled,
      });
    },
  });

  menuItems.push({ type: "separator" });

  // Add version
  menuItems.push({
    label: `Version ${version}`,
    enabled: false,
  });

  // Add Quit
  menuItems.push({
    label: "Quit",
    click: (): void => {
      app.quit();
    },
  });

  const menu = Menu.buildFromTemplate(menuItems);
  tray.setContextMenu(menu);

  if (usage) {
    tray.setToolTip(
      `Copilot: ${Math.round(usage.discountQuantity)}/${usage.userPremiumRequestEntitlement}`,
    );

    // Update tray icon with live numbers (like Swift app)
    try {
      const iconWithNumbers = createTrayIconWithNumbers(
        Math.round(usage.discountQuantity),
        usage.userPremiumRequestEntitlement,
        usage.netBilledAmount,
      );
      tray.setImage(iconWithNumbers);
    } catch (e) {
      devLog.error("[Tray] Failed to create custom icon:", e);
      // Fallback to default icon (no change)
    }
  }
}

// ============= Prediction Algorithm =============

/**
 * Calculates end-of-month usage prediction
 * Ported from Swift app's UsagePredictor
 */
function calculatePrediction(
  history: UsageHistory,
  currentUsage: CopilotUsage,
  period: number,
): UsagePrediction {
  const days = history.days;

  // Edge case: No data
  if (days.length === 0) {
    return {
      predictedMonthlyRequests: 0,
      predictedBilledAmount: 0,
      confidenceLevel: "low",
      daysUsedForPrediction: 0,
    };
  }

  // Step 1: Get weights for the period
  const weights = getWeightsForPeriod(period);

  // Step 2: Calculate weighted average daily usage
  const sortedDays = [...days].sort(
    (a, b) => new Date(b.date).getTime() - new Date(a.date).getTime(),
  );
  let weightedSum = 0;
  let totalWeight = 0;
  const daysToUse = Math.min(sortedDays.length, weights.length);

  for (let i = 0; i < daysToUse; i++) {
    const day = sortedDays[i];
    const totalRequests = day.includedRequests + day.billedRequests;
    const weight = weights[i];
    weightedSum += totalRequests * weight;
    totalWeight += weight;
  }

  const weightedAvgDailyUsage = totalWeight > 0 ? weightedSum / totalWeight : 0;

  // Step 3: Calculate weekend/weekday ratio
  let weekdaySum = 0;
  let weekendSum = 0;
  let weekdayCount = 0;
  let weekendCount = 0;

  for (const day of days) {
    const date = new Date(day.date);
    const dayOfWeek = date.getDay(); // 0 = Sunday, 6 = Saturday
    const totalRequests = day.includedRequests + day.billedRequests;

    if (dayOfWeek === 0 || dayOfWeek === 6) {
      weekendSum += totalRequests;
      weekendCount++;
    } else {
      weekdaySum += totalRequests;
      weekdayCount++;
    }
  }

  const weekdayAvg = weekdayCount > 0 ? weekdaySum / weekdayCount : 0;
  const weekendAvg = weekendCount > 0 ? weekendSum / weekendCount : 0;
  const weekendRatio = weekdayAvg > 0 ? weekendAvg / weekdayAvg : 1.0;

  // Step 4: Calculate remaining days
  const today = new Date();
  const daysInMonth = new Date(
    today.getFullYear(),
    today.getMonth() + 1,
    0,
  ).getDate();
  const currentDay = today.getDate();
  const remainingDays = daysInMonth - currentDay;

  let remainingWeekdays = 0;
  let remainingWeekends = 0;

  for (let i = 1; i <= remainingDays; i++) {
    const futureDate = new Date(
      today.getFullYear(),
      today.getMonth(),
      today.getDate() + i,
    );
    const dayOfWeek = futureDate.getDay();

    if (dayOfWeek === 0 || dayOfWeek === 6) {
      remainingWeekends++;
    } else {
      remainingWeekdays++;
    }
  }

  // Step 5: Predict total monthly usage
  const currentTotalUsage = days.reduce(
    (sum, day) => sum + day.includedRequests + day.billedRequests,
    0,
  );
  const predictedRemainingWeekdayUsage =
    weightedAvgDailyUsage * remainingWeekdays;
  const predictedRemainingWeekendUsage =
    weightedAvgDailyUsage * weekendRatio * remainingWeekends;
  const predictedMonthlyTotal =
    currentTotalUsage +
    predictedRemainingWeekdayUsage +
    predictedRemainingWeekendUsage;

  // Step 6: Calculate predicted add-on cost
  const limit = currentUsage.userPremiumRequestEntitlement;
  const costPerRequest = config.costPerRequest;
  const predictedBilledAmount =
    predictedMonthlyTotal > limit
      ? (predictedMonthlyTotal - limit) * costPerRequest
      : 0;

  // Step 7: Determine confidence level
  let confidenceLevel: "low" | "medium" | "high";
  if (days.length < 3) {
    confidenceLevel = "low";
  } else if (days.length < 7) {
    confidenceLevel = "medium";
  } else {
    confidenceLevel = "high";
  }

  return {
    predictedMonthlyRequests: predictedMonthlyTotal,
    predictedBilledAmount,
    confidenceLevel,
    daysUsedForPrediction: days.length,
  };
}

/**
 * Get weights for prediction period
 * Matched to Swift app's weights
 */
function getWeightsForPeriod(period: number): number[] {
  switch (period) {
    case 7:
      return [1.5, 1.5, 1.2, 1.2, 1.2, 1.0, 1.0];
    case 14:
      return [
        1.5, 1.5, 1.4, 1.4, 1.3, 1.3, 1.2, 1.2, 1.1, 1.1, 1.0, 1.0, 1.0, 1.0,
      ];
    case 21:
      return [
        1.5, 1.5, 1.4, 1.4, 1.3, 1.3, 1.2, 1.2, 1.2, 1.1, 1.1, 1.1, 1.0, 1.0,
        1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
      ];
    default:
      return [1.5, 1.5, 1.2, 1.2, 1.2, 1.0, 1.0];
  }
}

// ============= Custom Tray Icon with Numbers =============

/**
 * Creates a custom tray icon with usage numbers overlaid
 * Similar to the Swift app's StatusBarIconView
 */
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
    const canvasObj = createCanvas(size * 2, size); // Double width for icon + text
    const ctx = canvasObj.getContext("2d");

    // Clear background (transparent)
    ctx.clearRect(0, 0, size * 2, size);

    // Load base icon - use cached buffer if available, otherwise read from disk
    let iconLoaded = false;
    try {
      if (!trayBaseIconBuffer) {
        const basePath = join(
          __dirname,
          "../../resources/tray/trayTemplate.png",
        );
        devLog.log("[TrayIcon] Loading icon from:", basePath);
        const fs = require("fs");
        trayBaseIconBuffer = fs.readFileSync(basePath);
      }

      const img = new (require("canvas").Image)();
      img.src = trayBaseIconBuffer;

      // Draw icon at 16x16 size
      ctx.drawImage(img, 0, 0, size, size);
      iconLoaded = true;
      // console.log("[TrayIcon] Base icon loaded successfully");
    } catch (e) {
      devLog.log("[TrayIcon] Failed to load base icon:", e);
    }

    // Fallback: draw a simple icon if image didn't load
    if (!iconLoaded) {
      ctx.fillStyle = "#FFFFFF";
      ctx.font = "12px Arial";
      ctx.fillText("🤖", 0, 12);
    }

    // If there's add-on cost, show dollar amount
    if (addOnCost > 0) {
      const costText =
        addOnCost >= 10
          ? `$${addOnCost.toFixed(1)}`
          : `$${addOnCost.toFixed(2)}`;
      ctx.fillStyle = "#FFFFFF";
      ctx.font = "bold 10px monospace";
      ctx.fillText(costText, size + 2, 12);
      devLog.log("[TrayIcon] Drawing cost text:", costText);
    } else {
      // Show usage count
      const countText = used.toString();
      ctx.fillStyle = "#FFFFFF";
      ctx.font = "bold 11px monospace";
      ctx.fillText(countText, size + 2, 12);
      devLog.log("[TrayIcon] Drawing count text:", countText);

      // Draw small progress circle
      const percentage = limit > 0 ? Math.min((used / limit) * 100, 100) : 0;
      const circleX = size + 2 + ctx.measureText(countText).width + 8;
      const circleY = 8;
      const radius = 4;

      // Background circle
      ctx.strokeStyle = "rgba(255, 255, 255, 0.3)";
      ctx.lineWidth = 2;
      ctx.beginPath();
      ctx.arc(circleX, circleY, radius, 0, Math.PI * 2);
      ctx.stroke();

      // Progress arc
      const startAngle = -Math.PI / 2;
      const endAngle = startAngle + (Math.PI * 2 * percentage) / 100;

      // Color based on percentage
      if (percentage < 50) {
        ctx.strokeStyle = "#22c55e"; // green
      } else if (percentage < 75) {
        ctx.strokeStyle = "#eab308"; // yellow
      } else if (percentage < 90) {
        ctx.strokeStyle = "#f97316"; // orange
      } else {
        ctx.strokeStyle = "#ef4444"; // red
      }

      ctx.beginPath();
      ctx.arc(circleX, circleY, radius, startAngle, endAngle);
      ctx.stroke();
    }

    const dataUrl = canvasObj.toDataURL();
    const nativeImg = nativeImage.createFromDataURL(dataUrl);
    devLog.log("[TrayIcon] Created native image, size:", nativeImg.getSize());
    return nativeImg;
  } catch (e) {
    devLog.error("[TrayIcon] Failed to create custom icon:", e);
    return nativeImage.createFromPath(icon); // Fallback instead of throw
  }
}

// ============= Authentication =============

function destroyAuthView(): void {
  if (!authView) return;

  // Remove all event listeners
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const wc = (authView as any).webContents as Electron.WebContents;
  if (wc && !wc.isDestroyed()) {
    wc.removeAllListeners("did-navigate");
    wc.removeAllListeners("will-redirect");
    wc.removeAllListeners("did-finish-load");
    wc.close();
  }

  authView = null;
  customerId = null;
}

function createAuthView(): void {
  if (!mainWindow) return;

  // Clean up existing auth view if any
  if (authView) {
    destroyAuthView();
  }

  authView = new WebContentsView({
    webPreferences: {
      partition: "persist:github",
      nodeIntegration: false,
      contextIsolation: true,
    },
  });

  // Navigate to billing page
  authView.webContents.loadURL(config.githubBillingUrl);

  // Monitor navigation
  authView.webContents.on("did-navigate", (_event, url) => {
    devLog.log("[Auth] Navigated to:", url);
    if (url.includes("/login") || url.includes("/session")) {
      devLog.log("[Auth] Detected login page");
      mainWindow?.webContents.send("auth:state-changed", "unauthenticated");
      stopRefreshTimer(); // Stop timer when logged out
      // Don't auto-show login window here - let the explicit login request handle it
    } else if (url.includes("/settings/billing")) {
      devLog.log("[Auth] Detected billing page - user is authenticated");
      mainWindow?.webContents.send("auth:state-changed", "authenticated");
      hideLoginWindow();
      startRefreshTimer(); // Start timer when authenticated
      fetchUsageData();
    }
  });

  // Also check for redirects
  authView.webContents.on("will-redirect", (_event, url) => {
    if (url.includes("/login")) {
      mainWindow?.webContents.send("auth:state-changed", "unauthenticated");
    }
  });

  // Check auth state after page loads
  authView.webContents.on("did-finish-load", async () => {
    const url = authView?.webContents.getURL() || "";
    devLog.log("[Auth] Page finished loading:", url);
    if (url.includes("/login") || url.includes("/session")) {
      devLog.log("[Auth] Login page loaded");
      mainWindow?.webContents.send("auth:state-changed", "unauthenticated");
      stopRefreshTimer(); // Ensure timer is stopped when logged out
    } else if (url.includes("/settings/billing")) {
      devLog.log("[Auth] Billing page loaded - authenticated");
      mainWindow?.webContents.send("auth:state-changed", "authenticated");
      hideLoginWindow();
      // Only start timer if not already running
      if (!refreshTimer) {
        startRefreshTimer();
      }
      fetchUsageData();
    }
  });
}

function showLoginWindow(): void {
  if (!mainWindow) return;

  // If authWindow exists but is destroyed, recreate it
  if (authWindow) {
    if (authWindow.isDestroyed()) {
      authWindow = null;
    } else {
      authWindow.show();
      authWindow.focus();
      // Always reload the login URL to ensure fresh state
      authWindow.loadURL(config.githubLoginUrl);
      return;
    }
  }

  authWindow = new BrowserWindow({
    width: 900,
    height: 700,
    parent: mainWindow,
    show: false,
    autoHideMenuBar: true,
    title: "GitHub Login",
    webPreferences: {
      partition: "persist:github",
      nodeIntegration: false,
      contextIsolation: true,
    },
  });

  authWindow.on("ready-to-show", () => {
    authWindow?.show();
  });

  authWindow.on("closed", () => {
    // Notify renderer that auth was cancelled by user
    if (mainWindow && !mainWindow.isDestroyed()) {
      mainWindow.webContents.send("auth:state-changed", "unauthenticated");
    }
    authWindow = null;
  });

  // Track if user has logged in (navigated away from login page)
  let hasNavigatedFromLogin = false;

  const handleAuthWindowNavigation = (): void => {
    const url = authWindow?.webContents.getURL() || "";
    devLog.log("[Auth] AuthWindow navigation:", url);

    // If on login page, just record that we've seen it
    if (url.includes("/login") || url.includes("/session")) {
      hasNavigatedFromLogin = true;
      return;
    }

    // Only load billing URL if we've previously seen the login page
    // and now we're on a different page (user logged in)
    if (hasNavigatedFromLogin && authView) {
      devLog.log("[Auth] User logged in, loading billing page");
      authView.webContents.loadURL(config.githubBillingUrl);
    }
  };

  authWindow.webContents.on("did-navigate", handleAuthWindowNavigation);
  authWindow.webContents.on("did-finish-load", handleAuthWindowNavigation);

  authWindow.loadURL(config.githubLoginUrl);
}

function hideLoginWindow(): void {
  if (!authWindow) return;
  authWindow.close();
  authWindow = null;
}

// ============= Data Fetching =============

async function getCustomerId(): Promise<number | null> {
  if (!authView) return null;
  if (customerId) return customerId;

  // Method 0: Extract from URL query parameters
  try {
    const url = authView.webContents.getURL();
    devLog.log("[Auth] Getting customer ID from URL:", url);
    const match = url.match(/[?&]customer=(\d+)/);
    if (match) {
      customerId = parseInt(match[1], 10);
      devLog.log("[Auth] Found customer ID in URL:", customerId);
      return customerId;
    }
  } catch (e) {
    devLog.error("[Auth] Method 0 (URL) failed:", e);
  }

  // Method 1: API call
  try {
    const result = await authView.webContents.executeJavaScript(`
      (async function() {
        try {
          const response = await fetch('/api/v3/user', {
            headers: { 'Accept': 'application/json' }
          });
          if (!response.ok) throw new Error('API failed');
          const data = await response.json();
          return JSON.stringify({ success: true, id: data.id });
        } catch (error) {
          return JSON.stringify({ success: false, error: error.message });
        }
      })()
    `);
    const parsed = JSON.parse(result);
    if (parsed.success) {
      customerId = parsed.id;
      return customerId;
    }
  } catch (e) {
    devLog.error("Method 1 failed:", e);
  }

  // Method 2: DOM extraction from script tag (matching copilot-usage-monitor Swift implementation)
  try {
    devLog.log("[Auth] Trying Method 2: DOM extraction from script tag");
    const result = await authView.webContents.executeJavaScript(`
      (function() {
        try {
          const el = document.querySelector('script[data-target="react-app.embeddedData"]');
          if (!el) return 'no_element';
          try {
            const data = JSON.parse(el.textContent);
            const id = data?.payload?.customer?.customerId;
            return id ? id.toString() : 'no_id_in_json';
          } catch(e) {
            return 'parse_error: ' + e.message;
          }
        } catch (error) {
          return 'error: ' + error.message;
        }
      })()
    `);
    devLog.log("[Auth] Method 2 result:", result);

    // Check if result is a valid customer ID (numeric string)
    if (result && /^\d+$/.test(result)) {
      customerId = parseInt(result, 10);
      devLog.log("[Auth] Method 2 success - Customer ID:", customerId);
      return customerId;
    } else {
      devLog.log("[Auth] Method 2 failed:", result);
    }
  } catch (e) {
    devLog.error("[Auth] Method 2 exception:", e);
  }

  // Method 3: Regex
  try {
    const result = await authView.webContents.executeJavaScript(`
      (function() {
        try {
          const html = document.body.innerHTML;
          const patterns = [/customerId":(\\d+)/, /customerId&quot;:(\\d+)/, /customer_id=(\\d+)/];
          for (const pattern of patterns) {
            const match = html.match(pattern);
            if (match) return JSON.stringify({ success: true, id: parseInt(match[1]) });
          }
          return JSON.stringify({ success: false, error: 'No match' });
        } catch (error) {
          return JSON.stringify({ success: false, error: error.message });
        }
      })()
    `);
    const parsed = JSON.parse(result);
    if (parsed.success) {
      customerId = parsed.id;
      return customerId;
    }
  } catch (e) {
    devLog.error("Method 3 failed:", e);
  }

  return null;
}

async function fetchUsageData(): Promise<void> {
  if (!authView || !mainWindow) return;

  devLog.log("[Usage] Starting fetchUsageData");
  mainWindow.webContents.send("usage:loading", true);

  try {
    const id = await getCustomerId();
    devLog.log("[Usage] Got customer ID:", id);
    if (!id) {
      mainWindow.webContents.send("usage:data", {
        success: false,
        error: "Could not get customer ID. Please log in again.",
      });
      return;
    }

    // Fetch usage card
    devLog.log("[Usage] Fetching usage card for customer:", id);
    const usageResult = await authView.webContents.executeJavaScript(`
      (async function() {
        try {
          const res = await fetch('/settings/billing/copilot_usage_card?customer_id=${id}&period=3', {
            headers: { 'Accept': 'application/json', 'x-requested-with': 'XMLHttpRequest' }
          });
          console.log('[Usage] Usage card response status:', res.status);
          if (!res.ok) throw new Error('Request failed: ' + res.status);
          const data = await res.json();
          console.log('[Usage] Usage card data:', JSON.stringify(data).slice(0, 200));
          return JSON.stringify({ success: true, data });
        } catch (error) {
          console.error('[Usage] Usage card error:', error.message);
          return JSON.stringify({ success: false, error: error.message });
        }
      })()
    `);

    const usageParsed = JSON.parse(usageResult);
    console.log(
      "[Usage] Usage card parsed:",
      usageParsed.success ? "success" : "failed",
      usageParsed.error || "",
    );
    if (!usageParsed.success) {
      mainWindow.webContents.send("usage:data", {
        success: false,
        error: usageParsed.error,
      });
      return;
    }

    // Fetch usage history
    devLog.log("[Usage] Fetching usage history");
    const historyResult = await authView.webContents.executeJavaScript(`
      (async function() {
        try {
          const res = await fetch('/settings/billing/copilot_usage_table?customer_id=${id}&group=0&period=3&query=&page=1', {
            headers: { 'Accept': 'application/json', 'x-requested-with': 'XMLHttpRequest' }
          });
          console.log('[Usage] History response status:', res.status);
          if (!res.ok) throw new Error('Request failed: ' + res.status);
          const data = await res.json();
          console.log('[Usage] History data rows:', data.rows ? data.rows.length : 0);
          return JSON.stringify({ success: true, data });
        } catch (error) {
          console.error('[Usage] History error:', error.message);
          return JSON.stringify({ success: false, error: error.message });
        }
      })()
    `);

    const historyParsed = JSON.parse(historyResult);
    devLog.log(
      "[Usage] History parsed:",
      historyParsed.success ? "success" : "failed",
    );

    // DEBUG: Log raw history data structure
    if (historyParsed.success && historyParsed.data) {
      const rawHistory = historyParsed.data;
      devLog.log(
        "[Usage] History data keys:",
        Object.keys(rawHistory).join(", "),
      );
      devLog.log(
        "[Usage] History has 'table' field?:",
        Object.prototype.hasOwnProperty.call(rawHistory, "table"),
      );
      devLog.log(
        "[Usage] History has 'rows' field?:",
        Object.prototype.hasOwnProperty.call(rawHistory, "rows"),
      );

      // Check for nested table.rows structure
      if (rawHistory.table && rawHistory.table.rows) {
        devLog.log("[Usage] Found table.rows structure");
        devLog.log(
          "[Usage] Number of history rows:",
          rawHistory.table.rows.length,
        );
        if (rawHistory.table.rows.length > 0) {
          devLog.log(
            "[Usage] First row sample:",
            JSON.stringify(rawHistory.table.rows[0]).slice(0, 200),
          );
        }
      } else if (rawHistory.rows) {
        devLog.log("[Usage] Number of history rows:", rawHistory.rows.length);
        if (rawHistory.rows.length > 0) {
          devLog.log(
            "[Usage] First row sample:",
            JSON.stringify(rawHistory.rows[0]).slice(0, 200),
          );
        }
      }
    } else {
      devLog.log("[Usage] History data missing or failed:", historyParsed);
    }

    // Parse usage data
    const usageData = usageParsed.data;
    devLog.log(
      "[Usage] Raw usage data keys:",
      Object.keys(usageData).join(", "),
    );
    devLog.log("[Usage] Sample values:", {
      netBilledAmount:
        usageData.netBilledAmount ?? usageData.net_billed_amount ?? "missing",
      discountQuantity:
        usageData.discountQuantity ?? usageData.discount_quantity ?? "missing",
      entitlement:
        usageData.userPremiumRequestEntitlement ??
        usageData.user_premium_request_entitlement ??
        "missing",
    });

    const usage: CopilotUsage = {
      netBilledAmount:
        usageData.netBilledAmount ?? usageData.net_billed_amount ?? 0,
      netQuantity: usageData.netQuantity ?? usageData.net_quantity ?? 0,
      discountQuantity:
        usageData.discountQuantity ?? usageData.discount_quantity ?? 0,
      userPremiumRequestEntitlement:
        usageData.userPremiumRequestEntitlement ??
        usageData.user_premium_request_entitlement ??
        0,
      filteredUserPremiumRequestEntitlement:
        usageData.filteredUserPremiumRequestEntitlement ??
        usageData.filtered_user_premium_request_entitlement ??
        0,
    };
    devLog.log("[Usage] Parsed usage:", usage);

    // Parse history data - handle nested table.rows.cells structure
    const historyData = historyParsed.success
      ? historyParsed.data
      : { table: { rows: [] } };

    // Extract rows from either table.rows or rows directly
    const rawRows = historyData.table?.rows || historyData.rows || [];

    const history: UsageHistory = {
      fetchedAt: new Date().toISOString(),
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      days: rawRows.map((row: any) => {
        // Handle Swift-style nested cells structure
        if (row.cells && Array.isArray(row.cells)) {
          const cells = row.cells;

          // Helper to parse cell value
          // eslint-disable-next-line @typescript-eslint/no-explicit-any
          const parseCell = (cell: any): number => {
            if (!cell) return 0;
            const value = cell.value || cell.sortValue || "";
            if (typeof value === "number") return value;
            if (typeof value === "string") {
              // Remove commas and currency symbols
              const cleaned = value.replace(/[$,]/g, "");
              return parseFloat(cleaned) || 0;
            }
            return 0;
          };

          // Parse date from sortValue (format: "yyyy-MM-dd HH:mm:ss Z utc")
          const dateCell = cells[0];
          const dateStr = dateCell?.sortValue || dateCell?.value || "";
          const dateMatch = dateStr.match(/^(\d{4}-\d{2}-\d{2})/);
          const date = dateMatch
            ? dateMatch[1]
            : new Date().toISOString().split("T")[0];

          return {
            date,
            includedRequests: cells[1] ? parseCell(cells[1]) : 0,
            billedRequests: cells[2] ? parseCell(cells[2]) : 0,
            grossAmount: cells[3] ? parseCell(cells[3]) : 0,
            billedAmount: cells[4] ? parseCell(cells[4]) : 0,
          };
        }

        // Fallback: handle flat structure
        return {
          date: row.date || new Date().toISOString().split("T")[0],
          includedRequests: row.included_requests ?? row.includedRequests ?? 0,
          billedRequests: row.billed_requests ?? row.billedRequests ?? 0,
          grossAmount: row.gross_amount ?? row.grossAmount ?? 0,
          billedAmount: row.billed_amount ?? row.billedAmount ?? 0,
        };
      }),
    };

    // Store history globally for tray menu
    usageHistory = history;

    // Calculate prediction if we have enough data
    if (history.days.length >= 3 && usage) {
      const settings = store.get("settings") as Settings;
      const prediction = calculatePrediction(
        history,
        usage,
        settings.predictionPeriod,
      );
      usagePrediction = prediction;
    }

    // Update tray
    updateTrayMenu(usage);

    // Send to renderer
    mainWindow.webContents.send("usage:data", {
      success: true,
      usage,
      history,
    });

    // Cache data
    store.set("cache", {
      usage,
      history,
      lastFetched: new Date().toISOString(),
    });
  } catch (error) {
    devLog.error("Failed to fetch usage:", error);
    mainWindow.webContents.send("usage:data", {
      success: false,
      error: error instanceof Error ? error.message : "Unknown error",
    });
  } finally {
    mainWindow.webContents.send("usage:loading", false);
  }
}

// ============= Refresh Timer =============

function startRefreshTimer(): void {
  stopRefreshTimer();
  const settings = store.get("settings") as Settings;
  refreshTimer = setInterval(() => {
    fetchUsageData();
  }, settings.refreshInterval * 1000);
}

function stopRefreshTimer(): void {
  if (refreshTimer) {
    clearInterval(refreshTimer);
    refreshTimer = null;
  }
}

// ============= IPC Handlers =============

function setupIpcHandlers(): void {
  // Auth
  ipcMain.on("auth:login", () => {
    try {
      // Destroy existing authWindow if any
      if (authWindow && !authWindow.isDestroyed()) {
        authWindow.destroy();
        authWindow = null;
      }

      // Clear customer ID to force re-authentication
      customerId = null;

      // Show login window - this will handle the auth flow
      // and reload billing in authView after successful login
      showLoginWindow();
    } catch (error) {
      devLog.error("Auth login failed:", error);
      mainWindow?.webContents.send("auth:state-changed", "error");
    }
  });

  ipcMain.on("auth:logout", () => {
    destroyAuthView();
    mainWindow?.webContents.send("auth:state-changed", "unauthenticated");
  });

  ipcMain.on("auth:check", () => {
    try {
      if (!authView) {
        createAuthView();
        if (!authView) {
          throw new Error("Failed to create auth view");
        }
        mainWindow?.webContents.send("auth:state-changed", "checking");
        return;
      }
      const url = authView.webContents.getURL();
      if (url.includes("/login") || url.includes("/session")) {
        mainWindow?.webContents.send("auth:state-changed", "unauthenticated");
      } else if (url.includes("/settings/billing")) {
        mainWindow?.webContents.send("auth:state-changed", "authenticated");
      } else {
        mainWindow?.webContents.send("auth:state-changed", "checking");
      }
    } catch (error) {
      devLog.error("Auth check failed:", error);
      mainWindow?.webContents.send("auth:state-changed", "error");
    }
  });

  // Usage
  ipcMain.on("usage:fetch", () => fetchUsageData());
  ipcMain.on("usage:refresh", () => fetchUsageData());

  // Settings
  ipcMain.handle("settings:get", () => store.get("settings"));
  ipcMain.handle("settings:set", (_event, settings) => {
    const current = store.get("settings") as Settings;
    const updated = { ...current, ...settings };
    store.set("settings", updated);

    // Update launch at login
    if (settings.launchAtLogin !== undefined) {
      app.setLoginItemSettings({ openAtLogin: settings.launchAtLogin });
    }

    // Restart refresh timer if interval changed
    if (settings.refreshInterval !== undefined) {
      startRefreshTimer();
    }

    mainWindow?.webContents.send("settings:changed", updated);
  });
  ipcMain.handle("settings:reset", () => {
    store.set("settings", DEFAULT_SETTINGS);
    mainWindow?.webContents.send("settings:changed", DEFAULT_SETTINGS);
  });

  // App
  ipcMain.on("app:quit", () => app.quit());
  ipcMain.on("window:show", () => {
    mainWindow?.show();
    mainWindow?.focus();
  });
  ipcMain.on("window:hide", () => mainWindow?.hide());
}

// ============= App Lifecycle =============

app.whenReady().then(() => {
  // Set app user model id for Windows
  app.setAppUserModelId("com.copilot-tracker");

  // Default open or close DevTools by F12 in development
  app.on("browser-window-created", (_, window) => {
    // Register F12 to toggle DevTools
    window.webContents.on("before-input-event", (event, input) => {
      if (input.key === "F12") {
        window.webContents.toggleDevTools();
        event.preventDefault();
      }
    });
  });

  // Setup IPC handlers
  setupIpcHandlers();

  // Create window and tray
  createWindow();
  createTray();
  createAuthView();

  // Don't start refresh timer yet - wait for auth state
  // Timer will be started when auth state becomes 'authenticated'

  // On macOS, re-create window when dock icon is clicked
  app.on("activate", function () {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    } else {
      mainWindow?.show();
    }
  });
});

// Quit when all windows are closed, except on macOS
app.on("window-all-closed", () => {
  if (process.platform !== "darwin") {
    app.quit();
  }
});

// Cleanup on quit
app.on("before-quit", () => {
  stopRefreshTimer();

  // Destroy auth view using the centralized function
  destroyAuthView();

  // Destroy auth window and remove listeners
  if (authWindow && !authWindow.isDestroyed()) {
    const wc = authWindow.webContents;
    if (wc && !wc.isDestroyed()) {
      wc.removeAllListeners("did-navigated");
      wc.removeAllListeners("did-finish-load");
    }
    authWindow.destroy();
    authWindow = null;
  }

  // Destroy tray
  if (tray) {
    tray.destroy();
    tray = null;
  }

  // Remove main window close listener to allow proper exit on macOS
  if (mainWindow && !mainWindow.isDestroyed()) {
    mainWindow.removeAllListeners("close");
  }
});
