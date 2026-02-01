/**
 * LoginPrompt Component
 * Shown when user needs to authenticate with GitHub
 */

import { Button } from "../ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "../ui/card";
import { useAuth } from "@renderer/hooks/useAuth";
import { Github, Loader2 } from "lucide-react";

export function LoginPrompt() {
  const { login, isLoading, hasError } = useAuth();

  return (
    <div className="min-h-screen bg-background flex items-center justify-center p-4">
      <Card className="w-full max-w-md">
        <CardHeader className="text-center">
          <div className="w-16 h-16 rounded-full bg-primary/10 flex items-center justify-center mx-auto mb-4">
            <svg
              viewBox="0 0 24 24"
              fill="none"
              className="w-8 h-8 text-primary"
              stroke="currentColor"
              strokeWidth="2"
            >
              <path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5" />
            </svg>
          </div>
          <CardTitle className="text-2xl">Copilot Tracker</CardTitle>
          <CardDescription>
            Monitor your GitHub Copilot premium request usage
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <p className="text-sm text-muted-foreground text-center">
            Sign in with your GitHub account to track your Copilot usage, view
            predictions, and get notified before exceeding your quota.
          </p>

          {hasError && (
            <div className="p-3 rounded-lg bg-destructive/10 border border-destructive/20 text-center">
              <p className="text-sm text-destructive">
                Authentication failed. Please try again.
              </p>
            </div>
          )}

          <Button
            className="w-full"
            size="lg"
            onClick={login}
            disabled={isLoading}
          >
            {isLoading ? (
              <>
                <Loader2 className="mr-2 h-5 w-5 animate-spin" />
                Connecting...
              </>
            ) : (
              <>
                <Github className="mr-2 h-5 w-5" />
                Login with GitHub
              </>
            )}
          </Button>

          <p className="text-xs text-muted-foreground text-center">
            This app uses secure GitHub OAuth. Your credentials are never
            stored.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
