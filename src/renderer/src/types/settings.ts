/**
 * Settings Types
 * User preferences and application settings
 */

// Tray icon format constants - must match TRAY_ICON_FORMATS in Rust (src-tauri/src/store.rs)
export const TRAY_FORMAT_CURRENT = "current" as const;
export const TRAY_FORMAT_CURRENT_TOTAL = "currentTotal" as const;
export const TRAY_FORMAT_REMAINING_TOTAL = "remainingTotal" as const;
export const TRAY_FORMAT_PERCENTAGE = "percentage" as const;
export const TRAY_FORMAT_REMAINING_PERCENT = "remainingPercent" as const;
export const TRAY_FORMAT_COMBINED = "combined" as const;
export const TRAY_FORMAT_REMAINING_COMBINED = "remainingCombined" as const;

// Default tray icon format - must match DEFAULT_TRAY_ICON_FORMAT in Rust
export const DEFAULT_TRAY_FORMAT = TRAY_FORMAT_CURRENT_TOTAL;

export type TrayIconFormat =
  | typeof TRAY_FORMAT_CURRENT
  | typeof TRAY_FORMAT_CURRENT_TOTAL
  | typeof TRAY_FORMAT_REMAINING_TOTAL
  | typeof TRAY_FORMAT_PERCENTAGE
  | typeof TRAY_FORMAT_REMAINING_PERCENT
  | typeof TRAY_FORMAT_COMBINED
  | typeof TRAY_FORMAT_REMAINING_COMBINED;

export interface NotificationSettings {
  enabled: boolean;
  thresholds: number[]; // e.g., [50, 75, 90, 100]
}

export interface Settings {
  refreshInterval: 10 | 30 | 60 | 300 | 1800; // seconds
  predictionPeriod: 7 | 14 | 21; // days
  launchAtLogin: boolean;
  startMinimized: boolean; // Auto-hide window on startup/login
  notifications: NotificationSettings;
  theme: "light" | "dark" | "system";
  trayIconFormat: TrayIconFormat;
}

export const DEFAULT_SETTINGS: Settings = {
  refreshInterval: 60,
  predictionPeriod: 7,
  launchAtLogin: false,
  startMinimized: true, // Auto-hide window on startup by default
  notifications: {
    enabled: true,
    thresholds: [75, 90, 100],
  },
  theme: "system",
  trayIconFormat: DEFAULT_TRAY_FORMAT,
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

// Tray icon format options for UI
export const TRAY_ICON_FORMAT_OPTIONS = [
  { value: TRAY_FORMAT_CURRENT, label: "Current only", example: "450" },
  {
    value: TRAY_FORMAT_CURRENT_TOTAL,
    label: "Current / Total",
    example: "450/1200",
  },
  {
    value: TRAY_FORMAT_REMAINING_TOTAL,
    label: "Remaining / Total",
    example: "750/1200",
  },
  { value: TRAY_FORMAT_PERCENTAGE, label: "Percentage used", example: "38%" },
  {
    value: TRAY_FORMAT_REMAINING_PERCENT,
    label: "Percentage remaining",
    example: "62%",
  },
  {
    value: TRAY_FORMAT_COMBINED,
    label: "Current + Percentage",
    example: "450/1200 (38%)",
  },
  {
    value: TRAY_FORMAT_REMAINING_COMBINED,
    label: "Remaining + Percentage",
    example: "750/1200 (62%)",
  },
] as const;
