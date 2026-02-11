/**
 * Floating Usage Widget Component
 * A draggable, always-on-top widget showing Copilot usage data
 */

import { useEffect, useState } from "react";
import { useUsageStore } from "@renderer/stores/usageStore";
import {
  getUsedRequests,
  getLimitRequests,
  getUsagePercentage,
} from "@renderer/types/usage";
import { WidgetHeader } from "./WidgetHeader";
import { listen, emit } from "@renderer/types/tauri";
import { getCurrentWindow, PhysicalPosition } from "@tauri-apps/api/window";
import { invoke as tauriInvoke } from "@tauri-apps/api/core";

export function Widget() {
  const { usage, prediction, setUsageData } = useUsageStore();
  const [isPinned, setIsPinned] = useState(true);
  const [isDragging, setIsDragging] = useState(false);

  // Calculate usage values
  const used = usage ? getUsedRequests(usage) : 0;
  const limit = usage ? getLimitRequests(usage) : 0;
  const percentage = usage ? getUsagePercentage(usage) : 0;

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

  // Get color class based on percentage
  const getColorClass = () => {
    if (percentage <= 50) return "bg-green-500";
    if (percentage <= 80) return "bg-yellow-500";
    if (percentage <= 100) return "bg-orange-500";
    return "bg-red-500";
  };

  const progressColor = getColorClass();

  // Get prediction status
  const getPredictionStatus = () => {
    if (!prediction || !limit) return null;
    const predicted = prediction.predictedMonthlyRequests;
    if (predicted > limit) {
      return {
        text: `Predicted: ${predicted.toLocaleString()} (may exceed)`,
        emoji: "⚠️",
        color: "text-orange-400",
      };
    }
    return {
      text: `Predicted: ${predicted.toLocaleString()} (on track)`,
      emoji: "✅",
      color: "text-green-400",
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

  // Set initial pin state and restore widget position
  useEffect(() => {
    const initWidgetState = async () => {
      try {
        const currentWindow = getCurrentWindow();

        // Restore widget position from settings
        const position = await tauriInvoke<{ x: number; y: number }>(
          "get_widget_position",
        );
        await currentWindow.setPosition(
          new PhysicalPosition({
            x: position.x,
            y: position.y,
          }),
        );

        // Default to pinned
        await currentWindow.setAlwaysOnTop(true);
      } catch (error) {
        console.error("Failed to initialize widget state:", error);
      }
    };

    initWidgetState();
  }, []);

  return (
    <div
      className={`widget-window h-full w-full flex flex-col ${
        isDragging ? "cursor-grabbing" : ""
      }`}
      style={{
        background: "rgba(30, 30, 30, 0.85)",
        backdropFilter: "blur(12px)",
        border: "1px solid rgba(255, 255, 255, 0.1)",
        borderRadius: "12px",
        boxShadow: "0 8px 32px rgba(0, 0, 0, 0.3)",
      }}
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

      {/* Content */}
      <div className="flex-1 px-4 py-3 flex flex-col gap-2">
        {/* Progress Bar */}
        <div className="flex items-center gap-2">
          {/* Custom progress bar with dynamic color */}
          <div
            className="flex-1 h-2 rounded-full overflow-hidden"
            style={{ backgroundColor: "rgba(255, 255, 255, 0.1)" }}
          >
            <div
              className="h-full rounded-full transition-all duration-300"
              style={{
                width: `${Math.min(percentage, 100)}%`,
                backgroundColor:
                  progressColor === "bg-green-500"
                    ? "#22c55e"
                    : progressColor === "bg-yellow-500"
                      ? "#eab308"
                      : progressColor === "bg-orange-500"
                        ? "#f97316"
                        : "#ef4444",
              }}
            />
          </div>
          <span className="text-xs font-medium text-white min-w-[35px] text-right">
            {Math.round(percentage)}%
          </span>
        </div>

        {/* Usage Text */}
        <div className="text-sm text-white">
          <span className="font-medium">{used.toLocaleString()}</span>
          <span className="text-muted-foreground"> / </span>
          <span className="text-muted-foreground">
            {limit.toLocaleString()} requests
          </span>
        </div>

        {/* Prediction */}
        {predictionStatus && (
          <div className="text-xs text-muted-foreground flex items-start gap-1">
            <span>{predictionStatus.emoji}</span>
            <span className={predictionStatus.color}>
              {predictionStatus.text}
            </span>
          </div>
        )}

        {/* Confidence */}
        {prediction && (
          <div className="text-xs text-muted-foreground flex items-center gap-1">
            <span>
              {prediction.confidenceLevel === "high" && "🟢"}
              {prediction.confidenceLevel === "medium" && "🟡"}
              {prediction.confidenceLevel === "low" && "🔴"}
            </span>
            <span>
              Based on {prediction.daysUsedForPrediction} day(s) of data
            </span>
          </div>
        )}
      </div>
    </div>
  );
}
