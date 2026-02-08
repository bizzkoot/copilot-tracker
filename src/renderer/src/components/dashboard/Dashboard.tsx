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
import { Button } from "../ui/button";
import { RefreshCw, AlertCircle, LogIn } from "lucide-react";

export function Dashboard() {
  const { usage, history, prediction, isLoading, error, lastUpdated, refresh } =
    useUsage();
  const { login, isAuthenticated } = useAuth();

  // Check if error is auth-related
  const isAuthError =
    error &&
    (error.toLowerCase().includes("customer id") ||
      error.toLowerCase().includes("login") ||
      error.toLowerCase().includes("auth") ||
      error.toLowerCase().includes("unauthorized"));

  // Calculate daily average for chart reference line
  const dailyAverage =
    history && history.days.length > 0
      ? history.days.reduce(
        (sum, day) => sum + day.includedRequests + day.billedRequests,
        0,
      ) / history.days.length
      : undefined;

  return (
    <div className="space-y-6">
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

      {/* Usage Chart */}
      <UsageChart
        history={history}
        isLoading={isLoading && !history}
        dailyAverage={dailyAverage}
      />

      {/* History Table */}
      <HistoryTable history={history} isLoading={isLoading && !history} />
    </div>
  );
}
