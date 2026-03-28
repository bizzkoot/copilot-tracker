/**
 * UsageChart Component
 * Line chart showing daily usage trends
 */

import { Card, CardContent, CardHeader, CardTitle } from "../ui/card";
import { Skeleton } from "../ui/skeleton";
import { useEffect, useMemo, useRef, useState } from "react";
import type {
  UsageHistory,
  CopilotUsage,
  DailyUsage,
} from "@renderer/types/usage";
import {
  getTotalRequests,
  isWeekend,
  formatDate,
  formatRequestCount,
  COST_PER_REQUEST,
} from "@renderer/types/usage";
import {
  LineChart,
  Line,
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
} from "recharts";
import { Tabs, TabsList, TabsTrigger } from "../ui/tabs";

interface UsageChartProps {
  history: UsageHistory | null;
  usage?: CopilotUsage | null;
  isLoading?: boolean;
}

interface ChartDataPoint {
  rawDate: Date;
  date: string;
  fullDate: string;
  usage: number; // Total usage for daily (or just total overall)
  included?: number; // For stacked
  billed?: number; // For stacked
  quota?: number; // Monthly/yearly quota
  utilization?: number; // Quota utilization percentage
  trend?: number;
  budget?: number;
  isWeekend: boolean;
  /** True when the quota for this point is an estimate (using current plan as fallback). */
  quotaEstimated?: boolean;
  /** True when this month/year has a different quota than the previous period,
   *  indicating a plan change occurred at or before this data point. */
  quotaChanged?: boolean;
}

type Timeframe = "current_month" | "monthly" | "yearly";

/**
 * Calculate Simple Moving Average (SMA)
 * period = number of days to look back (e.g. 7 for a weekly average)
 */
function calculateSMA(data: number[], period: number = 7): number[] {
  if (data.length === 0) return [];

  const sma: number[] = [];

  for (let i = 0; i < data.length; i++) {
    let sum = 0;
    let count = 0;

    for (let j = Math.max(0, i - period + 1); j <= i; j++) {
      sum += data[j];
      count++;
    }

    sma.push(sum / count);
  }

  return sma;
}

