/**
 * Widget Header Component
 * Draggable header with pin, minimize, and close buttons
 */

import { useState, useRef } from "react";
import { invoke, getCurrentWindow } from "@renderer/types/tauri";

interface WidgetHeaderProps {
  isPinned: boolean;
  onTogglePin: () => void;
  onMinimize: () => void;
  onClose: () => void;
  onDragStart: () => void;
  onDragEnd: () => void;
}

export function WidgetHeader({
  isPinned,
  onTogglePin,
  onMinimize,
  onClose,
  onDragStart,
  onDragEnd,
}: WidgetHeaderProps) {
  const [isDragging, setIsDragging] = useState(false);
  const isDraggingRef = useRef(false); // Stable ref for event handlers
  const dragStartPos = useRef({ x: 0, y: 0 });

  const handleMouseDown = (e: React.MouseEvent) => {
    // Only start drag if clicking on the header area (not buttons)
    if ((e.target as HTMLElement).closest("button")) return;

    setIsDragging(true);
    isDraggingRef.current = true;
    dragStartPos.current = { x: e.clientX, y: e.clientY };
    onDragStart();

    const handleMouseMove = async (moveEvent: MouseEvent) => {
      if (!isDraggingRef.current) return; // Use ref for stable check

      const deltaX = moveEvent.clientX - dragStartPos.current.x;
      const deltaY = moveEvent.clientY - dragStartPos.current.y;

      // Use Tauri window API to move the window
      const currentWindow = getCurrentWindow();
      if (currentWindow) {
        try {
          const pos = await currentWindow.getPosition();
          if (!isDraggingRef.current) return; // Check again after await
          await currentWindow.setPosition({
            x: pos.x + deltaX,
            y: pos.y + deltaY,
          });
        } catch (error) {
          console.error("Failed to move widget:", error);
        }
      }

      dragStartPos.current = { x: moveEvent.clientX, y: moveEvent.clientY };
    };

    const handleMouseUp = async () => {
      isDraggingRef.current = false;
      setIsDragging(false);
      onDragEnd();

      // Save widget position to settings
      try {
        const currentWindow = getCurrentWindow();
        const pos = await currentWindow.getPosition();
        await invoke("set_widget_position", { x: pos.x, y: pos.y });
      } catch (error) {
        console.error("Failed to save widget position:", error);
      }

      document.removeEventListener("mousemove", handleMouseMove);
      document.removeEventListener("mouseup", handleMouseUp);
    };

    document.addEventListener("mousemove", handleMouseMove);
    document.addEventListener("mouseup", handleMouseUp);
  };

  return (
    <div
      className={`flex items-center justify-between px-3 py-2 cursor-grab select-none ${
        isDragging ? "cursor-grabbing" : ""
      }`}
      style={{
        background: "rgba(255, 255, 255, 0.05)",
        borderTopLeftRadius: "12px",
        borderTopRightRadius: "12px",
        borderBottom: "1px solid rgba(255, 255, 255, 0.1)",
      }}
      onMouseDown={handleMouseDown}
    >
      {/* Title */}
      <div className="flex items-center gap-2">
        <span className="text-white text-sm font-medium">Copilot Usage</span>
      </div>

      {/* Buttons */}
      <div className="flex items-center gap-1">
        {/* Pin Button */}
        <button
          onClick={onTogglePin}
          className="w-6 h-6 flex items-center justify-center rounded hover:bg-white/10 transition-colors"
          title={isPinned ? "Always on top" : "Desktop only"}
        >
          {isPinned ? (
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="currentColor"
              className="text-white"
            >
              <path d="M16 12V4H17V2H7V4H8V12L6 14V16H11.2V22H12.8V16H18V14L16 12Z" />
            </svg>
          ) : (
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
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
          className="w-6 h-6 flex items-center justify-center rounded hover:bg-white/10 transition-colors"
          title="Minimize to tray"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
            className="text-white"
          >
            <line x1="5" y1="12" x2="19" y2="12" />
          </svg>
        </button>

        {/* Close Button */}
        <button
          onClick={onClose}
          className="w-6 h-6 flex items-center justify-center rounded hover:bg-red-500/20 hover:text-red-400 transition-colors"
          title="Close widget"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
            className="text-white"
          >
            <line x1="18" y1="6" x2="6" y2="18" />
            <line x1="6" y1="6" x2="18" y2="18" />
          </svg>
        </button>
      </div>
    </div>
  );
}
