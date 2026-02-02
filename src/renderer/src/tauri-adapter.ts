import {
  Settings,
  UsageFetchResult,
  AuthState,
  UpdateInfo,
  UpdateCheckStatus,
  ElectronAPI,
} from "./types";

// Rust payload types
interface RustUsageSummary {
  used: number;
  limit: number;
  remaining: number;
  percentage: number;
  timestamp: number;
}

// Rust AppSettings (full struct)
interface RustAppSettings extends Settings {
  customerId?: number;
  usageLimit: number;
  lastUsage: number;
  lastFetchTimestamp: number;
  isAuthenticated: boolean;
  updateChannel: string;
}

// Rust AuthState result
interface RustAuthState {
  is_authenticated: boolean;
  customer_id: number | null;
}

declare global {
  interface Window {
    __TAURI__?: {
      core: {
        invoke: <T>(cmd: string, args?: unknown) => Promise<T>;
      };
      event: {
        listen: <T>(
          event: string,
          handler: (event: { payload: T }) => void,
        ) => Promise<() => void>;
        emit: (event: string, payload?: unknown) => Promise<void>;
      };
    };
  }
}

// Listener storage for manual notifications (handling race conditions)
const authListeners: ((state: AuthState) => void)[] = [];
const usageListeners: ((data: UsageFetchResult) => void)[] = [];
let isAuthListenerSetup = false;
let isUsageListenerSetup = false;

// Helper to notify listeners
const notifyAuthListeners = (state: AuthState) => {
  authListeners.forEach((cb) => cb(state));
};

const notifyUsageListeners = (data: UsageFetchResult) => {
  usageListeners.forEach((cb) => cb(data));
};

// Helper to convert Rust usage summary to UsageFetchResult
const convertUsageData = (summary: RustUsageSummary): UsageFetchResult => {
  return {
    success: true,
    usage: {
      netQuantity: summary.used,
      netBilledAmount: 0,
      discountQuantity: 0,
      userPremiumRequestEntitlement: summary.limit,
      filteredUserPremiumRequestEntitlement: 0,
    },
  };
};

