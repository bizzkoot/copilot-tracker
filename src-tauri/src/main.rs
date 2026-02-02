use std::sync::{Arc, Mutex};

use serde::Deserialize;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Listener, Manager};

use copilot_tracker::{
    init_store_manager,
    AuthManager,
    StoreManager,
    TrayIconRenderer,
    UsageManager,
};

// ============================================================================
// Tray State
// ============================================================================

struct TrayState {
    tray: Mutex<Option<tauri::tray::TrayIcon>>,
    renderer: Arc<TrayIconRenderer>,
}

#[derive(Debug, Deserialize)]
struct UsagePayload {
    used: u32,
    limit: u32,
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

// ============================================================================
// IPC Commands - Authentication
// ============================================================================

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
    let result = tokio::task::spawn_blocking(move || {
        let mut auth_manager = auth_manager_state.lock().unwrap();
        tauri::async_runtime::block_on(auth_manager.perform_extraction(&app_clone))
    })
    .await
    .map_err(|err| format!("Auth extraction task failed: {}", err))??;

    // Update store if successful
    if let Some(customer_id) = result.customer_id {
        if let Some(store) = app.try_state::<StoreManager>() {
            let _ = store.set_customer_id(customer_id);
            // Emit auth state change
            let _ = app.emit("auth:state-changed", "authenticated");
        }
    }

    Ok(result)
}

#[tauri::command]
async fn handle_auth_redirect(
    app: AppHandle,
    state: tauri::State<'_, AuthManagerState>,
) -> Result<(), String> {
    log::info!("Auth redirect detected, starting extraction...");

    // 1. Hide/Close the auth window
    {
        let mut auth_manager = state.auth_manager.lock().unwrap();
        auth_manager.hide_auth_window();
    }

    // 2. Perform extraction
    let app_clone = app.clone();
    let auth_manager_state = state.auth_manager.clone();

    // Run extraction in background so we don't block the command return
    tokio::spawn(async move {
        // We need to lock the mutex to call perform_extraction
        // We wrap it in spawn_blocking because locking a std::sync::Mutex is blocking
        let app_for_extraction = app_clone.clone();
        let result = tokio::task::spawn_blocking(move || {
            let mut auth_manager = auth_manager_state.lock().unwrap();
            tauri::async_runtime::block_on(auth_manager.perform_extraction(&app_for_extraction))
        }).await;

        match result {
            Ok(Ok(extraction)) => {
                if let Some(customer_id) = extraction.customer_id {
                    log::info!("Extraction successful, customer_id: {}", customer_id);
                    if let Some(store) = app_clone.try_state::<StoreManager>() {
                        let _ = store.set_customer_id(customer_id);
                        
                        // Show main window FIRST
                        if let Some(window) = app_clone.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }

                        // Wait for frontend to be ready (handle race condition)
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

                        // Emit auth state change
                        let _ = app_clone.emit("auth:state-changed", "authenticated");
                    }
                } else {
                    log::warn!("Extraction returned no customer ID");
                }
            }
            Ok(Err(e)) => log::error!("Extraction failed: {}", e),
            Err(e) => log::error!("Task join error: {}", e),
        }
    });

    Ok(())
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
    let auth_manager = {
        let guard = state.auth_manager.lock().unwrap();
        (*guard).clone()
    };
    let mut usage_manager = UsageManager::new(auth_manager);
    usage_manager.fetch_usage(&app).await
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
    
    // Emit event to frontend
    let _ = app.emit("settings:changed", settings);
    
    Ok(())
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
        // Register plugins
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        // Register IPC commands
        .invoke_handler(tauri::generate_handler![
            // Auth commands
            show_auth_window,
            perform_auth_extraction,
            handle_auth_redirect,
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
            set_launch_at_login,
            // Tray commands
            update_tray_usage,
            // App commands
            get_app_version,
        ])
        // Setup application
        .setup(move |app| {
            log::info!("Initializing Copilot Tracker (Tauri)");

            // Initialize store manager
            let app_handle = app.handle();
            init_store_manager(&app_handle)?;

            // Create tray menu
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let show_i = MenuItem::with_id(app, "show", "Show Dashboard", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_i, &quit_i])?;

            let initial_image = renderer.render_text("1", 16).into_tauri_image();
            let tray = TrayIconBuilder::new()
                .icon(initial_image)
                .menu(&menu)
                .icon_as_template(true)
                .tooltip("Copilot Tracker")
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "quit" => {
                            app.exit(0);
                        }
                        "show" => {
                             if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        _ => {}
                    }
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
                let _ = update_tray_icon(&state, parsed.used, parsed.limit);
            });

            // Load initial usage and update tray
            let store = app.state::<StoreManager>();
            let (used, limit) = store.get_usage();
            if used > 0 {
                let state = app.state::<TrayState>();
                let _ = update_tray_icon(&state, used, limit);
            }

            // Start background usage polling (every 15 minutes)
            // NOTE: Deferred until after startup - needs proper async runtime initialization
            // let app_clone = app_handle.clone();
            // UsageManager::start_polling(app_clone, 15);

            log::info!("Copilot Tracker initialized successfully");

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri app");
}
