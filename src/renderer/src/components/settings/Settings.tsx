/**
 * Settings Component
 * User preferences panel
 */

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../ui/card'
import { Button } from '../ui/button'
import { useSettingsStore } from '@renderer/stores/settingsStore'
import { useAuth } from '@renderer/hooks/useAuth'
import {
  REFRESH_INTERVAL_OPTIONS,
  PREDICTION_PERIOD_OPTIONS,
  THEME_OPTIONS
} from '@renderer/types/settings'
import { ArrowLeft, RotateCcw } from 'lucide-react'

interface SettingsProps {
  onClose: () => void
}

export function Settings({ onClose }: SettingsProps) {
  const { login, isAuthenticated } = useAuth()
  const {
    refreshInterval,
    predictionPeriod,
    theme,
    launchAtLogin,
    notifications,
    setRefreshInterval,
    setPredictionPeriod,
    setTheme,
    setLaunchAtLogin,
    setNotificationsEnabled,
    setNotificationThresholds,
    resetSettings
  } = useSettingsStore()

  const handleThresholdToggle = (threshold: number) => {
    const newThresholds = notifications.thresholds.includes(threshold)
      ? notifications.thresholds.filter((t) => t !== threshold)
      : [...notifications.thresholds, threshold].sort((a, b) => a - b)
    setNotificationThresholds(newThresholds)
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <Button variant="ghost" size="icon" onClick={onClose}>
            <ArrowLeft className="h-5 w-5" />
          </Button>
          <div>
            <h2 className="text-2xl font-semibold">Settings</h2>
            <p className="text-sm text-muted-foreground">Configure your preferences</p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <Button variant="outline" size="sm" onClick={login}>
            {isAuthenticated ? 'Re-Login' : 'Login'}
          </Button>
          <Button variant="outline" size="sm" onClick={resetSettings}>
            <RotateCcw className="h-4 w-4 mr-2" />
            Reset
          </Button>
        </div>
      </div>

      {/* Refresh Interval */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Refresh Interval</CardTitle>
          <CardDescription>How often to fetch usage data from GitHub</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap gap-2">
            {REFRESH_INTERVAL_OPTIONS.map((option) => (
              <Button
                key={option.value}
                variant={refreshInterval === option.value ? 'default' : 'outline'}
                size="sm"
                onClick={() => setRefreshInterval(option.value as typeof refreshInterval)}
              >
                {option.label}
              </Button>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Prediction Period */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Prediction Period</CardTitle>
          <CardDescription>Days of history used for monthly prediction</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap gap-2">
            {PREDICTION_PERIOD_OPTIONS.map((option) => (
              <Button
                key={option.value}
                variant={predictionPeriod === option.value ? 'default' : 'outline'}
                size="sm"
                onClick={() => setPredictionPeriod(option.value as typeof predictionPeriod)}
              >
                {option.label}
              </Button>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Theme */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Theme</CardTitle>
          <CardDescription>Choose your preferred color scheme</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap gap-2">
            {THEME_OPTIONS.map((option) => (
              <Button
                key={option.value}
                variant={theme === option.value ? 'default' : 'outline'}
                size="sm"
                onClick={() => setTheme(option.value as typeof theme)}
              >
                {option.label}
              </Button>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* Notifications */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Notifications</CardTitle>
          <CardDescription>Get alerts when approaching usage limits</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <span className="text-sm">Enable notifications</span>
            <Button
              variant={notifications.enabled ? 'default' : 'outline'}
              size="sm"
              onClick={() => setNotificationsEnabled(!notifications.enabled)}
            >
              {notifications.enabled ? 'Enabled' : 'Disabled'}
            </Button>
          </div>

          {notifications.enabled && (
            <div className="space-y-2">
              <span className="text-sm text-muted-foreground">Alert at these thresholds:</span>
              <div className="flex flex-wrap gap-2">
                {[50, 75, 90, 100].map((threshold) => (
                  <Button
                    key={threshold}
                    variant={notifications.thresholds.includes(threshold) ? 'default' : 'outline'}
                    size="sm"
                    onClick={() => handleThresholdToggle(threshold)}
                  >
                    {threshold}%
                  </Button>
                ))}
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Launch at Login */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Startup</CardTitle>
          <CardDescription>Launch behavior when your computer starts</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-between">
            <span className="text-sm">Launch at login</span>
            <Button
              variant={launchAtLogin ? 'default' : 'outline'}
              size="sm"
              onClick={() => setLaunchAtLogin(!launchAtLogin)}
            >
              {launchAtLogin ? 'Enabled' : 'Disabled'}
            </Button>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
