/**
 * UsageChart Component
 * Line chart showing daily usage trends
 */

import { Card, CardContent, CardHeader, CardTitle } from "../ui/card";
import { Skeleton } from "../ui/skeleton";
import type { UsageHistory } from "@renderer/types/usage";
import { getTotalRequests, isWeekend, formatDate } from "@renderer/types/usage";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
} from "recharts";

interface UsageChartProps {
  history: UsageHistory | null;
  isLoading?: boolean;
}

interface ChartDataPoint {
  date: string;
  fullDate: string;
  usage: number;
  trend?: number;
  isWeekend: boolean;
}

/**
 * Calculate Exponential Moving Average (EMA)
 * Alpha = smoothing factor (0.25 = ~4 day emphasis)
 */
function calculateEMA(data: number[], alpha: number = 0.25): number[] {
  if (data.length === 0) return [];

  const ema: number[] = [data[0]]; // Start with first value

  for (let i = 1; i < data.length; i++) {
    ema.push(alpha * data[i] + (1 - alpha) * ema[i - 1]);
  }

  return ema;
}

export function UsageChart({ history, isLoading }: UsageChartProps) {
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

  if (!history || history.days.length === 0) {
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

  // Transform data for chart (reverse to show oldest first)
  const rawData = [...history.days]
    .reverse()
    .slice(-14) // Last 14 days
    .map((day) => {
      const dateObj =
        typeof day.date === "string" ? new Date(day.date) : day.date;
      return {
        date: formatDate(day.date),
        fullDate: dateObj.toLocaleDateString(),
        usage: getTotalRequests(day),
        isWeekend: isWeekend(day.date),
      };
    });

  // Calculate EMA trend
  const usageValues = rawData.map((d) => d.usage);
  const trendValues = calculateEMA(usageValues);

  // Combine data with trend
  const chartData: ChartDataPoint[] = rawData.map((d, i) => ({
    ...d,
    trend: trendValues[i],
  }));

  // Calculate max for Y axis
  const maxUsage = Math.max(
    ...chartData.map((d) => d.usage),
    ...chartData.map((d) => d.trend || 0),
  );
  const yAxisMax = Math.ceil(maxUsage * 1.2);

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-lg flex items-center justify-between">
          <span>Usage Trend</span>
          <span className="text-sm font-normal text-muted-foreground">
            Last {chartData.length} days
          </span>
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="h-[300px] w-full">
          <ResponsiveContainer width="100%" height="100%">
            <LineChart
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
              />
              <YAxis
                domain={[0, yAxisMax]}
                tick={{ fontSize: 12 }}
                tickLine={false}
                axisLine={false}
                className="text-muted-foreground"
                width={40}
              />
              <Tooltip
                content={({ active, payload }) => {
                  if (active && payload && payload.length) {
                    const data = payload[0].payload as ChartDataPoint;
                    return (
                      <div className="rounded-lg border bg-background p-3 shadow-md space-y-2">
                        <div>
                          <p className="text-sm font-medium">{data.fullDate}</p>
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
                              {data.usage.toLocaleString()}
                            </span>
                          </div>
                          {data.trend !== undefined && (
                            <div className="flex items-center gap-2 text-sm">
                              <span className="h-2 w-2 rounded-full bg-muted-foreground/50" />
                              <span className="text-muted-foreground">
                                Trend:
                              </span>
                              <span className="font-medium">
                                {data.trend.toFixed(1)}
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
                    {payload?.map((entry, index) => (
                      <div key={index} className="flex items-center gap-1.5">
                        {entry.value === "Usage" ? (
                          <div className="flex items-center">
                            <span className="h-2 w-2 rounded-full bg-primary mr-1.5" />
                            <span>Daily Usage</span>
                          </div>
                        ) : (
                          <div
                            className="flex items-center"
                            title="Exponential Moving Average (smoothed trend)"
                          >
                            <span className="h-0 w-3 border-t-2 border-dashed border-muted-foreground mr-1.5" />
                            <span>Trend (EMA)</span>
                          </div>
                        )}
                      </div>
                    ))}
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
                dataKey="trend"
                stroke="hsl(var(--muted-foreground))"
                strokeWidth={2}
                strokeDasharray="5 5"
                dot={false}
                activeDot={false}
                name="Trend"
              />
            </LineChart>
          </ResponsiveContainer>
        </div>
      </CardContent>
    </Card>
  );
}