export function UsageChart({ history, usage, isLoading }: UsageChartProps) {
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const [timeframe, setTimeframe] = useState<Timeframe>("current_month");

  const historyDays = history?.days ?? [];

  // Transform data for chart (oldest first), padding missing days with 0 usage
  const rawData = useMemo(() => {
    if (historyDays.length === 0) return [];

    const sortedDays = [...historyDays].reverse();
    const paddedData: {
      rawDate: Date;
      date: string;
      fullDate: string;
      usage: number;
      included: number;
      billed: number;
      limit?: number;
      quotaEstimated?: boolean;
      isWeekend: boolean;
    }[] = [];

    const firstDay = sortedDays[0];
    const lastDay = sortedDays[sortedDays.length - 1];

    const startDate =
      typeof firstDay.date === "string"
        ? new Date(firstDay.date)
        : new Date(firstDay.date);
    const endDate =
      typeof lastDay.date === "string"
        ? new Date(lastDay.date)
        : new Date(lastDay.date);

    // Normalize to start of UTC day to avoid timezone shifts during loop
    startDate.setUTCHours(0, 0, 0, 0);
    endDate.setUTCHours(0, 0, 0, 0);

    const dayMap = new Map<string, typeof firstDay>();
    for (const day of sortedDays) {
      const d = typeof day.date === "string" ? new Date(day.date) : day.date;
      dayMap.set(d.toISOString().split("T")[0], day);
    }

    const current = new Date(startDate);
    while (current <= endDate) {
      const dateKey = current.toISOString().split("T")[0];
      const existingDay = dayMap.get(dateKey);

      paddedData.push({
        rawDate: new Date(current),
        date: formatDate(current),
        fullDate: current.toLocaleDateString(),
        usage: existingDay ? getTotalRequests(existingDay) : 0,
        included: existingDay ? existingDay.includedRequests : 0,
        billed: existingDay ? existingDay.billedRequests : 0,
        limit: existingDay ? (existingDay as DailyUsage).limit : undefined,
        quotaEstimated: existingDay
          ? (existingDay as DailyUsage).quotaEstimated
          : undefined,
        isWeekend: isWeekend(current),
      });

      current.setUTCDate(current.getUTCDate() + 1);
    }

    return paddedData;
  }, [historyDays]);

  // Aggregate data based on timeframe
  const aggregatedData: ChartDataPoint[] = useMemo(() => {
    if (timeframe === "current_month") {
      // Filter the padded data for the CURRENT display month only
      const now = new Date();
      const currentYear = now.getUTCFullYear();
      const currentMonth = now.getUTCMonth();

      const filtered = rawData.filter((d) => {
        return (
          d.rawDate.getUTCFullYear() === currentYear &&
          d.rawDate.getUTCMonth() === currentMonth
        );
      });

      const usageValues = filtered.map((d) => d.usage);
      const trendValues = calculateSMA(usageValues, 7);

      let cumulativeUsage = 0;
      const totalQuota = usage?.userPremiumRequestEntitlement || 0;

      return filtered.map((d, i) => {
        const usageBeforeToday = cumulativeUsage;
        cumulativeUsage += d.usage;

        let dailyBudget: number | undefined = undefined;

        if (totalQuota > 0) {
          const daysInMonth = new Date(
            Date.UTC(currentYear, currentMonth + 1, 0),
          ).getUTCDate();
          const currentDayOfMonth = d.rawDate.getUTCDate();

          const remainingDays = daysInMonth - currentDayOfMonth + 1;
          const remainingQuota = Math.max(0, totalQuota - usageBeforeToday);

          if (remainingDays > 0) {
            dailyBudget = remainingQuota / remainingDays;
          }
        }

        return {
          ...d,
          trend: trendValues[i],
          budget: dailyBudget,
        };
      });
    } else if (timeframe === "monthly") {
      // Group by YYYY-MM and track quota information
      const monthlyMap = new Map<string, ChartDataPoint>();

      for (const d of rawData) {
        const year = d.rawDate.getUTCFullYear();
        const month = d.rawDate.getUTCMonth();
        const key = `${year}-${String(month + 1).padStart(2, "0")}`;

        // Month name for label (e.g. "Feb 2026")
        const dateStr = new Intl.DateTimeFormat("en-US", {
          month: "short",
          year: "numeric",
        }).format(d.rawDate);

        if (!monthlyMap.has(key)) {
          monthlyMap.set(key, {
            rawDate: new Date(Date.UTC(year, month, 1)),
            date: dateStr,
            fullDate: dateStr,
            usage: 0,
            included: 0,
            billed: 0,
            quota: 0,
            utilization: 0,
            isWeekend: false,
          });
        }

        const current = monthlyMap.get(key)!;
        current.usage += d.usage;
        current.included = (current.included || 0) + d.included;
        current.billed = (current.billed || 0) + d.billed;
      }

      const result = Array.from(monthlyMap.values()).sort(
        (a, b) => a.rawDate.getTime() - b.rawDate.getTime(),
      );

      // Calculate quota and utilization per month
      // If historical quota data is available (limit > 0), use it; otherwise fall back to current quota
      const currentQuota = usage?.userPremiumRequestEntitlement || 1200;

      // Pre-group rawData by YYYY-MM for O(n) lookup (avoids O(n²) filter in the loop below)
      const rawDataByMonth = new Map<string, typeof rawData>();
      for (const d of rawData) {
        const year = d.rawDate.getUTCFullYear();
        const month = d.rawDate.getUTCMonth();
        const key = `${year}-${String(month + 1).padStart(2, "0")}`;
        if (!rawDataByMonth.has(key)) {
          rawDataByMonth.set(key, []);
        }
        rawDataByMonth.get(key)!.push(d);
      }

      let previousMonthQuota: number | undefined;

      for (const r of result) {
        // O(1) lookup using the pre-grouped Map
        const mKey = `${r.rawDate.getUTCFullYear()}-${String(r.rawDate.getUTCMonth() + 1).padStart(2, "0")}`;
        const monthData = rawDataByMonth.get(mKey) || [];

        // Calculate quota based on historical data if available
        const historicalLimits = monthData
          .map((d) => d.limit)
          .filter((limit): limit is number => limit !== undefined && limit > 0);

        if (historicalLimits.length > 0) {
          // Use the maximum limit found in historical data for this month
          r.quota = Math.max(...historicalLimits);
        } else {
          // Fall back to current quota for months without historical limit data
          r.quota = currentQuota;
        }

        // A month is "estimated" if all of its days use estimated quota
        const nonEstimatedCount = monthData.filter(
          (d) => d.quotaEstimated === false,
        ).length;
        r.quotaEstimated = nonEstimatedCount === 0 && monthData.length > 0;

        // Detect a plan change: quota differs from the previous month's quota
        if (
          previousMonthQuota !== undefined &&
          previousMonthQuota > 0 &&
          r.quota > 0 &&
          previousMonthQuota !== r.quota
        ) {
          r.quotaChanged = true;
        }
        previousMonthQuota = r.quota;

        if (r.quota > 0) {
          r.utilization = (r.usage / r.quota) * 100;
        }
      }

      return result;
    } else {
      // yearly
      // Group by YYYY
      const yearlyMap = new Map<string, ChartDataPoint>();

      for (const d of rawData) {
        const year = d.rawDate.getUTCFullYear();
        const key = `${year}`;

        if (!yearlyMap.has(key)) {
          yearlyMap.set(key, {
            rawDate: new Date(Date.UTC(year, 0, 1)),
            date: key,
            fullDate: key,
            usage: 0,
            included: 0,
            billed: 0,
            quota: 0,
            utilization: 0,
            isWeekend: false,
          });
        }

        const current = yearlyMap.get(key)!;
        current.usage += d.usage;
        current.included = (current.included || 0) + d.included;
        current.billed = (current.billed || 0) + d.billed;
      }

      const result = Array.from(yearlyMap.values()).sort(
        (a, b) => a.rawDate.getTime() - b.rawDate.getTime(),
      );

      // Calculate quota and utilization per year
      // If historical quota data is available (limit > 0), use it; otherwise fall back to current quota
      const currentMonthlyQuota = usage?.userPremiumRequestEntitlement || 1200;
      const currentYearlyQuota = currentMonthlyQuota * 12;

      // Pre-group rawData by year for O(n) lookup (avoids O(n²) filter in the loop below)
      const rawDataByYear = new Map<number, typeof rawData>();
      for (const d of rawData) {
        const year = d.rawDate.getUTCFullYear();
        if (!rawDataByYear.has(year)) {
          rawDataByYear.set(year, []);
        }
        rawDataByYear.get(year)!.push(d);
      }

      let previousYearQuota: number | undefined;

      for (const r of result) {
        // O(1) lookup using the pre-grouped Map
        const yearData = rawDataByYear.get(r.rawDate.getUTCFullYear()) || [];

        // Calculate quota based on historical data if available
        const historicalLimits = yearData
          .map((d) => d.limit)
          .filter((limit): limit is number => limit !== undefined && limit > 0);

        if (historicalLimits.length > 0) {
          // Sum up the quota for each distinct calendar month in the year.
          // This correctly handles mid-year plan changes: if the user had
          // 1000 req/month for 6 months then upgraded to 2000/month, the
          // yearly quota is 6×1000 + 6×2000 = 18 000, not max(1000,2000)×12 = 24 000.
          const monthlyQuotaMap = new Map<number, number>();
          for (const d of yearData) {
            if (d.limit && d.limit > 0) {
              const monthKey = d.rawDate.getUTCMonth();
              if (!monthlyQuotaMap.has(monthKey)) {
                monthlyQuotaMap.set(monthKey, d.limit);
              }
            }
          }

          let yearlyQuota = 0;
          for (const quota of monthlyQuotaMap.values()) {
            yearlyQuota += quota;
          }

          // For months without historical data, assume current monthly quota.
          // Note: we use the current plan quota as the best available estimate —
          // GitHub's API does not expose historical quotas for past months that
          // were not recorded at fetch time, so this is an intentional approximation.
          const monthsWithData = monthlyQuotaMap.size;
          if (monthsWithData < 12) {
            yearlyQuota += (12 - monthsWithData) * currentMonthlyQuota;
          }

          r.quota = yearlyQuota;

          // Mark as a plan-change year when different monthly quotas appear
          const uniqueMonthlyQuotas = new Set(monthlyQuotaMap.values());
          if (uniqueMonthlyQuotas.size > 1) {
            r.quotaChanged = true;
          }
        } else {
          // Fall back to current yearly quota
          r.quota = currentYearlyQuota;
        }

        // Estimated if no day in this year has a confirmed quota
        const nonEstimatedCount = yearData.filter(
          (d) => d.quotaEstimated === false,
        ).length;
        r.quotaEstimated = nonEstimatedCount === 0 && yearData.length > 0;

        // Detect plan change across years
        if (
          previousYearQuota !== undefined &&
          previousYearQuota > 0 &&
          r.quota > 0 &&
          previousYearQuota !== r.quota
        ) {
          r.quotaChanged = true;
        }
        previousYearQuota = r.quota;

        if (r.quota > 0) {
          r.utilization = (r.usage / r.quota) * 100;
        }
      }

      return result;
    }
  }, [rawData, timeframe, usage]);

  const chartWidth =
    timeframe === "current_month"
      ? Math.max(700, aggregatedData.length * 56)
      : Math.max(700, aggregatedData.length * 80); // Wider bars for monthly/yearly

  const xAxisInterval =
    aggregatedData.length > 12 ? Math.ceil(aggregatedData.length / 12) - 1 : 0;

  useEffect(() => {
    const container = scrollContainerRef.current;
    if (!container) return;

    const scrollToLatest = () => {
      container.scrollLeft = Math.max(
        0,
        container.scrollWidth - container.clientWidth,
      );
    };

    requestAnimationFrame(() => {
      scrollToLatest();
      setTimeout(scrollToLatest, 60);
    });
  }, [aggregatedData.length, timeframe]);

  // Calculate max for Y axis
  const maxActualUsage =
    aggregatedData.length > 0
      ? Math.max(
          ...aggregatedData.map((d) => d.usage),
          ...aggregatedData.map((d) => d.trend || 0),
        )
      : 0;

  // Baseline budget logic is only really relevant for daily view ceiling
  const baselineBudget =
    timeframe === "current_month"
      ? (usage?.userPremiumRequestEntitlement || 0) / 30
      : 0;

  const yAxisMax = Math.max(
    1,
    Math.ceil(Math.max(maxActualUsage, baselineBudget) * 1.2),
  );

  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Usage Trend</CardTitle>
        </CardHeader>
        <CardContent>
          <Skeleton className="h-[300px] w-full" />
        </CardContent>
      </Card>
    );
  }

  if (historyDays.length === 0) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Usage Trend</CardTitle>
        </CardHeader>
        <CardContent className="h-[300px] flex items-center justify-center">
          <p className="text-muted-foreground">No usage history available</p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="overflow-hidden">
      <CardHeader className="pb-2 border-b bg-muted/20">
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
          <div className="flex items-center gap-2">
            <CardTitle className="text-lg">Usage Trend</CardTitle>
            <span className="text-sm font-normal text-muted-foreground hidden sm:inline-block">
              {aggregatedData.length}{" "}
              {timeframe === "current_month"
                ? "days"
                : timeframe === "monthly"
                  ? "months"
                  : "years"}
            </span>
          </div>

          <Tabs
            value={timeframe}
            onValueChange={(v) => {
              if (
                v === "current_month" ||
                v === "monthly" ||
                v === "yearly"
              ) {
                setTimeframe(v);
              }
            }}
            className="w-full sm:w-auto"
          >
            <TabsList className="grid w-full grid-cols-3 sm:w-[320px]">
              <TabsTrigger value="current_month">This Month</TabsTrigger>
              <TabsTrigger value="monthly">Monthly</TabsTrigger>
              <TabsTrigger value="yearly">Yearly</TabsTrigger>
            </TabsList>
          </Tabs>
        </div>
      </CardHeader>
      <CardContent className="pt-6">
        <div className="h-[300px] w-full flex">
          <div className="w-12 shrink-0 border-r border-border/40 pr-1">
            {timeframe === "current_month" ? (
              <LineChart
                width={48}
                height={300}
                data={aggregatedData}
                margin={{ top: 46, right: 0, left: 0, bottom: 20 }}
              >
                <YAxis
                  domain={[0, yAxisMax]}
                  tick={{ fontSize: 12 }}
                  tickLine={false}
                  axisLine={false}
                  className="text-muted-foreground"
                  width={44}
                  allowDataOverflow={true}
                />
              </LineChart>
            ) : (
              <BarChart
                width={48}
                height={300}
                data={aggregatedData}
                margin={{ top: 46, right: 0, left: 0, bottom: 20 }}
              >
                <YAxis
                  domain={[0, yAxisMax]}
                  tick={{ fontSize: 12 }}
                  tickLine={false}
                  axisLine={false}
                  className="text-muted-foreground"
                  width={44}
                  allowDataOverflow={true}
                />
              </BarChart>
            )}
          </div>

          <div
            ref={scrollContainerRef}
            className="h-[300px] w-full overflow-x-auto overflow-y-hidden"
          >
            <div style={{ width: chartWidth, height: "100%" }}>
              {/* CURRENT MONTH LINE CHART */}
              {timeframe === "current_month" && (
                <LineChart
                  width={chartWidth}
                  height={300}
                  data={aggregatedData}
                  margin={{ top: 10, right: 10, left: 0, bottom: 0 }}
                >
                  <CartesianGrid
                    strokeDasharray="3 3"
                    className="stroke-muted"
                    vertical={false}
                  />
                  <XAxis
                    dataKey="date"
                    tick={{ fontSize: 12 }}
                    tickLine={false}
                    axisLine={false}
                    className="text-muted-foreground"
                    interval={xAxisInterval}
                  />
                  <YAxis hide domain={[0, yAxisMax]} allowDataOverflow={true} />
                  <Tooltip
                    content={({ active, payload }) => {
                      if (active && payload && payload.length) {
                        const data = payload[0].payload as ChartDataPoint;
                        return (
                          <div className="rounded-lg border bg-background p-3 shadow-md space-y-2">
                            <div>
                              <p className="text-sm font-medium">
                                {data.fullDate}
                              </p>
                              <p className="text-xs text-muted-foreground">
                                {data.isWeekend ? "Weekend" : "Weekday"}
                              </p>
                            </div>
                            <div className="space-y-1">
                              <div className="flex items-center gap-2 text-sm">
                                <span className="h-2 w-2 rounded-full bg-primary" />
                                <span className="text-muted-foreground">
                                  Usage:
                                </span>
                                <span className="font-medium">
                                  {formatRequestCount(data.usage)}
                                </span>
                              </div>
                              {data.trend !== undefined && (
                                <div className="flex items-center gap-2 text-sm">
                                  <span className="h-2 w-2 rounded-full bg-muted-foreground/50" />
                                  <span className="text-muted-foreground">
                                    7-Day Avg:
                                  </span>
                                  <span className="font-medium">
                                    {data.trend.toFixed(1)}
                                  </span>
                                </div>
                              )}
                              {data.budget !== undefined && (
                                <div className="flex items-center gap-2 text-sm">
                                  <span className="h-2 w-2 rounded-full bg-orange-500/80" />
                                  <span className="text-muted-foreground">
                                    Budget:
                                  </span>
                                  <span className="font-medium">
                                    {data.budget.toFixed(1)} / day
                                  </span>
                                </div>
                              )}
                            </div>
                          </div>
                        );
                      }
                      return null;
                    }}
                  />
                  <Legend
                    verticalAlign="top"
                    height={36}
                    content={({ payload }) => (
                      <div className="flex justify-end gap-4 text-xs text-muted-foreground pb-2">
                        {payload?.map((entry, index) => {
                          let icon = (
                            <span className="h-2 w-2 rounded-full bg-primary mr-1.5" />
                          );
                          let label = "Daily Usage";
                          let tip = "Actual requests used per day";

                          if (entry.value === "Trend") {
                            icon = (
                              <span className="h-0 w-3 border-t-2 border-dashed border-muted-foreground mr-1.5" />
                            );
                            label = "7-Day Avg";
                            tip = "Rolling average over the last 7 days";
                          } else if (entry.value === "Budget") {
                            icon = (
                              <span className="h-0 w-3 border-t-2 border-orange-500/80 mr-1.5" />
                            );
                            label = "Daily Budget";
                            tip =
                              "Required daily pace to stay within monthly usage limit";
                          }

                          return (
                            <div
                              key={index}
                              className="flex items-center gap-1.5"
                              title={tip}
                            >
                              {icon}
                              <span>{label}</span>
                            </div>
                          );
                        })}
                      </div>
                    )}
                  />
                  <Line
                    type="monotone"
                    dataKey="usage"
                    name="Usage"
                    stroke="hsl(var(--primary))"
                    strokeWidth={2}
                    dot={{ fill: "hsl(var(--primary))", strokeWidth: 0, r: 4 }}
                    activeDot={{
                      fill: "hsl(var(--primary))",
                      strokeWidth: 0,
                      r: 6,
                    }}
                  />
                  <Line
                    type="monotone"
                    dataKey="budget"
                    stroke="#f97316"
                    strokeOpacity={0.8}
                    strokeWidth={2}
                    dot={false}
                    activeDot={false}
                    name="Budget"
                  />
                  <Line
                    type="monotone"
                    dataKey="trend"
                    stroke="hsl(var(--muted-foreground))"
                    strokeWidth={2}
                    strokeDasharray="5 5"
                    dot={false}
                    activeDot={false}
                    name="Trend"
                  />
                </LineChart>
              )}

              {/* MONTHLY / YEARLY STACKED BAR CHART */}
              {timeframe !== "current_month" && (
                <BarChart
                  width={chartWidth}
                  height={300}
                  data={aggregatedData}
                  margin={{ top: 10, right: 10, left: 0, bottom: 0 }}
                  barSize={40}
                >
                  <CartesianGrid
                    strokeDasharray="3 3"
                    className="stroke-muted"
                    vertical={false}
                  />
                  <XAxis
                    dataKey="date"
                    tick={{ fontSize: 12 }}
                    tickLine={false}
                    axisLine={false}
                    className="text-muted-foreground"
                    interval={xAxisInterval}
                  />
                  <YAxis hide domain={[0, yAxisMax]} allowDataOverflow={true} />
                  <Tooltip
                    cursor={{ fill: "var(--muted)", opacity: 0.2 }}
                    content={({ active, payload }) => {
                      if (active && payload && payload.length) {
                        const data = payload[0].payload as ChartDataPoint;
                        const billedCost =
                          (data.billed || 0) * COST_PER_REQUEST;

                        return (
                          <div className="rounded-lg border bg-background p-3 shadow-md space-y-2 min-w-[180px]">
                            <div>
                              <p className="text-sm font-medium">
                                {data.fullDate}
                              </p>
                              <p className="text-xs text-muted-foreground">
                                Total: {formatRequestCount(data.usage)} requests
                              </p>
                            </div>
                            <div className="space-y-1 pt-1 border-t">
                              <div className="flex items-center justify-between text-sm">
                                <div className="flex items-center gap-2">
                                  <span className="h-2 w-2 rounded-full bg-emerald-500" />
                                  <span className="text-muted-foreground">
                                    Included:
                                  </span>
                                </div>
                                <span className="font-medium">
                                  {formatRequestCount(data.included || 0)}
                                </span>
                              </div>
                              <div className="flex items-center justify-between text-sm">
                                <div className="flex items-center gap-2">
                                  <span className="h-2 w-2 rounded-full bg-red-500" />
                                  <span className="text-muted-foreground">
                                    Billed:
                                  </span>
                                </div>
                                <span className="font-medium">
                                  {formatRequestCount(data.billed || 0)}
                                </span>
                              </div>
                            </div>
                            {(data.billed || 0) > 0 && (
                              <div className="pt-1 mt-1 border-t flex items-center justify-between text-sm">
                                <span className="text-muted-foreground font-medium">
                                  Est. Cost:
                                </span>
                                <span className="font-bold text-red-500">
                                  ${billedCost.toFixed(2)}
                                </span>
                              </div>
                            )}
                            {data.utilization !== undefined && data.quota && (
                              <div className="pt-1 mt-1 border-t flex items-center justify-between text-sm">
                                <span className="text-muted-foreground font-medium">
                                  Quota Used:
                                </span>
                                <span
                                  className={`font-bold ${
                                    (data.utilization || 0) >= 90
                                      ? "text-red-500"
                                      : (data.utilization || 0) >= 75
                                        ? "text-orange-500"
                                        : "text-green-500"
                                  }`}
                                >
                                  {data.utilization.toFixed(1)}%
                                  {data.quotaEstimated && (
                                    <span
                                      className="ml-1 text-xs text-muted-foreground font-normal"
                                      title="Quota estimated using current plan — historical plan data not yet recorded for this period"
                                    >
                                      (~est.)
                                    </span>
                                  )}
                                </span>
                              </div>
                            )}
                            {data.quotaChanged && (
                              <div className="pt-1 mt-1 border-t flex items-center gap-1.5 text-xs text-amber-500">
                                <span>⚡</span>
                                <span>Plan quota changed this period</span>
                              </div>
                            )}
                          </div>
                        );
                      }
                      return null;
                    }}
                  />
                  <Legend
                    verticalAlign="top"
                    height={36}
                    content={() => (
                      <div className="flex justify-end gap-4 text-xs text-muted-foreground pb-2">
                        <div
                          className="flex items-center gap-1.5"
                          title="Requests within your included quota"
                        >
                          <span className="h-3 w-3 rounded-sm bg-emerald-500 mr-1.5" />
                          <span>Included Quota</span>
                        </div>
                        <div
                          className="flex items-center gap-1.5"
                          title="Requests billed as overage"
                        >
                          <span className="h-3 w-3 rounded-sm bg-red-500 mr-1.5" />
                          <span>Billed Overage</span>
                        </div>
                      </div>
                    )}
                  />
                  <Bar
                    dataKey="included"
                    name="Included"
                    stackId="a"
                    fill="#10b981"
                    radius={[0, 0, 4, 4]}
                  />
                  <Bar
                    dataKey="billed"
                    name="Billed"
                    stackId="a"
                    fill="#ef4444"
                    radius={[4, 4, 0, 0]}
                  />
                </BarChart>
              )}
            </div>

            {/* Quota estimation footnote — shown in monthly/yearly view when any
                period uses estimated quota or a plan change was detected */}
            {timeframe !== "current_month" &&
              (() => {
                const hasEstimated = aggregatedData.some(
                  (d) => d.quotaEstimated,
                );
                const hasPlanChange = aggregatedData.some(
                  (d) => d.quotaChanged,
                );
                if (!hasEstimated && !hasPlanChange) return null;
                return (
                  <div className="mt-2 text-xs text-muted-foreground space-y-0.5 px-1">
                    {hasEstimated && (
                      <p>
                        ~est. = utilization calculated using current plan quota.
                        Accuracy improves as monthly data is recorded over time.
                      </p>
                    )}
                    {hasPlanChange && (
                      <p className="text-amber-500/80">
                        ⚡ Plan quota change detected across periods.
                        Utilization percentages reflect each period&apos;s
                        recorded quota.
                      </p>
                    )}
                  </div>
                );
              })()}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
