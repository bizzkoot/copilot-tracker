// Prevent console window on Windows in release builds
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use chrono::Datelike;
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{MouseButton, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Listener, Manager};
use tauri_plugin_http::reqwest;
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_opener::OpenerExt;

use copilot_tracker::{AuthManager, StoreManager, TrayIconRenderer, UsageManager, WidgetPosition};
mod theme;

const GITHUB_API_URL: &str =
    "https://api.github.com/repos/bizzkoot/copilot-tracker/releases/latest";

use crate::theme::text_color_for_theme_preference;

// ============================================================================
// Helper: Resolve App Directory
// ============================================================================

/// Resolve the app data directory manually without requiring an AppHandle.
/// This allows us to initialize StoreManager before the Tauri builder runs.
fn resolve_app_dir(identifier: &str) -> std::path::PathBuf {
    #[cfg(target_os = "macos")]
    let base = std::env::var("HOME")
        .map(|h| std::path::PathBuf::from(h).join("Library/Application Support"))
        .unwrap_or_else(|_| std::env::current_dir().unwrap());

    #[cfg(target_os = "windows")]
    let base = std::env::var("LOCALAPPDATA")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| std::env::current_dir().unwrap());

    #[cfg(target_os = "linux")]
    let base = std::env::var("XDG_DATA_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|_| {
            std::env::var("HOME").map(|h| std::path::PathBuf::from(h).join(".local/share"))
        })
        .unwrap_or_else(|_| std::env::current_dir().unwrap());

    base.join(identifier)
}

// ============================================================================
// Tray State
// ============================================================================

struct TrayState {
    tray: Mutex<Option<tauri::tray::TrayIcon>>,
    renderer: Arc<TrayIconRenderer>,
    last_menu_rebuild: Mutex<std::time::Instant>,
}

// ============================================================================
// Background Polling State
// ============================================================================

/// Debounce window for polling restart (milliseconds)
const POLLING_RESTART_DEBOUNCE_MS: u64 = 500;

struct PollingState {
    cancel_tx: Mutex<Option<tokio::sync::mpsc::Sender<()>>>,
    /// Timestamp of last restart to implement debounce
    last_restart: Mutex<std::time::Instant>,
    /// Last interval used to avoid duplicate restarts
    last_interval: Mutex<u64>,
    /// Flag to prevent restarts during app shutdown
    is_shutting_down: Mutex<bool>,
}

impl PollingState {
    fn new() -> Self {
        Self {
            cancel_tx: Mutex::new(None),
            last_restart: Mutex::new(std::time::Instant::now()),
            last_interval: Mutex::new(0),
            is_shutting_down: Mutex::new(false),
        }
    }

    /// Start or restart background polling with new interval
    /// Includes debounce to prevent rapid restarts and shutdown protection
    fn restart_polling(&self, app: AppHandle, interval_seconds: u64) {
        // Check if we're shutting down - don't start new polling tasks
        {
            let shutting_down = self.is_shutting_down.lock().unwrap();
            if *shutting_down {
                log::warn!("[PollingState] Ignoring restart request during shutdown");
                return;
            }
        }

        // Debounce: Skip if called with same interval within debounce window
        {
            let now = std::time::Instant::now();
            let mut last_restart = self.last_restart.lock().unwrap();
            let mut last_interval = self.last_interval.lock().unwrap();

            if *last_interval == interval_seconds
                && now.duration_since(*last_restart)
                    < std::time::Duration::from_millis(POLLING_RESTART_DEBOUNCE_MS)
            {
                log::debug!(
                    "[PollingState] Skipping duplicate restart request (interval: {}s)",
                    interval_seconds
                );
                return;
            }

            // Update tracking before restart
            *last_restart = now;
            *last_interval = interval_seconds;
        }

        // Cancel existing polling task if any
        if let Ok(mut guard) = self.cancel_tx.lock() {
            if let Some(tx) = guard.take() {
                // Properly handle cancellation result
                match tx.try_send(()) {
                    Ok(_) => log::info!("[PollingState] Cancelled previous polling task"),
                    Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                        log::warn!(
                            "[PollingState] Cancel channel full, task may already be stopping"
                        );
                    }
                    Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                        log::warn!("[PollingState] Cancel channel closed, task already stopped");
                    }
                }
            }

