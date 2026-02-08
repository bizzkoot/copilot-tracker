import { app } from "electron";
import * as fs from "fs";
import * as path from "path";

// Check if running in development mode
export const isDevelopment = !app.isPackaged;

// Load configuration from environment variables with defaults
export const config = {
  appId: process.env.APP_ID || "com.electron.copilot-tracker",
  productName: process.env.PRODUCT_NAME || "copilot-tracker",
  githubBillingUrl:
    process.env.GITHUB_BILLING_URL ||
    "https://github.com/settings/billing/premium_requests_usage",
  githubLoginUrl: process.env.GITHUB_LOGIN_URL || "https://github.com/login",
  costPerRequest: parseFloat(process.env.COST_PER_REQUEST || "0.04"),
} as const;

// Debug log file path (in home directory for easy access)
const DEBUG_LOG_FILE = path.join(
  process.env.HOME || "~",
  "copilot-tracker-debug.log",
);

// Write to debug log file
function writeToDebugLog(level: string, args: unknown[]): void {
  const timestamp = new Date().toISOString();
  const message = args
    .map((arg) => {
      if (typeof arg === "object") {
        return JSON.stringify(arg);
      }
      return String(arg);
    })
    .join(" ");

  const logLine = `[${timestamp}] [${level}] ${message}\n`;

  try {
    fs.appendFileSync(DEBUG_LOG_FILE, logLine);
  } catch (e) {
    // If file writing fails, fall back to console
    console.error("[DebugLog] Failed to write to file:", e);
  }
}

// Development logger - logs to console in dev, and always writes to file
export const devLog = {
  log: (...args: unknown[]) => {
    writeToDebugLog("LOG", args);
    if (isDevelopment) {
      console.log("[Dev]", ...args);
    }
  },
  info: (...args: unknown[]) => {
    writeToDebugLog("INFO", args);
    if (isDevelopment) {
      console.info("[Dev]", ...args);
    }
  },
  warn: (...args: unknown[]) => {
    writeToDebugLog("WARN", args);
    if (isDevelopment) {
      console.warn("[Dev]", ...args);
    }
  },
  error: (...args: unknown[]) => {
    writeToDebugLog("ERROR", args);
    if (isDevelopment) {
      console.error("[Dev]", ...args);
    }
  },
};
