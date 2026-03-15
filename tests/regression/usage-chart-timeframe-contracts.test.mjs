import test from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

const chartPath = path.resolve(
  "src/renderer/src/components/dashboard/UsageChart.tsx",
);
const usageRsPath = path.resolve("src-tauri/src/usage.rs");

const chartSource = fs.readFileSync(chartPath, "utf8");
const usageRsSource = fs.readFileSync(usageRsPath, "utf8");

// ─── UsageChart: Timeframe type ─────────────────────────────────────────────

test("UsageChart defines Timeframe type with all three values", () => {
  assert.match(
    chartSource,
    /type Timeframe = "current_month" \| "monthly" \| "yearly"/,
    "UsageChart must define Timeframe type with current_month, monthly, and yearly",
  );
});

// ─── UsageChart: Default timeframe is current_month ─────────────────────────

test("UsageChart initializes timeframe state to current_month", () => {
  assert.match(
    chartSource,
    /useState<Timeframe>\("current_month"\)/,
    "UsageChart must initialize timeframe state to 'current_month'",
  );
});

// ─── UsageChart: COST_PER_REQUEST import ────────────────────────────────────

test("UsageChart imports COST_PER_REQUEST from types/usage", () => {
  assert.match(
    chartSource,
    /COST_PER_REQUEST/,
    "UsageChart must import COST_PER_REQUEST from usage types",
  );
});

// ─── UsageChart: BarChart and Bar imports from recharts ─────────────────────

test("UsageChart imports BarChart and Bar from recharts for aggregated views", () => {
  assert.match(
    chartSource,
    /BarChart/,
    "UsageChart must import BarChart from recharts for monthly/yearly views",
  );
  assert.match(
    chartSource,
    /import \{[^}]*Bar[^}]*\} from "recharts"/,
    "UsageChart must import Bar from recharts",
  );
});

// ─── UsageChart: Tabs component imports ─────────────────────────────────────

test("UsageChart imports Tabs, TabsList, TabsTrigger for timeframe switching", () => {
  assert.match(
    chartSource,
    /import \{ Tabs, TabsList, TabsTrigger \} from/,
    "UsageChart must import Tabs components for timeframe selection UI",
  );
});

// ─── UsageChart: ChartDataPoint has new optional fields ─────────────────────

test("ChartDataPoint interface includes included and billed optional fields", () => {
  assert.match(
    chartSource,
    /included\?: number/,
    "ChartDataPoint must have optional included field for stacked data",
  );
  assert.match(
    chartSource,
    /billed\?: number/,
    "ChartDataPoint must have optional billed field for stacked data",
  );
});

test("ChartDataPoint interface includes quota and utilization optional fields", () => {
  assert.match(
    chartSource,
    /quota\?: number/,
    "ChartDataPoint must have optional quota field for monthly/yearly views",
  );
  assert.match(
    chartSource,
    /utilization\?: number/,
    "ChartDataPoint must have optional utilization field for quota percentage",
  );
});

// ─── UsageChart: aggregatedData useMemo depends on timeframe ────────────────

test("aggregatedData memoization includes timeframe in its dependency array", () => {
  assert.match(
    chartSource,
    /\}, \[rawData, timeframe, usage\]\)/,
    "aggregatedData useMemo must include timeframe in its dependency array",
  );
});

// ─── UsageChart: Tabs UI uses timeframe state ───────────────────────────────

