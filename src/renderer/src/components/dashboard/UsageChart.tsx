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
  ReferenceLine,
} from "recharts";

interface UsageChartProps {
  history: UsageHistory | null;
  isLoading?: boolean;
  dailyAverage?: number;
}

interface ChartDataPoint {
  date: string;
  fullDate: string;
  usage: number;
  isWeekend: boolean;
}

export function UsageChart({
  history,
  isLoading,
  dailyAverage,
}: UsageChartProps) {
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
  const chartData: ChartDataPoint[] = [...history.days]
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

  // Calculate max for Y axis
  const maxUsage = Math.max(
    ...chartData.map((d) => d.usage),
    dailyAverage || 0,
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
                      <div className="rounded-lg border bg-background p-2 shadow-md">
                        <p className="text-sm font-medium">{data.fullDate}</p>
                        <p className="text-sm text-muted-foreground">
                          {data.usage.toLocaleString()} requests
                          {data.isWeekend && " (weekend)"}
                        </p>
                      </div>
                    );
                  }
                  return null;
                }}
              />
              {dailyAverage && (
                <ReferenceLine
                  y={dailyAverage}
                  stroke="hsl(var(--muted-foreground))"
                  strokeDasharray="5 5"
                  label={{
                    value: "Avg",
                    position: "right",
                    fill: "hsl(var(--muted-foreground))",
                    fontSize: 12,
                  }}
                />
              )}
              <Line
                type="monotone"
                dataKey="usage"
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
            </LineChart>
          </ResponsiveContainer>
        </div>
      </CardContent>
    </Card>
  );
}
