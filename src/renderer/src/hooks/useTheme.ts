/**
 * useTheme Hook
 * Manages dark/light theme with system detection
 */

import { useEffect, useCallback } from "react";
import { useSettingsStore } from "../stores/settingsStore";

export function useTheme() {
  const theme = useSettingsStore((state) => state.theme);
  const setTheme = useSettingsStore((state) => state.setTheme);

  // Apply theme to document
  const applyTheme = useCallback((themeValue: "light" | "dark" | "system") => {
    const root = document.documentElement;

    if (themeValue === "system") {
      const isDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
      root.classList.toggle("dark", isDark);
    } else {
      root.classList.toggle("dark", themeValue === "dark");
    }
  }, []);

  // Listen for system theme changes
  useEffect(() => {
    applyTheme(theme);

    if (theme === "system") {
      const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
      const handleChange = () => applyTheme("system");

      mediaQuery.addEventListener("change", handleChange);
      return () => mediaQuery.removeEventListener("change", handleChange);
    }
  }, [theme, applyTheme]);

  // Toggle between light and dark
  const toggleTheme = useCallback(() => {
    const root = document.documentElement;
    const isDark = root.classList.contains("dark");
    const newTheme = isDark ? "light" : "dark";

    // Optimistic update
    setTheme(newTheme);

    // Sync with backend to prevent reset on other settings updates
    window.electron.setSettings({ theme: newTheme }).catch((err) => {
      console.error("Failed to sync theme to backend:", err);
      // Revert on failure
      setTheme(isDark ? "dark" : "light");
    });
  }, [setTheme]);

  // Get current effective theme (resolves 'system' to actual value)
  const getEffectiveTheme = useCallback((): "light" | "dark" => {
    if (theme === "system") {
      return window.matchMedia("(prefers-color-scheme: dark)").matches
        ? "dark"
        : "light";
    }
    return theme;
  }, [theme]);

  return {
    theme,
    setTheme,
    toggleTheme,
    getEffectiveTheme,
    isDark: getEffectiveTheme() === "dark",
  };
}
