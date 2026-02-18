/**
 * Settings Component
 * User preferences panel with tabbed layout
 */

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "../ui/card";
import { Button } from "../ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "../ui/tabs";
import { useSettingsStore } from "@renderer/stores/settingsStore";
import {
  REFRESH_INTERVAL_OPTIONS,
  PREDICTION_PERIOD_OPTIONS,
  THEME_OPTIONS,
  TRAY_ICON_FORMAT_OPTIONS,
} from "@renderer/types/settings";
import {
  ArrowLeft,
  RefreshCw,
  Bug,
  ExternalLink,
  Settings2,
  Palette,
  Monitor,
  Info,
  ChevronDown,
  ChevronRight,
} from "lucide-react";
import { GitHubIcon } from "@renderer/components/icons/GitHubIcon";
import { useEffect, useState } from "react";
import { markLocalSettingsUpdate } from "@renderer/hooks/useSettingsSync";

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
  const [widgetEnabled, setWidgetEnabled] = useState(false);
  const [widgetLoading, setWidgetLoading] = useState(true);
  const [openAccordionSection, setOpenAccordionSection] = useState<
    string | null
  >(null);

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

  // Fetch app version
  useEffect(() => {
    window.electron.getVersion().then(setAppVersion);
  }, []);

  // Fetch initial widget state
  useEffect(() => {
    const fetchWidgetState = async () => {
      try {
        const enabled = await window.electron.isWidgetEnabled();
        setWidgetEnabled(enabled);
      } catch (err) {
        console.error("Failed to get widget state:", err);
      } finally {
        setWidgetLoading(false);
      }
    };

    fetchWidgetState();
  }, []);

  // Listen for widget state changes from tray menu
  useEffect(() => {
    const cleanup = window.electron.onWidgetEnabledChanged((enabled) => {
      setWidgetEnabled(enabled);
    });

    return cleanup;
  }, []);

  // Listen for update checks
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

  const handleLaunchAtLoginToggle = async () => {
    const newValue = !launchAtLogin;
    setLaunchAtLogin(newValue);
    await window.electron.setSettings({ launchAtLogin: newValue });
  };

  const handleStartMinimizedToggle = async () => {
    const newValue = !startMinimized;
    setStartMinimized(newValue);
    await window.electron.setSettings({ startMinimized: newValue });
  };

  const handleToggleWidget = async () => {
    const newValue = !widgetEnabled;
    setWidgetEnabled(newValue);

    try {
      await window.electron.setWidgetEnabled(newValue);
    } catch (err) {
      console.error("Failed to toggle widget:", err);
      setWidgetEnabled(!newValue);
    }
  };

  const getNextThresholds = (threshold: number) => {
    return notifications.thresholds.includes(threshold)
      ? notifications.thresholds.filter((t) => t !== threshold)
      : [...notifications.thresholds, threshold].sort((a, b) => a - b);
  };

  const handleCheckForUpdate = () => {
    setCheckingForUpdate(true);
    setUpdateStatus("checking");
    setUpdateStatusMessage("Checking for updates...");
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
      </div>

      {/* Tabbed Settings */}
      <Tabs defaultValue="general" className="w-full">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="general" className="gap-2">
            <Settings2 className="h-4 w-4" />
            <span className="hidden sm:inline">General</span>
          </TabsTrigger>
          <TabsTrigger value="appearance" className="gap-2">
            <Palette className="h-4 w-4" />
            <span className="hidden sm:inline">Appearance</span>
          </TabsTrigger>
          <TabsTrigger value="behavior" className="gap-2">
            <Monitor className="h-4 w-4" />
            <span className="hidden sm:inline">Behavior</span>
          </TabsTrigger>
          <TabsTrigger value="about" className="gap-2">
            <Info className="h-4 w-4" />
            <span className="hidden sm:inline">About</span>
          </TabsTrigger>
        </TabsList>

        {/* General Tab */}
        <TabsContent value="general" className="space-y-4 mt-4">
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
                      setRefreshInterval(
                        option.value as typeof refreshInterval,
                      );
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
                      setPredictionPeriod(
                        option.value as typeof predictionPeriod,
                      );
                      window.electron.setSettings({
                        predictionPeriod:
                          option.value as typeof predictionPeriod,
                      });
                    }}
                  >
                    {option.label}
                  </Button>
                ))}
              </div>
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
        </TabsContent>

        {/* Appearance Tab */}
        <TabsContent value="appearance" className="space-y-4 mt-4">
          {/* Theme */}
          <Card>
            <CardHeader>
              <CardTitle className="text-lg">Theme</CardTitle>
              <CardDescription>
                Choose your preferred color scheme
              </CardDescription>
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

                      markLocalSettingsUpdate();
                      setTheme(newTheme);

                      try {
                        await window.electron.setSettings({
                          theme: newTheme,
                        });
                      } catch (err) {
                        console.error("Failed to save theme setting:", err);
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
        </TabsContent>

        {/* Behavior Tab */}
        <TabsContent value="behavior" className="space-y-4 mt-4">
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
                  disabled={widgetLoading}
                >
                  {widgetEnabled ? "Enabled" : "Disabled"}
                </Button>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* About Tab */}
        <TabsContent value="about" className="mt-4">
          <div className="flex flex-col items-center text-center space-y-6">
            {/* Hero Section */}
            <div className="w-full max-w-lg space-y-4 pb-6 border-b">
              <div className="space-y-3">
                <p className="text-lg text-muted-foreground">
                  Version {appVersion}
                  <span className="mx-2">•</span>
                  by bizzkoot
                  <span className="mx-2">•</span>
                  MIT License
                </p>

                <p className="text-base text-foreground">
                  Track your Copilot usage, not your limits
                </p>
              </div>

              {/* Quick Actions */}
              <div className="flex flex-wrap items-center justify-center gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() =>
                    window.electron.openExternal(
                      "https://github.com/bizzkoot/copilot-tracker",
                    )
                  }
                  title="Star on GitHub"
                >
                  <GitHubIcon className="mr-2 h-4 w-4" />
                  Star
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() =>
                    window.electron.openExternal(
                      "https://github.com/bizzkoot/copilot-tracker/issues",
                    )
                  }
                  title="Report an issue"
                >
                  <Bug className="mr-2 h-4 w-4" />
                  Issue
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() =>
                    window.electron.openExternal(
                      "https://github.com/settings/billing/premium_requests_usage",
                    )
                  }
                  title="Open GitHub Billing"
                >
                  <ExternalLink className="mr-2 h-4 w-4" />
                  Billing
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleCheckForUpdate}
                  disabled={checkingForUpdate}
                  title="Check for updates"
                >
                  <RefreshCw
                    className={`mr-2 h-4 w-4 ${checkingForUpdate ? "animate-spin" : ""}`}
                  />
                  Updates
                </Button>
              </div>

              {/* Update Status */}
              {updateStatusMessage && (
                <p
                  className={`text-sm ${
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
            </div>

            {/* Accordion Sections */}
            <div className="w-full max-w-2xl space-y-3">
              {/* Features Section */}
              <Card
                className="overflow-hidden"
                onClick={() =>
                  setOpenAccordionSection(
                    openAccordionSection === "features" ? null : "features",
                  )
                }
              >
                <button className="w-full">
                  <CardHeader className="py-4 px-5 hover:bg-muted/50 transition-colors">
                    <div className="flex items-center justify-between">
                      <CardTitle className="text-base flex items-center gap-2">
                        <span>✨</span>
                        <span>What it does</span>
                      </CardTitle>
                      {openAccordionSection === "features" ? (
                        <ChevronDown className="h-5 w-5 text-muted-foreground" />
                      ) : (
                        <ChevronRight className="h-5 w-5 text-muted-foreground" />
                      )}
                    </div>
                  </CardHeader>
                </button>
                {openAccordionSection === "features" && (
                  <CardContent className="px-5 pb-4 pt-0 border-t">
                    <div className="grid grid-cols-1 sm:grid-cols-2 gap-3 text-sm pt-4">
                      <div className="flex items-start gap-2">
                        <span className="text-primary">📊</span>
                        <span className="text-muted-foreground">
                          Real-time usage tracking with gauge visualization
                        </span>
                      </div>
                      <div className="flex items-start gap-2">
                        <span className="text-primary">🔮</span>
                        <span className="text-muted-foreground">
                          Smart monthly predictions based on usage trends
                        </span>
                      </div>
                      <div className="flex items-start gap-2">
                        <span className="text-primary">🔔</span>
                        <span className="text-muted-foreground">
                          Configurable alerts when approaching limits
                        </span>
                      </div>
                      <div className="flex items-start gap-2">
                        <span className="text-primary">🎯</span>
                        <span className="text-muted-foreground">
                          System tray integration with custom display formats
                        </span>
                      </div>
                      <div className="flex items-start gap-2">
                        <span className="text-primary">🎨</span>
                        <span className="text-muted-foreground">
                          Floating widget for persistent usage visibility
                        </span>
                      </div>
                      <div className="flex items-start gap-2">
                        <span className="text-primary">🌓</span>
                        <span className="text-muted-foreground">
                          Dark/Light theme with automatic detection
                        </span>
                      </div>
                      <div className="flex items-start gap-2">
                        <span className="text-primary">🔐</span>
                        <span className="text-muted-foreground">
                          Secure WebView-based GitHub OAuth
                        </span>
                      </div>
                      <div className="flex items-start gap-2">
                        <span className="text-primary">🔄</span>
                        <span className="text-muted-foreground">
                          Automatic updates via GitHub Releases
                        </span>
                      </div>
                    </div>
                  </CardContent>
                )}
              </Card>

              {/* Tech Stack Section */}
              <Card
                className="overflow-hidden"
                onClick={() =>
                  setOpenAccordionSection(
                    openAccordionSection === "tech" ? null : "tech",
                  )
                }
              >
                <button className="w-full">
                  <CardHeader className="py-4 px-5 hover:bg-muted/50 transition-colors">
                    <div className="flex items-center justify-between">
                      <CardTitle className="text-base flex items-center gap-2">
                        <span>🛠️</span>
                        <span>How it&apos;s built</span>
                      </CardTitle>
                      {openAccordionSection === "tech" ? (
                        <ChevronDown className="h-5 w-5 text-muted-foreground" />
                      ) : (
                        <ChevronRight className="h-5 w-5 text-muted-foreground" />
                      )}
                    </div>
                  </CardHeader>
                </button>
                {openAccordionSection === "tech" && (
                  <CardContent className="px-5 pb-4 pt-0 border-t">
                    <div className="pt-4 text-sm leading-relaxed">
                      <div className="text-foreground font-semibold mb-4 text-center">
                        Copilot Tracker Architecture
                      </div>
                      <div className="space-y-4 text-left">
                        <div>
                          <div className="text-foreground font-medium flex items-center gap-2 mb-2">
                            <span>⚛️</span>
                            <span>Frontend</span>
                          </div>
                          <div className="pl-7 space-y-1 text-muted-foreground">
                            <div>• React 18 + TypeScript</div>
                            <div>• Tailwind CSS</div>
                            <div>• shadcn/ui</div>
                          </div>
                        </div>

                        <div>
                          <div className="text-foreground font-medium flex items-center gap-2 mb-2">
                            <span>📦</span>
                            <span>State & Visualization</span>
                          </div>
                          <div className="pl-7 space-y-1 text-muted-foreground">
                            <div>• Zustand (state management)</div>
                            <div>• Recharts (charts)</div>
                          </div>
                        </div>

                        <div>
                          <div className="text-foreground font-medium flex items-center gap-2 mb-2">
                            <span>🦀</span>
                            <span>Backend</span>
                          </div>
                          <div className="pl-7 space-y-1 text-muted-foreground">
                            <div>• Tauri 2.0 (Rust)</div>
                          </div>
                        </div>
                      </div>
                    </div>
                  </CardContent>
                )}
              </Card>

              {/* Privacy Section */}
              <Card
                className="overflow-hidden"
                onClick={() =>
                  setOpenAccordionSection(
                    openAccordionSection === "privacy" ? null : "privacy",
                  )
                }
              >
                <button className="w-full">
                  <CardHeader className="py-4 px-5 hover:bg-muted/50 transition-colors">
                    <div className="flex items-center justify-between">
                      <CardTitle className="text-base flex items-center gap-2">
                        <span>🔒</span>
                        <span>Privacy policy</span>
                      </CardTitle>
                      {openAccordionSection === "privacy" ? (
                        <ChevronDown className="h-5 w-5 text-muted-foreground" />
                      ) : (
                        <ChevronRight className="h-5 w-5 text-muted-foreground" />
                      )}
                    </div>
                  </CardHeader>
                </button>
                {openAccordionSection === "privacy" && (
                  <CardContent className="px-5 pb-4 pt-0 border-t space-y-3">
                    <div className="pt-4 space-y-3">
                      <p className="text-sm text-muted-foreground">
                        <strong className="text-foreground">
                          Privacy First:
                        </strong>{" "}
                        No API tokens stored. Authentication happens via secure
                        WebView directly with GitHub. All data is stored locally
                        on your machine. No tracking or analytics collected.
                      </p>
                      <p className="text-xs text-muted-foreground pt-2 border-t">
                        This application is not officially affiliated with
                        GitHub or Microsoft. It uses GitHub&apos;s internal
                        billing APIs which may change without notice.
                      </p>
                    </div>
                  </CardContent>
                )}
              </Card>

              {/* Support Section */}
              <Card
                className="overflow-hidden"
                onClick={() =>
                  setOpenAccordionSection(
                    openAccordionSection === "support" ? null : "support",
                  )
                }
              >
                <button className="w-full">
                  <CardHeader className="py-4 px-5 hover:bg-muted/50 transition-colors">
                    <div className="flex items-center justify-between">
                      <CardTitle className="text-base flex items-center gap-2">
                        <span>💬</span>
                        <span>Support</span>
                      </CardTitle>
                      {openAccordionSection === "support" ? (
                        <ChevronDown className="h-5 w-5 text-muted-foreground" />
                      ) : (
                        <ChevronRight className="h-5 w-5 text-muted-foreground" />
                      )}
                    </div>
                  </CardHeader>
                </button>
                {openAccordionSection === "support" && (
                  <CardContent className="px-5 pb-4 pt-0 border-t">
                    <div className="pt-4 space-y-3">
                      <p className="text-sm text-muted-foreground">
                        Found a bug or have a feature request?
                      </p>
                      <div className="flex flex-wrap gap-2">
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={(e) => {
                            e.stopPropagation();
                            window.electron.openExternal(
                              "https://github.com/bizzkoot/copilot-tracker/issues",
                            );
                          }}
                        >
                          <Bug className="mr-2 h-4 w-4" />
                          Report Issue
                        </Button>
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={(e) => {
                            e.stopPropagation();
                            window.electron.openExternal(
                              "https://github.com/bizzkoot/copilot-tracker/discussions",
                            );
                          }}
                        >
                          <ExternalLink className="mr-2 h-4 w-4" />
                          Discussions
                        </Button>
                      </div>
                    </div>
                  </CardContent>
                )}
              </Card>
            </div>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
}
