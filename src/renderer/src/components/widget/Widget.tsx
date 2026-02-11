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
import { invoke, listen, emit } from "@renderer/types/tauri";
import { getCurrentWindow } from "@tauri-apps/api/window";

export function Widget() {
  const { usage, prediction } = useUsageStore();
  const [isPinned, setIsPinned] = useState(true);
  const [isDragging, setIsDragging] = useState(false);

  // Calculate usage values
  const used = usage ? getUsedRequests(usage) : 0;
  const limit = usage ? getLimitRequests(usage) : 0;
  const percentage = usage ? getUsagePercentage(usage) : 0;

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

  // Minimize widget
  const minimizeWidget = async () => {
    try {
      await emit("widget:minimize");
      const currentWindow = getCurrentWindow();
      await currentWindow.hide();
    } catch (error) {
      console.error("Failed to minimize widget:", error);
    }
  };

  // Close widget
  const closeWidget = async () => {
    try {
      await emit("widget:close");
      const currentWindow = getCurrentWindow();
      await currentWindow.hide();
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
        const position = await invoke<{ x: number; y: number }>(
          "get_widget_position",
        );
        await currentWindow.setPosition({
          x: position.x,
          y: position.y,
        });

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
