/**
 * Layout Component
 * Main layout wrapper with header and content area
 */

import { useState, useEffect } from "react";
import { Header } from "./Header";
import { Dashboard } from "../dashboard/Dashboard";
import { Settings } from "../settings/Settings";
import { LoginPrompt } from "../auth/LoginPrompt";
import { UpdateBanner } from "../ui/UpdateBanner";
import { DashboardSkeleton } from "../dashboard/DashboardSkeleton";
import { useAuth } from "@renderer/hooks/useAuth";

type View = "dashboard" | "settings";

export function Layout() {
  const [currentView, setCurrentView] = useState<View>("dashboard");
  const { needsLogin, isLoading } = useAuth();

  useEffect(() => {
    const cleanup = window.electron.onNavigate((route: string) => {
      if (route === "settings") {
        setCurrentView("settings");
      } else if (route === "dashboard") {
        setCurrentView("dashboard");
      }
    });
    return cleanup;
  }, []);

  const handleSettingsClick = () => {
    setCurrentView(currentView === "settings" ? "dashboard" : "settings");
  };

  // Show login prompt if not authenticated
  if (needsLogin) {
    return <LoginPrompt />;
  }

  // Show skeleton loading state while determining auth/initial data
  if (isLoading) {
    return (
      <div className="min-h-screen bg-background flex flex-col">
        <Header onSettingsClick={handleSettingsClick} />
        <main className="flex-1 container mx-auto px-4 py-4">
          <DashboardSkeleton />
        </main>
        <UpdateBanner />
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-background flex flex-col">
      <Header onSettingsClick={handleSettingsClick} />
      <main className="flex-1 container mx-auto px-4 py-4">
        {currentView === "dashboard" ? (
          <Dashboard />
        ) : (
          <Settings onClose={() => setCurrentView("dashboard")} />
        )}
      </main>
      <UpdateBanner />
    </div>
  );
}
