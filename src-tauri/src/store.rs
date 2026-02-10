use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

use crate::usage::UsageEntry;

const STORE_FILENAME: &str = "settings.json";
const HISTORY_FILENAME: &str = "usage_history.json";

/// Valid tray icon display formats
pub const TRAY_ICON_FORMATS: &[&str] = &[
    "current",
    "currentTotal",
    "remainingTotal",
    "percentage",
    "remainingPercent",
    "combined",
    "remainingCombined",
];

/// Default tray icon format - must be one of TRAY_ICON_FORMATS
pub const DEFAULT_TRAY_ICON_FORMAT: &str = "currentTotal";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    /// Customer ID from GitHub
    pub customer_id: Option<u64>,
    /// Usage limit for the current period
    pub usage_limit: u32,
    /// Last known usage count
    pub last_usage: u32,
    /// Last time usage was fetched (timestamp)
    pub last_fetch_timestamp: i64,
    /// Whether to launch at login
    pub launch_at_login: bool,
    /// Whether to show notifications
    pub show_notifications: bool,
    /// Notification thresholds
    #[serde(default = "default_thresholds")]
    pub notification_thresholds: Vec<u32>,
    /// Update channel (stable, beta)
    pub update_channel: String,
    /// Authenticated state
    pub is_authenticated: bool,
    /// Refresh interval in seconds
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval: u32,
    /// Prediction period in days
    #[serde(default = "default_prediction_period")]
    pub prediction_period: u32,
    /// Start minimized
    #[serde(default = "default_start_minimized")]
    pub start_minimized: bool,
    /// Theme
    #[serde(default = "default_theme")]
    pub theme: String,
    /// Tray icon display format
    #[serde(default = "default_tray_icon_format")]
    pub tray_icon_format: String,
    /// Widget enabled
    #[serde(default = "default_widget_enabled")]
    pub widget_enabled: bool,
    /// Widget position (x, y)
    #[serde(default = "default_widget_position")]
    pub widget_position: WidgetPosition,
    /// Widget pinned (always on top)
    #[serde(default = "default_widget_pinned")]
    pub widget_pinned: bool,
    /// Widget visible
    #[serde(default = "default_widget_visible")]
    pub widget_visible: bool,
}

/// Widget position on screen
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetPosition {
    pub x: i32,
    pub y: i32,
}

impl Default for WidgetPosition {
    fn default() -> Self {
        Self { x: 100, y: 100 }
    }
}

fn default_thresholds() -> Vec<u32> {
    vec![75, 90, 100]
}

fn default_refresh_interval() -> u32 {
    60
}

fn default_prediction_period() -> u32 {
    7
}

fn default_start_minimized() -> bool {
    true
}

fn default_theme() -> String {
    "system".to_string()
}

fn default_tray_icon_format() -> String {
    DEFAULT_TRAY_ICON_FORMAT.to_string()
}

fn default_widget_enabled() -> bool {
    false
}

fn default_widget_pinned() -> bool {
    true
}

fn default_widget_visible() -> bool {
    false
}

