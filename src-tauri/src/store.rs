use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::usage::UsageEntry;

const STORE_FILENAME: &str = "settings.json";
const USAGE_CACHE_FILENAME: &str = "usage_cache.json";
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

/// Backup frequency for auto-backup
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BackupFrequency {
    EveryRefresh,
    Daily,
    Every3Days,
    Weekly,
}

impl Default for BackupFrequency {
    fn default() -> Self {
        BackupFrequency::Daily
    }
}

/// AppSettings - PASSIVE data (user preferences + auth)
/// Stored in settings.json
/// Changes infrequently: only when user changes settings or logs in/out
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    /// Customer ID from GitHub (passive - only on login)
    pub customer_id: Option<u64>,
    /// Whether to launch at login
    pub launch_at_login: bool,
    // -------------------------------------------------------------------------
    // BACKWARD COMPATIBILITY FIELDS
    // These fields are no longer used by new code (data lives in usage_cache.json).
    // They are kept in this struct so that:
    //   1. New code serializes them as defaults → old builds can parse settings.json
    //      without crashing (safe downgrade from new → old build).
    //   2. Old settings.json files with real values in these fields are silently
    //      accepted by new code without a parse error (safe upgrade path).
    // On upgrade: smart startup fetch (last_fetch_timestamp==0) immediately refreshes.
    // On downgrade: old build reads 0/1200 → treats as first launch → fetches data.
    // -------------------------------------------------------------------------
    #[serde(default = "legacy_default_usage_limit")]
    pub usage_limit: u32,
    #[serde(default)]
    pub last_usage: f64,
    #[serde(default)]
    pub last_fetch_timestamp: i64,
    #[serde(default)]
    pub last_update_check_timestamp: i64,
    /// Whether to show notifications
    pub show_notifications: bool,
    /// Notification thresholds
    #[serde(default = "default_thresholds")]
    pub notification_thresholds: Vec<u32>,
    /// Update channel (stable, beta)
    pub update_channel: String,
    /// Authenticated state (passive - only on login/logout)
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
    /// Enable debug tools in dashboard
    #[serde(default = "default_debug_tools_enabled")]
    pub debug_tools_enabled: bool,
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
    /// Auto backup usage data after each fetch
    #[serde(default = "default_auto_backup_enabled")]
    pub auto_backup_enabled: bool,
    /// Backup frequency for auto-backup
    #[serde(default = "default_backup_frequency")]
    pub backup_frequency: BackupFrequency,
    /// Maximum number of backups to keep (0 = unlimited)
    #[serde(default = "default_backup_retention_count")]
    pub backup_retention_count: u32,
    /// Timestamp of last auto-backup (ISO 8601)
    #[serde(default)]
    pub last_auto_backup_at: Option<String>,
    /// Custom backup directory (None = default ./backups/)
    #[serde(default)]
    pub backup_directory: Option<String>,
}

/// UsageCacheData - ACTIVE data (volatile usage information)
/// Stored in usage_cache.json
/// Changes frequently: on every usage refresh/poll
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UsageCacheData {
    /// Usage limit for the current period (cached from API)
    pub usage_limit: u32,
    /// Last known usage count
    pub last_usage: f64,
    /// Last time usage was fetched (timestamp)
    pub last_fetch_timestamp: i64,
    /// Last time update check was performed (timestamp)
    #[serde(default)]
    pub last_update_check_timestamp: i64,
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

fn default_debug_tools_enabled() -> bool {
    false
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

fn default_auto_backup_enabled() -> bool {
    false
}

fn default_backup_frequency() -> BackupFrequency {
    BackupFrequency::Daily
}

fn default_backup_retention_count() -> u32 {
    10
}

fn legacy_default_usage_limit() -> u32 {
    1200
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            customer_id: None,
            launch_at_login: false,
            // Legacy compat fields — always zero/default in new builds
            usage_limit: legacy_default_usage_limit(),
            last_usage: 0.0,
            last_fetch_timestamp: 0,
            last_update_check_timestamp: 0,
            show_notifications: true,
            notification_thresholds: default_thresholds(),
            update_channel: "stable".to_string(),
            is_authenticated: false,
            refresh_interval: default_refresh_interval(),
            prediction_period: default_prediction_period(),
            start_minimized: default_start_minimized(),
            debug_tools_enabled: default_debug_tools_enabled(),
            theme: default_theme(),
            tray_icon_format: default_tray_icon_format(),
            widget_enabled: default_widget_enabled(),
            widget_position: default_widget_position(),
            widget_pinned: default_widget_pinned(),
            widget_visible: default_widget_visible(),
            auto_backup_enabled: default_auto_backup_enabled(),
            backup_frequency: default_backup_frequency(),
            backup_retention_count: default_backup_retention_count(),
            last_auto_backup_at: None,
            backup_directory: None,
        }
    }
}

