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

import { useState } from "react";
import { ChevronDown, ChevronRight } from "lucide-react";

// ... existing imports ...

interface TableRowProps {
  day: DailyUsage;
  isExpanded: boolean;
  onToggle: () => void;
}

function TableRow({ day, isExpanded, onToggle }: TableRowProps) {
  const total = getTotalRequests(day);
  const weekend = isWeekend(day.date);
  const hasModels = day.models && day.models.length > 0;

  return (
    <>
      <tr
        className={`border-b border-border hover:bg-muted/50 transition-colors cursor-pointer ${isExpanded ? "bg-muted/30" : ""}`}
        onClick={onToggle}
      >
        <td className="py-3 px-4">
          <div className="flex items-center gap-2">
            {hasModels ? (
              isExpanded ? (
                <ChevronDown className="h-4 w-4 text-muted-foreground" />
              ) : (
                <ChevronRight className="h-4 w-4 text-muted-foreground" />
              )
            ) : (
              <span className="w-4" />
            )}
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
      {isExpanded && hasModels && (
        <tr className="bg-muted/20">
          <td colSpan={5} className="p-0">
            <div className="px-4 py-2 pl-12">
              <table className="w-full text-sm">
                <thead>
                  <tr className="text-xs text-muted-foreground border-b border-border/50">
                    <th className="py-2 text-left font-medium">Model</th>
                    <th className="py-2 text-right font-medium">Included</th>
                    <th className="py-2 text-right font-medium">Billed</th>
                    <th className="py-2 text-right font-medium">Cost</th>
                  </tr>
                </thead>
                <tbody>
                  {day.models!.map((model, idx) => (
                    <tr
                      key={idx}
                      className="border-b border-border/20 last:border-0 hover:bg-muted/30"
                    >
                      <td className="py-2 text-muted-foreground">
                        {model.name}
                      </td>
                      <td className="py-2 text-right text-muted-foreground">
                        {model.includedRequests}
                      </td>
                      <td className="py-2 text-right text-muted-foreground">
                        {model.billedRequests}
                      </td>
                      <td className="py-2 text-right text-muted-foreground">
                        {formatCurrency(model.billedAmount)}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </td>
        </tr>
      )}
    </>
  );
}

export function HistoryTable({ history, isLoading }: HistoryTableProps) {
  const [expandedRows, setExpandedRows] = useState<Set<number>>(new Set());
  const [isTotalExpanded, setIsTotalExpanded] = useState(false);

  const toggleRow = (index: number) => {
    const newExpanded = new Set(expandedRows);
    if (newExpanded.has(index)) {
      newExpanded.delete(index);
    } else {
      newExpanded.add(index);
    }
    setExpandedRows(newExpanded);
  };

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

  // Calculate model totals
  const modelTotals = history.days.reduce(
    (acc, day) => {
      day.models?.forEach((model) => {
        if (!acc[model.name]) {
          acc[model.name] = {
            name: model.name,
            includedRequests: 0,
            billedRequests: 0,
            billedAmount: 0,
          };
        }
        acc[model.name].includedRequests += model.includedRequests;
        acc[model.name].billedRequests += model.billedRequests;
        acc[model.name].billedAmount += model.billedAmount;
      });
      return acc;
    },
    {} as Record<
      string,
      {
        name: string;
        includedRequests: number;
        billedRequests: number;
        billedAmount: number;
      }
    >,
  );

  const sortedModelTotals = Object.values(modelTotals).sort(
    (a, b) => b.billedAmount - a.billedAmount,
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
                <TableRow
                  key={index}
                  day={day}
                  isExpanded={expandedRows.has(index)}
                  onToggle={() => toggleRow(index)}
                />
              ))}
            </tbody>
            <tfoot>
              <tr
                className={`border-t-2 border-border font-medium cursor-pointer hover:bg-muted/50 transition-colors ${isTotalExpanded ? "bg-muted/30" : "bg-muted/30"}`}
                onClick={() => setIsTotalExpanded(!isTotalExpanded)}
              >
                <td className="py-3 px-4 flex items-center gap-2">
                  {sortedModelTotals.length > 0 ? (
                    isTotalExpanded ? (
                      <ChevronDown className="h-4 w-4 text-muted-foreground" />
                    ) : (
                      <ChevronRight className="h-4 w-4 text-muted-foreground" />
                    )
                  ) : (
                    <span className="w-4" />
                  )}
                  <span>Total</span>
                </td>
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
              {isTotalExpanded && sortedModelTotals.length > 0 && (
                <tr className="bg-muted/20">
                  <td colSpan={5} className="p-0">
                    <div className="px-4 py-2 pl-12">
                      <table className="w-full text-sm">
                        <thead>
                          <tr className="text-xs text-muted-foreground border-b border-border/50">
                            <th className="py-2 text-left font-medium">
                              Model
                            </th>
                            <th className="py-2 text-right font-medium">
                              Included
                            </th>
                            <th className="py-2 text-right font-medium">
                              Billed
                            </th>
                            <th className="py-2 text-right font-medium">
                              Cost
                            </th>
                          </tr>
                        </thead>
                        <tbody>
                          {sortedModelTotals.map((model, idx) => (
                            <tr
                              key={idx}
                              className="border-b border-border/20 last:border-0 hover:bg-muted/30"
                            >
                              <td className="py-2 text-muted-foreground">
                                {model.name}
                              </td>
                              <td className="py-2 text-right text-muted-foreground">
                                {model.includedRequests.toLocaleString()}
                              </td>
                              <td className="py-2 text-right text-muted-foreground">
                                {model.billedRequests.toLocaleString()}
                              </td>
                              <td className="py-2 text-right text-muted-foreground">
                                {formatCurrency(model.billedAmount)}
                              </td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  </td>
                </tr>
              )}
            </tfoot>
          </table>
        </div>
      </CardContent>
    </Card>
  );
}
