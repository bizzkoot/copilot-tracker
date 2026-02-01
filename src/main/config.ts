import { app } from "electron";

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

// Development logger - only logs in development
export const devLog = {
  log: (...args: unknown[]) => {
    if (isDevelopment) {
      console.log("[Dev]", ...args);
    }
  },
  error: (...args: unknown[]) => {
    if (isDevelopment) {
      console.error("[Dev]", ...args);
    }
  },
};
