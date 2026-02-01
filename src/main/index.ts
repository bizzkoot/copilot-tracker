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
import { electronApp, optimizer, is } from "@electron-toolkit/utils";
const icon = join(__dirname, "../../resources/icon.png");
import Store from "electron-store";

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

// Constants
const GITHUB_BILLING_URL =
  "https://github.com/settings/billing/premium_requests_usage";
const GITHUB_LOGIN_URL = "https://github.com/login";
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
    // On macOS, hide window instead of closing
    if (process.platform === "darwin") {
      event.preventDefault();
      mainWindow?.hide();
    }
  });

  mainWindow.webContents.setWindowOpenHandler((details) => {
    shell.openExternal(details.url);
    return { action: "deny" };
  });

  // HMR for renderer
  if (is.dev && process.env["ELECTRON_RENDERER_URL"]) {
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

  const usageLabel = usage
    ? `Used: ${Math.round(usage.discountQuantity)} / ${usage.userPremiumRequestEntitlement} (${((usage.discountQuantity / usage.userPremiumRequestEntitlement) * 100).toFixed(1)}%)`
    : "Loading...";

  const menu = Menu.buildFromTemplate([
    {
      label: usageLabel,
      enabled: false,
    },
    { type: "separator" },
    {
      label: "Open Dashboard",
      click: (): void => {
        mainWindow?.show();
        mainWindow?.focus();
      },
    },
    {
      label: "Refresh",
      click: (): void => {
        fetchUsageData();
      },
    },
    { type: "separator" },
    {
      label: "Settings",
      click: (): void => {
        mainWindow?.show();
        mainWindow?.focus();
        mainWindow?.webContents.send("navigate", "settings");
      },
    },
    { type: "separator" },
    {
      label: "Quit",
      click: (): void => {
        app.quit();
      },
    },
  ]);

  tray.setContextMenu(menu);

  if (usage) {
    tray.setToolTip(
      `Copilot: ${Math.round(usage.discountQuantity)}/${usage.userPremiumRequestEntitlement}`,
    );
  }
}

// ============= Authentication =============

