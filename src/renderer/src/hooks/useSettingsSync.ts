/**
 * useSettingsSync Hook
 * Global hook to sync settings between tray menu and dashboard
 * Runs at app initialization and handles settings changes from any source
 */

import { useEffect, useCallback, useRef } from "react";
import { useSettingsStore } from "../stores/settingsStore";

// Track last local update to prevent race conditions
let lastLocalUpdate = 0;
const SYNC_DEBOUNCE_MS = 1000; // 1 second debounce

/** Call this when updating settings locally to prevent sync overwriting them */
export function markLocalSettingsUpdate(): void {
  lastLocalUpdate = Date.now();
  console.log("[useSettingsSync] Marked local update at:", lastLocalUpdate);
}

export function useSettingsSync() {
  const updateSettings = useSettingsStore((state) => state.updateSettings);
  const currentTheme = useSettingsStore((state) => state.theme);
  const syncInProgress = useRef(false);

  // Function to sync settings from main process
  const syncSettings = useCallback(async () => {
    if (typeof window.electron === "undefined") return;

    // Skip sync if settings were recently updated locally (prevent race condition)
    const timeSinceLastUpdate = Date.now() - lastLocalUpdate;
    if (timeSinceLastUpdate < SYNC_DEBOUNCE_MS) {
      console.log(
        `[useSettingsSync] Skipping sync - local update was ${timeSinceLastUpdate}ms ago`,
      );
      return;
    }

    if (syncInProgress.current) {
      console.log("[useSettingsSync] Skipping sync - already in progress");
      return;
    }

    syncInProgress.current = true;
    try {
      console.log("[useSettingsSync] Syncing settings from main process...");
      const settings = await window.electron.getSettings();

      // Preserve local theme to prevent race condition
      // Theme should only change from explicit user action, not from periodic syncs
      if (settings.theme !== currentTheme) {
        console.log(
          `[useSettingsSync] Preserving local theme "${currentTheme}" over fetched theme "${settings.theme}"`,
        );
        settings.theme = currentTheme;
      }

      updateSettings(settings);
      console.log("[useSettingsSync] Settings synced:", settings);
    } catch (err) {
      console.error("[useSettingsSync] Failed to sync settings:", err);
    } finally {
      syncInProgress.current = false;
    }
  }, [updateSettings, currentTheme]);

  // Initial sync on mount
  useEffect(() => {
    syncSettings();
  }, [syncSettings]);

  // Listen for settings changes from main process (e.g., tray menu)
  useEffect(() => {
    if (typeof window.electron === "undefined") return;

    const cleanup = window.electron.onSettingsChanged((settings) => {
      console.log(
        "[useSettingsSync] Received settings:changed event:",
        settings,
      );
      updateSettings(settings);
    });

    return cleanup;
  }, [updateSettings]);

  // Sync when window becomes visible (user opens dashboard)
  useEffect(() => {
    const handleVisibilityChange = () => {
      if (document.visibilityState === "visible") {
        console.log(
          "[useSettingsSync] Window became visible, syncing settings...",
        );
        syncSettings();
      }
    };

    document.addEventListener("visibilitychange", handleVisibilityChange);
    return () => {
      document.removeEventListener("visibilitychange", handleVisibilityChange);
    };
  }, [syncSettings]);

  // Sync when window gets focus
  useEffect(() => {
    const handleFocus = () => {
      console.log("[useSettingsSync] Window got focus, syncing settings...");
      syncSettings();
    };

    window.addEventListener("focus", handleFocus);
    return () => {
      window.removeEventListener("focus", handleFocus);
    };
  }, [syncSettings]);
}
