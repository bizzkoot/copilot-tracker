/**
 * Usage Store
 * Zustand store for managing usage data state
 */

import { create } from "zustand";
import type {
  CopilotUsage,
  UsageHistory,
  UsagePrediction,
} from "../types/usage";
import type { AuthState } from "../types/app";

interface UsageState {
  // Auth state
  authState: AuthState;
  setAuthState: (state: AuthState) => void;

  // Usage data
  usage: CopilotUsage | null;
  history: UsageHistory | null;
  prediction: UsagePrediction | null;
  setUsage: (usage: CopilotUsage | null) => void;
  setHistory: (history: UsageHistory | null) => void;
  setPrediction: (prediction: UsagePrediction | null) => void;
  setUsageData: (data: {
    usage?: CopilotUsage | null;
    history?: UsageHistory | null;
    prediction?: UsagePrediction | null;
  }) => void;

  // Loading state
  isLoading: boolean;
  setIsLoading: (loading: boolean) => void;

  // Error state
  error: string | null;
  setError: (error: string | null) => void;

  // Last updated timestamp
  lastUpdated: Date | null;
  setLastUpdated: (date: Date | null) => void;

  // Reset all data
  reset: () => void;
}

export const useUsageStore = create<UsageState>((set) => ({
  // Auth state
  authState: "unknown",
  setAuthState: (authState) => set({ authState }),

  // Usage data
  usage: null,
  history: null,
  prediction: null,
  setUsage: (usage) => set({ usage }),
  setHistory: (history) => set({ history }),
  setPrediction: (prediction) => set({ prediction }),
  setUsageData: (data) =>
    set((state) => ({
      usage: data.usage !== undefined ? data.usage : state.usage,
      history: data.history !== undefined ? data.history : state.history,
      prediction:
        data.prediction !== undefined ? data.prediction : state.prediction,
      lastUpdated: new Date(),
      error: null,
    })),

  // Loading state
  isLoading: false,
  setIsLoading: (isLoading) => set({ isLoading }),

  // Error state
  error: null,
  setError: (error) => set({ error, isLoading: false }),

  // Last updated timestamp
  lastUpdated: null,
  setLastUpdated: (lastUpdated) => set({ lastUpdated }),

  // Reset all data
  reset: () =>
    set({
      authState: "unknown",
      usage: null,
      history: null,
      prediction: null,
      isLoading: false,
      error: null,
      lastUpdated: null,
    }),
}));

// Selectors for commonly used derived state
export const useAuthState = () => useUsageStore((state) => state.authState);
export const useIsAuthenticated = () =>
  useUsageStore((state) => state.authState === "authenticated");
export const useIsLoading = () => useUsageStore((state) => state.isLoading);
export const useUsageData = () =>
  useUsageStore((state) => ({
    usage: state.usage,
    history: state.history,
    prediction: state.prediction,
  }));
