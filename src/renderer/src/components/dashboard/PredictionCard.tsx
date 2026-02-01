/**
 * PredictionCard Component
 * Displays end of month usage prediction
 */

import { Card, CardContent, CardHeader, CardTitle } from "../ui/card";
import { Skeleton } from "../ui/skeleton";
import { Tooltip } from "../ui/tooltip";
import type { UsagePrediction, CopilotUsage } from "@renderer/types/usage";
import { getLimitRequests, formatCurrency } from "@renderer/types/usage";
import { getConfidenceDescription } from "@renderer/services/predictor";
import { TrendingUp, AlertTriangle, CheckCircle } from "lucide-react";

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
      <Card>
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
      <Card>
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
  const willExceed = prediction.predictedMonthlyRequests > limit;
  const excessAmount = Math.max(0, prediction.predictedMonthlyRequests - limit);
  const percentOfLimit = (prediction.predictedMonthlyRequests / limit) * 100;

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-lg flex items-center justify-between">
          <span>Monthly Prediction</span>
          <Tooltip
            content={getConfidenceDescription(prediction.confidenceLevel)}
          >
            <span
              className={`text-xs px-2 py-1 rounded-full ${
                prediction.confidenceLevel === "high"
                  ? "bg-green-500/20 text-green-500"
                  : prediction.confidenceLevel === "medium"
                    ? "bg-yellow-500/20 text-yellow-500"
                    : "bg-red-500/20 text-red-500"
              }`}
            >
              {prediction.confidenceLevel.charAt(0).toUpperCase() +
                prediction.confidenceLevel.slice(1)}{" "}
              confidence
            </span>
          </Tooltip>
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Predicted Total */}
        <div className="flex items-center gap-3">
          {willExceed ? (
            <TrendingUp className="h-8 w-8 text-orange-500" />
          ) : percentOfLimit > 75 ? (
            <AlertTriangle className="h-8 w-8 text-yellow-500" />
          ) : (
            <CheckCircle className="h-8 w-8 text-green-500" />
          )}
          <div>
            <div className="flex items-baseline gap-2">
              <span className="text-3xl font-bold">
                {prediction.predictedMonthlyRequests.toLocaleString()}
              </span>
              <span className="text-muted-foreground">requests</span>
            </div>
            <div className="text-sm text-muted-foreground">
              {percentOfLimit.toFixed(0)}% of monthly limit
            </div>
          </div>
        </div>

        {/* Will Exceed Warning */}
        {willExceed && (
          <div className="p-3 rounded-lg bg-orange-500/10 border border-orange-500/20">
            <div className="flex items-center gap-2 text-orange-500">
              <TrendingUp className="h-4 w-4" />
              <span className="font-medium">May exceed limit</span>
            </div>
            <div className="mt-1 text-sm text-muted-foreground">
              Estimated {excessAmount.toLocaleString()} requests over limit
            </div>
            {prediction.predictedBilledAmount > 0 && (
              <div className="mt-1 text-sm font-medium text-orange-500">
                Est. add-on cost:{" "}
                {formatCurrency(prediction.predictedBilledAmount)}
              </div>
            )}
          </div>
        )}

        {/* Safe Notice */}
        {!willExceed && percentOfLimit < 75 && (
          <div className="p-3 rounded-lg bg-green-500/10 border border-green-500/20">
            <div className="flex items-center gap-2 text-green-500">
              <CheckCircle className="h-4 w-4" />
              <span className="font-medium">On track</span>
            </div>
            <div className="mt-1 text-sm text-muted-foreground">
              Likely to stay within your included quota
            </div>
          </div>
        )}

        {/* Based on data note */}
        <div className="text-xs text-muted-foreground">
          Based on {prediction.daysUsedForPrediction} days of usage data
        </div>
      </CardContent>
    </Card>
  );
}