impl Default for UsageCacheData {
    fn default() -> Self {
        Self {
            usage_limit: 1200, // Default Copilot limit
            last_usage: 0.0,
            last_fetch_timestamp: 0,
            last_update_check_timestamp: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageCache {
    pub customer_id: u64,
    pub net_quantity: f64,
    pub discount_quantity: f64,
    pub user_premium_request_entitlement: u32,
    pub filtered_user_premium_request_entitlement: u32,
    pub net_billed_amount: f64,
    pub timestamp: i64,
}

pub struct StoreManager {
    settings_path: PathBuf,
    usage_cache_path: PathBuf,
    history_path: PathBuf,
    settings: Mutex<AppSettings>,
    usage_cache: Mutex<UsageCacheData>,
    usage_history: Mutex<Vec<UsageEntry>>,
}

impl StoreManager {
    /// Create a new store manager with the given app directory
    pub fn new(app_dir: PathBuf) -> Result<Self, String> {
        // Ensure directory exists (moved from init_store_manager)
        if !app_dir.exists() {
            std::fs::create_dir_all(&app_dir)
                .map_err(|e| format!("Failed to create app data dir: {}", e))?;
        }

        let settings_path = app_dir.join(STORE_FILENAME);
        let usage_cache_path = app_dir.join(USAGE_CACHE_FILENAME);
        let history_path = app_dir.join(HISTORY_FILENAME);

        // Load existing settings or create defaults
        // Note: load_settings_from_disk now handles corrupted files by backing them up
        // and returning default settings, so this will never fail
        let settings = if settings_path.exists() {
            match Self::load_settings_from_disk(&settings_path) {
                Ok(s) => s,
                Err(e) => {
                    log::error!("Failed to load settings (will use defaults): {}", e);
                    AppSettings::default()
                }
            }
        } else {
            AppSettings::default()
        };

        // Load existing usage cache or create defaults
        // Note: load_usage_cache_from_disk handles corrupted files by backing them up
        // and returning default cache, so this will never fail
        let usage_cache = if usage_cache_path.exists() {
            match Self::load_usage_cache_from_disk(&usage_cache_path) {
                Ok(c) => c,
                Err(e) => {
                    log::error!("Failed to load usage cache (will use defaults): {}", e);
                    UsageCacheData::default()
                }
            }
        } else {
            UsageCacheData::default()
        };

        // Load existing history or create empty
        // Note: load_history_from_disk now handles corrupted files by backing them up
        // and returning empty history, so this will never fail
        let history = if history_path.exists() {
            match Self::load_history_from_disk(&history_path) {
                Ok(h) => h,
                Err(e) => {
                    log::error!("Failed to load history (will use empty): {}", e);
                    Vec::new()
                }
            }
        } else {
            Vec::new()
        };

        Ok(Self {
            settings_path,
            usage_cache_path,
            history_path,
            settings: Mutex::new(settings),
            usage_cache: Mutex::new(usage_cache),
            usage_history: Mutex::new(history),
        })
    }

    /// Load settings from disk with automatic corruption recovery
    fn load_settings_from_disk(path: &PathBuf) -> Result<AppSettings, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read settings file: {}", e))?;

        let settings: AppSettings = serde_json::from_str(&content)
            .map_err(|e| {
                // JSON parse error - file is corrupted
                log::error!("Corrupted settings file: {}", e);
                backup_corrupted_file(path);
                format!(
                    "Failed to parse settings file (corrupted). Backed up and recovered with defaults: {}",
                    e
                )
            })?;

        Ok(settings)
    }

    /// Save settings to disk using atomic write pattern (prevents corruption)
    fn save_settings_to_disk(path: &PathBuf, settings: &AppSettings) -> Result<(), String> {
        let content = serde_json::to_string_pretty(settings)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        write_file_atomic(path, &content)
    }

    /// Load history from disk with automatic corruption recovery
    fn load_history_from_disk(path: &PathBuf) -> Result<Vec<UsageEntry>, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read history file: {}", e))?;

        let history: Vec<UsageEntry> = serde_json::from_str(&content)
            .map_err(|e| {
                // JSON parse error - file is corrupted
                log::error!("Corrupted history file: {}", e);
                backup_corrupted_file(path);
                format!(
                    "Failed to parse history file (corrupted). Backed up and recovered with empty history: {}",
                    e
                )
            })?;

        Ok(history)
    }

    /// Save history to disk using atomic write pattern (prevents corruption)
    fn save_history_to_disk(path: &PathBuf, history: &Vec<UsageEntry>) -> Result<(), String> {
        let content = serde_json::to_string_pretty(history)
            .map_err(|e| format!("Failed to serialize history: {}", e))?;

        write_file_atomic(path, &content)
    }

    /// Load usage cache from disk with automatic corruption recovery
    fn load_usage_cache_from_disk(path: &PathBuf) -> Result<UsageCacheData, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read usage cache file: {}", e))?;

        let cache: UsageCacheData = serde_json::from_str(&content)
            .map_err(|e| {
                // JSON parse error - file is corrupted
                log::error!("Corrupted usage cache file: {}", e);
                backup_corrupted_file(path);
                format!(
                    "Failed to parse usage cache file (corrupted). Backed up and recovered with defaults: {}",
                    e
                )
            })?;

        Ok(cache)
    }

