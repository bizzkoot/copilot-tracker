/**
 * UsageCard Component
 * Displays current Copilot usage with progress bar
 */

import { Card, CardContent, CardHeader, CardTitle } from "../ui/card";
import { Progress } from "../ui/progress";
import { Skeleton } from "../ui/skeleton";
import { Tooltip } from "../ui/tooltip";
import type { CopilotUsage } from "@renderer/types/usage";
import {
  getUsedRequests,
  getLimitRequests,
  getUsagePercentage,
  getProgressColor,
  formatCurrency,
} from "@renderer/types/usage";

interface UsageCardProps {
  usage: CopilotUsage | null;
  isLoading?: boolean;
}

export function UsageCard({ usage, isLoading }: UsageCardProps) {
  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Current Usage</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <Skeleton className="h-4 w-full" />
          <Skeleton className="h-8 w-32" />
          <Skeleton className="h-4 w-24" />
        </CardContent>
      </Card>
    );
  }

  if (!usage) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Current Usage</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-muted-foreground">No usage data available</p>
        </CardContent>
      </Card>
    );
  }

  const used = getUsedRequests(usage);
  const limit = getLimitRequests(usage);
  const percentage = getUsagePercentage(usage);
  const progressColor = getProgressColor(percentage);
  const addOnCost = usage.netBilledAmount;

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-lg flex items-center justify-between">
          <span>Current Usage</span>
          <span className="text-sm font-normal text-muted-foreground">
            This billing period
          </span>
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Progress Bar */}
        <div className="space-y-2">
          <Tooltip
            content={`${used.toLocaleString()} of ${limit.toLocaleString()} requests used`}
          >
            <Progress
              value={percentage}
              className="h-3"
              indicatorClassName={progressColor}
            />
          </Tooltip>
        </div>

        {/* Usage Numbers */}
        <div className="flex items-baseline gap-2">
          <span className="text-3xl font-bold">{used.toLocaleString()}</span>
          <span className="text-muted-foreground">
            / {limit.toLocaleString()}
          </span>
          <span
            className={`text-sm font-medium ${percentage >= 90 ? "text-red-500" : percentage >= 75 ? "text-yellow-500" : "text-green-500"}`}
          >
            ({percentage.toFixed(1)}%)
          </span>
        </div>

        {/* Add-on Cost */}
        {addOnCost > 0 ? (
          <div className="flex items-center gap-2 text-orange-500">
            <span className="text-sm">Add-on charges:</span>
            <span className="font-semibold">{formatCurrency(addOnCost)}</span>
          </div>
        ) : (
          <div className="text-sm text-muted-foreground">
            No add-on charges this period
          </div>
        )}
      </CardContent>
    </Card>
  );
}
