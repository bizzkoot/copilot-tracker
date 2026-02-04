/**
 * useAuth Hook
 * Manages authentication state and login/logout
 */

import { useCallback, useEffect } from "react";
import { useUsageStore } from "../stores/usageStore";
import type { AuthState } from "../types/electron";

export function useAuth() {
  const authState = useUsageStore((state) => state.authState);
  const setAuthState = useUsageStore((state) => state.setAuthState);
  const reset = useUsageStore((state) => state.reset);

  // Login - opens GitHub auth window
  const login = useCallback(() => {
    console.log(
      "[Auth] login() called, electron available:",
      typeof window.electron !== "undefined",
    );
    if (typeof window.electron !== "undefined") {
      window.electron.login();
    } else {
      console.error("[Auth] window.electron is undefined - IPC not available");
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

    // Initial auth check
    checkAuth();

    return () => {
      unsubAuthState?.();
      unsubSessionExpired?.();
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
