/**
 * PredictionCard Component
 * Displays end of month usage prediction
 */

import {
  TrendingUp,
  AlertTriangle,
  CheckCircle,
  Calendar,
  Zap,
} from "lucide-react";
import { Card, CardContent, CardHeader, CardTitle } from "../ui/card";
import { Skeleton } from "../ui/skeleton";
import { Tooltip } from "../ui/tooltip";
import type { UsagePrediction, CopilotUsage } from "@renderer/types/usage";
import {
  getLimitRequests,
  formatCurrency,
  getUsedRequests,
} from "@renderer/types/usage";
import {
  getConfidenceDescription,
  getDaysInMonth,
} from "@renderer/services/predictor";

interface PredictionCardProps {
  prediction: UsagePrediction | null;
  usage: CopilotUsage | null;
  isLoading?: boolean;
}

export function PredictionCard({
  prediction,
  usage,
  isLoading,
}: PredictionCardProps) {
  if (isLoading) {
    return (
      <Card className="h-full">
        <CardHeader>
          <CardTitle className="text-lg">Monthly Prediction</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <Skeleton className="h-8 w-40" />
          <Skeleton className="h-4 w-32" />
          <Skeleton className="h-4 w-28" />
        </CardContent>
      </Card>
    );
  }

  if (!prediction || !usage) {
    return (
      <Card className="h-full">
        <CardHeader>
          <CardTitle className="text-lg">Monthly Prediction</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-muted-foreground">
            Need more usage data for prediction
          </p>
        </CardContent>
      </Card>
    );
  }

  const limit = getLimitRequests(usage);
  const used = getUsedRequests(usage);
  const willExceed = prediction.predictedMonthlyRequests > limit;
  const excessAmount = Math.max(0, prediction.predictedMonthlyRequests - limit);
  const percentOfLimit = (prediction.predictedMonthlyRequests / limit) * 100;

  // Calculate days remaining in the month
  const today = new Date();
  const daysInMonth = getDaysInMonth(today);
  const currentDay = today.getDate();
  const daysRemaining = Math.max(1, daysInMonth - currentDay);

  // Calculate daily budget
  const remainingRequests = Math.max(0, limit - used);
  const dailyBudget = Math.floor(remainingRequests / daysRemaining);

  return (
    <Card className="h-full border-primary/20 bg-gradient-to-br from-card to-primary/5">
      <CardHeader className="pb-2">
        <CardTitle className="text-base font-medium text-muted-foreground uppercase tracking-wider flex items-center justify-between">
          <span>Monthly Forecast</span>
          <Tooltip
            content={getConfidenceDescription(prediction.confidenceLevel)}
          >
            <span
              className={`text-[10px] px-2 py-0.5 rounded-full border ${
                prediction.confidenceLevel === "high"
                  ? "bg-green-500/10 text-green-600 border-green-500/20"
                  : prediction.confidenceLevel === "medium"
                    ? "bg-yellow-500/10 text-yellow-600 border-yellow-500/20"
                    : "bg-red-500/10 text-red-600 border-red-500/20"
              }`}
            >
              {prediction.confidenceLevel.toUpperCase()} CONFIDENCE
            </span>
          </Tooltip>
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-6 pt-2">
        {/* Forecasted Total */}
        <div className="flex items-center gap-4">
          <div
            className={`p-3 rounded-2xl ${willExceed ? "bg-orange-500/10" : "bg-green-500/10"}`}
          >
            {willExceed ? (
              <TrendingUp className="h-8 w-8 text-orange-500" />
            ) : percentOfLimit > 75 ? (
              <AlertTriangle className="h-8 w-8 text-yellow-500" />
            ) : (
              <CheckCircle className="h-8 w-8 text-green-500" />
            )}
          </div>
          <div>
            <div className="flex items-baseline gap-2">
              <span className="text-3xl font-bold tracking-tight">
                {prediction.predictedMonthlyRequests.toLocaleString()}
              </span>
              <span className="text-sm text-muted-foreground font-medium uppercase">
                Expected
              </span>
            </div>
            <div className="text-sm font-medium text-muted-foreground">
              {percentOfLimit.toFixed(0)}% of monthly limit
            </div>
          </div>
        </div>

        {/* Actionable Metrics */}
        <div className="grid grid-cols-2 gap-4">
          <div className="p-3 rounded-xl bg-secondary/30 border border-border/50">
            <div className="flex items-center gap-2 mb-1">
              <Calendar className="h-4 w-4 text-primary" />
              <span className="text-[10px] font-bold text-muted-foreground uppercase">
                Ends In
              </span>
            </div>
            <div className="text-xl font-bold">{daysRemaining} Days</div>
          </div>
          <div className="p-3 rounded-xl bg-secondary/30 border border-border/50">
            <div className="flex items-center gap-2 mb-1">
              <Zap className="h-4 w-4 text-primary" />
              <span className="text-[10px] font-bold text-muted-foreground uppercase">
                Daily Budget
              </span>
            </div>
            <div className="text-xl font-bold">
              {dailyBudget}
              <span className="text-xs font-normal text-muted-foreground ml-1">
                req
              </span>
            </div>
          </div>
        </div>

        {/* Status Alert */}
        {willExceed ? (
          <div className="p-3 rounded-xl bg-orange-500/10 border border-orange-500/20">
            <div className="flex items-center gap-2 text-orange-600 dark:text-orange-400">
              <AlertTriangle className="h-4 w-4" />
              <span className="text-sm font-bold">Capacity Alert</span>
            </div>
            <div className="mt-1 text-xs text-muted-foreground">
              Estimated {excessAmount.toLocaleString()} requests over limit.
              {prediction.predictedBilledAmount > 0 && (
                <span className="block mt-1 font-bold text-orange-600 dark:text-orange-400">
                  Est. extra cost:{" "}
                  {formatCurrency(prediction.predictedBilledAmount)}
                </span>
              )}
            </div>
          </div>
        ) : (
          <div className="p-3 rounded-xl bg-green-500/10 border border-green-500/20">
            <div className="flex items-center gap-2 text-green-600 dark:text-green-400">
              <CheckCircle className="h-4 w-4" />
              <span className="text-sm font-bold">Safe Consumption</span>
            </div>
            <p className="mt-1 text-xs text-muted-foreground">
              You are likely to stay within your included quota this month.
            </p>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
