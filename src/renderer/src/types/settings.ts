/**
 * Settings Types
 * User preferences and application settings
 */

export interface NotificationSettings {
  enabled: boolean;
  thresholds: number[]; // e.g., [50, 75, 90, 100]
}

export interface Settings {
  refreshInterval: 10 | 30 | 60 | 300 | 1800; // seconds
  predictionPeriod: 7 | 14 | 21; // days
  launchAtLogin: boolean;
  notifications: NotificationSettings;
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

// Refresh interval options for UI
export const REFRESH_INTERVAL_OPTIONS = [
  { value: 10, label: "10 seconds" },
  { value: 30, label: "30 seconds" },
  { value: 60, label: "1 minute" },
  { value: 300, label: "5 minutes" },
  { value: 1800, label: "30 minutes" },
] as const;

// Prediction period options for UI
export const PREDICTION_PERIOD_OPTIONS = [
  { value: 7, label: "7 days" },
  { value: 14, label: "14 days" },
  { value: 21, label: "21 days" },
] as const;

// Theme options for UI
export const THEME_OPTIONS = [
  { value: "light", label: "Light" },
  { value: "dark", label: "Dark" },
  { value: "system", label: "System" },
] as const;
