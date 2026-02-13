/**
 * Floating Usage Widget Component
 * A draggable, always-on-top widget showing Copilot usage data
 */

import { useEffect, useState } from "react";
import { useUsageStore } from "@renderer/stores/usageStore";
import { useTrayIconFormat } from "@renderer/stores/settingsStore";
import { useSettingsSync } from "@renderer/hooks/useSettingsSync";
import {
  getUsedRequests,
  getLimitRequests,
  getUsagePercentage,
} from "@renderer/types/usage";
import {
  TRAY_FORMAT_REMAINING_TOTAL,
  TRAY_FORMAT_REMAINING_PERCENT,
  TRAY_FORMAT_REMAINING_COMBINED,
} from "@renderer/types/settings";
import { WidgetHeader } from "./WidgetHeader";
import { listen, emit } from "@renderer/types/tauri";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke as tauriInvoke } from "@tauri-apps/api/core";

// Helper to determine if we should show remaining values
const isRemainingFormat = (format: string): boolean => {
  return (
    format === TRAY_FORMAT_REMAINING_TOTAL ||
    format === TRAY_FORMAT_REMAINING_PERCENT ||
    format === TRAY_FORMAT_REMAINING_COMBINED
  );
};

export function Widget() {
  // Sync settings from main process to keep widget in sync with dashboard/settings
  useSettingsSync();

  const { usage, prediction, lastUpdated, setUsageData } = useUsageStore();
  const [isPinned, setIsPinned] = useState(true);
  const [isDragging, setIsDragging] = useState(false);
  const trayIconFormat = useTrayIconFormat();

  // Calculate usage values
  const used = usage ? getUsedRequests(usage) : 0;
  const limit = usage ? getLimitRequests(usage) : 0;
  const percentage = usage ? getUsagePercentage(usage) : 0;

  // Determine if we should show remaining values based on tray icon format
  const showRemaining = isRemainingFormat(trayIconFormat);

  // Calculate display value based on tray icon format
  const displayValue = showRemaining ? limit - used : used;
  const displayPercentage = showRemaining ? 100 - percentage : percentage;

  // Fetch cached usage data when widget mounts
  // This ensures the widget has data even if it missed the initial usage:data event
  useEffect(() => {
    const fetchInitialUsage = async () => {
      try {
        console.log("[Widget] Fetching cached usage data on mount...");
        const result = await window.electron.getCachedUsage();

        if (result && result.success) {
          console.log("[Widget] Received cached usage data:", result);
          setUsageData({
            usage: result.usage,
            history: result.history,
            prediction: result.prediction,
          });
        } else {
          console.log("[Widget] No cached usage data available");
        }
      } catch (error) {
        console.error("[Widget] Failed to fetch cached usage:", error);
      }
    };

    fetchInitialUsage();
  }, [setUsageData]);

  // Listen for usage data updates from backend
  useEffect(() => {
    const unsubscribe = window.electron.onUsageData?.((data) => {
      console.log("[Widget] Received usage:data event:", data);

      if (data.success) {
        setUsageData({
          usage: data.usage,
          history: data.history,
          prediction: data.prediction,
        });
      }
    });

    return () => {
      unsubscribe?.();
    };
  }, [setUsageData]);

  // Get color based on percentage - refined palette with better contrast
  const getProgressColor = () => {
    if (percentage <= 50) return "#10b981"; // emerald-500
    if (percentage <= 80) return "#f59e0b"; // amber-500
    if (percentage <= 100) return "#f97316"; // orange-500
    return "#ef4444"; // red-500
  };

  const progressColor = getProgressColor();

  // Get prediction status with refined styling
  const getPredictionStatus = () => {
    if (!prediction || !limit) return null;
    const predicted = prediction.predictedMonthlyRequests;
    if (predicted > limit) {
      return {
        text: `Forecast: ${predicted.toLocaleString()} (exceeds limit)`,
        color: "#fb923c", // orange-400
      };
    }
    return {
      text: `Forecast: ${predicted.toLocaleString()} (on track)`,
      color: "#4ade80", // green-400
    };
  };

  const predictionStatus = getPredictionStatus();

  // Toggle pin mode
  const togglePin = async () => {
    const newPinned = !isPinned;
    setIsPinned(newPinned);

    try {
      const currentWindow = getCurrentWindow();
      await currentWindow.setAlwaysOnTop(newPinned);

      // Emit event to update settings
      await emit("widget:pin-changed", { pinned: newPinned });
    } catch (error) {
      console.error("Failed to toggle pin mode:", error);
    }
  };

  // Minimize widget - calls backend to properly update state and tray menu
  const minimizeWidget = async () => {
    try {
      // Call the backend command that updates store and rebuilds tray menu
      await tauriInvoke("minimize_widget");
    } catch (error) {
      console.error("Failed to minimize widget:", error);
    }
  };

  // Close widget - calls backend to properly update state and tray menu
  const closeWidget = async () => {
    try {
      // Call the backend command that updates store and rebuilds tray menu
      await tauriInvoke("hide_widget");
    } catch (error) {
      console.error("Failed to close widget:", error);
    }
  };

  // Listen for pin state changes from other windows
  useEffect(() => {
    const unlisten = listen<boolean>("widget:set-pin", (event) => {
      setIsPinned(event.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  // Set initial pin state (position is already restored by backend on startup)
  useEffect(() => {
    const initWidgetState = async () => {
      try {
        const currentWindow = getCurrentWindow();

        // Position is already restored by the backend before the widget is shown
        // We only need to set the initial pin state here
        await currentWindow.setAlwaysOnTop(true);
      } catch (error) {
        console.error("Failed to initialize widget state:", error);
      }
    };

    initWidgetState();
  }, []);

  return (
    <div
      className={`widget-window h-full w-full flex flex-col rounded-xl border border-white/10 bg-[#1e1e1e]/85 shadow-[0_8px_32px_rgba(0,0,0,0.3)] backdrop-blur-[12px] ${
        isDragging ? "cursor-grabbing" : ""
      }`}
    >
      {/* Header */}
      <WidgetHeader
        isPinned={isPinned}
        onTogglePin={togglePin}
        onMinimize={minimizeWidget}
        onClose={closeWidget}
        onDragStart={() => setIsDragging(true)}
        onDragEnd={() => setIsDragging(false)}
      />

      {/* Content - Side-by-side layout (reduced height) */}
      <div className="flex-1 px-4 py-2.5 flex gap-4">
        {/* Left Panel: Stacked text */}
        <div className="flex-1 flex flex-col justify-center gap-1.5 min-w-0">
          {/* Usage Text - refined typography */}
          <div className="text-sm text-white leading-tight">
            <span className="font-semibold tracking-tight">
              {displayValue.toLocaleString()}
            </span>
            <span className="text-white/50 font-normal mx-1">/</span>
            <span className="text-white/70 font-normal">
              {limit.toLocaleString()} requests
            </span>
          </div>

          {/* Prediction - refined without emoji */}
          {predictionStatus && (
            <div className="text-xs leading-tight">
              <span style={{ color: predictionStatus.color, fontWeight: 500 }}>
                {predictionStatus.text}
              </span>
            </div>
          )}

          {/* Confidence - refined with colored dot */}
          {prediction && (
            <div className="text-xs text-white/60 leading-tight flex items-center gap-1.5">
              <span
                className="w-1.5 h-1.5 rounded-full flex-shrink-0"
                style={{
                  backgroundColor:
                    prediction.confidenceLevel === "high"
                      ? "#10b981"
                      : prediction.confidenceLevel === "medium"
                        ? "#f59e0b"
                        : "#ef4444",
                }}
              />
              <span className="truncate">
                Based on {prediction.daysUsedForPrediction} day(s) of data
              </span>
            </div>
          )}

          {/* Last Updated - refined typography */}
          {lastUpdated && (
            <div className="text-xs text-white/50 leading-tight font-normal">
              Updated{" "}
              {lastUpdated.toLocaleTimeString([], {
                hour: "2-digit",
                minute: "2-digit",
              })}
            </div>
          )}
        </div>

        {/* Right Panel: Circular Gauge */}
        <div className="flex items-center justify-center flex-shrink-0">
          <div className="relative w-14 h-14">
            {/* Background Circle */}
            <svg className="w-full h-full" viewBox="0 0 36 36">
              {/* Background track */}
              <path
                d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
                fill="none"
                stroke="rgba(255, 255, 255, 0.12)"
                strokeWidth="3"
                strokeLinecap="round"
              />
              {/* Progress arc */}
              <path
                d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831"
                fill="none"
                stroke={progressColor}
                strokeWidth="3"
                strokeLinecap="round"
                strokeDasharray={`${Math.min(displayPercentage, 100)}, 100`}
                style={{
                  transition: "stroke-dasharray 500ms ease-out",
                  filter: `drop-shadow(0 0 4px ${progressColor}40)`,
                }}
              />
            </svg>
            {/* Percentage text in center */}
            <div className="absolute inset-0 flex items-center justify-center">
              <span className="text-sm font-bold text-white leading-none">
                {Math.round(displayPercentage)}%
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
