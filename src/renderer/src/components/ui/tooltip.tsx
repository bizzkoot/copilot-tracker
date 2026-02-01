import * as React from "react";
import { cn } from "@renderer/lib/utils";

interface TooltipProps {
  children: React.ReactNode;
  content: React.ReactNode;
  side?: "top" | "right" | "bottom" | "left";
  className?: string;
}

const Tooltip: React.FC<TooltipProps> = ({
  children,
  content,
  side = "top",
  className,
}) => {
  const [isVisible, setIsVisible] = React.useState(false);

  const positionClasses = {
    top: "bottom-full left-1/2 -translate-x-1/2 mb-2",
    bottom: "top-full left-1/2 -translate-x-1/2 mt-2",
    left: "right-full top-1/2 -translate-y-1/2 mr-2",
    right: "left-full top-1/2 -translate-y-1/2 ml-2",
  };

  return (
    <div
      className="relative inline-block"
      onMouseEnter={() => setIsVisible(true)}
      onMouseLeave={() => setIsVisible(false)}
    >
      {children}
      {isVisible && (
        <div
          className={cn(
            "absolute z-50 px-3 py-1.5 text-sm rounded-md bg-popover text-popover-foreground shadow-md border animate-in fade-in-0 zoom-in-95",
            positionClasses[side],
            className,
          )}
        >
          {content}
        </div>
      )}
    </div>
  );
};

export { Tooltip };