    /// Save usage cache to disk using atomic write pattern (prevents corruption)
    fn save_usage_cache_to_disk(path: &PathBuf, cache: &UsageCacheData) -> Result<(), String> {
        let content = serde_json::to_string_pretty(cache)
            .map_err(|e| format!("Failed to serialize usage cache: {}", e))?;

        write_file_atomic(path, &content)
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

    /// Set usage data (active data - written to usage_cache.json)
    pub fn set_usage(&self, used: f64, limit: u32) -> Result<(), String> {
        let mut cache = self.usage_cache.lock().unwrap();
        cache.last_usage = used;
        cache.usage_limit = limit;
        cache.last_fetch_timestamp = chrono::Utc::now().timestamp();

        // Persist to usage_cache.json
        Self::save_usage_cache_to_disk(&self.usage_cache_path, &cache)?;
        drop(cache);

        Ok(())
    }

    /// Get usage data (from usage_cache.json)
    pub fn get_usage(&self) -> (f64, u32) {
        let cache = self.usage_cache.lock().unwrap();
        (cache.last_usage, cache.usage_limit)
    }

    /// Get last fetch timestamp (from usage_cache.json)
    pub fn get_last_fetch_timestamp(&self) -> i64 {
        self.usage_cache.lock().unwrap().last_fetch_timestamp
    }

    /// Set last update check timestamp (active data - written to usage_cache.json)
    pub fn set_last_update_check_timestamp(&self, timestamp: i64) -> Result<(), String> {
        let mut cache = self.usage_cache.lock().unwrap();
        cache.last_update_check_timestamp = timestamp;

        // Persist to usage_cache.json
        Self::save_usage_cache_to_disk(&self.usage_cache_path, &cache)?;
        drop(cache);

        Ok(())
    }

    /// Get last update check timestamp (from usage_cache.json)
    pub fn get_last_update_check_timestamp(&self) -> i64 {
        self.usage_cache.lock().unwrap().last_update_check_timestamp
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

    /// Export usage cache for API responses (legacy compatibility)
    pub fn export_usage_cache(&self) -> Result<UsageCache, String> {
        let settings = self.settings.lock().unwrap();
        let cache = self.usage_cache.lock().unwrap();

        let customer_id = settings.customer_id.ok_or("No customer ID available")?;

        Ok(UsageCache {
            customer_id,
            net_quantity: cache.last_usage,
            // last_usage is the rounded discount_quantity (written by set_usage)
            discount_quantity: cache.last_usage,
            user_premium_request_entitlement: cache.usage_limit,
            filtered_user_premium_request_entitlement: cache.usage_limit,
            net_billed_amount: 0.0,
            timestamp: cache.last_fetch_timestamp,
        })
    }

    pub fn set_usage_cache(&self, cache: UsageCache) {
        // Updates auth state in settings.json (passive file — infrequent writes).
        // NOTE: Usage numbers (last_usage, usage_limit, last_fetch_timestamp) are
        // intentionally NOT updated here — they are already written to usage_cache.json
        // by set_usage(), which is always called before this method in every fetch path.
        // Mixing both writes here would overwrite the correct rounded discount_quantity
        // value (from set_usage) with the raw net_quantity from the API response.
        if let Err(e) = self.update_settings(|s| {
            s.customer_id = Some(cache.customer_id);
            s.is_authenticated = true;
        }) {
            log::error!(
                "[set_usage_cache] Failed to persist auth state to settings.json: {}",
                e
            );
        }
    }

    pub fn get_usage_cache(&self) -> Option<UsageCache> {
        let settings = self.settings.lock().unwrap();
        let cache = self.usage_cache.lock().unwrap();

        settings.customer_id.map(|customer_id| UsageCache {
            customer_id,
            net_quantity: cache.last_usage,
            // last_usage stores the rounded discount_quantity (set by set_usage).
            // Using it here so fallback history entries show the correct `used` value.
            discount_quantity: cache.last_usage,
            user_premium_request_entitlement: cache.usage_limit,
            filtered_user_premium_request_entitlement: cache.usage_limit,
            net_billed_amount: 0.0,
            timestamp: cache.last_fetch_timestamp,
        })
    }

    pub fn clear_usage_cache(&self) {
        let defaults = UsageCacheData::default();
        {
            let mut cache = self.usage_cache.lock().unwrap();
            *cache = defaults.clone();
        }
        // Persist the cleared state so next startup doesn't load stale data
        if let Err(e) = Self::save_usage_cache_to_disk(&self.usage_cache_path, &defaults) {
            log::error!(
                "[clear_usage_cache] Failed to persist cleared cache to disk: {}",
                e
            );
        }
        log::info!("Usage cache cleared and persisted");
    }

    pub fn clear_usage_history(&self) {
        let mut guard = self.usage_history.lock().unwrap();
        guard.clear();
        // Also delete the history file
        if self.history_path.exists() {
            let _ = std::fs::remove_file(&self.history_path);
        }
        log::info!("Usage history cleared");
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

        // Clear usage cache to defaults AND persist to disk
        {
            let cleared_cache = UsageCacheData::default();
            {
                let mut cache = self.usage_cache.lock().unwrap();
                *cache = cleared_cache.clone();
            }
            // Must persist — otherwise next restart loads stale cache from disk
            if let Err(e) = Self::save_usage_cache_to_disk(&self.usage_cache_path, &cleared_cache) {
                log::error!(
                    "[reset_settings] Failed to persist cleared usage cache: {}",
                    e
                );
            }
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

    /// Get auto backup enabled state
    pub fn get_auto_backup_enabled(&self) -> bool {
        self.settings.lock().unwrap().auto_backup_enabled
    }

    /// Set auto backup enabled state
    pub fn set_auto_backup_enabled(&self, enabled: bool) -> Result<(), String> {
        self.update_settings(|s| {
            s.auto_backup_enabled = enabled;
        })
    }

    /// Check if an auto-backup should run now based on frequency settings
    pub fn should_auto_backup(&self) -> bool {
        let settings = self.settings.lock().unwrap();
        if !settings.auto_backup_enabled {
            return false;
        }

        let threshold_hours: i64 = match settings.backup_frequency {
            BackupFrequency::EveryRefresh => 0,
            BackupFrequency::Daily => 24,
            BackupFrequency::Every3Days => 72,
            BackupFrequency::Weekly => 168,
        };

        if threshold_hours == 0 {
            return true; // always backup
        }

        if let Some(ref ts) = settings.last_auto_backup_at {
            if let Ok(last) = chrono::DateTime::parse_from_rfc3339(ts) {
                let now = chrono::Utc::now();
                let elapsed = now.signed_duration_since(last.with_timezone(&chrono::Utc));
                return elapsed.num_hours() >= threshold_hours;
            }
        }

        true // no previous backup recorded, should backup
    }

    /// Record the current time as the last auto-backup timestamp
    pub fn record_auto_backup_time(&self) -> Result<(), String> {
        let timestamp = chrono::Utc::now().to_rfc3339();
        self.update_settings(|s| {
            s.last_auto_backup_at = Some(timestamp.clone());
        })
    }

    /// Get backup directory
    pub fn get_backup_directory(&self) -> Option<String> {
        self.settings.lock().unwrap().backup_directory.clone()
    }

    /// Set backup directory
    pub fn set_backup_directory(&self, directory: Option<String>) -> Result<(), String> {
        self.update_settings(|s| {
            s.backup_directory = directory;
        })
    }

    /// Get the backups directory path
    pub fn get_backups_path(&self) -> PathBuf {
        if let Some(ref custom) = self.get_backup_directory() {
            PathBuf::from(custom)
        } else {
            self.settings_path.parent().unwrap().join("backups")
        }
    }

    /// Create a backup of usage data (history + cache)
    pub fn create_backup(&self) -> Result<String, String> {
        let backups_dir = self.get_backups_path();

        // Ensure backups directory exists
        std::fs::create_dir_all(&backups_dir)
            .map_err(|e| format!("Failed to create backups directory: {}", e))?;

        // Generate timestamp-based backup ID
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let backup_id = format!("backup_{}", timestamp);
        let backup_dir = backups_dir.join(&backup_id);

        std::fs::create_dir_all(&backup_dir)
            .map_err(|e| format!("Failed to create backup directory: {}", e))?;

        // Copy usage_history.json if exists
        if self.history_path.exists() {
            let dest = backup_dir.join("usage_history.json");
            std::fs::copy(&self.history_path, &dest)
                .map_err(|e| format!("Failed to backup history: {}", e))?;
        }

        // Copy usage_cache.json if exists
        if self.usage_cache_path.exists() {
            let dest = backup_dir.join("usage_cache.json");
            std::fs::copy(&self.usage_cache_path, &dest)
                .map_err(|e| format!("Failed to backup cache: {}", e))?;
        }

        // Write metadata
        let mut files = Vec::new();
        if self.history_path.exists() {
            files.push("usage_history.json");
        }
        if self.usage_cache_path.exists() {
            files.push("usage_cache.json");
        }

        let metadata = serde_json::json!({
            "backup_id": backup_id,
            "created_at": chrono::Utc::now().to_rfc3339(),
            "files": files
        });

        let metadata_path = backup_dir.join("metadata.json");
        std::fs::write(
            &metadata_path,
            serde_json::to_string_pretty(&metadata).unwrap(),
        )
        .map_err(|e| format!("Failed to write metadata: {}", e))?;

        log::info!("Backup created: {}", backup_id);

        // Prune old backups if retention limit is set
        let retention = self.settings.lock().unwrap().backup_retention_count;
        if retention > 0 {
            if let Err(e) = self.prune_backups(retention) {
                log::warn!("Backup pruning failed (non-fatal): {}", e);
            }
        }

        Ok(backup_id)
    }

    /// Restore usage data from a backup
    pub fn restore_backup(&self, backup_id: &str) -> Result<(), String> {
        let backups_dir = self.get_backups_path();
        let backup_dir = backups_dir.join(backup_id);

        if !backup_dir.exists() {
            return Err(format!("Backup not found: {}", backup_id));
        }

        // Read metadata to verify
        let metadata_path = backup_dir.join("metadata.json");
        if metadata_path.exists() {
            let _metadata: serde_json::Value = serde_json::from_str(
                &std::fs::read_to_string(&metadata_path)
                    .map_err(|e| format!("Failed to read metadata: {}", e))?,
            )
            .map_err(|e| format!("Invalid metadata: {}", e))?;
        }

        // Restore usage_history.json if exists in backup
        let history_backup = backup_dir.join("usage_history.json");
        if history_backup.exists() {
            // Load and validate
            let history: Vec<crate::usage::UsageEntry> = serde_json::from_str(
                &std::fs::read_to_string(&history_backup)
                    .map_err(|e| format!("Failed to read history backup: {}", e))?,
            )
            .map_err(|e| format!("Invalid history backup: {}", e))?;

            // Restore to current history
            self.set_usage_history(history);
        }

        // Restore usage_cache.json if exists in backup
        let cache_backup = backup_dir.join("usage_cache.json");
        if cache_backup.exists() {
            let cache: UsageCacheData = serde_json::from_str(
                &std::fs::read_to_string(&cache_backup)
                    .map_err(|e| format!("Failed to read cache backup: {}", e))?,
            )
            .map_err(|e| format!("Invalid cache backup: {}", e))?;

            // Restore to current cache
            {
                let mut current_cache = self.usage_cache.lock().unwrap();
                *current_cache = cache.clone();
            }
            Self::save_usage_cache_to_disk(&self.usage_cache_path, &cache)?;
        }

        log::info!("Backup restored: {}", backup_id);
        Ok(())
    }

    /// List all available backups
    pub fn list_backups(&self) -> Result<Vec<BackupInfo>, String> {
        let backups_dir = self.get_backups_path();

        if !backups_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();

        for entry in std::fs::read_dir(&backups_dir)
            .map_err(|e| format!("Failed to read backups directory: {}", e))?
        {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            if path.is_dir() {
                let backup_id = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                // Read metadata if exists
                let metadata_path = path.join("metadata.json");
                let (created_at, files) = if metadata_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&metadata_path) {
                        if let Ok(meta) = serde_json::from_str::<serde_json::Value>(&content) {
                            let created = meta
                                .get("created_at")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown")
                                .to_string();
                            let files: Vec<String> = meta
                                .get("files")
                                .and_then(|v| v.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| v.as_str().map(String::from))
                                        .collect()
                                })
                                .unwrap_or_default();
                            (created, files)
                        } else {
                            (String::new(), Vec::new())
                        }
                    } else {
                        (String::new(), Vec::new())
                    }
                } else {
                    (String::new(), Vec::new())
                };

                // Calculate size
                let size = std::fs::read_dir(&path)
                    .map(|entries| {
                        entries
                            .filter_map(|e| e.ok())
                            .filter_map(|e| e.metadata().ok())
                            .map(|m| m.len())
                            .sum()
                    })
                    .unwrap_or(0);

                backups.push(BackupInfo {
                    backup_id,
                    created_at,
                    files,
                    size_bytes: size,
                });
            }
        }

        // Sort by creation date (newest first)
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(backups)
    }

