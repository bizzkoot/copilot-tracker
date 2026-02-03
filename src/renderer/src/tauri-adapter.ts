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
const settingsListeners: ((settings: Settings) => void)[] = [];
let isAuthListenerSetup = false;
let isUsageListenerSetup = false;

// Helper to notify listeners
const notifyAuthListeners = (state: AuthState) => {
  authListeners.forEach((cb) => cb(state));
};

const notifyUsageListeners = (data: UsageFetchResult) => {
  usageListeners.forEach((cb) => cb(data));
};

const notifySettingsListeners = (settings: Settings) => {
  settingsListeners.forEach((cb) => cb(settings));
};

// Helper to convert Rust usage summary to UsageFetchResult
const convertUsageData = (summary: RustUsageSummary): UsageFetchResult => {
  return {
    success: true,
    usage: {
      netQuantity: summary.used,
      netBilledAmount: 0,
      discountQuantity: summary.used,
      userPremiumRequestEntitlement: summary.limit,
      filteredUserPremiumRequestEntitlement: summary.limit,
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
        notifyAuthListeners(event.payload);
      }).catch((err) => console.error("Failed to set up auth listener:", err));
    }

    if (!isUsageListenerSetup) {
      isUsageListenerSetup = true;
      listen<RustUsageSummary>("usage:updated", (event) => {
        const result = convertUsageData(event.payload);
        notifyUsageListeners(result);
      }).catch((err) => console.error("Failed to set up usage listener:", err));

      listen<{
        summary: RustUsageSummary;
        history: Array<{
          timestamp: number;
          used: number;
          limit: number;
          billed_requests?: number;
          gross_amount?: number;
          billed_amount?: number;
        }>;
        prediction?: {
          predicted_monthly_requests: number;
          predicted_billed_amount: number;
          confidence_level: string;
          days_used_for_prediction: number;
        };
      }>("usage:data", (event) => {
        const payload = event.payload;
        const result: UsageFetchResult = {
          success: true,
          usage: {
            netQuantity: payload.summary.used,
            netBilledAmount: 0,
            discountQuantity: payload.summary.used,
            userPremiumRequestEntitlement: payload.summary.limit,
            filteredUserPremiumRequestEntitlement: payload.summary.limit,
          },
          history: {
            fetchedAt: new Date(payload.summary.timestamp * 1000),
            days: payload.history.map((entry) => ({
              date: new Date(entry.timestamp * 1000),
              includedRequests:
                entry.used - (entry.billed_requests ?? 0) < 0
                  ? 0
                  : entry.used - (entry.billed_requests ?? 0),
              billedRequests: entry.billed_requests ?? 0,
              grossAmount: entry.gross_amount ?? 0,
              billedAmount: entry.billed_amount ?? 0,
            })),
          },
          prediction: payload.prediction
            ? {
                predictedMonthlyRequests:
                  payload.prediction.predicted_monthly_requests,
                predictedBilledAmount:
                  payload.prediction.predicted_billed_amount,
                confidenceLevel: payload.prediction.confidence_level as
                  | "low"
                  | "medium"
                  | "high",
                daysUsedForPrediction:
                  payload.prediction.days_used_for_prediction,
              }
            : undefined,
        };
        notifyUsageListeners(result);
      }).catch((err) =>
        console.error("Failed to set up usage:data listener:", err),
      );
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
        let unlisten: (() => void) | null = null;
        listen("auth:session-expired", () => callback())
          .then((stop) => {
            unlisten = stop;
          })
          .catch((err) =>
            console.error("Failed to listen auth:session-expired", err),
          );
        return () => {
          unlisten?.();
        };
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
        let unlisten: (() => void) | null = null;
        listen<boolean>("usage:loading", (event) => callback(event.payload))
          .then((stop) => {
            unlisten = stop;
          })
          .catch((err) => console.error("Failed to listen usage:loading", err));
        return () => {
          unlisten?.();
        };
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
      resetSettings: async () => {
        console.log("[TauriAdapter] Resetting all settings and data...");
        const rustSettings = await invoke<RustAppSettings>("reset_settings");
        console.log("[TauriAdapter] Backend reset complete");

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
        notifySettingsListeners(settings);

        // Force a checkAuth to ensure the auth state is properly updated
        console.log(
          "[TauriAdapter] Reset complete - forcing auth check to verify unauthenticated state...",
        );
        try {
          const result = await invoke<RustAuthState>("check_auth_status");
          const state: AuthState = result.is_authenticated
            ? "authenticated"
            : "unauthenticated";
          console.log("[TauriAdapter] Post-reset checkAuth result:", state);
          notifyAuthListeners(state);
        } catch (err) {
          console.error("[TauriAdapter] Post-reset checkAuth failed:", err);
          notifyAuthListeners("error");
        }
      },
      onSettingsChanged: (callback: (settings: Settings) => void) => {
        settingsListeners.push(callback);
        let unlisten: (() => void) | null = null;
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
          notifySettingsListeners(settings);
        })
          .then((stop) => {
            unlisten = stop;
          })
          .catch((err) =>
            console.error("Failed to listen settings:changed", err),
          );
        return () => {
          unlisten?.();
          const idx = settingsListeners.indexOf(callback);
          if (idx !== -1) settingsListeners.splice(idx, 1);
        };
      },

      // App
      quit: () => invoke("plugin:process|exit"),
      showWindow: () =>
        invoke("plugin:window|show").catch((e) =>
          console.error("Failed to show window", e),
        ),
      hideWindow: () =>
        invoke("hide_main_window").catch((e) =>
          console.error("Failed to hide window", e),
        ),
      openExternal: (url: string) => invoke("open_external_url", { url }),
      checkForUpdates: () => invoke("check_for_updates"),
      onNavigate: (callback: (route: string) => void) => {
        let unlisten: (() => void) | null = null;
        listen<string>("navigate", (event) => callback(event.payload))
          .then((stop) => {
            unlisten = stop;
          })
          .catch((err) => console.error("Failed to listen navigate", err));
        return () => {
          unlisten?.();
        };
      },
      onUpdateAvailable: (callback: (info: UpdateInfo) => void) => {
        let unlisten: (() => void) | null = null;
        listen<UpdateInfo>("update:available", (event) =>
          callback(event.payload),
        )
          .then((stop) => {
            unlisten = stop;
          })
          .catch((err) =>
            console.error("Failed to listen update:available", err),
          );
        return () => {
          unlisten?.();
        };
      },
      onUpdateChecked: (callback: (status: UpdateCheckStatus) => void) => {
        let unlisten: (() => void) | null = null;
        listen<UpdateCheckStatus>("update:checked", (event) =>
          callback(event.payload),
        )
          .then((stop) => {
            unlisten = stop;
          })
          .catch((err) =>
            console.error("Failed to listen update:checked", err),
          );
        return () => {
          unlisten?.();
        };
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