function destroyAuthView(): void {
  if (!authView) return;

  // Remove all event listeners
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
  authView.webContents.loadURL(GITHUB_BILLING_URL);

  // Monitor navigation
  authView.webContents.on("did-navigate", (_event, url) => {
    if (url.includes("/login") || url.includes("/session")) {
      mainWindow?.webContents.send("auth:state-changed", "unauthenticated");
      showLoginWindow();
    } else if (url.includes("/settings/billing")) {
      mainWindow?.webContents.send("auth:state-changed", "authenticated");
      hideLoginWindow();
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
    if (url.includes("/login") || url.includes("/session")) {
      mainWindow?.webContents.send("auth:state-changed", "unauthenticated");
    } else if (url.includes("/settings/billing")) {
      mainWindow?.webContents.send("auth:state-changed", "authenticated");
      hideLoginWindow();
      fetchUsageData();
    }
  });
}

function showLoginWindow(): void {
  if (!mainWindow) return;

  if (authWindow) {
    authWindow.show();
    authWindow.focus();
    return;
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

  const handleAuthWindowNavigation = (): void => {
    const url = authWindow?.webContents.getURL() || "";
    if (url.includes("/login") || url.includes("/session")) return;
    authView?.webContents.loadURL(GITHUB_BILLING_URL);
  };

  authWindow.webContents.on("did-navigate", handleAuthWindowNavigation);
  authWindow.webContents.on("did-finish-load", handleAuthWindowNavigation);

  authWindow.loadURL(GITHUB_LOGIN_URL);
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
    console.error("Method 1 failed:", e);
  }

  // Method 2: DOM extraction
  try {
    const result = await authView.webContents.executeJavaScript(`
      (function() {
        try {
          const el = document.querySelector('script[data-target="react-app.embeddedData"]');
          if (!el) return JSON.stringify({ success: false, error: 'Element not found' });
          const data = JSON.parse(el.textContent);
          return JSON.stringify({ success: true, id: data.payload.customer.customerId });
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
    console.error("Method 2 failed:", e);
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
    console.error("Method 3 failed:", e);
  }

  return null;
}

async function fetchUsageData(): Promise<void> {
  if (!authView || !mainWindow) return;

  mainWindow.webContents.send("usage:loading", true);

  try {
    const id = await getCustomerId();
    if (!id) {
      mainWindow.webContents.send("usage:data", {
        success: false,
        error: "Could not get customer ID. Please log in again.",
      });
      return;
    }

    // Fetch usage card
    const usageResult = await authView.webContents.executeJavaScript(`
      (async function() {
        try {
          const res = await fetch('/settings/billing/copilot_usage_card?customer_id=${id}&period=3', {
            headers: { 'Accept': 'application/json', 'x-requested-with': 'XMLHttpRequest' }
          });
          if (!res.ok) throw new Error('Request failed: ' + res.status);
          const data = await res.json();
          return JSON.stringify({ success: true, data });
        } catch (error) {
          return JSON.stringify({ success: false, error: error.message });
        }
      })()
    `);

    const usageParsed = JSON.parse(usageResult);
    if (!usageParsed.success) {
      mainWindow.webContents.send("usage:data", {
        success: false,
        error: usageParsed.error,
      });
      return;
    }

    // Fetch usage history
    const historyResult = await authView.webContents.executeJavaScript(`
      (async function() {
        try {
          const res = await fetch('/settings/billing/copilot_usage_table?customer_id=${id}&group=0&period=3&query=&page=1', {
            headers: { 'Accept': 'application/json', 'x-requested-with': 'XMLHttpRequest' }
          });
          if (!res.ok) throw new Error('Request failed: ' + res.status);
          const data = await res.json();
          return JSON.stringify({ success: true, data });
        } catch (error) {
          return JSON.stringify({ success: false, error: error.message });
        }
      })()
    `);

    const historyParsed = JSON.parse(historyResult);

    // Parse usage data
    const usageData = usageParsed.data;
    const usage: CopilotUsage = {
      netBilledAmount: usageData.net_billed_amount ?? 0,
      netQuantity: usageData.net_quantity ?? 0,
      discountQuantity: usageData.discount_quantity ?? 0,
      userPremiumRequestEntitlement:
        usageData.user_premium_request_entitlement ?? 0,
      filteredUserPremiumRequestEntitlement:
        usageData.filtered_user_premium_request_entitlement ?? 0,
    };

    // Parse history data
    const historyData = historyParsed.success
      ? historyParsed.data
      : { rows: [] };
    const history = {
      fetchedAt: new Date().toISOString(),
      days: (historyData.rows || []).map(
        (row: {
          date: string;
          included_requests?: number;
          billed_requests?: number;
          gross_amount?: number;
          billed_amount?: number;
        }) => ({
          date: row.date,
          includedRequests: row.included_requests ?? 0,
          billedRequests: row.billed_requests ?? 0,
          grossAmount: row.gross_amount ?? 0,
          billedAmount: row.billed_amount ?? 0,
        }),
      ),
    };

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
    console.error("Failed to fetch usage:", error);
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
      if (!authView) {
        createAuthView();
      }
      if (!authView) {
        throw new Error("Failed to create auth view");
      }
      authView.webContents.loadURL(GITHUB_BILLING_URL);
      showLoginWindow();
    } catch (error) {
      console.error("Auth login failed:", error);
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
      console.error("Auth check failed:", error);
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
  electronApp.setAppUserModelId("com.copilot-tracker");

  // Default open or close DevTools by F12 in development
  app.on("browser-window-created", (_, window) => {
    optimizer.watchWindowShortcuts(window);
  });

  // Setup IPC handlers
  setupIpcHandlers();

  // Create window and tray
  createWindow();
  createTray();
  createAuthView();

  // Start refresh timer
  startRefreshTimer();

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
      wc.removeAllListeners("did-navigate");
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