    /// Delete a backup
    pub fn delete_backup(&self, backup_id: &str) -> Result<(), String> {
        let backups_dir = self.get_backups_path();
        let backup_dir = backups_dir.join(backup_id);

        if !backup_dir.exists() {
            return Err(format!("Backup not found: {}", backup_id));
        }

        std::fs::remove_dir_all(&backup_dir)
            .map_err(|e| format!("Failed to delete backup: {}", e))?;

        log::info!("Backup deleted: {}", backup_id);
        Ok(())
    }

    /// Prune old backups, keeping only the most recent `keep` backups.
    /// Deletes the oldest backups (sorted by creation date) until count <= keep.
    pub fn prune_backups(&self, keep: u32) -> Result<(), String> {
        let mut backups = self.list_backups()?;
        if backups.len() <= keep as usize {
            return Ok(());
        }

        // list_backups returns newest-first; trim from the tail (oldest)
        let to_delete = backups.split_off(keep as usize);

        for backup in to_delete {
            let backups_dir = self.get_backups_path();
            let backup_dir = backups_dir.join(&backup.backup_id);
            if backup_dir.exists() {
                std::fs::remove_dir_all(&backup_dir).map_err(|e| {
                    format!("Failed to prune backup {}: {}", backup.backup_id, e)
                })?;
                log::info!("Pruned old backup: {}", backup.backup_id);
            }
        }

        Ok(())
    }
}

