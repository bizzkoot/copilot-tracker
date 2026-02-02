use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager};

const STORE_FILENAME: &str = "settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Update channel (stable, beta)
    pub update_channel: String,
    /// Authenticated state
    pub is_authenticated: bool,
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
            update_channel: "stable".to_string(),
            is_authenticated: false,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
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
    settings: Mutex<AppSettings>,
}

impl StoreManager {
    /// Create a new store manager with the given app directory
    pub fn new(app_dir: PathBuf) -> Result<Self, String> {
        let settings_path = app_dir.join(STORE_FILENAME);

        // Load existing settings or create defaults
        let settings = if settings_path.exists() {
            Self::load_settings_from_disk(&settings_path)?
        } else {
            AppSettings::default()
        };

        Ok(Self {
            settings_path,
            settings: Mutex::new(settings),
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

        let customer_id = settings.customer_id
            .ok_or("No customer ID available")?;

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
}

/// Initialize the store manager and attach to app
pub fn init_store_manager(app: &AppHandle) -> Result<(), String> {
    let app_dir = app.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;

    // Ensure directory exists
    std::fs::create_dir_all(&app_dir)
        .map_err(|e| format!("Failed to create app data dir: {}", e))?;

    let store_manager = StoreManager::new(app_dir)?;

    app.manage(store_manager);

    Ok(())
}
