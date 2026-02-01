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
    authState,
    setUsageData,
    setIsLoading,
    setError,
    setPrediction,
  } = useUsageStore();

  const predictionPeriod = useSettingsStore((state) => state.predictionPeriod);
  const refreshInterval = useSettingsStore((state) => state.refreshInterval);

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

  // Setup IPC listeners
  useEffect(() => {
    if (typeof window.electron === "undefined") return;

    // Listen for usage data
    const unsubUsage = window.electron.onUsageData?.((data) => {
      if (data.success) {
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

  // Auto-refresh on interval
  useEffect(() => {
    if (authState !== "authenticated") return;

    const intervalId = setInterval(() => {
      refresh();
    }, refreshInterval * 1000);

    return () => clearInterval(intervalId);
  }, [authState, refreshInterval, refresh]);

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