export function initTauriAdapter() {
  console.log("Initializing Tauri Adapter...");

  try {
    const isTauri =
      typeof window !== "undefined" && window.__TAURI__ !== undefined;

    // FIX: Check if we are already in an Electron environment
    const isElectron =
      typeof window !== "undefined" && (window as any).electron !== undefined;

    // If running in Electron (and not Tauri), do NOT overwrite the API
    if (isElectron && !isTauri) {
      console.log(
        "Electron environment detected. Skipping Tauri adapter initialization.",
      );
      return;
    }

    if (!isTauri) {
      console.log("Not running in Tauri environment. Using mock adapter.");
      setupMockAdapter();
      return;
    }

    const core = window.__TAURI__?.core;
    const event = window.__TAURI__?.event;

    if (!core || !event) {
      console.error(
        "Tauri core or event module missing, falling back to mock adapter.",
        { core, event },
      );
      setupMockAdapter();
      return;
    }

    const { invoke } = core;
    const { listen } = event;

    // Set up global listeners once
    if (!isAuthListenerSetup) {
      isAuthListenerSetup = true;
      // Backend emits string "authenticated" | "unauthenticated"
      listen<AuthState>("auth:state-changed", (event) => {
        console.log("[TauriAdapter] Event received: auth:state-changed", event.payload); console.log("[TauriAdapter] Notifying listeners. Count:", authListeners.length);
        notifyAuthListeners(event.payload);
      }).catch((err) => console.error("Failed to set up auth listener:", err));
    }

    if (!isUsageListenerSetup) {
      isUsageListenerSetup = true;
      listen<RustUsageSummary>("usage:updated", (event) => {
        console.log("Tauri event received: usage:updated", event.payload);
        const result = convertUsageData(event.payload);
        notifyUsageListeners(result);
      }).catch((err) => console.error("Failed to set up usage listener:", err));
    }

    const electronAPI: ElectronAPI = {
      platform: "darwin", // Todo: use window.__TAURI__.os.platform() if available

      // Auth
      login: () => invoke("show_auth_window"),
      logout: async () => {
        await invoke("logout");
        // Manual notification handled by backend event, but safe to add here if needed
      },
      checkAuth: async () => {
        try {
          const result = await invoke<RustAuthState>("check_auth_status");
          const state: AuthState = result.is_authenticated
            ? "authenticated"
            : "unauthenticated";
          console.log("[TauriAdapter] Manual checkAuth result:", state);
          notifyAuthListeners(state);
        } catch (err) {
          console.error("checkAuth failed:", err);
          notifyAuthListeners("error");
        }
      },
      onAuthStateChanged: (callback: (state: AuthState) => void) => {
        authListeners.push(callback);
        return () => {
          const idx = authListeners.indexOf(callback);
          if (idx !== -1) authListeners.splice(idx, 1);
        };
      },
      onSessionExpired: (callback: () => void) => {
        listen("auth:session-expired", () => callback());
        return () => {};
      },

      // Usage
      fetchUsage: async () => {
        try {
          const summary = await invoke<RustUsageSummary>("fetch_usage");
          const result = convertUsageData(summary);
          notifyUsageListeners(result);
        } catch (err) {
          console.error("fetchUsage failed:", err);
          notifyUsageListeners({
            success: false,
            error: String(err),
          });
        }
      },
      refreshUsage: async () => {
        try {
          const summary = await invoke<RustUsageSummary>("fetch_usage");
          const result = convertUsageData(summary);
          notifyUsageListeners(result);
        } catch (err) {
          console.error("refreshUsage failed:", err);
          notifyUsageListeners({
            success: false,
            error: String(err),
          });
        }
      },
      onUsageData: (callback: (data: UsageFetchResult) => void) => {
        usageListeners.push(callback);
        return () => {
          const idx = usageListeners.indexOf(callback);
          if (idx !== -1) usageListeners.splice(idx, 1);
        };
      },
      onUsageLoading: (callback: (loading: boolean) => void) => {
        listen<boolean>("usage:loading", (event) => callback(event.payload));
        return () => {};
      },

      // Settings
      getSettings: async () => {
        try {
          const rustSettings = await invoke<RustAppSettings>("get_settings");
          return {
            refreshInterval: rustSettings.refreshInterval,
            predictionPeriod: rustSettings.predictionPeriod as any,
            launchAtLogin: rustSettings.launchAtLogin,
            startMinimized: rustSettings.startMinimized,
            theme: rustSettings.theme as any,
            notifications: {
              enabled: (rustSettings as any).showNotifications,
              thresholds: (rustSettings as any).notificationThresholds || [
                75, 90, 100,
              ],
            },
          };
        } catch (e) {
          console.error("Failed to get settings", e);
          throw e;
        }
      },
      setSettings: async (newSettings: Partial<Settings>) => {
        try {
          // 1. Get current settings from Rust
          const current = await invoke<RustAppSettings>("get_settings");

          // 2. Merge updates
          let showNotifications = (current as any).showNotifications;
          let notificationThresholds = (current as any).notificationThresholds;

          if (newSettings.notifications) {
            if (newSettings.notifications.enabled !== undefined) {
              showNotifications = newSettings.notifications.enabled;
            }
            if (newSettings.notifications.thresholds) {
              notificationThresholds = newSettings.notifications.thresholds;
            }
          }

          const merged = {
            ...current,
            ...newSettings,
            showNotifications,
            notificationThresholds,
          };

          // 3. Send back to Rust
          await invoke("update_settings", { settings: merged });
        } catch (e) {
          console.error("Failed to set settings", e);
          throw e;
        }
      },
      resetSettings: () => {
        console.warn("resetSettings not fully implemented in Tauri adapter");
        return Promise.resolve();
      },
      onSettingsChanged: (callback: (settings: Settings) => void) => {
        listen<RustAppSettings>("settings:changed", (event) => {
          const rustSettings = event.payload;
          const settings: Settings = {
            refreshInterval: rustSettings.refreshInterval,
            predictionPeriod: rustSettings.predictionPeriod as any,
            launchAtLogin: rustSettings.launchAtLogin,
            startMinimized: rustSettings.startMinimized,
            theme: rustSettings.theme as any,
            notifications: {
              enabled: (rustSettings as any).showNotifications,
              thresholds: (rustSettings as any).notificationThresholds,
            },
          };
          callback(settings);
        });
        return () => {};
      },

      // App
      quit: () => invoke("plugin:process|exit"),
      showWindow: () =>
        invoke("plugin:window|show").catch((e) =>
          console.error("Failed to show window", e),
        ),
      hideWindow: () =>
        invoke("plugin:window|hide").catch((e) =>
          console.error("Failed to hide window", e),
        ),
      openExternal: (url: string) => invoke("plugin:shell|open", { path: url }),
      checkForUpdates: () => console.log("Updates not implemented"),
      onNavigate: (callback: (route: string) => void) => {
        listen<string>("navigate", (event) => callback(event.payload));
        return () => {};
      },
      onUpdateAvailable: (_callback: (info: UpdateInfo) => void) => {
        return () => {};
      },
      onUpdateChecked: (_callback: (status: UpdateCheckStatus) => void) => {
        return () => {};
      },
      getVersion: () => invoke("get_app_version"),
    };

    (window as any).electron = electronAPI;
    console.log("Tauri Adapter initialized successfully.");
  } catch (err) {
    console.error("Critical error initializing Tauri Adapter:", err);
    setupMockAdapter();
  }
}

function setupMockAdapter() {
  console.warn("Setting up Mock/Fallback Adapter");
  const mockAPI: ElectronAPI = {
    platform: "darwin",
    login: async () => {},
    logout: async () => {},
    checkAuth: async () => {},
    onAuthStateChanged: (cb) => {
      // Immediate mock response for testing
      setTimeout(() => cb("unauthenticated"), 100);
      return () => {};
    },
    onSessionExpired: () => () => {},
    fetchUsage: async () => {},
    refreshUsage: async () => {},
    onUsageData: () => () => {},
    onUsageLoading: () => () => {},
    getSettings: async () =>
      ({
        refreshInterval: 60,
        predictionPeriod: "30days",
        launchAtLogin: false,
        startMinimized: false,
        theme: "system",
        notifications: { enabled: true, thresholds: [90] },
      }) as any,
    setSettings: async () => {},
    resetSettings: async () => {},
    onSettingsChanged: () => () => {},
    quit: async () => {},
    showWindow: async () => {},
    hideWindow: async () => {},
    openExternal: async () => {},
    checkForUpdates: async () => {},
    onNavigate: () => () => {},
    onUpdateAvailable: () => () => {},
    onUpdateChecked: () => () => {},
    getVersion: async () => "1.0.0-mock",
  };
  (window as any).electron = mockAPI;
}
