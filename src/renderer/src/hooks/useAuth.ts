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
  const reset = useUsageStore((state) => state.reset);

  // Login - opens GitHub auth window
  const login = useCallback(() => {
    if (isDev) {
      console.log(
        "[Auth] login() called, electron available:",
        typeof window.electron !== "undefined",
      );
    }
    if (typeof window.electron !== "undefined") {
      window.electron.login();
    } else {
      if (isDev) {
        console.error(
          "[Auth] window.electron is undefined - IPC not available",
        );
      }
    }
  }, []);

  // Logout - clears session
  const logout = useCallback(() => {
    if (typeof window.electron !== "undefined") {
      window.electron.logout();
      reset();
    }
  }, [reset]);

  // Check current auth state
  const checkAuth = useCallback(() => {
    if (typeof window.electron !== "undefined") {
      window.electron.checkAuth();
    }
  }, []);

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
    checkAuth();

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
