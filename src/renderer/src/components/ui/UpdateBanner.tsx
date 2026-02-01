import { useEffect, useState } from "react";
import { Download, X } from "lucide-react";
import { Button } from "./button";
import { Card } from "./card";
import { UpdateInfo } from "@renderer/types/electron";

export function UpdateBanner() {
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [isVisible, setIsVisible] = useState(false);

  useEffect(() => {
    // Listen for update available events
    const cleanup = window.electron.onUpdateAvailable((info) => {
      console.log("Update available:", info);
      setUpdateInfo(info);
      setIsVisible(true);
    });
    return cleanup;
  }, []);

  const handleDownload = () => {
    if (!updateInfo) return;

    // Open the specific release tag URL
    // If version doesn't start with 'v', prepend it
    const version = updateInfo.version.startsWith("v")
      ? updateInfo.version
      : `v${updateInfo.version}`;

    const url = `https://github.com/bizzkoot/copilot-tracker/releases/tag/${version}`;
    window.electron.openExternal(url);
  };

  const handleDismiss = () => {
    setIsVisible(false);
  };

  if (!isVisible || !updateInfo) return null;

  return (
    <div className="fixed bottom-4 right-4 z-50 w-full max-w-sm animate-in slide-in-from-bottom-5 fade-in duration-300">
      <Card className="border-primary/50 shadow-lg bg-card/95 backdrop-blur supports-[backdrop-filter]:bg-card/80">
        <div className="p-4 flex items-start gap-4">
          <div className="bg-primary/10 p-2 rounded-full text-primary">
            <Download className="h-5 w-5" />
          </div>

          <div className="flex-1 space-y-1">
            <h4 className="text-sm font-semibold">Update Available</h4>
            <p className="text-xs text-muted-foreground">
              Version {updateInfo.version} is now available.
            </p>

            <div className="pt-2 flex gap-2">
              <Button
                size="sm"
                onClick={handleDownload}
                className="h-8 text-xs"
              >
                Download
              </Button>
              <Button
                variant="ghost"
                size="sm"
                onClick={handleDismiss}
                className="h-8 text-xs"
              >
                Later
              </Button>
            </div>
          </div>

          <button
            onClick={handleDismiss}
            className="text-muted-foreground hover:text-foreground transition-colors"
          >
            <X className="h-4 w-4" />
          </button>
        </div>
      </Card>
    </div>
  );
}