/// Backup information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    pub backup_id: String,
    pub created_at: String,
    pub files: Vec<String>,
    pub size_bytes: u64,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Write content to file atomically (prevents corruption on crash/restart)
///
/// This uses an atomic write pattern:
/// 1. Write content to a temporary file (.tmp)
/// 2. Sync the temporary file to disk
/// 3. Atomically rename the temporary file to the target file
///
/// The atomic rename ensures that either the old file remains intact or the new
/// file is completely written - never a partially written/corrupted state.
/// This is critical for preventing data loss during system crashes or restarts.
fn write_file_atomic(path: &PathBuf, content: &str) -> Result<(), String> {
    let temp_path = path.with_extension("tmp");

    // Step 1: Write to temporary file
    let mut file = std::fs::File::create(&temp_path)
        .map_err(|e| format!("Failed to create temp file: {}", e))?;

    file.write_all(content.as_bytes())
        .map_err(|e| format!("Failed to write to temp file: {}", e))?;

    // Step 2: Sync temp file to disk (ensure data is physically written)
    file.sync_all()
        .map_err(|e| format!("Failed to sync temp file: {}", e))?;

    // Step 3: Atomic rename (POSIX guarantees this is atomic)
    // On success: old file is replaced, new file is complete
    // On failure: old file remains intact
    std::fs::rename(&temp_path, path).map_err(|e| format!("Failed to rename temp file: {}", e))?;

    log::debug!("File written atomically: {:?}", path);
    Ok(())
}

/// Backup a corrupted file by renaming it with a timestamp
///
/// When JSON parsing fails, we preserve the corrupted file for debugging
/// by renaming it to .bak.TIMESTAMP. This allows investigation of what went wrong.
fn backup_corrupted_file(path: &PathBuf) {
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_path = path.with_extension(format!("bak.{}", timestamp));

    match std::fs::rename(path, &backup_path) {
        Ok(_) => {
            log::warn!(
                "Backed up corrupted file from {:?} to {:?}",
                path,
                backup_path
            );
        }
        Err(e) => {
            log::error!("Failed to backup corrupted file {:?}: {}", path, e);
        }
    }
}
