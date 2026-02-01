/**
 * HistoryTable Component
 * Table showing daily usage breakdown
 */

import { Card, CardContent, CardHeader, CardTitle } from "../ui/card";
import { Skeleton } from "../ui/skeleton";
import type { UsageHistory, DailyUsage } from "@renderer/types/usage";
import {
  getTotalRequests,
  isWeekend,
  formatCurrency,
} from "@renderer/types/usage";

interface HistoryTableProps {
  history: UsageHistory | null;
  isLoading?: boolean;
}

function formatDateLong(date: Date | string): string {
  const dateObj = typeof date === "string" ? new Date(date) : date;
  return new Intl.DateTimeFormat("en-US", {
    weekday: "short",
    month: "short",
    day: "numeric",
  }).format(dateObj);
}

function TableRow({ day }: { day: DailyUsage }) {
  const total = getTotalRequests(day);
  const weekend = isWeekend(day.date);

  return (
    <tr className="border-b border-border hover:bg-muted/50 transition-colors">
      <td className="py-3 px-4">
        <div className="flex items-center gap-2">
          <span>{formatDateLong(day.date)}</span>
          {weekend && (
            <span className="text-xs px-1.5 py-0.5 rounded bg-muted text-muted-foreground">
              Weekend
            </span>
          )}
        </div>
      </td>
      <td className="py-3 px-4 text-right font-medium">
        {total.toLocaleString()}
      </td>
      <td className="py-3 px-4 text-right">
        {day.includedRequests.toLocaleString()}
      </td>
      <td className="py-3 px-4 text-right">
        {day.billedRequests > 0 ? (
          <span className="text-orange-500">
            {day.billedRequests.toLocaleString()}
          </span>
        ) : (
          <span className="text-muted-foreground">0</span>
        )}
      </td>
      <td className="py-3 px-4 text-right">
        {day.billedAmount > 0 ? (
          <span className="text-orange-500">
            {formatCurrency(day.billedAmount)}
          </span>
        ) : (
          <span className="text-muted-foreground">$0.00</span>
        )}
      </td>
    </tr>
  );
}

export function HistoryTable({ history, isLoading }: HistoryTableProps) {
  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Daily Breakdown</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-2">
            {Array.from({ length: 5 }).map((_, i) => (
              <Skeleton key={i} className="h-12 w-full" />
            ))}
          </div>
        </CardContent>
      </Card>
    );
  }

  if (!history || history.days.length === 0) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Daily Breakdown</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-muted-foreground">No history data available</p>
        </CardContent>
      </Card>
    );
  }

  // Calculate totals
  const totals = history.days.reduce(
    (acc, day) => ({
      total: acc.total + getTotalRequests(day),
      included: acc.included + day.includedRequests,
      billed: acc.billed + day.billedRequests,
      amount: acc.amount + day.billedAmount,
    }),
    { total: 0, included: 0, billed: 0, amount: 0 },
  );

  return (
    <Card>
      <CardHeader className="pb-2">
        <CardTitle className="text-lg flex items-center justify-between">
          <span>Daily Breakdown</span>
          <span className="text-sm font-normal text-muted-foreground">
            {history.days.length} days
          </span>
        </CardTitle>
      </CardHeader>
      <CardContent className="p-0">
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-border bg-muted/50">
                <th className="py-3 px-4 text-left font-medium">Date</th>
                <th className="py-3 px-4 text-right font-medium">Total</th>
                <th className="py-3 px-4 text-right font-medium">Included</th>
                <th className="py-3 px-4 text-right font-medium">Billed</th>
                <th className="py-3 px-4 text-right font-medium">Cost</th>
              </tr>
            </thead>
            <tbody>
              {history.days.slice(0, 10).map((day, index) => (
                <TableRow key={index} day={day} />
              ))}
            </tbody>
            <tfoot>
              <tr className="border-t-2 border-border bg-muted/30 font-medium">
                <td className="py-3 px-4">Total</td>
                <td className="py-3 px-4 text-right">
                  {totals.total.toLocaleString()}
                </td>
                <td className="py-3 px-4 text-right">
                  {totals.included.toLocaleString()}
                </td>
                <td className="py-3 px-4 text-right">
                  {totals.billed > 0 ? (
                    <span className="text-orange-500">
                      {totals.billed.toLocaleString()}
                    </span>
                  ) : (
                    "0"
                  )}
                </td>
                <td className="py-3 px-4 text-right">
                  {totals.amount > 0 ? (
                    <span className="text-orange-500">
                      {formatCurrency(totals.amount)}
                    </span>
                  ) : (
                    "$0.00"
                  )}
                </td>
              </tr>
            </tfoot>
          </table>
        </div>
      </CardContent>
    </Card>
  );
}
