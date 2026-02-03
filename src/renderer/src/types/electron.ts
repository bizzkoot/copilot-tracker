/**
 * Electron API Types
 * Type definitions for IPC communication between main and renderer
 */

import type { CopilotUsage, UsageHistory, UsagePrediction } from "./usage";
import type { Settings } from "./settings";

// Auth state
export type AuthState =
  | "unknown"
  | "checking"
  | "authenticated"
  | "unauthenticated"
  | "error";

// Customer ID result
export interface CustomerIdResult {
  success: boolean;
  id?: number;
  error?: string;
}

// Usage fetch result
export interface UsageFetchResult {
  success: boolean;
  usage?: CopilotUsage;
  history?: UsageHistory;
  prediction?: UsagePrediction;
  error?: string;
}

export interface UpdateCheckStatus {
  status: "available" | "none" | "error";
  message?: string;
}

// Update info
export interface UpdateInfo {
  version: string;
  files: {
    url: string;
    sha512: string;
    size: number;
  }[];
  path: string;
  sha512: string;
  releaseUrl?: string;
  downloadUrl?: string;
  releaseName?: string;
  releaseNotes?: string | Array<{ version: string; note: string | null }>;
  releaseDate: string;
}

// IPC Events from main to renderer
export interface MainToRendererEvents {
  "auth:state-changed": (state: AuthState) => void;
  "auth:session-expired": () => void;
  "auth:ready": () => void;
  "usage:data": (data: UsageFetchResult) => void;
  "usage:loading": (loading: boolean) => void;
  "settings:changed": (settings: Settings) => void;
  "update:available": (info: UpdateInfo) => void;
  "update:checked": (status: UpdateCheckStatus) => void;
}

// IPC Events from renderer to main
export interface RendererToMainEvents {
  "auth:login": () => void;
  "auth:logout": () => void;
  "auth:check": () => void;
  "usage:fetch": () => void;
  "usage:refresh": () => void;
  "settings:get": () => Settings;
  "settings:set": (settings: Partial<Settings>) => void;
  "settings:reset": () => void;
  "app:quit": () => void;
  "window:show": () => void;
  "window:hide": () => void;
}

// Electron API exposed to renderer
export interface ElectronAPI {
  // Auth
  login: () => void;
  logout: () => void;
  checkAuth: () => void;
  onAuthStateChanged: (callback: (state: AuthState) => void) => () => void;
  onSessionExpired: (callback: () => void) => () => void;

  // Usage
  fetchUsage: () => void;
  refreshUsage: () => void;
  onUsageData: (callback: (data: UsageFetchResult) => void) => () => void;
  onUsageLoading: (callback: (loading: boolean) => void) => () => void;

  // Settings
  getSettings: () => Promise<Settings>;
  setSettings: (settings: Partial<Settings>) => Promise<void>;
  resetSettings: () => Promise<void>;
  onSettingsChanged: (callback: (settings: Settings) => void) => () => void;

  // App
  quit: () => void;
  showWindow: () => void;
  hideWindow: () => void;
  openExternal: (url: string) => void;
  checkForUpdates: () => void;
  onNavigate: (callback: (route: string) => void) => () => void;
  onUpdateAvailable: (callback: (info: UpdateInfo) => void) => () => void;
  onUpdateChecked: (
    callback: (status: UpdateCheckStatus) => void,
  ) => () => void;

  // Platform info
  platform: NodeJS.Platform;
  getVersion: () => Promise<string>;
}

// Window API for global access
declare global {
  interface Window {
    electron: ElectronAPI;
  }
}
