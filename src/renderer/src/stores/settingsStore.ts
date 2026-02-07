/**
 * Settings Store
 * Zustand store for managing application settings
 */

import { create } from "zustand";
import { persist } from "zustand/middleware";
import type { Settings } from "../types/settings";
import { DEFAULT_SETTINGS } from "../types/settings";

interface SettingsState extends Settings {
  // Actions
  setRefreshInterval: (interval: Settings["refreshInterval"]) => void;
  setPredictionPeriod: (period: Settings["predictionPeriod"]) => void;
  setLaunchAtLogin: (enabled: boolean) => void;
  setStartMinimized: (enabled: boolean) => void;
  setNotificationsEnabled: (enabled: boolean) => void;
  setNotificationThresholds: (thresholds: number[]) => void;
  setTheme: (theme: Settings["theme"]) => void;
  setTrayIconFormat: (format: Settings["trayIconFormat"]) => void;
  updateSettings: (settings: Partial<Settings>) => void;
  resetSettings: () => void;
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      // Default values
      ...DEFAULT_SETTINGS,

      // Actions
      setRefreshInterval: (refreshInterval) => set({ refreshInterval }),

      setPredictionPeriod: (predictionPeriod) => set({ predictionPeriod }),

      setLaunchAtLogin: (launchAtLogin) => set({ launchAtLogin }),

      setStartMinimized: (startMinimized) => set({ startMinimized }),

      setNotificationsEnabled: (enabled) =>
        set((state) => ({
          notifications: { ...state.notifications, enabled },
        })),

      setNotificationThresholds: (thresholds) =>
        set((state) => ({
          notifications: { ...state.notifications, thresholds },
        })),

      setTheme: (theme) => set({ theme }),

      setTrayIconFormat: (trayIconFormat) => set({ trayIconFormat }),

      updateSettings: (settings) =>
        set((state) => ({
          ...state,
          ...settings,
          notifications: settings.notifications
            ? { ...state.notifications, ...settings.notifications }
            : state.notifications,
        })),

      resetSettings: () => set(DEFAULT_SETTINGS),
    }),
    {
      name: "copilot-tracker-settings",
      version: 1,
    },
  ),
);

// Selectors
export const useTheme = () => useSettingsStore((state) => state.theme);
export const useRefreshInterval = () =>
  useSettingsStore((state) => state.refreshInterval);
export const usePredictionPeriod = () =>
  useSettingsStore((state) => state.predictionPeriod);
export const useNotifications = () =>
  useSettingsStore((state) => state.notifications);