test("UsageChart renders a Tabs component bound to timeframe state", () => {
  assert.match(
    chartSource,
    /value=\{timeframe\}/,
    "Tabs component must be bound to timeframe state",
  );
  assert.match(
    chartSource,
    /onValueChange=\{/,
    "Tabs must have onValueChange handler to update timeframe",
  );
});

test("UsageChart renders TabsTrigger for each timeframe option", () => {
  assert.match(
    chartSource,
    /value="current_month"/,
    "UsageChart must render TabsTrigger for current_month",
  );
  assert.match(
    chartSource,
    /value="monthly"/,
    "UsageChart must render TabsTrigger for monthly",
  );
  assert.match(
    chartSource,
    /value="yearly"/,
    "UsageChart must render TabsTrigger for yearly",
  );
});

// ─── UsageChart: current_month aggregation uses current UTC month ────────────

test("current_month aggregation filters by UTC year and month", () => {
  assert.match(
    chartSource,
    /getUTCFullYear\(\) === currentYear/,
    "current_month filter must compare UTC year to current year",
  );
  assert.match(
    chartSource,
    /getUTCMonth\(\) === currentMonth/,
    "current_month filter must compare UTC month to current month",
  );
});

// ─── UsageChart: monthly aggregation groups by YYYY-MM ──────────────────────

test("monthly aggregation uses YYYY-MM key for grouping", () => {
  assert.match(
    chartSource,
    /`\$\{year\}-\$\{String\(month \+ 1\)\.padStart\(2, "0"\)\}`/,
    "monthly aggregation must use YYYY-MM key format",
  );
});

// ─── UsageChart: yearly aggregation groups by year ──────────────────────────

test("yearly aggregation groups data by calendar year", () => {
  assert.match(
    chartSource,
    /const yearlyMap = new Map</,
    "yearly aggregation must use a Map to group by year",
  );
  assert.match(
    chartSource,
    /rawDate\.getUTCFullYear\(\)/,
    "yearly aggregation must use getUTCFullYear for consistent grouping",
  );
});

// ─── UsageChart: baselineBudget is only applied for current_month ────────────

test("baselineBudget is restricted to current_month timeframe only", () => {
  assert.match(
    chartSource,
    /timeframe === "current_month"\s*\?\s*\(usage\?\.userPremiumRequestEntitlement \|\| 0\) \/ 30\s*:\s*0/,
    "baselineBudget must be non-zero only for current_month timeframe",
  );
});

// ─── UsageChart: scroll reset when timeframe changes ────────────────────────

test("useEffect for scroll-to-latest includes timeframe in dependency array", () => {
  assert.match(
    chartSource,
    /\}, \[aggregatedData\.length, timeframe\]\)/,
    "scroll-to-latest effect must re-run when timeframe changes",
  );
});

// ─── usage.rs: wall-clock polling replaces tokio interval ───────────────────

test("usage.rs start_polling uses wall-clock time with chrono::Utc::now()", () => {
  assert.match(
    usageRsSource,
    /let mut last_tick = chrono::Utc::now\(\)/,
    "start_polling must use chrono::Utc::now() for wall-clock time tracking",
  );
});

test("usage.rs start_polling uses 1-second sleep loop for responsiveness", () => {
  assert.match(
    usageRsSource,
    /tokio::time::sleep\(tokio::time::Duration::from_secs\(1\)\)/,
    "start_polling must use 1-second sleep for responsive cancellation",
  );
});

test("usage.rs start_polling checks elapsed time against interval_duration", () => {
  assert.match(
    usageRsSource,
    /if elapsed >= interval_duration/,
    "start_polling must compare elapsed wall-clock time against interval_duration",
  );
});

test("usage.rs start_polling updates last_tick after each actual polling tick", () => {
  assert.match(
    usageRsSource,
    /last_tick = now/,
    "start_polling must update last_tick after each poll to avoid consecutive triggers",
  );
});

test("usage.rs start_polling doc comment mentions wall-clock time and sleep/hibernation", () => {
  assert.match(
    usageRsSource,
    /wall-clock time/,
    "start_polling must be documented as using wall-clock time",
  );
});

test("usage.rs start_polling no longer uses tokio::time::interval", () => {
  assert.doesNotMatch(
    usageRsSource,
    /tokio::time::interval\(/,
    "start_polling must not use tokio::time::interval (replaced by wall-clock approach)",
  );
});

test("usage.rs no longer imports tokio::time::Duration directly", () => {
  assert.doesNotMatch(
    usageRsSource,
    /use tokio::time::Duration/,
    "usage.rs must not have a bare tokio::time::Duration import (removed in this PR)",
  );
});