/**
 * useAuth Hook
 * Manages authentication state and login/logout
 */

import { useCallback, useEffect } from "react";
import { useUsageStore } from "../stores/usageStore";
import type { AuthState } from "../types/app";

const isDev = import.meta.env.DEV;

export function useAuth() {
  const authState = useUsageStore((state) => state.authState);
  const setAuthState = useUsageStore((state) => state.setAuthState);
  const setError = useUsageStore((state) => state.setError);

  // Login - opens GitHub auth window
  const login = useCallback(async () => {
    if (isDev) {
      console.log(
        "[Auth] login() called, electron available:",
        typeof window.electron !== "undefined",
      );
    }
    if (typeof window.electron !== "undefined") {
      try {
        await window.electron.login();
        setError(null);
      } catch (err) {
        console.error("[Auth] login failed:", err);
        setError(err instanceof Error ? err.message : "Failed to start login");
      }
      return;
    }

    if (isDev) {
      console.error("[Auth] window.electron is undefined - IPC not available");
    }
  }, [setError]);

  // Logout - clears session
  const logout = useCallback(async () => {
    if (typeof window.electron !== "undefined") {
      try {
        await window.electron.logout();
        setAuthState("unauthenticated");
        setError(null);
      } catch (err) {
        console.error("[Auth] logout failed:", err);
        setError(err instanceof Error ? err.message : "Failed to log out");
      }
    }
  }, [setAuthState, setError]);

  // Check current auth state
  const checkAuth = useCallback(async () => {
    if (typeof window.electron !== "undefined") {
      try {
        await window.electron.checkAuth();
      } catch (err) {
        console.error("[Auth] checkAuth failed:", err);
        setAuthState("error");
        setError(
          err instanceof Error ? err.message : "Failed to check auth state",
        );
      }
    }
  }, [setAuthState, setError]);

  // Setup IPC listeners
  useEffect(() => {
    if (typeof window.electron === "undefined") return;

    // Listen for auth state changes
    const unsubAuthState = window.electron.onAuthStateChanged?.(
      (state: AuthState) => {
        setAuthState(state);
      },
    );

    // Listen for session expiry
    const unsubSessionExpired = window.electron.onSessionExpired?.(() => {
      setAuthState("unauthenticated");
    });

    // Listen for already authenticated notification
    const unsubAlreadyAuthenticated = window.electron.onAlreadyAuthenticated?.(
      () => {
        console.log("[Auth] Already authenticated - data will be refreshed");
        // User will see the data refresh in the dashboard
        // No need for a toast since the state is already showing as authenticated
      },
    );

    // Listen for auth extraction failures
    const unsubExtractionFailed = window.electron.onAuthExtractionFailed?.(
      (error: string) => {
        console.error("[Auth] Extraction failed:", error);
        setError(
          "Unable to retrieve Copilot data. GitHub may have changed their interface. Please try again or report this issue.",
        );
        setAuthState("error");
      },
    );

    // Initial auth check
    void checkAuth();

    return () => {
      unsubAuthState?.();
      unsubSessionExpired?.();
      unsubAlreadyAuthenticated?.();
      unsubExtractionFailed?.();
    };
  }, [setAuthState, checkAuth]);

  // Computed properties
  const isAuthenticated = authState === "authenticated";
  const isLoading = authState === "checking" || authState === "unknown";
  const needsLogin = authState === "unauthenticated";
  const hasError = authState === "error";

  return {
    authState,
    isAuthenticated,
    isLoading,
    needsLogin,
    hasError,
    login,
    logout,
    checkAuth,
  };
}