fn default_widget_position() -> WidgetPosition {
    WidgetPosition::default()
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            customer_id: None,
            usage_limit: 1200, // Default Copilot limit
            last_usage: 0,
            last_fetch_timestamp: 0,
            launch_at_login: false,
            show_notifications: true,
            notification_thresholds: default_thresholds(),
            update_channel: "stable".to_string(),
            is_authenticated: false,
            refresh_interval: default_refresh_interval(),
            prediction_period: default_prediction_period(),
            start_minimized: default_start_minimized(),
            theme: default_theme(),
            tray_icon_format: default_tray_icon_format(),
            widget_enabled: default_widget_enabled(),
            widget_position: default_widget_position(),
            widget_pinned: default_widget_pinned(),
            widget_visible: default_widget_visible(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageCache {
    pub customer_id: u64,
    pub net_quantity: u64,
    pub discount_quantity: u64,
    pub user_premium_request_entitlement: u64,
    pub filtered_user_premium_request_entitlement: u64,
    pub net_billed_amount: f64,
    pub timestamp: i64,
}

pub struct StoreManager {
    settings_path: PathBuf,
    history_path: PathBuf,
    settings: Mutex<AppSettings>,
    usage_cache: Mutex<Option<UsageCache>>,
    usage_history: Mutex<Vec<UsageEntry>>,
}

impl StoreManager {
    /// Create a new store manager with the given app directory
    pub fn new(app_dir: PathBuf) -> Result<Self, String> {
        let settings_path = app_dir.join(STORE_FILENAME);
        let history_path = app_dir.join(HISTORY_FILENAME);

        // Load existing settings or create defaults
        let settings = if settings_path.exists() {
            Self::load_settings_from_disk(&settings_path)?
        } else {
            AppSettings::default()
        };

        // Load existing history or create empty
        let history = if history_path.exists() {
            Self::load_history_from_disk(&history_path)?
        } else {
            Vec::new()
        };

        Ok(Self {
            settings_path,
            history_path,
            settings: Mutex::new(settings),
            usage_cache: Mutex::new(None),
            usage_history: Mutex::new(history),
        })
    }

    /// Load settings from disk
    fn load_settings_from_disk(path: &PathBuf) -> Result<AppSettings, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read settings file: {}", e))?;

        let settings: AppSettings = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse settings file: {}", e))?;

        Ok(settings)
    }

    /// Save settings to disk
    fn save_settings_to_disk(path: &PathBuf, settings: &AppSettings) -> Result<(), String> {
        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        std::fs::write(path, content)
            .map_err(|e| format!("Failed to write settings file: {}", e))?;

        Ok(())
    }

    /// Load history from disk
    fn load_history_from_disk(path: &PathBuf) -> Result<Vec<UsageEntry>, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read history file: {}", e))?;

        let history: Vec<UsageEntry> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse history file: {}", e))?;

        Ok(history)
    }

    /// Save history to disk
    fn save_history_to_disk(path: &PathBuf, history: &Vec<UsageEntry>) -> Result<(), String> {
        let content = serde_json::to_string_pretty(history)
            .map_err(|e| format!("Failed to serialize history: {}", e))?;

        std::fs::write(path, content)
            .map_err(|e| format!("Failed to write history file: {}", e))?;

        Ok(())
    }

    /// Get a copy of current settings
    pub fn get_settings(&self) -> AppSettings {
        self.settings.lock().unwrap().clone()
    }

    /// Update settings and persist to disk
    pub fn update_settings<F>(&self, updater: F) -> Result<(), String>
    where
        F: FnOnce(&mut AppSettings),
    {
        let mut settings = self.settings.lock().unwrap();
        updater(&mut settings);

        // Persist to disk
        Self::save_settings_to_disk(&self.settings_path, &settings)?;

        Ok(())
    }

    /// Set customer ID
    pub fn set_customer_id(&self, id: u64) -> Result<(), String> {
        self.update_settings(|s| {
            s.customer_id = Some(id);
            s.is_authenticated = true;
        })
    }

    /// Get customer ID
    pub fn get_customer_id(&self) -> Option<u64> {
        self.settings.lock().unwrap().customer_id
    }

    /// Set usage data
    pub fn set_usage(&self, used: u32, limit: u32) -> Result<(), String> {
        self.update_settings(|s| {
            s.last_usage = used;
            s.usage_limit = limit;
            s.last_fetch_timestamp = chrono::Utc::now().timestamp();
        })
    }

    /// Get usage data
    pub fn get_usage(&self) -> (u32, u32) {
        let settings = self.settings.lock().unwrap();
        (settings.last_usage, settings.usage_limit)
    }

    /// Set launch at login preference
    pub fn set_launch_at_login(&self, enabled: bool) -> Result<(), String> {
        self.update_settings(|s| {
            s.launch_at_login = enabled;
        })
    }

    /// Get launch at login preference
    pub fn get_launch_at_login(&self) -> bool {
        self.settings.lock().unwrap().launch_at_login
    }

    /// Set show notifications preference
    pub fn set_show_notifications(&self, enabled: bool) -> Result<(), String> {
        self.update_settings(|s| {
            s.show_notifications = enabled;
        })
    }

    /// Get show notifications preference
    pub fn get_show_notifications(&self) -> bool {
        self.settings.lock().unwrap().show_notifications
    }

    /// Check if authenticated
    pub fn is_authenticated(&self) -> bool {
        self.settings.lock().unwrap().is_authenticated
    }

    /// Clear authentication (logout)
    pub fn clear_auth(&self) -> Result<(), String> {
        self.update_settings(|s| {
            s.customer_id = None;
            s.is_authenticated = false;
        })
    }

    /// Export usage cache for persistence
    pub fn export_usage_cache(&self) -> Result<UsageCache, String> {
        let settings = self.settings.lock().unwrap();

        let customer_id = settings.customer_id.ok_or("No customer ID available")?;

        Ok(UsageCache {
            customer_id,
            net_quantity: settings.last_usage as u64,
            discount_quantity: 0,
            user_premium_request_entitlement: 0,
            filtered_user_premium_request_entitlement: 0,
            net_billed_amount: 0.0,
            timestamp: settings.last_fetch_timestamp,
        })
    }

    pub fn set_usage_cache(&self, cache: UsageCache) {
        let mut guard = self.usage_cache.lock().unwrap();
        *guard = Some(cache);
    }

    pub fn get_usage_cache(&self) -> Option<UsageCache> {
        self.usage_cache.lock().unwrap().clone()
    }

    pub fn set_usage_history(&self, history: Vec<UsageEntry>) {
        let mut guard = self.usage_history.lock().unwrap();
        *guard = history.clone();
        drop(guard); // Release lock before disk I/O

        // Persist to disk
        if let Err(e) = Self::save_history_to_disk(&self.history_path, &history) {
            log::error!("Failed to save usage history to disk: {}", e);
        } else {
            log::info!(
                "Successfully saved {} history entries to disk",
                history.len()
            );
        }
    }

    pub fn get_usage_history(&self) -> Vec<UsageEntry> {
        self.usage_history.lock().unwrap().clone()
    }

    pub fn reset_settings(&self) -> Result<AppSettings, String> {
        let defaults = AppSettings::default();
        self.update_settings(|s| {
            *s = defaults.clone();
        })?;

        // Clear usage cache
        {
            let mut cache = self.usage_cache.lock().unwrap();
            *cache = None;
        }

        // Clear usage history
        {
            let mut history = self.usage_history.lock().unwrap();
            history.clear();
        }

        // Delete history file from disk
        if self.history_path.exists() {
            std::fs::remove_file(&self.history_path)
                .map_err(|e| format!("Failed to delete history file: {}", e))?;
        }

        Ok(defaults)
    }

    /// Get the tray icon display format
    pub fn get_tray_icon_format(&self) -> String {
        self.settings.lock().unwrap().tray_icon_format.clone()
    }

    /// Set the tray icon display format with validation
    pub fn set_tray_icon_format(&self, format: String) -> Result<(), String> {
        if !TRAY_ICON_FORMATS.contains(&format.as_str()) {
            return Err(format!("Invalid tray icon format: {}", format));
        }

        self.update_settings(|s| {
            s.tray_icon_format = format;
        })
    }

    /// Get widget enabled state
    pub fn get_widget_enabled(&self) -> bool {
        self.settings.lock().unwrap().widget_enabled
    }

    /// Set widget enabled state
    pub fn set_widget_enabled(&self, enabled: bool) -> Result<(), String> {
        self.update_settings(|s| {
            s.widget_enabled = enabled;
        })
    }

    /// Get widget position
    pub fn get_widget_position(&self) -> WidgetPosition {
        self.settings.lock().unwrap().widget_position.clone()
    }

    /// Set widget position
    pub fn set_widget_position(&self, position: WidgetPosition) -> Result<(), String> {
        self.update_settings(|s| {
            s.widget_position = position;
        })
    }

    /// Get widget pinned state
    pub fn get_widget_pinned(&self) -> bool {
        self.settings.lock().unwrap().widget_pinned
    }

    /// Set widget pinned state
    pub fn set_widget_pinned(&self, pinned: bool) -> Result<(), String> {
        self.update_settings(|s| {
            s.widget_pinned = pinned;
        })
    }

    /// Get widget visible state
    pub fn get_widget_visible(&self) -> bool {
        self.settings.lock().unwrap().widget_visible
    }

    /// Set widget visible state
    pub fn set_widget_visible(&self, visible: bool) -> Result<(), String> {
        self.update_settings(|s| {
            s.widget_visible = visible;
        })
    }
}

