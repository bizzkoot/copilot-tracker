/**
 * Usage Predictor Service
 * Ported from UsagePredictor.swift - predicts end of month usage
 */

import {
  CopilotUsage,
  UsageHistory,
  UsagePrediction,
  DailyUsage,
  PREDICTION_PERIODS,
  COST_PER_REQUEST,
  getTotalRequests,
  isWeekend,
  getUsedRequests,
  getLimitRequests,
} from "../types/usage";

/**
 * Predict end-of-month usage based on history and current usage
 */
export function predictUsage(
  history: UsageHistory,
  currentUsage: CopilotUsage,
  predictionPeriod: number,
): UsagePrediction {
  const config = PREDICTION_PERIODS[predictionPeriod];
  if (!config) {
    throw new Error(`Invalid prediction period: ${predictionPeriod}`);
  }

  const dailyData = history.days.slice(0, predictionPeriod);

  // 1. Calculate weighted average daily usage
  const weightedAvgDaily = calculateWeightedAverage(dailyData, config.weights);

  // 2. Calculate weekend/weekday ratio
  const weekendRatio = calculateWeekendRatio(dailyData);

  // 3. Calculate remaining days
  const today = new Date();
  const daysInMonth = getDaysInMonth(today);
  const currentDay = today.getDate();
  const remainingDays = daysInMonth - currentDay;
  const { remainingWeekdays, remainingWeekends } = countRemainingDays(
    today,
    remainingDays,
  );

  // 4. Predict total monthly usage
  const currentTotal = getUsedRequests(currentUsage);
  const predictedRemaining =
    weightedAvgDaily * remainingWeekdays +
    weightedAvgDaily * weekendRatio * remainingWeekends;
  const predictedMonthlyTotal = currentTotal + predictedRemaining;

  // 5. Calculate predicted add-on cost
  const limit = getLimitRequests(currentUsage);
  const excessRequests = Math.max(0, predictedMonthlyTotal - limit);
  const predictedBilledAmount = excessRequests * COST_PER_REQUEST;

  // 6. Determine confidence level
  const confidenceLevel = getConfidenceLevel(dailyData.length);

  return {
    predictedMonthlyRequests: Math.round(predictedMonthlyTotal),
    predictedBilledAmount: Math.round(predictedBilledAmount * 100) / 100,
    confidenceLevel,
    daysUsedForPrediction: dailyData.length,
  };
}

/**
 * Calculate weighted average of daily usage
 */
function calculateWeightedAverage(
  dailyData: DailyUsage[],
  weights: number[],
): number {
  let weightedSum = 0;
  let totalWeight = 0;

  dailyData.forEach((day, index) => {
    const weight = weights[index] || 1.0;
    weightedSum += getTotalRequests(day) * weight;
    totalWeight += weight;
  });

  return totalWeight > 0 ? weightedSum / totalWeight : 0;
}

/**
 * Calculate ratio of weekend to weekday usage
 */
function calculateWeekendRatio(dailyData: DailyUsage[]): number {
  const weekendDays = dailyData.filter((d) => isWeekend(d.date));
  const weekdayDays = dailyData.filter((d) => !isWeekend(d.date));

  if (weekdayDays.length === 0) return 1.0;

  const avgWeekend = average(weekendDays.map((d) => getTotalRequests(d)));
  const avgWeekday = average(weekdayDays.map((d) => getTotalRequests(d)));

  return avgWeekday > 0 ? avgWeekend / avgWeekday : 1.0;
}

/**
 * Calculate average of number array
 */
function average(nums: number[]): number {
  return nums.length > 0 ? nums.reduce((a, b) => a + b, 0) / nums.length : 0;
}

/**
 * Get number of days in a month
 */
function getDaysInMonth(date: Date): number {
  return new Date(date.getFullYear(), date.getMonth() + 1, 0).getDate();
}

/**
 * Count remaining weekdays and weekends
 */
function countRemainingDays(
  today: Date,
  remaining: number,
): { remainingWeekdays: number; remainingWeekends: number } {
  let weekdays = 0;
  let weekends = 0;

  for (let i = 1; i <= remaining; i++) {
    const date = new Date(today);
    date.setDate(today.getDate() + i);
    if (isWeekend(date)) {
      weekends++;
    } else {
      weekdays++;
    }
  }

  return { remainingWeekdays: weekdays, remainingWeekends: weekends };
}

/**
 * Determine confidence level based on data availability
 */
function getConfidenceLevel(daysCount: number): "low" | "medium" | "high" {
  if (daysCount < 3) return "low";
  if (daysCount < 7) return "medium";
  return "high";
}

/**
 * Generate prediction with fallback for missing history
 */
export function generatePrediction(
  currentUsage: CopilotUsage | null,
  history: UsageHistory | null,
  predictionPeriod: number,
): UsagePrediction | null {
  if (!currentUsage || !history || history.days.length === 0) {
    return null;
  }

  try {
    return predictUsage(history, currentUsage, predictionPeriod);
  } catch (error) {
    console.error("Failed to generate prediction:", error);
    return null;
  }
}

/**
 * Get prediction confidence description
 */
export function getConfidenceDescription(
  level: "low" | "medium" | "high",
): string {
  switch (level) {
    case "low":
      return "Limited data available (< 3 days)";
    case "medium":
      return "Moderate data available (3-6 days)";
    case "high":
      return "Sufficient data available (7+ days)";
  }
}

/**
 * Get prediction confidence color
 */
export function getConfidenceColor(level: "low" | "medium" | "high"): string {
  switch (level) {
    case "low":
      return "text-red-500";
    case "medium":
      return "text-yellow-500";
    case "high":
      return "text-green-500";
  }
}
