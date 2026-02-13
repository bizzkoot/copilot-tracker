/**
 * UsageCard Component
 * Displays current Copilot usage with progress bar
 */

import { Card, CardContent, CardHeader, CardTitle } from "../ui/card";
import { Gauge } from "../ui/gauge";
import { Skeleton } from "../ui/skeleton";
import { Tooltip } from "../ui/tooltip";
import type { CopilotUsage } from "@renderer/types/usage";
import {
  getUsedRequests,
  getLimitRequests,
  getUsagePercentage,
  formatCurrency,
} from "@renderer/types/usage";
import { useTrayIconFormat } from "@renderer/stores/settingsStore";
import {
  TRAY_FORMAT_REMAINING_TOTAL,
  TRAY_FORMAT_REMAINING_PERCENT,
  TRAY_FORMAT_REMAINING_COMBINED,
} from "@renderer/types/settings";

// Helper to determine if we should show remaining values
const isRemainingFormat = (format: string): boolean => {
  return (
    format === TRAY_FORMAT_REMAINING_TOTAL ||
    format === TRAY_FORMAT_REMAINING_PERCENT ||
    format === TRAY_FORMAT_REMAINING_COMBINED
  );
};

interface UsageCardProps {
  usage: CopilotUsage | null;
  isLoading?: boolean;
}

export function UsageCard({ usage, isLoading }: UsageCardProps) {
  const trayIconFormat = useTrayIconFormat();

  if (isLoading) {
    return (
      <Card className="h-full">
        <CardHeader>
          <CardTitle className="text-lg">Quota Status</CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col items-center justify-center pt-2 pb-8">
          <Skeleton className="h-[120px] w-[120px] rounded-full" />
          <Skeleton className="h-6 w-32 mt-6" />
        </CardContent>
      </Card>
    );
  }

  if (!usage) {
    return (
      <Card className="h-full">
        <CardHeader>
          <CardTitle className="text-lg">Quota Status</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-muted-foreground">No usage data available.</p>
        </CardContent>
      </Card>
    );
  }

  const used = getUsedRequests(usage);
  const limit = getLimitRequests(usage);
  const percentage = getUsagePercentage(usage);

  // Determine if we should show remaining values based on tray icon format
  const showRemaining = isRemainingFormat(trayIconFormat);

  // Calculate display values based on tray icon format
  const displayUsed = showRemaining ? limit - used : used;
  const displayPercentage = showRemaining ? 100 - percentage : percentage;

  const displayLabel = showRemaining ? "REMAINING" : "CONSUMED";

  const getGaugeColor = (pct: number) => {
    if (pct >= 90) return "text-red-500";
    if (pct >= 75) return "text-yellow-500";
    return "text-green-500";
  };

  const addOnCost = usage.netBilledAmount;

  return (
    <Card className="h-full overflow-hidden border-primary/20 bg-gradient-to-br from-card to-primary/5">
      <CardHeader className="pb-2">
        <CardTitle className="text-base font-medium text-muted-foreground uppercase tracking-wider">
          Quota {displayLabel}
        </CardTitle>
      </CardHeader>
      <CardContent className="flex flex-col items-center justify-center pt-2 pb-6 space-y-6">
        <div className="relative group">
          <Tooltip
            content={`${used.toLocaleString()} of ${limit.toLocaleString()} requests used`}
          >
            <Gauge
              value={displayPercentage}
              size={140}
              strokeWidth={12}
              indicatorClassName={getGaugeColor(percentage)}
              className="drop-shadow-sm"
            />
          </Tooltip>
        </div>

        <div className="text-center">
          <div className="flex items-baseline justify-center gap-1">
            <span className="text-3xl font-bold tracking-tight">
              {displayUsed.toLocaleString()}
            </span>
            <span className="text-sm text-muted-foreground font-medium">
              / {limit.toLocaleString()}
            </span>
          </div>
          <p className="text-xs text-muted-foreground mt-1 font-medium">
            REQUESTS THIS PERIOD
          </p>
        </div>

        {addOnCost > 0 && (
          <div className="flex items-center gap-2 px-3 py-1 rounded-full bg-orange-500/10 border border-orange-500/20 text-orange-600 dark:text-orange-400">
            <span className="text-xs font-semibold whitespace-nowrap">
              Add-on: {formatCurrency(addOnCost)}
            </span>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
