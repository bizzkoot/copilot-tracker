/**
 * Settings Component
 * User preferences panel
 */

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "../ui/card";
import { Button } from "../ui/button";
import { useSettingsStore } from "@renderer/stores/settingsStore";
import {
  REFRESH_INTERVAL_OPTIONS,
  PREDICTION_PERIOD_OPTIONS,
  THEME_OPTIONS,
  TRAY_ICON_FORMAT_OPTIONS,
} from "@renderer/types/settings";
import { ArrowLeft, RefreshCw } from "lucide-react";
import { useEffect, useState } from "react";
import { markLocalSettingsUpdate } from "@renderer/hooks/useSettingsSync";

// Widget settings sub-component
function WidgetSettings() {
  const [widgetEnabled, setWidgetEnabled] = useState(false);
  const [isLoading, setIsLoading] = useState(true);

  // Fetch initial widget state
  useEffect(() => {
    const fetchWidgetState = async () => {
      try {
        const enabled = await window.electron.isWidgetEnabled();
        setWidgetEnabled(enabled);
      } catch (err) {
        console.error("Failed to get widget state:", err);
      } finally {
        setIsLoading(false);
      }
    };

    fetchWidgetState();
  }, []);

  // Listen for widget state changes from tray menu
  useEffect(() => {
    const cleanup = window.electron.onWidgetEnabledChanged((enabled) => {
      console.log("[WidgetSettings] Widget state changed from tray:", enabled);
      setWidgetEnabled(enabled);
    });

    return cleanup;
  }, []);

  const handleToggleWidget = async () => {
    const newValue = !widgetEnabled;
    setWidgetEnabled(newValue);

    try {
      await window.electron.setWidgetEnabled(newValue);
    } catch (err) {
      console.error("Failed to toggle widget:", err);
      setWidgetEnabled(!newValue); // Revert on error
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-lg">Floating Widget</CardTitle>
        <CardDescription>
          Show a floating widget with usage information
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="flex items-center justify-between">
          <div className="flex-1">
            <span className="text-sm">Enable widget</span>
            <p className="text-xs text-muted-foreground mt-1">
              Display floating widget on your desktop
            </p>
          </div>
          <Button
            variant={widgetEnabled ? "default" : "outline"}
            size="sm"
            onClick={handleToggleWidget}
            disabled={isLoading}
          >
            {widgetEnabled ? "Enabled" : "Disabled"}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}

interface SettingsProps {
  onClose: () => void;
}

export function Settings({ onClose }: SettingsProps) {
  const [checkingForUpdate, setCheckingForUpdate] = useState(false);
  const [updateStatus, setUpdateStatus] = useState<
    "idle" | "checking" | "none" | "available" | "error"
  >("idle");
  const [updateStatusMessage, setUpdateStatusMessage] = useState<string | null>(
    null,
  );
  const [appVersion, setAppVersion] = useState("Loading...");
  const {
    refreshInterval,
    predictionPeriod,
    theme,
    launchAtLogin,
    startMinimized,
    notifications,
    trayIconFormat,
    setRefreshInterval,
    setPredictionPeriod,
    setTheme,
    setLaunchAtLogin,
    setStartMinimized,
    setNotificationsEnabled,
    setNotificationThresholds,
    setTrayIconFormat,
  } = useSettingsStore();

  const handleLaunchAtLoginToggle = async () => {
    const newValue = !launchAtLogin;
    // Update local state immediately for responsive UI
    setLaunchAtLogin(newValue);
    // Sync with main process (which calls app.setLoginItemSettings)
    await window.electron.setSettings({ launchAtLogin: newValue });
  };

  const handleStartMinimizedToggle = async () => {
    const newValue = !startMinimized;
    // Update local state immediately for responsive UI
    setStartMinimized(newValue);
    // Sync with main process
    await window.electron.setSettings({ startMinimized: newValue });
  };

  // Fetch app version
  useEffect(() => {
    window.electron.getVersion().then(setAppVersion);
  }, []);

  useEffect(() => {
    const cleanup = window.electron.onUpdateChecked((status) => {
      setCheckingForUpdate(false);
      setUpdateStatus(status.status);

      if (status.status === "none") {
        setUpdateStatusMessage("Up to date");
      } else if (status.status === "available") {
        setUpdateStatusMessage("Update available");
      } else if (status.status === "error") {
        setUpdateStatusMessage(status.message ?? "Update check failed");
      } else {
        setUpdateStatusMessage(null);
      }
    });

    return cleanup;
  }, []);

  const getNextThresholds = (threshold: number) => {
    return notifications.thresholds.includes(threshold)
      ? notifications.thresholds.filter((t) => t !== threshold)
      : [...notifications.thresholds, threshold].sort((a, b) => a - b);
  };

  const handleCheckForUpdate = () => {
    setCheckingForUpdate(true);
    setUpdateStatus("checking");
    setUpdateStatusMessage("Checking for updates...");
    // Use the exposed API to check for updates
    window.electron.checkForUpdates();
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <Button variant="ghost" size="icon" onClick={onClose}>
            <ArrowLeft className="h-5 w-5" />
          </Button>
          <div>
            <h2 className="text-2xl font-semibold">Settings</h2>
            <p className="text-sm text-muted-foreground">
              Configure your preferences
            </p>
          </div>
        </div>
        {/* Removed Login and Reset buttons from header */}
      </div>

      {/* Refresh Interval */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Refresh Interval</CardTitle>
          <CardDescription>
            How often to fetch usage data from GitHub
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap gap-2">
            {REFRESH_INTERVAL_OPTIONS.map((option) => (
              <Button
                key={option.value}
                variant={
                  refreshInterval === option.value ? "default" : "outline"
                }
                size="sm"
                onClick={() => {
                  setRefreshInterval(option.value as typeof refreshInterval);
                  window.electron.setSettings({
                    refreshInterval: option.value as typeof refreshInterval,
                  });
                }}
              >
                {option.label}
              </Button>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Prediction Period */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Prediction Period</CardTitle>
          <CardDescription>
            Days of history used for monthly prediction
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap gap-2">
            {PREDICTION_PERIOD_OPTIONS.map((option) => (
              <Button
                key={option.value}
                variant={
                  predictionPeriod === option.value ? "default" : "outline"
                }
                size="sm"
                onClick={() => {
                  setPredictionPeriod(option.value as typeof predictionPeriod);
                  window.electron.setSettings({
                    predictionPeriod: option.value as typeof predictionPeriod,
                  });
                }}
              >
                {option.label}
              </Button>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Theme */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Theme</CardTitle>
          <CardDescription>Choose your preferred color scheme</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap gap-2">
            {THEME_OPTIONS.map((option) => (
              <Button
                key={option.value}
                variant={theme === option.value ? "default" : "outline"}
                size="sm"
                onClick={async () => {
                  const newTheme = option.value as typeof theme;
                  const oldTheme = theme;

                  // Mark local update to prevent race condition with sync
                  markLocalSettingsUpdate();

                  // Optimistic update
                  setTheme(newTheme);

                  try {
                    await window.electron.setSettings({
                      theme: newTheme,
                    });
                  } catch (err) {
                    console.error("Failed to save theme setting:", err);
                    // Revert on failure
                    setTheme(oldTheme);
                  }
                }}
              >
                {option.label}
              </Button>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Tray Icon Format */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Tray Icon Format</CardTitle>
          <CardDescription>
            Choose how usage is displayed in the system tray icon
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            {TRAY_ICON_FORMAT_OPTIONS.map((option) => (
              <button
                key={option.value}
                onClick={() => {
                  setTrayIconFormat(option.value);
                  window.electron.setSettings({
                    trayIconFormat: option.value,
                  });
                }}
                className={`
                  flex items-center justify-between p-4 rounded-lg border-2 transition-all
                  ${
                    trayIconFormat === option.value
                      ? "border-primary bg-primary/5"
                      : "border-border hover:border-primary/50"
                  }
                `}
              >
                <div className="text-left">
                  <div className="font-medium text-sm">{option.label}</div>
                  <div className="text-xs text-muted-foreground mt-1">
                    Example: {option.example}
                  </div>
                </div>
                {/* Mini tray icon preview */}
                <div className="bg-black/80 text-white px-2 py-1 rounded text-xs font-mono">
                  {option.example}
                </div>
              </button>
            ))}
          </div>
          <p className="text-xs text-muted-foreground mt-3">
            * Preview may differ slightly from actual tray icon appearance
          </p>
        </CardContent>
      </Card>

      {/* Notifications */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Notifications</CardTitle>
          <CardDescription>
            Get alerts when approaching usage limits
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <span className="text-sm">Enable notifications</span>
            <Button
              variant={notifications.enabled ? "default" : "outline"}
              size="sm"
              onClick={() => {
                const enabled = !notifications.enabled;
                setNotificationsEnabled(enabled);
                window.electron.setSettings({
                  notifications: {
                    ...notifications,
                    enabled,
                  },
                });
              }}
            >
              {notifications.enabled ? "Enabled" : "Disabled"}
            </Button>
          </div>

          {notifications.enabled && (
            <div className="space-y-2">
              <span className="text-sm text-muted-foreground">
                Alert at these thresholds:
              </span>
              <div className="flex flex-wrap gap-2">
                {[50, 75, 90, 100].map((threshold) => (
                  <Button
                    key={threshold}
                    variant={
                      notifications.thresholds.includes(threshold)
                        ? "default"
                        : "outline"
                    }
                    size="sm"
                    onClick={() => {
                      const newThresholds = getNextThresholds(threshold);
                      setNotificationThresholds(newThresholds);
                      window.electron.setSettings({
                        notifications: {
                          ...notifications,
                          thresholds: newThresholds,
                        },
                      });
                    }}
                  >
                    {threshold}%
                  </Button>
                ))}
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Startup */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Startup</CardTitle>
          <CardDescription>
            Launch behavior when your computer starts
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <span className="text-sm">Launch at login</span>
            <Button
              variant={launchAtLogin ? "default" : "outline"}
              size="sm"
              onClick={handleLaunchAtLoginToggle}
            >
              {launchAtLogin ? "Enabled" : "Disabled"}
            </Button>
          </div>
          <div className="flex items-center justify-between">
            <div className="flex-1">
              <span className="text-sm">Start minimized</span>
              <p className="text-xs text-muted-foreground mt-1">
                Hide window on startup (tray icon only)
              </p>
            </div>
            <Button
              variant={startMinimized ? "default" : "outline"}
              size="sm"
              onClick={handleStartMinimizedToggle}
            >
              {startMinimized ? "Enabled" : "Disabled"}
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Widget */}
      <WidgetSettings />

      {/* About */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">About</CardTitle>
          <CardDescription>App information and updates</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-between">
            <span className="text-sm">App Version</span>
            <div className="flex items-center gap-4">
              <span className="text-sm text-muted-foreground">
                v{appVersion}
              </span>
              <Button
                variant="outline"
                size="sm"
                onClick={handleCheckForUpdate}
                disabled={checkingForUpdate}
              >
                <RefreshCw
                  className={`h-4 w-4 mr-2 ${checkingForUpdate ? "animate-spin" : ""}`}
                />
                Check for Updates
              </Button>
            </div>
          </div>
          {updateStatusMessage && (
            <p
              className={`mt-2 text-xs ${
                updateStatus === "error"
                  ? "text-destructive"
                  : updateStatus === "available"
                    ? "text-primary"
                    : "text-muted-foreground"
              }`}
            >
              {updateStatusMessage}
            </p>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
