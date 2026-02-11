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

use copilot_tracker::{
    init_store_manager, AuthManager, StoreManager, TrayIconRenderer, UsageManager, WidgetPosition,
};

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
            
            if *last_interval == interval_seconds &&
                now.duration_since(*last_restart) < std::time::Duration::from_millis(POLLING_RESTART_DEBOUNCE_MS) {
                log::debug!("[PollingState] Skipping duplicate restart request (interval: {}s)", interval_seconds);
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
                        log::warn!("[PollingState] Cancel channel full, task may already be stopping");
                    }
                    Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                        log::warn!("[PollingState] Cancel channel closed, task already stopped");
                    }
                }
            }

            // Start new polling task
            let cancel_tx = UsageManager::start_polling(app, interval_seconds);
            *guard = Some(cancel_tx);
            log::info!("[PollingState] Started polling with interval: {}s", interval_seconds);
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
}

/// Format tray icon text based on the specified format
fn format_tray_text(used: u32, limit: u32, format: &str) -> String {
    // Handle unauthenticated state (limit == 0)
    if limit == 0 {
        return used.to_string();
    }

    let remaining = limit.saturating_sub(used);
    let percentage = (used as f32 / limit as f32) * 100.0;
    let remaining_pct = 100.0 - percentage;

    match format {
        "current" => used.to_string(),
        "currentTotal" => format!("{used}/{limit}"),
        "remainingTotal" => format!("{remaining}/{limit}"),
        "percentage" => format!("{:.0}%", percentage),
        "remainingPercent" => format!("{:.0}%", remaining_pct),
        "combined" => format!("{used}/{limit} ({:.0}%)", percentage),
        "remainingCombined" => format!("{remaining}/{limit} ({:.0}%)", remaining_pct),
        _ => format!("{used}/{limit}"), // fallback to current default
    }
}

fn update_tray_icon(state: &TrayState, used: u32, limit: u32, format: &str) -> Result<(), String> {
    let text = format_tray_text(used, limit, format);

    let image = state
        .renderer
        .render_text_only(&text, 16)
        .into_tauri_image();

    let tray_guard = state.tray.lock().map_err(|_| "tray lock poisoned".to_string())?;
    let tray = tray_guard.as_ref().ok_or("tray not initialized".to_string())?;
    tray.set_icon(Some(image)).map_err(|err| err.to_string())
}