            // Start new polling task
            let cancel_tx = UsageManager::start_polling(app, interval_seconds);
            *guard = Some(cancel_tx);
            log::info!(
                "[PollingState] Started polling with interval: {}s",
                interval_seconds
            );
        }
    }

    /// Stop background polling and mark as shutting down
    fn stop_polling(&self) {
        // Set shutdown flag FIRST to prevent restart attempts
        {
            let mut shutting_down = self.is_shutting_down.lock().unwrap();
            *shutting_down = true;
            log::info!("[PollingState] Shutdown flag set");
        }

        // Then cancel the polling task
        if let Ok(mut guard) = self.cancel_tx.lock() {
            if let Some(tx) = guard.take() {
                match tx.try_send(()) {
                    Ok(_) => log::info!("[PollingState] Stopped polling"),
                    Err(tokio::sync::mpsc::error::TrySendError::Full(_)) => {
                        log::warn!("[PollingState] Stop request queued (channel full)");
                    }
                    Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                        log::debug!("[PollingState] Task already stopped (channel closed)");
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, serde::Serialize)]
struct UpdateCheckStatus {
    status: String,
    message: Option<String>,
}

#[derive(Clone, Debug, Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateInfo {
    version: String,
    release_url: String,
    download_url: Option<String>,
    release_name: Option<String>,
    release_notes: Option<String>,
    release_date: Option<String>,
}

#[derive(Default)]
struct UpdateState {
    latest: Mutex<Option<UpdateInfo>>,
    last_check_time: Mutex<Option<chrono::DateTime<chrono::Local>>>,
}

/// Format a timestamp for display (HH:MM:SS for today, date only for other days)
fn format_timestamp(date: Option<chrono::DateTime<chrono::Local>>) -> String {
    match date {
        None => "Never".to_string(),
        Some(dt) => {
            let now = chrono::Local::now();
            let today = now.date_naive();
            let date = dt.date_naive();

            if date == today {
                // Same day - show time in HH:MM:SS format
                dt.format("%H:%M:%S").to_string()
            } else {
                // Different day - show date only
                dt.format("%b %d").to_string()
            }
        }
    }
}

/// Format tray icon text based on the specified format
fn format_request_count(value: f64) -> String {
    let rounded = (value * 10.0).round() / 10.0;
    if (rounded.fract()).abs() < 0.001 {
        format!("{:.0}", rounded)
    } else {
        format!("{:.1}", rounded)
    }
}

fn format_tray_text(used: f64, limit: u32, format: &str) -> String {
    // Handle unauthenticated state (limit == 0)
    if limit == 0 {
        return format_request_count(used);
    }

    let remaining = (limit as f64 - used).max(0.0);
    let percentage = (used / limit as f64 * 100.0) as f32;
    let remaining_pct = 100.0 - percentage;
    let used_text = format_request_count(used);
    let limit_text = format_request_count(limit as f64);
    let remaining_text = format_request_count(remaining);

    match format {
        "current" => used_text,
        "currentTotal" => format!("{used_text}/{limit_text}"),
        "remainingTotal" => format!("{remaining_text}/{limit_text}"),
        "percentage" => format!("{:.0}%", percentage),
        "remainingPercent" => format!("{:.0}%", remaining_pct),
        "combined" => format!("{used_text}/{limit_text} ({:.0}%)", percentage),
        "remainingCombined" => {
            format!("{remaining_text}/{limit_text} ({:.0}%)", remaining_pct)
        }
        _ => format!("{used_text}/{limit_text}"), // fallback to current default
    }
}

fn tray_text_color(theme_preference: &str) -> (u8, u8, u8) {
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    {
        let _ = theme_preference;
        text_color_for_theme_preference("system")
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        text_color_for_theme_preference(theme_preference)
    }
}

fn update_tray_icon(
    app: &AppHandle,
    state: &TrayState,
    used: f64,
    limit: u32,
    format: &str,
) -> Result<(), String> {
    let text = format_tray_text(used, limit, format);
    let theme_preference = app.state::<StoreManager>().get_settings().theme;
    let color = tray_text_color(&theme_preference);

    let image = state
        .renderer
        .render_text_only(&text, 16, color)
        .into_tauri_image();

    let tray_guard = state
        .tray
        .lock()
        .map_err(|_| "tray lock poisoned".to_string())?;
    let tray = tray_guard
        .as_ref()
        .ok_or("tray not initialized".to_string())?;
    tray.set_icon(Some(image)).map_err(|err| err.to_string())?;

    #[cfg(target_os = "macos")]
    {
        tray.set_icon_as_template(true)
            .map_err(|err| err.to_string())?;
    }

    Ok(())
}

/// Helper to update tray icon using current settings from store
fn update_tray_icon_from_store(app: &AppHandle) -> Result<(), String> {
    let store = app.state::<StoreManager>();
    let (used, limit) = store.get_usage();
    let format = store.get_tray_icon_format();
    let tray_state = app.state::<TrayState>();
    update_tray_icon(app, &tray_state, used, limit, &format)
}

fn build_tray_menu(
    app: &AppHandle,
    update: Option<&UpdateInfo>,
) -> Result<Menu<tauri::Wry>, String> {
    let store = app.state::<StoreManager>();
    let settings = store.get_settings();
    let version = app.package_info().version.to_string();
    let (used, limit) = store.get_usage();
    let usage_history = UsageManager::get_cached_history(app);
    let prediction = UsageManager::predict_usage_from_history(
        &usage_history,
        used,
        limit,
        settings.prediction_period,
    );

    // Calculate metrics for dual-perspective display
    let remaining = (limit as f64 - used).max(0.0);
    let percentage_used = if limit > 0 {
        (used / limit as f64 * 100.0) as f32
    } else {
        0.0
    };
    let _percentage_remaining = 100.0 - percentage_used;

    // Calculate daily metrics aligned with Dashboard
    let now = chrono::Utc::now();
    let current_day = now.day() as f32;
    let days_in_month = if now.month() == 12 {
        31
    } else {
        let next_month = chrono::NaiveDate::from_ymd_opt(now.year(), now.month() + 1, 1)
            .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(now.year() + 1, 1, 1).unwrap());
        let current_month = chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap();
        (next_month - current_month).num_days() as u32
    };
    // Match dashboard logic: Math.max(1, daysInMonth - currentDay)
    let days_remaining = (days_in_month as f32 - current_day).max(1.0);
    // Floor the daily budget to be conservative (synced with Dashboard)
    let daily_budget = (remaining as f32 / days_remaining).floor();

    // Get actual today's usage from history (if available)
    let today_timestamp = now
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp();
    let today_usage_actual = usage_history
        .iter()
        .find(|e| e.timestamp == today_timestamp)
        .map(|e| e.used)
        .unwrap_or(0.0);

    let menu = Menu::new(app).map_err(|e| e.to_string())?;

    // === QUOTA SUBMENU (simplified: TODAY + MONTH FORECAST) ===
    let quota_submenu =
        Submenu::with_id(app, "quota", "📊 Quota", true).map_err(|e| e.to_string())?;

    // TODAY'S STATUS header
    let today_header =
        MenuItem::with_id(app, "today_header", "📊 TODAY'S STATUS", true, None::<&str>)
            .map_err(|e| e.to_string())?;
    quota_submenu
        .append(&today_header)
        .map_err(|e| e.to_string())?;

    if limit > 0 {
        let budget = daily_budget as i64;
        let usage_percent = if budget > 0 {
            ((today_usage_actual / budget as f64) * 100.0).round() as i32
        } else if today_usage_actual > 0.0 {
            100
        } else {
            0
        };
        let (status_icon, percent_str) = if budget == 0 && today_usage_actual > 0.0 {
            ("⚠️", "Exceeded".to_string())
        } else if today_usage_actual > budget as f64 {
            ("⚠️", format!("{}%", usage_percent))
        } else if usage_percent > 75 {
            ("🟡", format!("{}%", usage_percent))
        } else {
            ("✅", format!("{}%", usage_percent))
        };
        let today_line = MenuItem::with_id(
            app,
            "today_line",
            format!(
                "   ⚡ Used today: {} req",
                format_request_count(today_usage_actual)
            ),
            true,
            None::<&str>,
        )
        .map_err(|e| e.to_string())?;
        quota_submenu
            .append(&today_line)
            .map_err(|e| e.to_string())?;

        let budget_line = MenuItem::with_id(
            app,
            "budget_line",
            format!(
                "   🎯 Daily budget: {} req",
                format_request_count(budget as f64)
            ),
            true,
            None::<&str>,
        )
        .map_err(|e| e.to_string())?;
        quota_submenu
            .append(&budget_line)
            .map_err(|e| e.to_string())?;

        let percent_line = MenuItem::with_id(
            app,
            "percent_line",
            format!("   {} {} of daily budget", status_icon, percent_str),
            true,
            None::<&str>,
        )
        .map_err(|e| e.to_string())?;
        quota_submenu
            .append(&percent_line)
            .map_err(|e| e.to_string())?;
    } else {
        let loading_line =
            MenuItem::with_id(app, "loading_line", "▶ Loading data...", true, None::<&str>)
                .map_err(|e| e.to_string())?;
        quota_submenu
            .append(&loading_line)
            .map_err(|e| e.to_string())?;
    }

    quota_submenu
        .append(&PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;

    // MONTH FORECAST section
    if let Some(prediction) = prediction {
        let forecast_header = MenuItem::with_id(
            app,
            "forecast_header",
            "🔮 END OF MONTH FORECAST",
            true,
            None::<&str>,
        )
        .map_err(|e| e.to_string())?;
        quota_submenu
            .append(&forecast_header)
            .map_err(|e| e.to_string())?;

        let status_label = if prediction.predicted_monthly_requests > limit as f64 {
            "   ⚠️ Alert: Will exceed quota".to_string()
        } else {
            "   ✅ Safe: Won't exceed quota".to_string()
        };
        let status_line = MenuItem::with_id(app, "status_line", status_label, true, None::<&str>)
            .map_err(|e| e.to_string())?;
        quota_submenu
            .append(&status_line)
            .map_err(|e| e.to_string())?;

        let forecast_line = MenuItem::with_id(
            app,
            "forecast_line",
            format!(
                "   📈 Est. final usage: {} / {} req",
                format_request_count(prediction.predicted_monthly_requests),
                format_request_count(limit as f64)
            ),
            true,
            None::<&str>,
        )
        .map_err(|e| e.to_string())?;
        quota_submenu
            .append(&forecast_line)
            .map_err(|e| e.to_string())?;
    } else {
        let forecast_header = MenuItem::with_id(
            app,
            "forecast_header",
            "🔮 END OF MONTH FORECAST",
            true,
            None::<&str>,
        )
        .map_err(|e| e.to_string())?;
        quota_submenu
            .append(&forecast_header)
            .map_err(|e| e.to_string())?;
        let no_data = MenuItem::with_id(app, "no_data", "   Insufficient data", true, None::<&str>)
            .map_err(|e| e.to_string())?;
        quota_submenu.append(&no_data).map_err(|e| e.to_string())?;
    }

    menu.append(&quota_submenu).map_err(|e| e.to_string())?;

    // === USAGE HISTORY SECTION ===
    let history_submenu = Submenu::with_id(app, "usage_history", "📜 Usage History", true)
        .map_err(|e| e.to_string())?;
    if !usage_history.is_empty() {
        for entry in usage_history.iter().take(7) {
            let date = chrono::DateTime::from_timestamp(entry.timestamp, 0)
                .map(|dt| dt.date_naive())
                .unwrap_or_else(|| chrono::Utc::now().date_naive());
            let label = format!(
                "{}: {} req",
                date.format("%b %d"),
                format_request_count(entry.used)
            );
            let item = MenuItem::new(app, label, true, None::<&str>).map_err(|e| e.to_string())?;
            history_submenu.append(&item).map_err(|e| e.to_string())?;
        }
    } else {
        let item =
            MenuItem::new(app, "No history yet", true, None::<&str>).map_err(|e| e.to_string())?;
        history_submenu.append(&item).map_err(|e| e.to_string())?;
    }
    menu.append(&history_submenu).map_err(|e| e.to_string())?;

    let prediction_period_submenu =
        Submenu::with_id(app, "prediction_period", "Prediction Period", true)
            .map_err(|e| e.to_string())?;
    for (label, value) in [("7 days", 7_u32), ("14 days", 14_u32), ("21 days", 21_u32)] {
        let item = CheckMenuItem::with_id(
            app,
            format!("prediction_period:{}", value),
            label,
            true,
            settings.prediction_period == value,
            None::<&str>,
        )
        .map_err(|e| e.to_string())?;
        prediction_period_submenu
            .append(&item)
            .map_err(|e| e.to_string())?;
    }
    menu.append(&prediction_period_submenu)
        .map_err(|e| e.to_string())?;

    let refresh_submenu =
        Submenu::with_id(app, "auto_refresh", "Auto Refresh", true).map_err(|e| e.to_string())?;
    let refresh_options = [
        ("10 seconds", 10_u32),
        ("30 seconds", 30_u32),
        ("1 minute", 60_u32),
        ("5 minutes", 300_u32),
        ("30 minutes", 1800_u32),
    ];
    for (label, value) in refresh_options {
        let item = CheckMenuItem::with_id(
            app,
            format!("refresh_interval:{}", value),
            label,
            true,
            settings.refresh_interval == value,
            None::<&str>,
        )
        .map_err(|e| e.to_string())?;
        refresh_submenu.append(&item).map_err(|e| e.to_string())?;
    }
    menu.append(&refresh_submenu).map_err(|e| e.to_string())?;

    menu.append(&PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;

    let open_dashboard =
        MenuItem::with_id(app, "open_dashboard", "Open Dashboard", true, None::<&str>)
            .map_err(|e| e.to_string())?;
    menu.append(&open_dashboard).map_err(|e| e.to_string())?;

    let open_billing = MenuItem::with_id(app, "open_billing", "Open Billing", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    menu.append(&open_billing).map_err(|e| e.to_string())?;

    // Refresh with last refresh time on the same row
    let store = app.state::<StoreManager>();
    let last_fetch_timestamp = store.get_last_fetch_timestamp();
    let last_refresh_time = if last_fetch_timestamp > 0 {
        chrono::DateTime::from_timestamp(last_fetch_timestamp, 0)
            .map(|dt| dt.with_timezone(&chrono::Local))
    } else {
        None
    };
    let last_refresh_label = format!("Refresh (Last: {})", format_timestamp(last_refresh_time));
    let refresh = MenuItem::with_id(app, "refresh", last_refresh_label, true, None::<&str>)
        .map_err(|e| e.to_string())?;
    menu.append(&refresh).map_err(|e| e.to_string())?;

    menu.append(&PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;

    // Widget menu item
    let widget_visible = if let Some(widget) = app.get_webview_window("widget") {
        widget.is_visible().unwrap_or(false)
    } else {
        false
    };
    let widget_label = if widget_visible {
        "Hide Widget"
    } else {
        "Show Widget"
    };
    let widget_item = MenuItem::with_id(app, "toggle_widget", widget_label, true, None::<&str>)
        .map_err(|e| e.to_string())?;
    menu.append(&widget_item).map_err(|e| e.to_string())?;

    let settings_item = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    menu.append(&settings_item).map_err(|e| e.to_string())?;

    menu.append(&PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;

    let update_label = if let Some(info) = update {
        format!("⬆️ Update Available: {}", info.version)
    } else {
        "Check for Updates".to_string()
    };
    let update_item = MenuItem::with_id(app, "update_check", update_label, true, None::<&str>)
        .map_err(|e| e.to_string())?;
    menu.append(&update_item).map_err(|e| e.to_string())?;

    // Show last check time below when no update is available (from persisted store)
    if update.is_none() {
        let store = app.state::<StoreManager>();
        let last_check_time = store.get_last_update_check_timestamp();
        let last_check_dt = if last_check_time > 0 {
            chrono::DateTime::from_timestamp(last_check_time, 0)
                .map(|dt| dt.with_timezone(&chrono::Local))
        } else {
            None
        };
        let last_check_label = format!("Last checked: {}", format_timestamp(last_check_dt));
        let last_check_item =
            MenuItem::with_id(app, "last_check", last_check_label, false, None::<&str>)
                .map_err(|e| e.to_string())?;
        menu.append(&last_check_item).map_err(|e| e.to_string())?;
    }

    menu.append(&PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;

    let launch_label = if settings.launch_at_login {
        "☑️ Launch at Login"
    } else {
        "☐ Launch at Login"
    };
    let launch_item = MenuItem::with_id(app, "launch_at_login", launch_label, true, None::<&str>)
        .map_err(|e| e.to_string())?;
    menu.append(&launch_item).map_err(|e| e.to_string())?;

    menu.append(&PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;

    // GitHub Links submenu
    let github_submenu =
        Submenu::with_id(app, "github", "⭐ GitHub", true).map_err(|e| e.to_string())?;
    let github_stars =
        MenuItem::with_id(app, "github_repo", "⭐ Star on GitHub", true, None::<&str>)
            .map_err(|e| e.to_string())?;
    github_submenu
        .append(&github_stars)
        .map_err(|e| e.to_string())?;

    let github_issues =
        MenuItem::with_id(app, "github_issues", "🐛 Report Issue", true, None::<&str>)
            .map_err(|e| e.to_string())?;
    github_submenu
        .append(&github_issues)
        .map_err(|e| e.to_string())?;
    menu.append(&github_submenu).map_err(|e| e.to_string())?;

    menu.append(&PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;

    let version_item = MenuItem::with_id(
        app,
        "version",
        format!("Version {}", version),
        false,
        None::<&str>,
    )
    .map_err(|e| e.to_string())?;
    menu.append(&version_item).map_err(|e| e.to_string())?;

    let quit_i =
        MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).map_err(|e| e.to_string())?;
    menu.append(&quit_i).map_err(|e| e.to_string())?;

    Ok(menu)
}

fn rebuild_tray_menu(app: &AppHandle, update: Option<&UpdateInfo>) -> Result<(), String> {
    let tray_state = app.state::<TrayState>();

    // Debounce: Don't rebuild more than once per second
    {
        let mut last_rebuild = tray_state
            .last_menu_rebuild
            .lock()
            .map_err(|_| "lock poisoned")?;
        let now = std::time::Instant::now();
        if now.duration_since(*last_rebuild).as_millis() < 1000 {
            log::debug!("Skipping tray menu rebuild - too soon since last rebuild");
            return Ok(());
        }
        *last_rebuild = now;
    }

    let menu = build_tray_menu(app, update)?;
    let tray_guard = tray_state
        .tray
        .lock()
        .map_err(|_| "tray lock poisoned".to_string())?;
    let tray = tray_guard
        .as_ref()
        .ok_or("tray not initialized".to_string())?;

    // Set new menu (Tauri automatically cleans up old menu)
    tray.set_menu(Some(menu)).map_err(|e| e.to_string())?;

    // Force cleanup of old menu references by dropping the guard early
    drop(tray_guard);

    log::debug!("Tray menu rebuilt successfully");
    Ok(())
}

#[tauri::command]
async fn show_auth_window(
    app: AppHandle,
    state: tauri::State<'_, AuthManagerState>,
) -> Result<bool, String> {
    let mut auth_manager = state
        .auth_manager
        .lock()
        .map_err(|e| format!("Failed to acquire auth manager lock: {}", e))?;
    auth_manager.show_auth_window(&app)?;
    Ok(true)
}

#[tauri::command]
async fn perform_auth_extraction(
    app: AppHandle,
    state: tauri::State<'_, AuthManagerState>,
) -> Result<copilot_tracker::ExtractionResult, String> {
    let app_clone = app.clone();
    let auth_manager_state = state.auth_manager.clone();
    let result = {
        let mut manager = AuthManager::new();
        manager.perform_extraction(&app_clone).await
    }?;

    if let Some(customer_id) = result.customer_id {
        if let Some(store) = app.try_state::<StoreManager>() {
            let _ = store.set_customer_id(customer_id);
            let _ = app.emit("auth:state-changed", "authenticated");
        }
    }

    {
        let mut manager = auth_manager_state.lock().unwrap();
        manager.finish_extraction();
    }

    Ok(result)
}

#[tauri::command]
async fn check_auth_status(app: AppHandle) -> Result<copilot_tracker::AuthState, String> {
    let store = app.state::<StoreManager>();
    let customer_id = store.get_customer_id();

    let is_authenticated = customer_id.is_some();
    let state_str = if is_authenticated {
        "authenticated"
    } else {
        "unauthenticated"
    };
    let _ = app.emit("auth:state-changed", state_str);

    Ok(copilot_tracker::AuthState {
        is_authenticated,
        customer_id,
    })
}

// ============================================================================
// IPC Commands - Usage
// ============================================================================

#[tauri::command]
async fn fetch_usage(
    app: AppHandle,
    _state: tauri::State<'_, AuthManagerState>,
) -> Result<copilot_tracker::UsageSummary, String> {
    let _ = app.emit("usage:loading", true);
    let mut usage_manager = UsageManager::new();
    let result = usage_manager.fetch_usage(&app).await;
    let _ = app.emit("usage:loading", false);

    if let Ok(summary) = &result {
        let history = UsageManager::get_cached_history(&app);
        let store = app.state::<StoreManager>();
        let settings = store.get_settings();
        let prediction = UsageManager::predict_usage_from_history(
            &history,
            summary.used,
            summary.limit,
            settings.prediction_period,
        );
        let payload = copilot_tracker::UsagePayload {
            summary: summary.clone(),
            history,
            prediction,
        };
        let _ = app.emit("usage:data", payload);
    }

    result
}

#[tauri::command]
async fn force_fetch_usage(
    app: AppHandle,
    _state: tauri::State<'_, AuthManagerState>,
) -> Result<copilot_tracker::UsageSummary, String> {
    log::info!("Force fetch usage - clearing cache first");

    // Clear cache first
    {
        let store = app.state::<StoreManager>();
        store.clear_usage_cache();
        store.clear_usage_history();
    }

    // Now fetch fresh data
    let _ = app.emit("usage:loading", true);
    let mut usage_manager = UsageManager::new();
    let result = usage_manager.fetch_usage(&app).await;
    let _ = app.emit("usage:loading", false);

    if let Ok(summary) = &result {
        let history = UsageManager::get_cached_history(&app);
        let store = app.state::<StoreManager>();
        let settings = store.get_settings();
        let prediction = UsageManager::predict_usage_from_history(
            &history,
            summary.used,
            summary.limit,
            settings.prediction_period,
        );
        let payload = copilot_tracker::UsagePayload {
            summary: summary.clone(),
            history,
            prediction,
        };
        let _ = app.emit("usage:data", payload);
    }

    result
}

#[tauri::command]
fn get_cached_usage(app: AppHandle) -> Result<copilot_tracker::UsageSummary, String> {
    UsageManager::get_cached_usage(&app)
}

#[tauri::command]
fn predict_eom_usage(app: AppHandle) -> Result<u32, String> {
    UsageManager::predict_eom_usage(&app)
}

#[tauri::command]
fn days_until_limit(app: AppHandle) -> Result<Option<i64>, String> {
    UsageManager::days_until_limit(&app)
}

#[tauri::command]
fn get_cached_usage_data(app: AppHandle) -> Result<Option<copilot_tracker::UsagePayload>, String> {
    let store = app.state::<StoreManager>();
    let (used, limit) = store.get_usage();
    let is_authenticated = store.is_authenticated();

    if !is_authenticated {
        return Ok(None);
    }

    let remaining = (limit as f64 - used).max(0.0);
    let percentage = if limit > 0 {
        (used / limit as f64 * 100.0) as f32
    } else {
        0.0
    };

    let summary = copilot_tracker::UsageSummary {
        used,
        limit,
        remaining,
        percentage,
        timestamp: chrono::Utc::now().timestamp(),
    };

    let history = UsageManager::get_cached_history(&app);
    let settings = store.get_settings();
    let prediction =
        UsageManager::predict_usage_from_history(&history, used, limit, settings.prediction_period);

    Ok(Some(copilot_tracker::UsagePayload {
        summary,
        history,
        prediction,
    }))
}

// ============================================================================
// IPC Commands - Settings
// ============================================================================

#[tauri::command]
fn get_settings(app: AppHandle) -> Result<copilot_tracker::AppSettings, String> {
    let store = app.state::<StoreManager>();
    Ok(store.get_settings())
}

#[tauri::command]
fn get_app_version(app: AppHandle) -> Result<String, String> {
    Ok(app.package_info().version.to_string())
}

#[tauri::command]
fn update_settings(app: AppHandle, settings: copilot_tracker::AppSettings) -> Result<(), String> {
    let store = app.state::<StoreManager>();
    let previous = store.get_settings();
    store.update_settings(|s| {
        *s = settings.clone();
    })?;

    if previous.launch_at_login != settings.launch_at_login {
        use tauri_plugin_autostart::ManagerExt;
        let result = if settings.launch_at_login {
            app.autolaunch().enable()
        } else {
            app.autolaunch().disable()
        };

        if let Err(e) = result {
            log::error!("Failed to set launch at login: {}", e);
            let _ = store.update_settings(|s| {
                s.launch_at_login = previous.launch_at_login;
            });
            return Err(format!("Failed to set launch at login: {}", e));
        }
    }

    let _ = app.emit("settings:changed", settings.clone());
    let update_state = app.state::<UpdateState>();
    let latest = update_state.latest.lock().unwrap();
    let _ = rebuild_tray_menu(&app, latest.as_ref());

    // Update tray icon with new format
    let _ = update_tray_icon_from_store(&app);

    Ok(())
}

#[tauri::command]
fn reset_settings(app: AppHandle) -> Result<copilot_tracker::AppSettings, String> {
    log::info!("Resetting all settings and data...");

    let store = app.state::<StoreManager>();
    let defaults = store.reset_settings()?;

    log::info!(
        "Store reset complete, customer_id is now: {:?}",
        store.get_customer_id()
    );

    // IMPORTANT: Emit auth state changed FIRST before settings changed
    // This ensures frontend clears auth state before any other events
    let _ = app.emit("auth:state-changed", "unauthenticated");
    log::info!("Emitted auth:state-changed = unauthenticated");

    // Small delay to ensure auth event is processed before settings event.
    // Note: This is a synchronous command, so blocking sleep is acceptable here.
    // The Tauri runtime handles this in a thread pool.
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Then emit settings changed
    let _ = app.emit("settings:changed", defaults.clone());
    log::info!("Emitted settings:changed with defaults");

    // CRITICAL: Emit usage:updated with empty data to reset tray icon
    let (used, limit) = store.get_usage();
    log::info!("Reset usage values: used={}, limit={}", used, limit);

    let summary = copilot_tracker::UsageSummary {
        used,
        limit,
        remaining: (limit as f64 - used).max(0.0),
        percentage: if limit > 0 {
            (used / limit as f64 * 100.0) as f32
        } else {
            0.0
        },
        timestamp: chrono::Utc::now().timestamp(),
    };
    let _ = app.emit("usage:updated", &summary);
    log::info!("Emitted usage:updated to reset tray icon");

    // Update tray icon directly to "1" (unauthenticated state)
    let tray_state = app.state::<TrayState>();
    let _ = update_tray_icon(&app, &tray_state, 1.0, 0, "currentTotal");
    log::info!("Updated tray icon to default '1' for unauthenticated state");

    // Rebuild tray menu
    let update_state = app.state::<UpdateState>();
    let latest = update_state.latest.lock().unwrap();
    let _ = rebuild_tray_menu(&app, latest.as_ref());

    Ok(defaults)
}

#[tauri::command]
async fn logout(app: AppHandle) -> Result<(), String> {
    let store = app.state::<StoreManager>();
    store.clear_auth()?;

    // Stop background polling when user logs out
    let polling_state = app.state::<PollingState>();
    polling_state.stop_polling();
    log::info!("[Logout] Background polling stopped");

    // Emit event to frontend
    let _ = app.emit("auth:state-changed", "unauthenticated");

    Ok(())
}

#[tauri::command]
fn set_launch_at_login(app: AppHandle, enabled: bool) -> Result<(), String> {
    let store = app.state::<StoreManager>();
    store.set_launch_at_login(enabled)?;

    // Enable/disable autostart using the plugin
    use tauri_plugin_autostart::ManagerExt;
    let result = if enabled {
        app.autolaunch().enable()
    } else {
        app.autolaunch().disable()
    };

    if let Err(e) = result {
        log::error!("Failed to set launch at login: {}", e);
        return Err(format!("Failed to set launch at login: {}", e));
    }

    let update_state = app.state::<UpdateState>();
    let latest = update_state.latest.lock().unwrap();
    let _ = rebuild_tray_menu(&app, latest.as_ref());

    Ok(())
}

#[tauri::command]
fn hide_main_window(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn open_external_url(app: AppHandle, url: String) -> Result<(), String> {
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| e.to_string())
}

// ============================================================================
// Widget Commands
// ============================================================================

#[tauri::command]
fn toggle_widget(app: AppHandle) -> Result<bool, String> {
    if let Some(widget) = app.get_webview_window("widget") {
        let store = app.state::<StoreManager>();
        if widget.is_visible().map_err(|e| e.to_string())? {
            widget.hide().map_err(|e| e.to_string())?;
            // Fully disable widget on hide (must re-enable from settings)
            store
                .set_widget_enabled(false)
                .map_err(|e| format!("Failed to persist widget_enabled=false: {}", e))?;
            store
                .set_widget_visible(false)
                .map_err(|e| format!("Failed to persist widget_visible=false: {}", e))?;
            // Notify all windows of widget state change
            let _ = app.emit("widget:enabled-changed", false);
            Ok(false)
        } else {
            // Restore position before showing
            let widget_position = store.get_widget_position();
            widget
                .set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                    x: widget_position.x,
                    y: widget_position.y,
                }))
                .map_err(|e| e.to_string())?;
            let widget_pinned = store.get_widget_pinned();
            show_widget_without_focus(&widget, widget_pinned)?;
            // Mark widget as enabled and visible so it restores on restart
            store
                .set_widget_enabled(true)
                .map_err(|e| format!("Failed to persist widget_enabled=true: {}", e))?;
            store
                .set_widget_visible(true)
                .map_err(|e| format!("Failed to persist widget_visible=true: {}", e))?;
            // Notify all windows of widget state change
            let _ = app.emit("widget:enabled-changed", true);
            Ok(true)
        }
    } else {
        Err("Widget window not found".to_string())
    }
}

/// Hide widget from the widget window's close button
/// Updates store and rebuilds tray menu
#[tauri::command]
fn hide_widget(app: AppHandle) -> Result<(), String> {
    if let Some(widget) = app.get_webview_window("widget") {
        let store = app.state::<StoreManager>();
        widget.hide().map_err(|e| e.to_string())?;
        // Fully disable widget when closing (must re-enable from settings)
        store
            .set_widget_enabled(false)
            .map_err(|e| format!("Failed to persist widget_enabled=false: {}", e))?;
        store
            .set_widget_visible(false)
            .map_err(|e| format!("Failed to persist widget_visible=false: {}", e))?;

        // Notify all windows of widget state change
        let _ = app.emit("widget:enabled-changed", false);

        // Rebuild tray menu to update "Show Widget" label
        if let Ok(menu) = build_tray_menu(&app, None) {
            if let Some(tray_state) = app.try_state::<TrayState>() {
                if let Ok(tray_guard) = tray_state.tray.lock() {
                    if let Some(ref tray) = *tray_guard {
                        let _ = tray.set_menu(Some(menu));
                    }
                }
            }
        }
    }
    Ok(())
}

/// Minimize widget from the widget window's minimize button
/// Updates store and rebuilds tray menu (same behavior as hide for widget)
#[tauri::command]
fn minimize_widget(app: AppHandle) -> Result<(), String> {
    // For the floating widget, minimize behaves the same as hide
    // Both just hide the window and update the tray menu
    hide_widget(app)
}

/// Show widget without stealing focus from current application
/// Uses platform-specific APIs to prevent focus stealing
fn show_widget_without_focus(
    widget: &tauri::WebviewWindow,
    always_on_top: bool,
) -> Result<(), String> {
    // Show the widget
    widget.show().map_err(|e| e.to_string())?;

    // Platform-specific focus prevention
    #[cfg(target_os = "macos")]
    {
        // On macOS, we need to prevent the widget from becoming the key window
        // The window is configured with skip_taskbar and decorations=false
        // which helps, but we also minimize and restore to avoid focus steal
        // This is a workaround since we can't easily access NSWindow APIs without objc

        // Small delay to let the show complete, then minimize and restore
        // This breaks the focus chain
        std::thread::sleep(std::time::Duration::from_millis(10));
        widget
            .set_always_on_top(always_on_top)
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, the window configuration (skip_taskbar, decorations=false)
        // already helps prevent focus stealing
        // We ensure always_on_top is set to keep it floating
        widget
            .set_always_on_top(always_on_top)
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        // On Linux, most window managers respect the window type
        // The skip_taskbar and decorations settings help prevent focus stealing
        widget
            .set_always_on_top(always_on_top)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[tauri::command]
fn is_widget_visible(app: AppHandle) -> Result<bool, String> {
    if let Some(widget) = app.get_webview_window("widget") {
        widget.is_visible().map_err(|e| e.to_string())
    } else {
        Ok(false)
    }
}

#[tauri::command]
async fn set_widget_position(app: AppHandle, x: i32, y: i32) -> Result<(), String> {
    let store = app.state::<StoreManager>();

    if let Some(widget) = app.get_webview_window("widget") {
        let should_set_position = match widget.outer_position() {
            Ok(current) => current.x != x || current.y != y,
            Err(_) => true,
        };

        if should_set_position {
            widget
                .set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }))
                .map_err(|e| e.to_string())?;
        }
    }

    // Save position to settings even if the widget window is not currently available
    store
        .set_widget_position(WidgetPosition { x, y })
        .map_err(|e| {
            log::error!(
                "[Widget] Failed to persist widget position ({}, {}): {}",
                x,
                y,
                e
            );
            e
        })?;

    Ok(())
}

#[tauri::command]
async fn get_widget_position(app: AppHandle) -> Result<WidgetPosition, String> {
    let store = app.state::<StoreManager>();

    if let Some(widget) = app.get_webview_window("widget") {
        let pos = widget.outer_position().map_err(|e| e.to_string())?;
        Ok(WidgetPosition { x: pos.x, y: pos.y })
    } else {
        // Widget window not yet created, return stored position
        Ok(store.get_widget_position())
    }
}

#[tauri::command]
async fn set_widget_pinned(app: AppHandle, pinned: bool) -> Result<(), String> {
    if let Some(widget) = app.get_webview_window("widget") {
        widget
            .set_always_on_top(pinned)
            .map_err(|e| e.to_string())?;
    }

    // Save pin state to settings even if the widget window is not currently available
    let store = app.state::<StoreManager>();
    store.set_widget_pinned(pinned).map_err(|e| {
        log::error!(
            "[Widget] Failed to persist widget pin state ({}): {}",
            pinned,
            e
        );
        e
    })?;

    // Emit event to notify widget window
    let _ = app.emit("widget:set-pin", pinned);

    Ok(())
}

#[tauri::command]
async fn is_widget_pinned(app: AppHandle) -> Result<bool, String> {
    let store = app.state::<StoreManager>();
    Ok(store.get_widget_pinned())
}

#[tauri::command]
async fn is_widget_enabled(app: AppHandle) -> Result<bool, String> {
    let store = app.state::<StoreManager>();
    Ok(store.get_widget_enabled())
}

#[tauri::command]
async fn set_widget_enabled(app: AppHandle, enabled: bool) -> Result<(), String> {
    let store = app.state::<StoreManager>();
    store
        .set_widget_enabled(enabled)
        .map_err(|e| e.to_string())?;

    log::info!("[Widget] set_widget_enabled called: enabled={}", enabled);

    // Always set widget_visible state regardless of window availability
    // This ensures the state persists even if window is not ready yet
    if enabled {
        store
            .set_widget_visible(true)
            .map_err(|e| format!("Failed to persist widget_visible=true: {}", e))?;
        log::info!("[Widget] Set widget_visible=true");
    } else {
        store
            .set_widget_visible(false)
            .map_err(|e| format!("Failed to persist widget_visible=false: {}", e))?;
        log::info!("[Widget] Set widget_visible=false");
    }

    // Emit event to notify all windows of widget state change
    let _ = app.emit("widget:enabled-changed", enabled);

    // If enabling, also show the widget
    if enabled {
        if let Some(widget) = app.get_webview_window("widget") {
            // Restore position before showing
            let widget_position = store.get_widget_position();
            widget
                .set_position(tauri::Position::Physical(tauri::PhysicalPosition {
                    x: widget_position.x,
                    y: widget_position.y,
                }))
                .map_err(|e| e.to_string())?;
            let widget_pinned = store.get_widget_pinned();
            show_widget_without_focus(&widget, widget_pinned)?;
            log::info!("[Widget] Widget window shown");
        } else {
            log::warn!("[Widget] Widget window not found when enabling");
        }
    } else {
        // If disabling, hide the widget
        if let Some(widget) = app.get_webview_window("widget") {
            widget.hide().map_err(|e| e.to_string())?;
            log::info!("[Widget] Widget window hidden");
        }
    }

    // Rebuild tray menu to update the widget toggle label
    let _ = rebuild_tray_menu(&app, None);

    Ok(())
}

// ============================================================================
// IPC Commands - App
// ============================================================================

/// Helper function to process release data and emit appropriate events
fn process_release_data(
    app: &AppHandle,
    release: serde_json::Value,
    send_status: &dyn Fn(&str, Option<&str>),
) -> Result<(), String> {
    // Store the last check time at the start (regardless of outcome)
    let update_state = app.state::<UpdateState>();
    let now = chrono::Local::now();
    *update_state.last_check_time.lock().unwrap() = Some(now);

    // Persist to store
    let store = app.state::<StoreManager>();
    let _ = store.set_last_update_check_timestamp(now.timestamp());

    let tag_name = release
        .get("tag_name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let latest_version = tag_name.trim_start_matches('v');
    let current_version = app.package_info().version.to_string();

    let latest = match semver::Version::parse(latest_version) {
        Ok(version) => version,
        Err(_) => {
            send_status("error", Some("Invalid version format"));
            return Err("Invalid version format".to_string());
        }
    };
    let current = match semver::Version::parse(&current_version) {
        Ok(version) => version,
        Err(_) => {
            send_status("error", Some("Invalid version format"));
            return Err("Invalid version format".to_string());
        }
    };

    if latest > current {
        let assets = release
            .get("assets")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let download_url = assets
            .iter()
            .find_map(|asset| asset.get("browser_download_url").and_then(|v| v.as_str()))
            .map(|s| s.to_string());

        let info = UpdateInfo {
            version: tag_name,
            release_url: release
                .get("html_url")
                .and_then(|v| v.as_str())
                .unwrap_or("https://github.com/bizzkoot/copilot-tracker/releases")
                .to_string(),
            download_url,
            release_name: release
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            release_notes: release
                .get("body")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            release_date: release
                .get("published_at")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        };

        *update_state.latest.lock().unwrap() = Some(info.clone());

        let _ = app.emit("update:available", info.clone());
        send_status("available", None);

        let store = app.state::<StoreManager>();
        if store.get_show_notifications() {
            let _ = app
                .notification()
                .builder()
                .title("Copilot Tracker Update Available")
                .body(format!("Version {} is available.", info.version))
                .show();
        }

        let _ = rebuild_tray_menu(app, Some(&info));
    } else {
        *update_state.latest.lock().unwrap() = None;
        send_status("none", None);

        // Show notification that app is up to date
        let store = app.state::<StoreManager>();
        if store.get_show_notifications() {
            let _ = app
                .notification()
                .builder()
                .title("Copilot Tracker")
                .body(format!(
                    "You're running the latest version ({}).",
                    current_version
                ))
                .show();
        }

        let _ = rebuild_tray_menu(app, None);
    }

    Ok(())
}

#[tauri::command]
async fn check_for_updates(app: AppHandle) -> Result<(), String> {
    let send_status = |status: &str, message: Option<&str>| {
        let payload = UpdateCheckStatus {
            status: status.to_string(),
            message: message.map(|s| s.to_string()),
        };
        let _ = app.emit("update:checked", payload);
    };

    // Windows-specific update check: prefer WebView-based fetch first, then
    // fall back to reqwest (standard, then relaxed TLS as last resort).
    // Unsigned Windows binaries are more likely to hit network middleware or
    // certificate-path issues, so we keep multiple transport strategies.
    #[cfg(target_os = "windows")]
    {
        log::info!("[Update] Windows detected: trying webview fetch first");

        let mut auth_manager = AuthManager::new();
        let webview_error: String = match auth_manager.fetch_github_releases(&app).await {
            Ok(release_json) => {
                if release_json
                    .get("success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    let release = release_json.get("data").cloned().unwrap_or(release_json);
                    process_release_data(&app, release, &send_status)?;
                    return Ok(());
                }

                release_json
                    .get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown webview error")
                    .to_string()
            }
            Err(err) => err,
        };

        log::warn!(
            "[Update] Windows webview fetch failed: {}. Trying reqwest...",
            webview_error
        );

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| format!("Failed to build reqwest client: {}", e))?;

        let response = client
            .get(GITHUB_API_URL)
            .header("User-Agent", "Copilot-Tracker-App")
            .send()
            .await;

        let reqwest_error = match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    let mut parse_error: Option<String> = None;
                    let release = match resp.json().await {
                        Ok(value) => value,
                        Err(err) => {
                            parse_error = Some(format!("failed to parse response: {}", err));
                            serde_json::Value::Null
                        }
                    };

                    if !release.is_null() {
                        log::info!("[Update] Reqwest fallback succeeded");
                        process_release_data(&app, release, &send_status)?;
                        return Ok(());
                    }

                    parse_error.unwrap_or_else(|| "failed to parse response".to_string())
                } else {
                    format!("HTTP {}", resp.status())
                }
            }
            Err(err) => err.to_string(),
        };

        log::warn!(
            "[Update] Windows reqwest failed: {}. Trying relaxed TLS fallback...",
            reqwest_error
        );

        let relaxed_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true)
            .build()
            .map_err(|e| format!("Failed to build relaxed reqwest client: {}", e))?;

        let relaxed_response = relaxed_client
            .get(GITHUB_API_URL)
            .header("User-Agent", "Copilot-Tracker-App")
            .send()
            .await;

        let relaxed_error = match relaxed_response {
            Ok(resp) => {
                if resp.status().is_success() {
                    let mut parse_error: Option<String> = None;
                    let release = match resp.json().await {
                        Ok(value) => value,
                        Err(err) => {
                            parse_error = Some(format!("failed to parse response: {}", err));
                            serde_json::Value::Null
                        }
                    };

                    if !release.is_null() {
                        log::info!("[Update] Relaxed reqwest fallback succeeded");
                        process_release_data(&app, release, &send_status)?;
                        return Ok(());
                    }

                    parse_error.unwrap_or_else(|| "failed to parse response".to_string())
                } else {
                    format!("HTTP {}", resp.status())
                }
            }
            Err(err) => err.to_string(),
        };

        let update_state = app.state::<UpdateState>();
        let now = chrono::Local::now();
        *update_state.last_check_time.lock().unwrap() = Some(now);

        let store = app.state::<StoreManager>();
        let _ = store.set_last_update_check_timestamp(now.timestamp());

        let final_error = format!(
            "Update check failed (webview: {}, reqwest: {}, reqwest_relaxed: {})",
            webview_error, reqwest_error, relaxed_error
        );
        send_status("error", Some(final_error.as_str()));

        if store.get_show_notifications() {
            let _ = app
                .notification()
                .builder()
                .title("Copilot Tracker")
                .body("Update check failed. Opening releases in your browser.")
                .show();
        }

        let _ = app.opener().open_url(
            "https://github.com/bizzkoot/copilot-tracker/releases/latest",
            None::<&str>,
        );

        let _ = rebuild_tray_menu(&app, None);
        Ok(())
    }

    // Non-Windows platforms (macOS, Linux): Use webview fetch as primary method
    #[cfg(not(target_os = "windows"))]
    {
        log::info!("[Update] Attempting update check via webview fetch...");

        let mut auth_manager = AuthManager::new();
        match auth_manager.fetch_github_releases(&app).await {
            Ok(release_json) => {
                // Successfully fetched via webview
                log::info!("[Update] Webview fetch succeeded");
                let release = if release_json
                    .get("success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false)
                {
                    release_json.get("data").cloned().unwrap_or(release_json)
                } else {
                    // Store last check time even on error
                    let update_state = app.state::<UpdateState>();
                    let now = chrono::Local::now();
                    *update_state.last_check_time.lock().unwrap() = Some(now);

                    // Persist to store
                    let store = app.state::<StoreManager>();
                    let _ = store.set_last_update_check_timestamp(now.timestamp());

                    let error_msg = format!(
                        "Webview fetch failed: {}",
                        release_json
                            .get("error")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown error")
                    );
                    send_status("error", Some(&error_msg));

                    // Show error notification
                    if store.get_show_notifications() {
                        let _ = app
                            .notification()
                            .builder()
                            .title("Copilot Tracker")
                            .body("Failed to check for updates. Please try again later.")
                            .show();
                    }
                    let _ = rebuild_tray_menu(&app, None);
                    return Ok(());
                };
                process_release_data(&app, release, &send_status)?;
            }
            Err(webview_err) => {
                log::warn!(
                    "[Update] Webview fetch failed: {}, trying reqwest fallback",
                    webview_err
                );

                // Solution #2: Fallback to reqwest with rustls TLS
                log::info!("[Update] Attempting update check via reqwest with rustls TLS...");

                let client = reqwest::Client::new();
                let response = client
                    .get(GITHUB_API_URL)
                    .header("User-Agent", "Copilot-Tracker-App")
                    .send()
                    .await;

                match response {
                    Ok(resp) => {
                        if !resp.status().is_success() {
                            // Store last check time even on error
                            let update_state = app.state::<UpdateState>();
                            let now = chrono::Local::now();
                            *update_state.last_check_time.lock().unwrap() = Some(now);

                            // Persist to store
                            let store = app.state::<StoreManager>();
                            let _ = store.set_last_update_check_timestamp(now.timestamp());

                            send_status(
                                "error",
                                Some(
                                    format!("GitHub API returned status: {}", resp.status())
                                        .as_str(),
                                ),
                            );

                            // Show error notification
                            if store.get_show_notifications() {
                                let _ = app
                                    .notification()
                                    .builder()
                                    .title("Copilot Tracker")
                                    .body("Failed to check for updates. Please try again later.")
                                    .show();
                            }
                            let _ = rebuild_tray_menu(&app, None);
                            return Ok(());
                        }

                        let release = match resp.json().await {
                            Ok(value) => {
                                log::info!("[Update] Reqwest fallback succeeded");
                                value
                            }
                            Err(err) => {
                                log::error!("[Update] Failed to parse response: {}", err);

                                // Store last check time even on error
                                let update_state = app.state::<UpdateState>();
                                let now = chrono::Local::now();
                                *update_state.last_check_time.lock().unwrap() = Some(now);

                                // Persist to store
                                let store = app.state::<StoreManager>();
                                let _ = store.set_last_update_check_timestamp(now.timestamp());

                                send_status(
                                    "error",
                                    Some(
                                        format!("Failed to parse update response: {}", err)
                                            .as_str(),
                                    ),
                                );

                                // Show error notification
                                if store.get_show_notifications() {
                                    let _ = app
                                        .notification()
                                        .builder()
                                        .title("Copilot Tracker")
                                        .body(
                                            "Failed to check for updates. Please try again later.",
                                        )
                                        .show();
                                }
                                let _ = rebuild_tray_menu(&app, None);
                                return Ok(());
                            }
                        };

                        process_release_data(&app, release, &send_status)?;
                    }
                    Err(reqwest_err) => {
                        log::error!(
                            "[Update] Both methods failed (webview + reqwest). Final error: {}",
                            reqwest_err
                        );

                        // Store last check time even on error
                        let update_state = app.state::<UpdateState>();
                        let now = chrono::Local::now();
                        *update_state.last_check_time.lock().unwrap() = Some(now);

                        // Persist to store
                        let store = app.state::<StoreManager>();
                        let _ = store.set_last_update_check_timestamp(now.timestamp());

                        send_status(
                            "error",
                            Some(
                                format!(
                                    "Update check failed (webview: {}, reqwest: {})",
                                    webview_err, reqwest_err
                                )
                                .as_str(),
                            ),
                        );

                        // Show error notification
                        if store.get_show_notifications() {
                            let _ = app
                                .notification()
                                .builder()
                                .title("Copilot Tracker")
                                .body("Failed to check for updates. Please check your connection.")
                                .show();
                        }
                        let _ = rebuild_tray_menu(&app, None);
                        return Ok(());
                    }
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(())
    }
}

#[tauri::command]
fn get_update_info(app: AppHandle) -> Result<Option<UpdateInfo>, String> {
    // Return the in-memory latest update info (if any).
    // This prevents missed "update:available" events by allowing the renderer
    // to pull current state on startup.
    let update_state = app.state::<UpdateState>();
    let latest = update_state.latest.lock().unwrap().clone();
    Ok(latest)
}

// ============================================================================
// IPC Commands - Tray
// ============================================================================

#[tauri::command]
fn update_tray_usage(
    app: AppHandle,
    state: tauri::State<TrayState>,
    used: f64,
    limit: u32,
) -> Result<(), String> {
    let store = app.state::<StoreManager>();
    let format = store.get_tray_icon_format();
    update_tray_icon(&app, &state, used, limit, &format)
}

// ============================================================================
// State Managers
// ============================================================================

#[derive(Clone)]
struct AuthManagerState {
    auth_manager: Arc<Mutex<AuthManager>>,
}

// ============================================================================
// Main Application
// ============================================================================

fn main() {
    // Initialize logger
    env_logger::init();

    // Create tray icon renderer with platform-specific DPI scaling
    // macOS/Linux: Fixed 2x scale for Retina/HiDPI
    // Windows: Use 2x for consistency (Tauri handles DPI scaling automatically)
    let scale_factor = 2;

    let renderer = TrayIconRenderer::from_font_bytes_with_scale(
        include_bytes!("../assets/fonts/RobotoMono-Medium.ttf"),
        14.0,
        scale_factor,
    )
    .expect("renderer from font bytes");
    let renderer = Arc::new(renderer);
    let tray_state = TrayState {
        tray: Mutex::new(None),
        renderer: Arc::clone(&renderer),
        last_menu_rebuild: Mutex::new(std::time::Instant::now()),
    };

    // Create auth manager state
    let auth_manager_state = AuthManagerState {
        auth_manager: Arc::new(Mutex::new(AuthManager::new())),
    };

    // CONTEXT GENERATION & STORE INITIALIZATION
    // We generate the context here to access config/identifier, then pass it to the runner
    let context = tauri::generate_context!();
    let identifier = context.config().identifier.clone();

    // Resolve app directory manually using helper (Standard paths for Win/Mac/Linux)
    let app_dir = resolve_app_dir(&identifier);
    log::info!("Resolved app data directory: {:?}", app_dir);

    // Initialize StoreManager BEFORE the builder runs
    // This ensures state is available for plugins and early lifecycle events
    let store_manager = StoreManager::new(app_dir).expect("Failed to initialize StoreManager");

    tauri::Builder::default()
        // Manage state (CRITICAL FIX: StoreManager managed here, not in setup)
        .manage(store_manager)
        .manage(tray_state)
        .manage(auth_manager_state)
        .manage(UpdateState::default())
        .manage(PollingState::new())
        // Register plugins
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--no-dev"]), // Pass flag to prevent dev mode detection on autostart
        ))
        // Register IPC commands
        .invoke_handler(tauri::generate_handler![
            // Auth commands
            show_auth_window,
            perform_auth_extraction,
            check_auth_status,
            logout,
            copilot_tracker::hidden_webview_event,
            // Usage commands
            fetch_usage,
            force_fetch_usage,
            get_cached_usage,
            predict_eom_usage,
            days_until_limit,
            get_cached_usage_data,
            // Settings commands
            get_settings,
            update_settings,
            reset_settings,
            set_launch_at_login,
            // Tray commands
            update_tray_usage,
            // Widget commands
            toggle_widget,
            hide_widget,
            minimize_widget,
            is_widget_visible,
            set_widget_position,
            get_widget_position,
            set_widget_pinned,
            is_widget_pinned,
            is_widget_enabled,
            set_widget_enabled,
            // App commands
            get_app_version,
            hide_main_window,
            open_external_url,
            check_for_updates,
            get_update_info,
        ])
        // Setup application
        .setup(move |app| {
            log::info!("Initializing Copilot Tracker (Tauri)");

            // Hide from dock on macOS immediately on startup (before window creation)
            // This prevents the dock icon from appearing briefly on launch
            #[cfg(target_os = "macos")]
            {
                log::info!("Setting activation policy to accessory on macOS startup");
                // Use set_activation_policy to completely hide from dock
                // NSApplicationActivationPolicyAccessory = 1 means no dock icon
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }

            // NOTE: StoreManager is already initialized and managed in Builder::default() above.
            // init_store_manager(app.handle())?; <--- REMOVED (Caused the race condition on Windows)

            log::info!("StoreManager initialized and managed successfully (in main)");

            // Now safe to build tray menu (it accesses StoreManager)
            let menu = build_tray_menu(app.handle(), None)?;

            let theme_preference = app.state::<StoreManager>().get_settings().theme;
            let color = tray_text_color(&theme_preference);
            let initial_image = renderer.render_text_only("1", 16, color).into_tauri_image();

            let tray = TrayIconBuilder::new()
                .icon(initial_image)
                .menu(&menu)
                .icon_as_template(cfg!(target_os = "macos"))
                .tooltip("Copilot Tracker")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        // Save widget state before shutdown
                        let store = app.state::<StoreManager>();
                        let widget_enabled = store.get_widget_enabled();

                        if let Some(widget) = app.get_webview_window("widget") {
                            let is_visible = widget.is_visible().unwrap_or(false);
                            let position = widget
                                .outer_position()
                                .map(|pos| WidgetPosition { x: pos.x, y: pos.y })
                                .unwrap_or_else(|_| store.get_widget_position());

                            if let Err(err) = store.set_widget_visible(is_visible) {
                                log::error!(
                                    "[Shutdown] Failed to persist widget visibility: {}",
                                    err
                                );
                            }

                            if let Err(err) = store.set_widget_position(position.clone()) {
                                log::error!(
                                    "[Shutdown] Failed to persist widget position: {}",
                                    err
                                );
                            }

                            log::info!(
                                "[Shutdown] Widget enabled={}, saving visibility={}, position=({}, {})",
                                widget_enabled,
                                is_visible,
                                position.x,
                                position.y
                            );
                        } else {
                            log::info!("[Shutdown] Widget window not found, widget_enabled={}", widget_enabled);
                        }

                        // Stop background polling before app exit
                        let polling_state = app.state::<PollingState>();
                        polling_state.stop_polling();
                        log::info!("[Shutdown] Background polling stopped, exiting app");
                        app.exit(0);
                    }
                    "open_dashboard" => {
                        if let Some(window) = app.get_webview_window("main") {
                            // Restore to taskbar/dock before showing
                            #[cfg(target_os = "windows")]
                            {
                                let _ = window.set_skip_taskbar(false);
                            }
                            #[cfg(target_os = "macos")]
                            {
                                // Set activation policy to regular to show in dock
                                let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
                                let _ = app.show();
                            }
                            // Linux doesn't need skipTaskbar manipulation
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                        let _ = app.emit("navigate", "dashboard");
                    }
                    "open_billing" => {
                        let _ = app.opener().open_url(
                            "https://github.com/settings/billing/premium_requests_usage",
                            None::<&str>,
                        );
                    }
                    "refresh" => {
                        // Use hidden webview to silently fetch fresh usage data
                        log::info!("Refresh triggered - using hidden webview to fetch fresh data");
                        let app_handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let mut usage_manager = UsageManager::new();
                            match usage_manager.fetch_usage(&app_handle).await {
                                Ok(summary) => {
                                    log::info!("Refresh successful: {}/{} ({}%)",
                                        summary.used, summary.limit, summary.percentage);

                                    // Rebuild tray menu to show updated timestamp
                                    let update_state = app_handle.state::<UpdateState>();
                                    let latest = update_state.latest.lock().unwrap();
                                    let _ = rebuild_tray_menu(&app_handle, latest.as_ref());

                                    // Show notification on success (if enabled)
                                    if let Some(store) = app_handle.try_state::<StoreManager>() {
                                        if store.get_show_notifications() {
                                            let _ = app_handle
                                                .notification()
                                                .builder()
                                                .title("Copilot Tracker")
                                                .body(format!("Usage updated: {} / {} requests", summary.used, summary.limit))
                                                .show();
                                        }
                                    }
                                }
                                Err(e) => {
                                    log::error!("Refresh failed: {}", e);
                                }
                            }
                        });
                    }
                    "settings" => {
                        if let Some(window) = app.get_webview_window("main") {
                            // Restore to taskbar/dock before showing
                            #[cfg(target_os = "windows")]
                            {
                                let _ = window.set_skip_taskbar(false);
                            }
                            #[cfg(target_os = "macos")]
                            {
                                // Set activation policy to regular to show in dock
                                let _ = app.set_activation_policy(tauri::ActivationPolicy::Regular);
                                let _ = app.show();
                            }
                            // Linux doesn't need skipTaskbar manipulation
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                        let _ = app.emit("navigate", "settings");
                    }
                    "toggle_widget" => {
                        let _ = toggle_widget(app.clone());
                        // Rebuild tray menu to update widget label
                        let update_state = app.state::<UpdateState>();
                        let latest = update_state.latest.lock().unwrap();
                        let _ = rebuild_tray_menu(app, latest.as_ref());
                    }
                    "update_check" => {
                        let info = app.state::<UpdateState>().latest.lock().unwrap().clone();
                        if let Some(info) = info {
                            let _ = app.opener().open_url(info.release_url, None::<&str>);
                        } else {
                            let app_handle = app.clone();
                            tauri::async_runtime::spawn(async move {
                                let _ = check_for_updates(app_handle).await;
                            });
                        }
                    }
                    "launch_at_login" => {
                        let store = app.state::<StoreManager>();
                        let enabled = !store.get_launch_at_login();
                        let _ = set_launch_at_login(app.clone(), enabled);
                        let _ = app.emit("settings:changed", store.get_settings());
                    }
                    id if id.starts_with("prediction_period:") => {
                        if let Ok(value) = id.split(':').nth(1).unwrap_or("0").parse::<u32>() {
                            let store = app.state::<StoreManager>();
                            let mut settings = store.get_settings();
                            settings.prediction_period = value;
                            let _ = update_settings(app.clone(), settings);
                        }
                    }
                    id if id.starts_with("refresh_interval:") => {
                        if let Ok(value) = id.split(':').nth(1).unwrap_or("0").parse::<u32>() {
                            let store = app.state::<StoreManager>();
                            let mut settings = store.get_settings();
                            let old_interval = settings.refresh_interval;
                            settings.refresh_interval = value;
                            let _ = update_settings(app.clone(), settings);

                            // Restart background polling with new interval
                            if old_interval != value {
                                let polling_state = app.state::<PollingState>();
                                let interval_seconds = value.max(10); // Minimum 10 seconds
                                polling_state.restart_polling(app.clone(), interval_seconds as u64);
                                log::info!("[Settings] Restarted polling with new interval: {}s (was: {}s)", interval_seconds, old_interval);
                            }
                        }
                    }
                    "github_repo" => {
                        let _ = app.opener().open_url(
                            "https://github.com/bizzkoot/copilot-tracker",
                            None::<&str>,
                        );
                    }
                    "github_issues" => {
                        let _ = app.opener().open_url(
                            "https://github.com/bizzkoot/copilot-tracker/issues",
                            None::<&str>,
                        );
                    }
                    _ => {}
                })
                .on_tray_icon_event(move |tray, event| {
                    if let TrayIconEvent::DoubleClick {
                        button: MouseButton::Left,
                        ..
                    } = event
                    {
                        log::info!("Tray icon double-clicked - toggling widget");
                        let app = tray.app_handle();
                        let _ = toggle_widget(app.clone());
                        // Rebuild tray menu to update widget label
                        let update_state = app.state::<UpdateState>();
                        let latest = update_state.latest.lock().unwrap();
                        let _ = rebuild_tray_menu(app, latest.as_ref());
                    }
                })
                // Note: Tray icon single click intentionally does NOT show dashboard
                // Dashboard only opens via "Open Dashboard" menu item
                // Double click toggles the widget visibility
                .build(app)?;

            // Store tray icon in state
            let tray_state = app.state::<TrayState>();
            *tray_state
                .tray
                .lock()
                .map_err(|e| format!("Failed to acquire tray lock: {}", e))? = Some(tray);

            // Listen for usage updates and update tray
            let app_handle = app.handle();
            let listener_handle = app_handle.clone();
            app_handle.listen("usage:updated", move |event| {
                let payload = event.payload();
                log::info!("[TrayListener] Received usage:updated event, payload: {}", payload);

                // usage:updated emits UsageSummary, not UsagePayload
                let parsed: copilot_tracker::UsageSummary = match serde_json::from_str(payload) {
                    Ok(parsed) => parsed,
                    Err(e) => {
                        log::error!("[TrayListener] Failed to parse usage:updated event: {}", e);
                        return;
                    }
                };
                log::info!("[TrayListener] Updating tray icon to: {} / {} ({}%)",
                    parsed.used, parsed.limit, parsed.percentage);
                let _ = update_tray_icon_from_store(&listener_handle);
                // Rebuild menu with fresh data from store (not using update state)
                let update_state = listener_handle.state::<UpdateState>();
                let latest = update_state.latest.lock().unwrap();
                let _ = rebuild_tray_menu(&listener_handle, latest.as_ref());
                log::info!("[TrayListener] Tray icon and menu updated successfully");
            });

            // Prevent app from quitting when main window is closed (hide instead)
            let main_window = app.get_webview_window("main").ok_or("Main window not found")?;
            let app_handle_close = app.handle().clone();
            main_window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    // Prevent the window from actually closing
                    api.prevent_close();
                    // Just hide the window instead
                    let app_handle = app_handle_close.clone();
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.hide();

                        // Hide app from dock/taskbar when window closes (cross-platform)
                        // macOS: Set activation policy to accessory to remove dock icon
                        #[cfg(target_os = "macos")]
                        {
                            // Keep the app activation policy as accessory (hide dock icon),
                            // but DO NOT call `app.hide()` here — hiding the entire app
                            // also hides the floating widget window. The widget's
                            // visibility should be managed independently by its own
                            // commands/close handlers.
                            let _ = app_handle.set_activation_policy(tauri::ActivationPolicy::Accessory);
                        }

                        // Windows: Hide from taskbar using skipTaskbar
                        #[cfg(target_os = "windows")]
                        {
                            let _ = window.set_skip_taskbar(true);
                        }

                        // Linux: Window manager handles taskbar visibility automatically
                    }
                }
            });

            // Load initial usage and update tray
            let store = app.state::<StoreManager>();
            let (used, limit) = store.get_usage();
            let is_authenticated = store.is_authenticated();

            log::info!("Startup: used={}, limit={}, authenticated={}", used, limit, is_authenticated);

            // Always emit if authenticated, even if used=0 (might have zero usage but still have history)
            if is_authenticated {
                if used > 0.0 {
                    let _ = update_tray_icon_from_store(app.handle());
                }

                // Emit initial usage data to frontend (delayed to allow frontend listeners to attach)
                let app_handle_for_emit = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    // Wait for frontend to initialize listeners
                    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

                    let store = app_handle_for_emit.state::<StoreManager>();
                    let (used, limit) = store.get_usage();

                    log::info!("About to emit startup data: used={}, limit={}", used, limit);

                    let remaining = (limit as f64 - used).max(0.0);
                    let percentage = if limit > 0 {
                        (used / limit as f64 * 100.0) as f32
                    } else {
                        0.0
                    };
                    let summary = copilot_tracker::UsageSummary {
                        used,
                        limit,
                        remaining,
                        percentage,
                        timestamp: chrono::Utc::now().timestamp(),
                    };

                    let history = UsageManager::get_cached_history(&app_handle_for_emit);
                    let store = app_handle_for_emit.state::<StoreManager>();
                    let settings = store.get_settings();
                    let prediction = UsageManager::predict_usage_from_history(
                        &history,
                        used,
                        limit,
                        settings.prediction_period,
                    );

                    log::info!("History entries: {}", history.len());

                    let payload = copilot_tracker::UsagePayload {
                        summary,
                        history,
                        prediction,
                    };

                    log::info!("Emitting initial usage:data on startup");
                    match app_handle_for_emit.emit("usage:data", payload) {
                        Ok(_) => log::info!("Successfully emitted startup usage:data"),
                        Err(e) => log::error!("Failed to emit startup usage:data: {:?}", e),
                    }
                });
            }

            // Show first-run notification on Windows to help users find tray icon
            // This shows every launch until the user authenticates for the first time
            #[cfg(target_os = "windows")]
            {
                let store = app.state::<StoreManager>();
                if !store.is_authenticated() && store.get_show_notifications() {
                    let _ = app
                        .notification()
                        .builder()
                        .title("Copilot Tracker - Tray Icon")
                        .body("Look for the Copilot Tracker icon in your system tray (bottom-right corner). Click the arrow to pin it for easy access.")
                        .show();
                }
            }

            // Get settings for startup configuration
            let store = app.state::<StoreManager>();
            let settings = store.get_settings();

            // CRITICAL: Start background polling AFTER setup completes to prevent race condition
            // Spawn a delayed task to ensure polling starts after initialization is complete
            let app_for_polling = app_handle.clone();
            let polling_interval = settings.refresh_interval.max(10) as u64;
            tauri::async_runtime::spawn(async move {
                // Small delay to ensure setup() completes and all state is managed
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

                let polling_state = app_for_polling.state::<PollingState>();
                polling_state.restart_polling(app_for_polling.clone(), polling_interval);
                log::info!("[Startup] Started background polling with interval: {}s", polling_interval);
            });

            // Initialize widget state from settings
            let store = app.state::<StoreManager>();
            let widget_enabled = store.get_widget_enabled();
            let widget_visible = store.get_widget_visible();
            let widget_pinned = store.get_widget_pinned();
            let widget_position = store.get_widget_position();

            log::info!("[Startup] Widget state loaded: enabled={}, visible={}, pinned={}, position=({},{})",
                widget_enabled, widget_visible, widget_pinned, widget_position.x, widget_position.y);

            // Restore widget state if enabled
            if widget_enabled {
                if let Some(widget) = app.get_webview_window("widget") {
                    // Set position
                    let _ = widget.set_position(tauri::Position::Physical(
                        tauri::PhysicalPosition {
                            x: widget_position.x,
                            y: widget_position.y
                        }
                    ));

                    // Set pinned state
                    let _ = widget.set_always_on_top(widget_pinned);

                    // Show widget if it was visible (without stealing focus)
                    if widget_visible {
                        log::info!("[Startup] Showing widget (was visible on last shutdown)");
                        let _ = show_widget_without_focus(&widget, widget_pinned);

                        // Rebuild tray menu immediately after showing widget to avoid stale label
                        let update_state = app.state::<UpdateState>();
                        let latest = update_state.latest.lock().unwrap();
                        let _ = rebuild_tray_menu(app.handle(), latest.as_ref());
                        drop(latest);
                    } else {
                        log::info!("[Startup] Widget enabled but was hidden on last shutdown, not showing");
                    }

                    log::info!("[Startup] Widget state restored successfully");
                } else {
                    log::warn!("[Startup] Widget enabled but window not found!");
                }
            } else {
                log::info!("[Startup] Widget is disabled, not restoring");
            }

            // Update tray menu after widget restoration to keep label in sync
            let update_state = app.state::<UpdateState>();
            let latest = update_state.latest.lock().unwrap();
            let _ = rebuild_tray_menu(app.handle(), latest.as_ref());
            // Explicitly drop the lock before moving on
            drop(latest);

            if !tauri::is_dev() {
                let app_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    let _ = check_for_updates(app_handle).await;
                });
            }

            // Show window on startup if startMinimized is false
            if !settings.start_minimized {
                if let Some(window) = app.get_webview_window("main") {
                    #[cfg(target_os = "macos")]
                    {
                        // Set activation policy to regular to show in dock
                        app.set_activation_policy(tauri::ActivationPolicy::Regular);
                        let _ = app.show();
                    }
                    let _ = window.show();
                    log::info!("Showing window on startup (startMinimized is false)");
                }
            } else {
                log::info!("Window hidden on startup (startMinimized is true)");
            }

            log::info!("Copilot Tracker initialized successfully");

            Ok(())
        })
        .run(context)
        .expect("error while running tauri app");
}
