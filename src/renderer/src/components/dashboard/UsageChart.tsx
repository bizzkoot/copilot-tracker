/**
 * UsageChart Component
 * Line chart showing daily usage trends
 */

import { Card, CardContent, CardHeader, CardTitle } from "../ui/card";
import { Skeleton } from "../ui/skeleton";
import { useEffect, useMemo, useRef } from "react";
import type { UsageHistory, CopilotUsage } from "@renderer/types/usage";
import {
  getTotalRequests,
  isWeekend,
  formatDate,
  formatRequestCount,
} from "@renderer/types/usage";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
} from "recharts";

interface UsageChartProps {
  history: UsageHistory | null;
  usage?: CopilotUsage | null;
  isLoading?: boolean;
}

interface ChartDataPoint {
  rawDate: Date;
  date: string;
  fullDate: string;
  usage: number;
  trend?: number;
  budget?: number;
  isWeekend: boolean;
}

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
        isWeekend: isWeekend(current),
      });

      current.setUTCDate(current.getUTCDate() + 1);
    }

    return paddedData;
  }, [historyDays]);

  // Calculate SMA trend and Dynamic Budget
  const chartData: ChartDataPoint[] = useMemo(() => {
    const usageValues = rawData.map((d) => d.usage);
    const trendValues = calculateSMA(usageValues, 7);

    let currentMonth = -1;
    let cumulativeUsage = 0;
    const totalQuota = usage?.userPremiumRequestEntitlement || 0;

    return rawData.map((d, i) => {
      const month = d.rawDate.getUTCMonth();

      // Reset cumulative sum on new month
      if (month !== currentMonth) {
        currentMonth = month;
        cumulativeUsage = 0;
      }

      const usageBeforeToday = cumulativeUsage;
      cumulativeUsage += d.usage;

      let dailyBudget: number | undefined = undefined;

      if (totalQuota > 0) {
        const year = d.rawDate.getUTCFullYear();
        const daysInMonth = new Date(Date.UTC(year, month + 1, 0)).getUTCDate();
        const currentDayOfMonth = d.rawDate.getUTCDate();

        // Target: spreading remaining quota over remaining days (including today)
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
  }, [rawData, usage]);

  const chartWidth = Math.max(700, chartData.length * 56);
  const xAxisInterval =
    chartData.length > 12 ? Math.ceil(chartData.length / 12) - 1 : 0;

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
  }, [chartData.length]);

  // Calculate max for Y axis
  // Calculate max of actual usage and 7-day trend
  const maxActualUsage =
    chartData.length > 0
      ? Math.max(
          ...chartData.map((d) => d.usage),
          ...chartData.map((d) => d.trend || 0),
        )
      : 0;

  // Provide a minimum ceiling based on a flat daily average
  const baselineBudget = (usage?.userPremiumRequestEntitlement || 0) / 30;

  // Cap the Y-axis to the max of actual usage or the baseline budget, ignoring extreme dynamic budget spikes
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
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-lg flex items-center justify-between">
          <span>Usage Trend</span>
          <span className="text-sm font-normal text-muted-foreground">
            {chartData.length} days
          </span>
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="h-[300px] w-full flex">
          <div className="w-12 shrink-0 border-r border-border/40 pr-1">
            <LineChart
              width={48}
              height={300}
              data={chartData}
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
          </div>

          <div
            ref={scrollContainerRef}
            className="h-[300px] w-full overflow-x-auto overflow-y-hidden"
          >
            <div style={{ width: chartWidth, height: "100%" }}>
              <LineChart
                width={chartWidth}
                height={300}
                data={chartData}
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
                          >
                            <div className="flex items-center" title={tip}>
                              {icon}
                              <span>{label}</span>
                            </div>
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
                  dot={{
                    fill: "hsl(var(--primary))",
                    strokeWidth: 0,
                    r: 4,
                  }}
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
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
