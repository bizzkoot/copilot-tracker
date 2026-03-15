/**
 * UsageChart Component
 * Line chart showing daily usage trends
 */

import { Card, CardContent, CardHeader, CardTitle } from "../ui/card";
import { Skeleton } from "../ui/skeleton";
import { useEffect, useMemo, useRef, useState } from "react";
import type { UsageHistory, CopilotUsage } from "@renderer/types/usage";
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
      // Group by YYYY-MM
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

      // Use current monthly quota for all months
      const monthlyQuota = usage?.userPremiumRequestEntitlement || 1200;

      const result = Array.from(monthlyMap.values()).sort(
        (a, b) => a.rawDate.getTime() - b.rawDate.getTime(),
      );

      // Calculate utilization percentage
      for (const r of result) {
        r.quota = monthlyQuota;
        if (monthlyQuota > 0) {
          r.utilization = (r.usage / monthlyQuota) * 100;
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

      // Use current monthly quota * 12 for yearly view
      const monthlyQuota = usage?.userPremiumRequestEntitlement || 1200;
      const yearlyQuota = monthlyQuota * 12;

      const result = Array.from(yearlyMap.values()).sort(
        (a, b) => a.rawDate.getTime() - b.rawDate.getTime(),
      );

      // Calculate utilization percentage
      for (const r of result) {
        r.quota = yearlyQuota;
        if (yearlyQuota > 0) {
          r.utilization = (r.usage / yearlyQuota) * 100;
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
            onValueChange={(v) => setTimeframe(v as Timeframe)}
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
                                </span>
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
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
