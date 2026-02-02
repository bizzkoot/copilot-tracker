use std::sync::{Arc, Mutex};

use serde::Deserialize;
use tauri::menu::Menu;
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Listener, Manager};

use copilot_tracker::{
    init_store_manager,
    AuthManager,
    StoreManager,
    TrayIconRenderer,
    UpdateManager,
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
        }
    }

    Ok(result)
}

#[tauri::command]
async fn check_auth_status(
    app: AppHandle,
) -> Result<copilot_tracker::AuthState, String> {
    let store = app.state::<StoreManager>();
    let customer_id = store.get_customer_id();

    Ok(copilot_tracker::AuthState {
        is_authenticated: customer_id.is_some(),
        customer_id,
    })
}

#[tauri::command]
async fn logout(app: AppHandle) -> Result<(), String> {
    let store = app.state::<StoreManager>();
    store.clear_auth()?;
    Ok(())
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
fn update_settings(
    app: AppHandle,
    settings: copilot_tracker::AppSettings,
) -> Result<(), String> {
    let store = app.state::<StoreManager>();
    store.update_settings(|s| {
        *s = settings;
    })
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
// IPC Commands - Updater
// ============================================================================

#[tauri::command]
async fn check_for_updates(
    app: AppHandle,
) -> Result<copilot_tracker::UpdateStatus, String> {
    UpdateManager::check_for_updates(&app).await
}

#[tauri::command]
async fn install_update(
    app: AppHandle,
) -> Result<(), String> {
    UpdateManager::install_update(&app).await
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
        .plugin(tauri_plugin_updater::Builder::new().build())
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
            set_launch_at_login,
            // Tray commands
            update_tray_usage,
            // Updater commands
            check_for_updates,
            install_update,
        ])
        // Setup application
        .setup(move |app| {
            log::info!("Initializing Copilot Tracker (Tauri)");

            // Initialize store manager
            let app_handle = app.handle();
            init_store_manager(&app_handle)?;

            // Create tray menu
            let menu = Menu::new(app)?;
            let initial_image = renderer.render_text("1", 16).into_tauri_image();
            let tray = TrayIconBuilder::new()
                .icon(initial_image)
                .menu(&menu)
                .icon_as_template(true)
                .tooltip("Copilot Tracker")
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
            let app_clone = app_handle.clone();
            UsageManager::start_polling(app_clone, 15);

            // Start automatic update checks (every 24 hours)
            let app_clone = app_handle.clone();
            UpdateManager::start_auto_check(app_clone, 24);

            log::info!("Copilot Tracker initialized successfully");

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri app");
}
