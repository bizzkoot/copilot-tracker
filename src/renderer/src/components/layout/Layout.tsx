/**
 * Layout Component
 * Main layout wrapper with header and content area
 */

import { useState } from "react";
import { Header } from "./Header";
import { Dashboard } from "../dashboard/Dashboard";
import { Settings } from "../settings/Settings";
import { LoginPrompt } from "../auth/LoginPrompt";
import { useAuth } from "@renderer/hooks/useAuth";

type View = "dashboard" | "settings";

export function Layout() {
  const [currentView, setCurrentView] = useState<View>("dashboard");
  const { needsLogin, isLoading } = useAuth();

  const handleSettingsClick = () => {
    setCurrentView(currentView === "settings" ? "dashboard" : "settings");
  };

  // Show login prompt if not authenticated
  if (needsLogin) {
    return <LoginPrompt />;
  }

  // Show loading state
  if (isLoading) {
    return (
      <div className="min-h-screen bg-background flex items-center justify-center">
        <div className="text-center">
          <div className="w-12 h-12 border-4 border-primary border-t-transparent rounded-full animate-spin mx-auto" />
          <p className="mt-4 text-muted-foreground">
            Checking authentication...
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background flex flex-col">
      {/* macOS traffic lights safe area - only on macOS */}
      <div
        className="h-12 flex-shrink-0 -webkit-app-region-drag"
        style={
          {
            WebkitAppRegion: "drag",
          } as React.CSSProperties
        }
      />
      <Header onSettingsClick={handleSettingsClick} />
      <main className="flex-1 container mx-auto px-4 py-6">
        {currentView === "dashboard" ? (
          <Dashboard />
        ) : (
          <Settings onClose={() => setCurrentView("dashboard")} />
        )}
      </main>
    </div>
  );
}
