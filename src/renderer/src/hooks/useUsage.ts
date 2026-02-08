/**
 * useUsage Hook
 * Manages usage data fetching and state
 */

import { useCallback, useEffect } from "react";
import { useUsageStore } from "../stores/usageStore";
import { useSettingsStore } from "../stores/settingsStore";
import { generatePrediction } from "../services/predictor";

export function useUsage() {
  const {
    usage,
    history,
    prediction,
    isLoading,
    error,
    lastUpdated,
    setUsageData,
    setIsLoading,
    setError,
    setPrediction,
  } = useUsageStore();

  const predictionPeriod = useSettingsStore((state) => state.predictionPeriod);
  // refreshInterval is managed by the main process, not needed here

  // Update prediction when usage, history, or period changes
  useEffect(() => {
    if (usage && history) {
      const newPrediction = generatePrediction(
        usage,
        history,
        predictionPeriod,
      );
      setPrediction(newPrediction);
    }
  }, [usage, history, predictionPeriod, setPrediction]);

  // Fetch usage data from main process
  const fetchUsage = useCallback(async () => {
    if (typeof window.electron === "undefined") {
      console.warn("Electron API not available");
      return;
    }

    setIsLoading(true);
    setError(null);

    try {
      window.electron.fetchUsage();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to fetch usage");
    }
  }, [setIsLoading, setError]);

  // Refresh usage data
  const refresh = useCallback(() => {
    if (typeof window.electron !== "undefined") {
      window.electron.refreshUsage();
    }
  }, []);

  // Setup IPC listeners and request cached data on mount
  useEffect(() => {
    if (typeof window.electron === "undefined") return;

    // Request cached data immediately on mount
    const loadCachedData = async () => {
      try {
        const cached = await window.electron.getCachedUsage();
        if (cached && cached.success) {
          console.log("[useUsage] Loaded cached data on mount:", cached);
          setUsageData({
            usage: cached.usage,
            history: cached.history,
            prediction: cached.prediction,
          });
        }
      } catch (err) {
        console.error("[useUsage] Failed to load cached data:", err);
      }
    };
    loadCachedData();

    // Listen for usage data
    const unsubUsage = window.electron.onUsageData?.((data) => {
      console.log("[FRONTEND] Received usage data:", data);

      // DEBUG: Log raw rows from backend if available
      if (data.debugRawRows) {
        console.log(
          "[FRONTEND] RAW ROWS FROM API:",
          JSON.stringify(data.debugRawRows, null, 2),
        );
      }

      if (data.success) {
        if (data.history && data.history.days) {
          console.log("[FRONTEND] History days:", data.history.days);
          console.log(
            "[FRONTEND] Dates:",
            data.history.days.map((d) => String(d.date)),
          );
        }
        setUsageData({
          usage: data.usage,
          history: data.history,
          prediction: data.prediction,
        });
      } else {
        setError(data.error || "Failed to fetch usage data");
      }
    });

    // Listen for loading state
    const unsubLoading = window.electron.onUsageLoading?.((loading) => {
      setIsLoading(loading);
    });

    return () => {
      unsubUsage?.();
      unsubLoading?.();
    };
  }, [setUsageData, setError, setIsLoading]);

  // Auto-refresh is handled by the main process timer
  // The renderer receives updates via IPC, no need for a separate timer

  return {
    usage,
    history,
    prediction,
    isLoading,
    error,
    lastUpdated,
    fetchUsage,
    refresh,
  };
}
