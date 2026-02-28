/**
 * Dashboard Component
 * Main dashboard view with all usage components
 */

import { UsageCard } from "./UsageCard";
import { PredictionCard } from "./PredictionCard";
import { UsageChart } from "./UsageChart";
import { HistoryTable } from "./HistoryTable";
import { useUsage } from "@renderer/hooks/useUsage";
import { useAuth } from "@renderer/hooks/useAuth";
import { useSettingsStore } from "@renderer/stores/settingsStore";
import { Button } from "../ui/button";
import {
  RefreshCw,
  AlertCircle,
  LogIn,
  Download,
  Zap,
  Bug,
} from "lucide-react";

function exportDebugData(
  usage: unknown,
  history: unknown,
  prediction: unknown,
  lastUpdated: Date | null,
) {
  const debugData = {
    exportedAt: new Date().toISOString(),
    appVersion: "debug",
    usage,
    history,
    prediction,
    lastUpdated: lastUpdated?.toISOString() ?? null,
  };

  const blob = new Blob([JSON.stringify(debugData, null, 2)], {
    type: "application/json",
  });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `copilot-debug-${new Date().toISOString().split("T")[0]}.json`;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

function exportCaptureData(payload: unknown) {
  const blob = new Blob([JSON.stringify(payload, null, 2)], {
    type: "application/json",
  });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `copilot-capture-${new Date().toISOString().split("T")[0]}.json`;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
}

export function Dashboard() {
  const {
    usage,
    history,
    prediction,
    isLoading,
    error,
    lastUpdated,
    refresh,
    forceRefresh,
  } = useUsage();
  const { login, isAuthenticated } = useAuth();
  const debugToolsEnabled = useSettingsStore((state) => state.debugToolsEnabled);

  // Check if error is auth-related
  const isAuthError =
    error &&
    (error.toLowerCase().includes("customer id") ||
      error.toLowerCase().includes("login") ||
      error.toLowerCase().includes("auth") ||
      error.toLowerCase().includes("unauthorized"));

  const captureExtractionDebug = async () => {
    try {
      const extraction = await window.electron.captureExtractionDebug();
      exportCaptureData({
        exportedAt: new Date().toISOString(),
        appVersion: "capture",
        usage,
        history,
        prediction,
        lastUpdated: lastUpdated?.toISOString() ?? null,
        extraction,
      });
    } catch (captureError) {
      console.error("Capture extraction debug failed:", captureError);
    }
  };

  return (
    <div className="space-y-4">
      {/* Error State */}
      {error && (
        <div className="p-4 rounded-lg bg-destructive/10 border border-destructive/20 flex items-center gap-3">
          <AlertCircle className="h-5 w-5 text-destructive" />
          <div className="flex-1">
            <p className="font-medium text-destructive">
              Failed to load usage data
            </p>
            <p className="text-sm text-muted-foreground">{error}</p>
          </div>
          <div className="flex items-center gap-2">
            {isAuthError && !isAuthenticated && (
              <Button variant="default" size="sm" onClick={login}>
                <LogIn className="h-4 w-4 mr-2" />
                Login
              </Button>
            )}
            <Button variant="outline" size="sm" onClick={refresh}>
              <RefreshCw className="h-4 w-4 mr-2" />
              Refresh
            </Button>
          </div>
        </div>
      )}

      {/* Last Updated */}
      {lastUpdated && !error && (
        <div className="flex items-center justify-between text-sm text-muted-foreground">
          <span>Last updated: {lastUpdated.toLocaleTimeString()}</span>
          <div className="flex items-center gap-2">
            {debugToolsEnabled && (
              <>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() =>
                    exportDebugData(usage, history, prediction, lastUpdated)
                  }
                  title="Export debug data"
                >
                  <Download className="h-4 w-4 mr-2" />
                  Debug
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={captureExtractionDebug}
                  title="Capture full extraction payload"
                >
                  <Bug className="h-4 w-4 mr-2" />
                  Capture
                </Button>
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={forceRefresh}
                  disabled={isLoading}
                  title="Clear cache and force fresh fetch"
                >
                  <Zap
                    className={`h-4 w-4 mr-2 ${isLoading ? "animate-pulse" : ""}`}
                  />
                  Force
                </Button>
              </>
            )}
            <Button
              variant="ghost"
              size="sm"
              onClick={refresh}
              disabled={isLoading}
            >
              <RefreshCw
                className={`h-4 w-4 mr-2 ${isLoading ? "animate-spin" : ""}`}
              />
              Refresh
            </Button>
          </div>
        </div>
      )}

      {/* Usage and Forecast Hero Section */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-6 items-stretch">
        <UsageCard usage={usage} isLoading={isLoading && !usage} />
        <PredictionCard
          prediction={prediction}
          usage={usage}
          isLoading={isLoading && !prediction}
        />
      </div>

      {/* Usage Trend Chart */}
      <UsageChart history={history} isLoading={isLoading} />

      {/* History Table */}
      <HistoryTable history={history} isLoading={isLoading && !history} />
    </div>
  );
}