/// Helper to update tray icon using current settings from store
fn update_tray_icon_from_store(app: &AppHandle) -> Result<(), String> {
    let store = app.state::<StoreManager>();
    let (used, limit) = store.get_usage();
    let format = store.get_tray_icon_format();
    let tray_state = app.state::<TrayState>();
    update_tray_icon(&tray_state, used, limit, &format)
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
    let prediction = UsageManager::predict_usage_from_history(&usage_history, used, limit, settings.prediction_period);
    
    // Calculate metrics for dual-perspective display
    let remaining = limit.saturating_sub(used);
    let percentage_used = if limit > 0 { (used as f32 / limit as f32) * 100.0 } else { 0.0 };
    let percentage_remaining = 100.0 - percentage_used;
    
    // Calculate daily metrics
    let now = chrono::Utc::now();
    let current_day = now.day() as f32;
    let days_in_month = if now.month() == 12 {
        31
    } else {
        let next_month = chrono::NaiveDate::from_ymd_opt(now.year(), now.month() + 1, 1)
            .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(now.year() + 1, 1, 1).unwrap());
        let current_month =
            chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap();
        (next_month - current_month).num_days() as u32
    };
    let days_remaining = days_in_month as f32 - current_day;
    let daily_rate = if current_day > 0.0 { used as f32 / current_day } else { 0.0 };
    // Floor the daily budget to be conservative (synced with Dashboard)
    let daily_budget = if days_remaining > 0.0 { (remaining as f32 / days_remaining).floor() } else { 0.0 };

    let menu = Menu::new(app).map_err(|e| e.to_string())?;
    
    // === USAGE OVERVIEW SECTION ===
    // === USAGE OVERVIEW SECTION ===
    let overview_header = MenuItem::with_id(app, "overview_header", "📊 QUOTA STATUS", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    menu.append(&overview_header).map_err(|e| e.to_string())?;
    
    if limit > 0 {
        let quota_line = MenuItem::with_id(app, "quota_line", 
            format!("   {used} / {limit} requests ({percentage_used:.0}%)"), true, None::<&str>)
            .map_err(|e| e.to_string())?;
        menu.append(&quota_line).map_err(|e| e.to_string())?;
        
        let remaining_line = MenuItem::with_id(app, "remaining_line", 
            format!("   {remaining} remaining ({percentage_remaining:.0}%)"), true, None::<&str>)
            .map_err(|e| e.to_string())?;
        menu.append(&remaining_line).map_err(|e| e.to_string())?;
    } else {
        let loading_line = MenuItem::with_id(app, "loading_line", "▶ Loading data...", true, None::<&str>)
            .map_err(|e| e.to_string())?;
        menu.append(&loading_line).map_err(|e| e.to_string())?;
    }
    
    menu.append(&PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    
    // === CONSUMPTION RATE SECTION ===
    if limit > 0 && current_day > 0.0 {
        let rate_header = MenuItem::with_id(app, "rate_header", "📈 ACTIVITY", true, None::<&str>)
            .map_err(|e| e.to_string())?;
        menu.append(&rate_header).map_err(|e| e.to_string())?;
        
        let daily_rate_line = MenuItem::with_id(app, "daily_rate_line", 
            format!("   ⚡ Usage: {:.0} req/day", daily_rate), true, None::<&str>)
            .map_err(|e| e.to_string())?;
        menu.append(&daily_rate_line).map_err(|e| e.to_string())?;
        
        if daily_budget > 0.0 {
            let budget_line = MenuItem::with_id(app, "budget_line", 
                format!("   🎯 Budget: {:.0} req/day", daily_budget), true, None::<&str>)
                .map_err(|e| e.to_string())?;
            menu.append(&budget_line).map_err(|e| e.to_string())?;
        }

        let days_left_line = MenuItem::with_id(app, "days_left_line", 
            format!("   🗓️ {days_remaining:.0} days remaining"), true, None::<&str>)
            .map_err(|e| e.to_string())?;
        menu.append(&days_left_line).map_err(|e| e.to_string())?;
        
        menu.append(&PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    }

    // === PREDICTION SECTION ===
    if let Some(prediction) = prediction {
        let prediction_header = MenuItem::with_id(app, "prediction_header", "🔮 FORECAST", true, None::<&str>)
            .map_err(|e| e.to_string())?;
        menu.append(&prediction_header).map_err(|e| e.to_string())?;
        
        let status_label = if prediction.predicted_monthly_requests > limit {
            format!("   ⚠️ Exceed by {}", prediction.predicted_monthly_requests - limit)
        } else {
            format!("   ✅ Safe ({} left)", limit - prediction.predicted_monthly_requests)
        };
        let status_line = MenuItem::with_id(app, "status_line", status_label, true, None::<&str>)
            .map_err(|e| e.to_string())?;
        menu.append(&status_line).map_err(|e| e.to_string())?;

        let confidence_icon = match prediction.confidence_level.as_str() {
            "high" => "🟢",
            "medium" => "🟡",
            _ => "🔴",
        };
        let forecast_line = MenuItem::with_id(app, "forecast_line",
            format!("   {confidence_icon} Expected: {} total", prediction.predicted_monthly_requests),
            true, None::<&str>)
            .map_err(|e| e.to_string())?;
        menu.append(&forecast_line).map_err(|e| e.to_string())?;
    } else {
        let prediction_header = MenuItem::with_id(app, "prediction_header", "🔮 FORECAST", true, None::<&str>)
            .map_err(|e| e.to_string())?;
        menu.append(&prediction_header).map_err(|e| e.to_string())?;
        let no_data = MenuItem::with_id(app, "no_data", "   Insufficient data", true, None::<&str>)
            .map_err(|e| e.to_string())?;
        menu.append(&no_data).map_err(|e| e.to_string())?;
    }
    
    menu.append(&PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;

    // === USAGE HISTORY SECTION ===
    let history_submenu =
        Submenu::with_id(app, "usage_history", "📜 Usage History ▶", true).map_err(|e| e.to_string())?;
    if !usage_history.is_empty() {
        for entry in usage_history.iter().take(7) {
            let date = chrono::DateTime::from_timestamp(entry.timestamp, 0)
                .map(|dt| dt.date_naive())
                .unwrap_or_else(|| chrono::Utc::now().date_naive());
            let label = format!("{}: {} req", date.format("%b %d"), entry.used);
            let item = MenuItem::new(app, label, false, None::<&str>).map_err(|e| e.to_string())?;
            history_submenu.append(&item).map_err(|e| e.to_string())?;
        }
    } else {
        let item =
            MenuItem::new(app, "No history yet", false, None::<&str>).map_err(|e| e.to_string())?;
        history_submenu.append(&item).map_err(|e| e.to_string())?;
    }
    menu.append(&history_submenu).map_err(|e| e.to_string())?;

    let prediction_period_submenu = Submenu::with_id(app, "prediction_period", "Prediction Period", true)
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
        prediction_period_submenu.append(&item).map_err(|e| e.to_string())?;
    }
    menu.append(&prediction_period_submenu).map_err(|e| e.to_string())?;

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

    let open_billing =
        MenuItem::with_id(app, "open_billing", "Open Billing", true, None::<&str>)
            .map_err(|e| e.to_string())?;
    menu.append(&open_billing).map_err(|e| e.to_string())?;

    let refresh = MenuItem::with_id(app, "refresh", "Refresh", true, None::<&str>)
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
    let widget_label = if widget_visible { "Hide Widget" } else { "Show Widget" };
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

    menu.append(&PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;

    let launch_label = if settings.launch_at_login { "☑️ Launch at Login" } else { "☐ Launch at Login" };
    let launch_item = MenuItem::with_id(app, "launch_at_login", launch_label, true, None::<&str>)
        .map_err(|e| e.to_string())?;
    menu.append(&launch_item).map_err(|e| e.to_string())?;

    menu.append(&PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;

    let version_item =
        MenuItem::with_id(app, "version", format!("Version {}", version), false, None::<&str>)
            .map_err(|e| e.to_string())?;
    menu.append(&version_item).map_err(|e| e.to_string())?;

    let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)
        .map_err(|e| e.to_string())?;
    menu.append(&quit_i).map_err(|e| e.to_string())?;

    Ok(menu)
}

fn rebuild_tray_menu(app: &AppHandle, update: Option<&UpdateInfo>) -> Result<(), String> {
    let tray_state = app.state::<TrayState>();
    
    // Debounce: Don't rebuild more than once per second
    {
        let mut last_rebuild = tray_state.last_menu_rebuild.lock().map_err(|_| "lock poisoned")?;
        let now = std::time::Instant::now();
        if now.duration_since(*last_rebuild).as_millis() < 1000 {
            log::debug!("Skipping tray menu rebuild - too soon since last rebuild");
            return Ok(());
        }
        *last_rebuild = now;
    }
    
    let menu = build_tray_menu(app, update)?;
    let tray_guard = tray_state.tray.lock().map_err(|_| "tray lock poisoned".to_string())?;
    let tray = tray_guard.as_ref().ok_or("tray not initialized".to_string())?;
    
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
async fn check_auth_status(
    app: AppHandle,
) -> Result<copilot_tracker::AuthState, String> {
    let store = app.state::<StoreManager>();
    let customer_id = store.get_customer_id();

    let is_authenticated = customer_id.is_some();
    let state_str = if is_authenticated { "authenticated" } else { "unauthenticated" };
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
fn get_cached_usage(
    app: AppHandle,
) -> Result<copilot_tracker::UsageSummary, String> {
    UsageManager::get_cached_usage(&app)
}

#[tauri::command]
fn predict_eom_usage(
    app: AppHandle,
) -> Result<u32, String> {
    UsageManager::predict_eom_usage(&app)
}

#[tauri::command]
fn days_until_limit(
    app: AppHandle,
) -> Result<Option<i64>, String> {
    UsageManager::days_until_limit(&app)
}

#[tauri::command]
fn get_cached_usage_data(
    app: AppHandle,
) -> Result<Option<copilot_tracker::UsagePayload>, String> {
    let store = app.state::<StoreManager>();
    let (used, limit) = store.get_usage();
    let is_authenticated = store.is_authenticated();
    
    if !is_authenticated {
        return Ok(None);
    }
    
    let remaining = limit.saturating_sub(used);
    let percentage = if limit > 0 {
        (used as f32 / limit as f32) * 100.0
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
    let prediction = UsageManager::predict_usage_from_history(&history, used, limit, settings.prediction_period);
    
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
fn get_settings(
    app: AppHandle,
) -> Result<copilot_tracker::AppSettings, String> {
    let store = app.state::<StoreManager>();
    Ok(store.get_settings())
}

#[tauri::command]
fn get_app_version(
    app: AppHandle,
) -> Result<String, String> {
    Ok(app.package_info().version.to_string())
}

#[tauri::command]
fn update_settings(
    app: AppHandle,
    settings: copilot_tracker::AppSettings,
) -> Result<(), String> {
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
    
    log::info!("Store reset complete, customer_id is now: {:?}", store.get_customer_id());
    
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
        remaining: limit.saturating_sub(used),
        percentage: if limit > 0 { (used as f32 / limit as f32) * 100.0 } else { 0.0 },
        timestamp: chrono::Utc::now().timestamp(),
    };
    let _ = app.emit("usage:updated", &summary);
    log::info!("Emitted usage:updated to reset tray icon");

    // Update tray icon directly to "1" (unauthenticated state)
    let tray_state = app.state::<TrayState>();
    let _ = update_tray_icon(&tray_state, 1, 0, "currentTotal");
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
fn set_launch_at_login(
    app: AppHandle,
    enabled: bool,
) -> Result<(), String> {
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
    app.opener().open_url(url, None::<&str>).map_err(|e| e.to_string())
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
            let _ = store.set_widget_visible(false);
            Ok(false)
        } else {
            widget.show().map_err(|e| e.to_string())?;
            let _ = store.set_widget_visible(true);
            Ok(true)
        }
    } else {
        Err("Widget window not found".to_string())
    }
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
    if let Some(widget) = app.get_webview_window("widget") {
        widget.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }))
            .map_err(|e| e.to_string())?;
        // Save position to settings
        let store = app.state::<StoreManager>();
        let _ = store.set_widget_position(WidgetPosition { x, y });
    }
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
        widget.set_always_on_top(pinned).map_err(|e| e.to_string())?;
        // Save pin state to settings
        let store = app.state::<StoreManager>();
        let _ = store.set_widget_pinned(pinned);
        // Emit event to notify widget window
        let _ = app.emit("widget:set-pin", pinned);
    }
    Ok(())
}

#[tauri::command]
async fn is_widget_pinned(app: AppHandle) -> Result<bool, String> {
    let store = app.state::<StoreManager>();
    Ok(store.get_widget_pinned())
}

#[tauri::command]
async fn check_for_updates(app: AppHandle) -> Result<(), String> {
    let send_status = |status: &str, message: Option<String>| {
        let payload = UpdateCheckStatus {
            status: status.to_string(),
            message,
        };
        let _ = app.emit("update:checked", payload);
    };

    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/repos/bizzkoot/copilot-tracker/releases/latest")
        .header("User-Agent", "Copilot-Tracker-App")
        .send()
        .await
        .map_err(|e| e.to_string());

    let response = match response {
        Ok(response) => response,
        Err(err) => {
            send_status("error", Some(format!("Network request failed: {err}")));
            return Ok(());
        }
    };

    if !response.status().is_success() {
        send_status("error", Some("Update check failed".to_string()));
        return Ok(());
    }

    let release: serde_json::Value = match response.json().await {
        Ok(value) => value,
        Err(err) => {
            send_status("error", Some(format!("Failed to parse update response: {err}")));
            return Ok(());
        }
    };
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
            send_status("error", Some("Invalid version format".to_string()));
            return Ok(());
        }
    };
    let current = match semver::Version::parse(&current_version) {
        Ok(version) => version,
        Err(_) => {
            send_status("error", Some("Invalid version format".to_string()));
            return Ok(());
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
            release_name: release.get("name").and_then(|v| v.as_str()).map(|s| s.to_string()),
            release_notes: release.get("body").and_then(|v| v.as_str()).map(|s| s.to_string()),
            release_date: release.get("published_at").and_then(|v| v.as_str()).map(|s| s.to_string()),
        };

        let update_state = app.state::<UpdateState>();
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

        let _ = rebuild_tray_menu(&app, Some(&info));
    } else {
        let update_state = app.state::<UpdateState>();
        *update_state.latest.lock().unwrap() = None;
        send_status("none", None);
        let _ = rebuild_tray_menu(&app, None);
    }

    Ok(())
}

