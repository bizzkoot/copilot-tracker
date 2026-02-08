/**
 * Header Component
 * App header with title, theme toggle, and settings
 */

import { Button } from "../ui/button";
import { useTheme } from "@renderer/hooks/useTheme";
import { Sun, Moon, Settings, LogOut } from "lucide-react";
import { useAuth } from "@renderer/hooks/useAuth";

interface HeaderProps {
  onSettingsClick?: () => void;
}

export function Header({ onSettingsClick }: HeaderProps) {
  const { toggleTheme, isDark } = useTheme();
  const { logout, isAuthenticated } = useAuth();

  return (
    <header
      className="border-b border-border bg-card"
      style={{ WebkitAppRegion: "drag" } as React.CSSProperties}
    >
      <div className="container mx-auto px-4 py-3 flex items-center justify-between">
        {/* Logo and Title */}
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg overflow-hidden flex items-center justify-center">
            <img
              src="./icon.png"
              alt="Copilot Tracker"
              className="w-full h-full object-contain"
            />
          </div>
          <div>
            <h1 className="text-lg font-semibold">Copilot Tracker</h1>
            <p className="text-xs text-muted-foreground">Monitor your usage</p>
          </div>
        </div>

        {/* Actions */}
        <div
          className="flex items-center gap-2"
          style={{ WebkitAppRegion: "no-drag" } as React.CSSProperties}
        >
          {/* Theme Toggle */}
          <Button
            variant="ghost"
            size="icon"
            onClick={toggleTheme}
            title={isDark ? "Switch to light mode" : "Switch to dark mode"}
          >
            {isDark ? (
              <Sun className="h-5 w-5" />
            ) : (
              <Moon className="h-5 w-5" />
            )}
          </Button>

          {/* Settings */}
          <Button
            variant="ghost"
            size="icon"
            onClick={onSettingsClick}
            title="Settings"
          >
            <Settings className="h-5 w-5" />
          </Button>

          {/* Logout */}
          {isAuthenticated && (
            <Button variant="ghost" size="icon" onClick={logout} title="Logout">
              <LogOut className="h-5 w-5" />
            </Button>
          )}
        </div>
      </div>
    </header>
  );
}
