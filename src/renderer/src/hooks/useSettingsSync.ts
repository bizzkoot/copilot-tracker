/**
 * useSettingsSync Hook
 * Global hook to sync settings between tray menu and dashboard
 * Runs at app initialization and handles settings changes from any source
 */

import { useEffect, useCallback } from "react";
import { useSettingsStore } from "../stores/settingsStore";

export function useSettingsSync() {
  const updateSettings = useSettingsStore((state) => state.updateSettings);

  // Function to sync settings from main process
  const syncSettings = useCallback(async () => {
    if (typeof window.electron === "undefined") return;

    try {
      console.log("[useSettingsSync] Syncing settings from main process...");
      const settings = await window.electron.getSettings();
      updateSettings(settings);
      console.log("[useSettingsSync] Settings synced:", settings);
    } catch (err) {
      console.error("[useSettingsSync] Failed to sync settings:", err);
    }
  }, [updateSettings]);

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