// ============================================================================
// IPC Commands - Tray
// ============================================================================

#[tauri::command]
fn update_tray_usage(
    app: AppHandle,
    state: tauri::State<TrayState>,
    used: u32,
    limit: u32,
) -> Result<(), String> {
    let store = app.state::<StoreManager>();
    let format = store.get_tray_icon_format();
    update_tray_icon(&state, used, limit, &format)
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

    tauri::Builder::default()
        // Manage state
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
            is_widget_visible,
            set_widget_position,
            get_widget_position,
            set_widget_pinned,
            is_widget_pinned,
            // App commands
            get_app_version,
            hide_main_window,
            open_external_url,
            check_for_updates,
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

            // CRITICAL: Initialize store manager FIRST before any operations that need it
            // This must happen before building tray menu, creating tray icon, or starting polling
            let app_handle = app.handle();
            init_store_manager(app_handle)?;
            log::info!("StoreManager initialized and managed successfully");

            // Now safe to build tray menu (it accesses StoreManager)
            let menu = build_tray_menu(app.handle(), None)?;

            let initial_image = renderer.render_text_only("1", 16).into_tauri_image();

            let tray = TrayIconBuilder::new()
                .icon(initial_image)
                .menu(&menu)
                // .icon_as_template(true) // Disabled to prevent system overlays on tray icon
                .tooltip("Copilot Tracker")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
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
                            let _ = app_handle.set_activation_policy(tauri::ActivationPolicy::Accessory);
                            let _ = app_handle.hide();
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
                if used > 0 {
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
                    
                    let remaining = limit.saturating_sub(used);
                    let percentage = if limit > 0 {
                        (used as f32 / limit as f32) * 100.0
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

            // Update tray menu at startup
            let update_state = app.state::<UpdateState>();
            let latest = update_state.latest.lock().unwrap();
            let _ = rebuild_tray_menu(app.handle(), latest.as_ref());
            // Explicitly drop the lock before moving on
            drop(latest);

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
            
            log::info!("Widget state: enabled={}, visible={}, pinned={}, position=({},{})",
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
                    
                    // Show widget if it was visible
                    if widget_visible {
                        let _ = widget.show();
                    }
                    
                    log::info!("Widget state restored successfully");
                }
            }

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
        .run(tauri::generate_context!())
        .expect("error while running tauri app");
}
