/**
 * Copilot Usage Types
 * Ported from the original macOS Swift application
 */

// Current usage data from GitHub's billing API
export interface CopilotUsage {
  netBilledAmount: number; // Add-on cost in dollars
  netQuantity: number; // Net requests used
  discountQuantity: number; // Requests within included quota
  userPremiumRequestEntitlement: number; // Monthly limit
  filteredUserPremiumRequestEntitlement: number;
}

// Cached usage with timestamp
export interface CachedUsage {
  usage: CopilotUsage;
  timestamp: Date | string;
}

// Daily usage breakdown
export interface DailyUsage {
  date: string | Date; // UTC date (string when from IPC, Date when parsed)
  includedRequests: number; // Requests within quota
  billedRequests: number; // Add-on billed requests
  grossAmount: number; // Gross amount
  billedAmount: number; // Add-on cost
}

// Usage history collection
export interface UsageHistory {
  fetchedAt: string | Date;
  days: DailyUsage[];
}

// Prediction result
export interface UsagePrediction {
  predictedMonthlyRequests: number;
  predictedBilledAmount: number;
  confidenceLevel: "low" | "medium" | "high";
  daysUsedForPrediction: number;
}

// Prediction weights configuration
export interface PredictionWeights {
  period: number;
  weights: number[];
}

// Predefined prediction periods
export const PREDICTION_PERIODS: Record<number, PredictionWeights> = {
  7: {
    period: 7,
    weights: [1.5, 1.5, 1.2, 1.2, 1.2, 1.0, 1.0],
  },
  14: {
    period: 14,
    weights: [
      2.0, 1.8, 1.6, 1.4, 1.2, 1.2, 1.0, 1.0, 1.0, 1.0, 0.8, 0.8, 0.6, 0.6,
    ],
  },
  21: {
    period: 21,
    weights: [
      2.5, 2.3, 2.1, 1.9, 1.7, 1.5, 1.3, 1.3, 1.2, 1.2, 1.1, 1.1, 1.0, 1.0, 0.9,
      0.9, 0.8, 0.8, 0.7, 0.7, 0.6,
    ],
  },
};

// Cost per add-on request
// Note: The main process uses the actual configured value from .env
// This default is only used for renderer-side calculations
export const COST_PER_REQUEST = 0.04;

// ============= Computed Helper Functions =============

/**
 * Get total used requests from usage data
 */
export function getUsedRequests(usage: CopilotUsage): number {
  return Math.round(usage.discountQuantity);
}

/**
 * Get monthly request limit
 */
export function getLimitRequests(usage: CopilotUsage): number {
  return usage.userPremiumRequestEntitlement;
}

/**
 * Get usage percentage (0-100)
 */
export function getUsagePercentage(usage: CopilotUsage): number {
  const limit = getLimitRequests(usage);
  if (limit === 0) return 0;
  return (getUsedRequests(usage) / limit) * 100;
}

/**
 * Get total requests for a day
 */
export function getTotalRequests(day: DailyUsage): number {
  return day.includedRequests + day.billedRequests;
}

/**
 * Check if a date is a weekend
 */
export function isWeekend(date: Date | string): boolean {
  const dateObj = typeof date === "string" ? new Date(date) : date;
  const day = dateObj.getDay();
  return day === 0 || day === 6;
}

/**
 * Get day of week (0-6, Sunday = 0)
 */
export function getDayOfWeek(date: Date | string): number {
  const dateObj = typeof date === "string" ? new Date(date) : date;
  return dateObj.getDay();
}

/**
 * Get color class based on usage percentage
 */
export function getUsageColor(percentage: number): string {
  if (percentage < 50) return "text-green-500";
  if (percentage < 75) return "text-yellow-500";
  if (percentage < 90) return "text-orange-500";
  return "text-red-500";
}

/**
 * Get progress bar color based on usage percentage
 */
export function getProgressColor(percentage: number): string {
  if (percentage < 50) return "bg-green-500";
  if (percentage < 75) return "bg-yellow-500";
  if (percentage < 90) return "bg-orange-500";
  return "bg-red-500";
}

/**
 * Format currency amount
 */
export function formatCurrency(amount: number): string {
  return new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: "USD",
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(amount);
}

/**
 * Format date for display
 */
export function formatDate(date: Date | string): string {
  const dateObj = typeof date === "string" ? new Date(date) : date;
  return new Intl.DateTimeFormat("en-US", {
    month: "short",
    day: "numeric",
  }).format(dateObj);
}
