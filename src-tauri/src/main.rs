use serde::Deserialize;
use std::sync::{Arc, Mutex};
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Listener, Manager};
use tauri_plugin_http::reqwest;
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_opener::OpenerExt;

use copilot_tracker::{
    init_store_manager,
    AuthManager,
    StoreManager,
    TrayIconRenderer,
    UsageManager,
    UsagePayload,
};

// ============================================================================
// Tray State
// ============================================================================

struct TrayState {
    tray: Mutex<Option<tauri::tray::TrayIcon>>,
    renderer: Arc<TrayIconRenderer>,
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

fn update_tray_icon(state: &TrayState, used: u32, _limit: u32) -> Result<(), String> {
    let image = state
        .renderer
        .render_text(&used.to_string(), 16)
        .into_tauri_image();
    let tray_guard = state.tray.lock().map_err(|_| "tray lock poisoned".to_string())?;
    let tray = tray_guard.as_ref().ok_or("tray not initialized".to_string())?;
    tray.set_icon(Some(image)).map_err(|err| err.to_string())
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
    let prediction = UsageManager::predict_usage_from_history(&usage_history, used, limit);
    let usage_label = if limit > 0 {
        let percentage = (used as f32 / limit as f32) * 100.0;
        format!("Used: {} / {} ({:.1}%)", used, limit, percentage)
    } else {
        "Loading...".to_string()
    };

    let usage_label_item = MenuItem::with_id(app, "usage_label", usage_label, false, None::<&str>)
        .map_err(|e| e.to_string())?;

    let menu = Menu::new(app).map_err(|e| e.to_string())?;
    menu.append(&usage_label_item).map_err(|e| e.to_string())?;
    menu.append(&PredefinedMenuItem::separator(app).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;

    let prediction_item =
        MenuItem::with_id(app, "prediction_label", "📊 Monthly Prediction", false, None::<&str>)
            .map_err(|e| e.to_string())?;
    menu.append(&prediction_item).map_err(|e| e.to_string())?;

    if let Some(prediction) = prediction {
        let percent_of_limit = if limit > 0 {
            ((prediction.predicted_monthly_requests as f32 / limit as f32) * 100.0).round() as u32
        } else {
            0
        };
        let status_label = if prediction.predicted_monthly_requests > limit {
            "⚠️ May exceed limit"
        } else {
            "✅ On track"
        };
        let confidence_icon = match prediction.confidence_level.as_str() {
            "high" => "🟢",
            "medium" => "🟡",
            _ => "🔴",
        };
        let prediction_line = MenuItem::new(
            app,
            format!(
                "   {} requests ({}% of limit)",
                prediction.predicted_monthly_requests,
                percent_of_limit
            ),
            false,
            None::<&str>,
        )
        .map_err(|e| e.to_string())?;
        let status_line = MenuItem::new(
            app,
            format!(
                "   {} | {} {} confidence",
                status_label,
                confidence_icon,
                prediction.confidence_level
            ),
            false,
            None::<&str>,
        )
        .map_err(|e| e.to_string())?;
        let based_on_line = MenuItem::new(
            app,
            format!(
                "   Based on {} day(s) of usage data",
                prediction.days_used_for_prediction
            ),
            false,
            None::<&str>,
        )
        .map_err(|e| e.to_string())?;
        menu.append(&prediction_line).map_err(|e| e.to_string())?;
        menu.append(&status_line).map_err(|e| e.to_string())?;
        menu.append(&based_on_line).map_err(|e| e.to_string())?;
    }

    let history_submenu =
        Submenu::with_id(app, "usage_history", "Usage History", true).map_err(|e| e.to_string())?;
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
    let menu = build_tray_menu(app, update)?;
    let tray_state = app.state::<TrayState>();
    let tray_guard = tray_state.tray.lock().map_err(|_| "tray lock poisoned".to_string())?;
    let tray = tray_guard.as_ref().ok_or("tray not initialized".to_string())?;
    tray.set_menu(Some(menu)).map_err(|e| e.to_string())
}

#[tauri::command]
async fn show_auth_window(
    app: AppHandle,
    state: tauri::State<'_, AuthManagerState>,
) -> Result<bool, String> {
    let mut auth_manager = state.auth_manager.lock().unwrap();
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
    state: tauri::State<'_, AuthManagerState>,
) -> Result<copilot_tracker::UsageSummary, String> {
    let _ = app.emit("usage:loading", true);
    let auth_manager = {
        let guard = state.auth_manager.lock().unwrap();
        (*guard).clone()
    };
    let mut usage_manager = UsageManager::new(auth_manager);
    let result = usage_manager.fetch_usage(&app).await;
    let _ = app.emit("usage:loading", false);

    if let Ok(summary) = &result {
        let history = UsageManager::get_cached_history(&app);
        let prediction = UsageManager::predict_usage_from_history(
            &history,
            summary.used,
            summary.limit,
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
    store.update_settings(|s| {
        *s = settings.clone();
    })?;

    let _ = app.emit("settings:changed", settings.clone());
    let update_state = app.state::<UpdateState>();
    let latest = update_state.latest.lock().unwrap();
    let _ = rebuild_tray_menu(&app, latest.as_ref());

    Ok(())
}

#[tauri::command]
fn reset_settings(app: AppHandle) -> Result<copilot_tracker::AppSettings, String> {
    let store = app.state::<StoreManager>();
    let defaults = store.reset_settings()?;
    let _ = app.emit("settings:changed", defaults.clone());

    let update_state = app.state::<UpdateState>();
    let latest = update_state.latest.lock().unwrap();
    let _ = rebuild_tray_menu(&app, latest.as_ref());

    Ok(defaults)
}

#[tauri::command]
async fn logout(app: AppHandle) -> Result<(), String> {
    let store = app.state::<StoreManager>();
    store.clear_auth()?;
    
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

    #[cfg(target_os = "macos")]
    {
        use tauri_plugin_autostart::ManagerExt;
        let _ = app.autolaunch().enable();
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

        let _ = app
            .notification()
            .builder()
            .title("Copilot Tracker Update Available")
            .body(format!("Version {} is available.", info.version))
            .show();

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
    state: tauri::State<TrayState>,
    used: u32,
    limit: u32,
) -> Result<(), String> {
    update_tray_icon(&state, used, limit)
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

    // Create tray icon renderer
    let renderer = TrayIconRenderer::from_font_bytes(
        include_bytes!("../assets/fonts/Arimo[wght].ttf"),
        12.0,
    )
    .expect("renderer from font bytes");
    let renderer = Arc::new(renderer);
    let tray_state = TrayState {
        tray: Mutex::new(None),
        renderer: Arc::clone(&renderer),
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
        // Register plugins
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        // Register IPC commands
        .invoke_handler(tauri::generate_handler![
            // Auth commands
            show_auth_window,
            perform_auth_extraction,
            check_auth_status,
            logout,
            // Usage commands
            fetch_usage,
            get_cached_usage,
            predict_eom_usage,
            days_until_limit,
            // Settings commands
            get_settings,
            update_settings,
            reset_settings,
            set_launch_at_login,
            // Tray commands
            update_tray_usage,
            // App commands
            get_app_version,
            hide_main_window,
            open_external_url,
            check_for_updates,
        ])
        // Setup application
        .setup(move |app| {
            log::info!("Initializing Copilot Tracker (Tauri)");

            // Initialize store manager
            let app_handle = app.handle();
            init_store_manager(app_handle)?;

            // Create tray menu
            let menu = build_tray_menu(app.handle(), None)?;

            let initial_image = renderer.render_text("1", 16).into_tauri_image();
            let tray = TrayIconBuilder::new()
                .icon(initial_image)
                .menu(&menu)
                .icon_as_template(true)
                .tooltip("Copilot Tracker")
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "open_dashboard" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "open_billing" => {
                        let _ = app.opener().open_url(
                            "https://github.com/settings/billing/premium_requests_usage",
                            None::<&str>,
                        );
                    }
                    "refresh" => {
                        let app_handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = fetch_usage(
                                app_handle.clone(),
                                app_handle.state::<AuthManagerState>(),
                            )
                            .await;
                        });
                    }
                    "settings" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                        let _ = app.emit("navigate", "settings");
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
                            settings.refresh_interval = value;
                            let _ = update_settings(app.clone(), settings);
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // Store tray icon in state
            let tray_state = app.state::<TrayState>();
            *tray_state.tray.lock().unwrap() = Some(tray);

            // Listen for usage updates and update tray
            let app_handle = app.handle();
            let listener_handle = app_handle.clone();
            app_handle.listen("usage:updated", move |event| {
                let payload = event.payload();
                let parsed: UsagePayload = match serde_json::from_str(payload) {
                    Ok(parsed) => parsed,
                    Err(_) => return,
                };
                let state = listener_handle.state::<TrayState>();
                let _ = update_tray_icon(&state, parsed.summary.used, parsed.summary.limit);
                let update_state = listener_handle.state::<UpdateState>();
                let latest = update_state.latest.lock().unwrap();
                let _ = rebuild_tray_menu(&listener_handle, latest.as_ref());
            });

            // Load initial usage and update tray
            let store = app.state::<StoreManager>();
            let (used, limit) = store.get_usage();
            if used > 0 {
                let state = app.state::<TrayState>();
                let _ = update_tray_icon(&state, used, limit);
            }

            // Update tray menu at startup
            let update_state = app.state::<UpdateState>();
            let latest = update_state.latest.lock().unwrap();
            let _ = rebuild_tray_menu(app.handle(), latest.as_ref());

            // Start background usage polling (every 15 minutes)
            // NOTE: Deferred until after startup - needs proper async runtime initialization
            // let app_clone = app_handle.clone();
            // UsageManager::start_polling(app_clone, 15);

            if !tauri::is_dev() {
                let app_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    let _ = check_for_updates(app_handle).await;
                });
            }

            log::info!("Copilot Tracker initialized successfully");

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri app");
}
