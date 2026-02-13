/**
 * Widget Header Component
 * Draggable header with pin, minimize, and close buttons
 *
 * Uses Tauri's data-tauri-drag-region for native window dragging
 * This is the recommended approach for Tauri v2 frameless windows
 */

import { useEffect, useRef } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke as tauriInvoke } from "@tauri-apps/api/core";

interface WidgetHeaderProps {
  isPinned: boolean;
  showRemaining: boolean;
  onTogglePin: () => void;
  onMinimize: () => void;
  onClose: () => void;
  onDragStart: () => void;
  onDragEnd: () => void;
}

export function WidgetHeader({
  isPinned,
  showRemaining,
  onTogglePin,
  onMinimize,
  onClose,
  onDragStart,
  onDragEnd,
}: WidgetHeaderProps) {
  const headerRef = useRef<HTMLDivElement>(null);
  const isDraggingRef = useRef(false);

  useEffect(() => {
    const header = headerRef.current;
    if (!header) return;

    // Handle drag start
    const handleMouseDown = (e: MouseEvent) => {
      // Don't start drag if clicking on buttons
      if ((e.target as HTMLElement).closest("button")) return;

      isDraggingRef.current = true;
      onDragStart();
    };

    // Handle drag end and save position
    const handleMouseUp = async () => {
      if (!isDraggingRef.current) return;

      isDraggingRef.current = false;
      onDragEnd();

      // Save widget position to settings after drag completes
      try {
        const currentWindow = getCurrentWindow();
        const pos = await currentWindow.outerPosition();
        await tauriInvoke("set_widget_position", {
          x: pos.x,
          y: pos.y,
        });
      } catch (error) {
        console.error("Failed to save widget position:", error);
      }
    };

    header.addEventListener("mousedown", handleMouseDown);
    document.addEventListener("mouseup", handleMouseUp);

    return () => {
      header.removeEventListener("mousedown", handleMouseDown);
      document.removeEventListener("mouseup", handleMouseUp);
    };
  }, [onDragStart, onDragEnd]);

  return (
    <div
      ref={headerRef}
      data-tauri-drag-region
      className="flex items-center justify-between px-3 py-1.5 cursor-grab select-none active:cursor-grabbing"
      style={{
        background: "rgba(255, 255, 255, 0.06)",
        borderTopLeftRadius: "12px",
        borderTopRightRadius: "12px",
        borderBottom: "1px solid rgba(255, 255, 255, 0.08)",
      }}
    >
      {/* Title */}
      <div className="flex items-center gap-2" data-tauri-drag-region>
        <span
          className="text-white text-sm font-semibold tracking-tight"
          data-tauri-drag-region
        >
          Copilot {showRemaining ? "(Remaining)" : "(Consumed)"}
        </span>
      </div>

      {/* Buttons - refined hover states */}
      <div className="flex items-center gap-0.5">
        {/* Pin Button */}
        <button
          onClick={onTogglePin}
          className="w-7 h-7 flex items-center justify-center rounded hover:bg-white/12 active:bg-white/18 transition-all duration-150"
          title={isPinned ? "Always on top" : "Desktop only"}
        >
          {isPinned ? (
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="13"
              height="13"
              viewBox="0 0 24 24"
              fill="currentColor"
              className="text-white"
            >
              <path d="M16 12V4H17V2H7V4H8V12L6 14V16H11.2V22H12.8V16H18V14L16 12Z" />
            </svg>
          ) : (
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="13"
              height="13"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2.5"
              strokeLinecap="round"
              strokeLinejoin="round"
              className="text-white"
            >
              <path d="M16 12V4H17V2H7V4H8V12L6 14V16H11.2V22H12.8V16H18V14L16 12Z" />
            </svg>
          )}
        </button>

        {/* Minimize Button */}
        <button
          onClick={onMinimize}
          className="w-7 h-7 flex items-center justify-center rounded hover:bg-white/12 active:bg-white/18 transition-all duration-150"
          title="Minimize to tray"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="13"
            height="13"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2.5"
            strokeLinecap="round"
            strokeLinejoin="round"
            className="text-white"
          >
            <line x1="5" y1="12" x2="19" y2="12" />
          </svg>
        </button>

        {/* Close Button - refined hover */}
        <button
          onClick={onClose}
          className="w-7 h-7 flex items-center justify-center rounded hover:bg-red-500/25 active:bg-red-500/35 transition-all duration-150"
          title="Close widget"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="13"
            height="13"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2.5"
            strokeLinecap="round"
            strokeLinejoin="round"
            className="text-white/90 hover:text-red-400 transition-colors"
          >
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>
    </div>
  );
}