/// Initialize the store manager and attach to app
pub fn init_store_manager(app: &AppHandle) -> Result<(), String> {
    let app_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    // Ensure directory exists
    std::fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Failed to create app data dir: {}", e))?;

    let store_manager = StoreManager::new(app_dir)?;

    app.manage(store_manager);

    Ok(())
}

#[tauri::command]
pub fn get_settings(store: tauri::State<StoreManager>) -> AppSettings {
    store.get_settings()
}

#[tauri::command]
pub fn update_settings(store: tauri::State<StoreManager>, settings: AppSettings) -> Result<(), String> {
    store.update_settings(|s| *s = settings)
}

#[tauri::command]
pub fn get_usage_cache(store: tauri::State<StoreManager>) -> Option<UsageCache> {
    store.get_usage_cache()
}

#[tauri::command]
pub fn get_usage_history(store: tauri::State<StoreManager>) -> Vec<UsageEntry> {
    store.get_usage_history()
}

#[tauri::command]
pub fn add_usage_entry(store: tauri::State<StoreManager>, entry: UsageEntry) -> Result<(), String> {
    let mut history = store.get_usage_history();
    history.push(entry);
    store.set_usage_history(history);
    Ok(())
}

#[tauri::command]
pub fn clear_usage_history(store: tauri::State<StoreManager>) -> Result<(), String> {
    store.set_usage_history(Vec::new());
    Ok(())
}
