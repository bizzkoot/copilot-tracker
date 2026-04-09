use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

use crate::usage::UsageEntry;

const STORE_FILENAME: &str = "settings.json";
const USAGE_CACHE_FILENAME: &str = "usage_cache.json";
const HISTORY_FILENAME: &str = "usage_history.json";
const QUOTA_HISTORY_FILENAME: &str = "quota_history.json";

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum BackupFrequency {
    EveryRefresh,
    #[default]
    Daily,
    Every3Days,
    Weekly,
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

type UsageStateSnapshot = (UsageCacheData, Vec<UsageEntry>, HashMap<String, u32>);

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
    /// Path to quota_history.json — records the monthly quota (limit) observed each month.
    /// Used to show accurate historical utilization even when the user's plan changes.
    quota_history_path: PathBuf,
    settings: Mutex<AppSettings>,
    usage_cache: Mutex<UsageCacheData>,
    usage_history: Mutex<Vec<UsageEntry>>,
    /// Maps "YYYY-MM" → monthly quota limit (u32) recorded at fetch time.
    quota_history: Mutex<HashMap<String, u32>>,
    state_operation: Mutex<()>,
    backup_creation: Mutex<()>,
    session_generation: AtomicU64,
    session_transition_in_progress: AtomicBool,
}

impl StoreManager {
    fn empty_usage_cache_data() -> UsageCacheData {
        UsageCacheData {
            usage_limit: 0,
            last_usage: 0.0,
            last_fetch_timestamp: 0,
            last_update_check_timestamp: 0,
        }
    }

    fn clear_usage_state_to_empty(&self) -> Result<(), String> {
        self.replace_usage_state(Self::empty_usage_cache_data(), Vec::new(), HashMap::new())
    }

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
        let quota_history_path = app_dir.join(QUOTA_HISTORY_FILENAME);

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

        // Load per-month quota history (YYYY-MM → limit)
        let quota_history: HashMap<String, u32> = if quota_history_path.exists() {
            match std::fs::read_to_string(&quota_history_path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
            {
                Some(map) => map,
                None => {
                    log::warn!("Failed to parse quota_history.json, starting fresh");
                    HashMap::new()
                }
            }
        } else {
            HashMap::new()
        };

        Ok(Self {
            settings_path,
            usage_cache_path,
            history_path,
            quota_history_path,
            settings: Mutex::new(settings),
            usage_cache: Mutex::new(usage_cache),
            usage_history: Mutex::new(history),
            quota_history: Mutex::new(quota_history),
            state_operation: Mutex::new(()),
            backup_creation: Mutex::new(()),
            session_generation: AtomicU64::new(1),
            session_transition_in_progress: AtomicBool::new(false),
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
    fn save_history_to_disk(path: &PathBuf, history: &[UsageEntry]) -> Result<(), String> {
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

    fn save_or_delete_history_file(path: &PathBuf, history: &[UsageEntry]) -> Result<(), String> {
        if history.is_empty() {
            if path.exists() {
                std::fs::remove_file(path)
                    .map_err(|e| format!("Failed to delete history file: {}", e))?;
            }
            return Ok(());
        }

        Self::save_history_to_disk(path, history)
    }

    fn save_or_delete_quota_history_file(
        path: &PathBuf,
        quota_history: &HashMap<String, u32>,
        failure_context: &str,
    ) -> Result<(), String> {
        if quota_history.is_empty() {
            if path.exists() {
                std::fs::remove_file(path).map_err(|e| {
                    format!(
                        "Failed to delete {} quota_history.json: {}",
                        failure_context, e
                    )
                })?;
            }
            return Ok(());
        }

        let json = serde_json::to_string_pretty(quota_history).map_err(|e| {
            format!(
                "Failed to serialize {} quota history: {}",
                failure_context, e
            )
        })?;
        write_file_atomic(path, &json).map_err(|e| {
            format!(
                "Failed to persist {} quota_history.json: {}",
                failure_context, e
            )
        })
    }

    fn snapshot_usage_state(&self) -> Result<UsageStateSnapshot, String> {
        let cache = self
            .usage_cache
            .lock()
            .map_err(|_| "Internal state error".to_string())?
            .clone();
        let history = self
            .usage_history
            .lock()
            .map_err(|_| "Internal state error".to_string())?
            .clone();
        let quota = self
            .quota_history
            .lock()
            .map_err(|_| "Internal state error".to_string())?
            .clone();

        Ok((cache, history, quota))
    }

    fn persist_usage_state_to_disk(
        &self,
        cache: &UsageCacheData,
        history: &[UsageEntry],
        quota: &HashMap<String, u32>,
        quota_failure_context: &str,
    ) -> Result<(), String> {
        Self::save_or_delete_history_file(&self.history_path, history)?;
        Self::save_usage_cache_to_disk(&self.usage_cache_path, cache)?;
        Self::save_or_delete_quota_history_file(
            &self.quota_history_path,
            quota,
            quota_failure_context,
        )
    }

    fn replace_usage_state(
        &self,
        cache: UsageCacheData,
        history: Vec<UsageEntry>,
        quota: HashMap<String, u32>,
    ) -> Result<(), String> {
        let previous_state = self.snapshot_usage_state()?;

        if let Err(error) = self.persist_usage_state_to_disk(&cache, &history, &quota, "new") {
            if let Err(rollback_error) = self.persist_usage_state_to_disk(
                &previous_state.0,
                &previous_state.1,
                &previous_state.2,
                "rolled back",
            ) {
                return Err(format!("{}; rollback failed: {}", error, rollback_error));
            }

            return Err(error);
        }

        {
            let mut current_cache = self
                .usage_cache
                .lock()
                .map_err(|_| "Internal state error".to_string())?;
            *current_cache = cache;
        }
        {
            let mut current_history = self
                .usage_history
                .lock()
                .map_err(|_| "Internal state error".to_string())?;
            *current_history = history;
        }
        {
            let mut current_quota = self
                .quota_history
                .lock()
                .map_err(|_| "Internal state error".to_string())?;
            *current_quota = quota;
        }

        Ok(())
    }

    pub fn persist_usage_snapshot(
        &self,
        used: f64,
        limit: u32,
        history: Vec<UsageEntry>,
        quota: HashMap<String, u32>,
    ) -> Result<(), String> {
        let mut cache = self.snapshot_usage_state()?.0;
        cache.last_usage = used;
        cache.usage_limit = limit;
        cache.last_fetch_timestamp = chrono::Utc::now().timestamp();
        self.replace_usage_state(cache, history, quota)
    }

    pub fn persist_history_snapshot(&self, history: Vec<UsageEntry>) -> Result<(), String> {
        let (cache, _, quota) = self.snapshot_usage_state()?;
        self.replace_usage_state(cache, history, quota)
    }

    /// Get a copy of current settings
    pub fn get_settings(&self) -> AppSettings {
        self.settings
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Update settings and persist to disk
    pub fn update_settings<F>(&self, updater: F) -> Result<(), String>
    where
        F: FnOnce(&mut AppSettings),
    {
        let mut settings = self
            .settings
            .lock()
            .map_err(|_| "Internal state error".to_string())?;
        let mut updated = settings.clone();
        updater(&mut updated);

        // Persist to disk
        Self::save_settings_to_disk(&self.settings_path, &updated)?;
        *settings = updated;

        Ok(())
    }

    fn replace_settings(&self, settings: AppSettings) -> Result<(), String> {
        let mut current_settings = self
            .settings
            .lock()
            .map_err(|_| "Internal state error".to_string())?;
        Self::save_settings_to_disk(&self.settings_path, &settings)?;
        *current_settings = settings;
        Ok(())
    }

    pub(crate) fn lock_state_operation(&self) -> Result<std::sync::MutexGuard<'_, ()>, String> {
        self.state_operation
            .lock()
            .map_err(|_| "Internal state error".to_string())
    }

    fn bump_session_generation(&self) -> u64 {
        self.session_generation.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn get_session_generation(&self) -> u64 {
        self.session_generation.load(Ordering::SeqCst)
    }

    pub fn begin_session_transition(&self) {
        self.session_transition_in_progress
            .store(true, Ordering::SeqCst);
        self.bump_session_generation();
    }

    pub fn finish_session_transition(&self) {
        self.bump_session_generation();
        self.session_transition_in_progress
            .store(false, Ordering::SeqCst);
    }

    pub fn is_session_transition_in_progress(&self) -> bool {
        self.session_transition_in_progress.load(Ordering::SeqCst)
    }

    /// Set customer ID
    pub fn set_customer_id(&self, id: u64) -> Result<(), String> {
        let _state_operation = self.lock_state_operation()?;
        self.set_customer_id_locked(id)
    }

    pub(crate) fn set_customer_id_locked(&self, id: u64) -> Result<(), String> {
        let previous_customer_id = self.get_customer_id();
        if previous_customer_id == Some(id) {
            return self.update_settings(|s| {
                s.customer_id = Some(id);
                s.is_authenticated = true;
            });
        }

        let previous_usage_state = self.snapshot_usage_state()?;
        self.begin_session_transition();
        let result = (|| {
            log::warn!(
                "Customer ID changed from {:?} to {}; clearing persisted usage state",
                previous_customer_id,
                id
            );
            self.clear_usage_state_to_empty()?;
            self.update_settings(|s| {
                s.customer_id = Some(id);
                s.is_authenticated = true;
            })
        })();
        if let Err(error) = result {
            if let Err(rollback_error) = self.replace_usage_state(
                previous_usage_state.0,
                previous_usage_state.1,
                previous_usage_state.2,
            ) {
                self.finish_session_transition();
                return Err(format!(
                    "{}; usage rollback failed: {}",
                    error, rollback_error
                ));
            }
            self.finish_session_transition();
            return Err(error);
        }

        self.finish_session_transition();
        Ok(())
    }

    /// Get customer ID
    pub fn get_customer_id(&self) -> Option<u64> {
        self.settings
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .customer_id
    }

    /// Set usage data (active data - written to usage_cache.json)
    pub fn set_usage(&self, used: f64, limit: u32) -> Result<(), String> {
        let _state_operation = self.lock_state_operation()?;
        let mut updated = self
            .usage_cache
            .lock()
            .map_err(|_| "Internal state error".to_string())?
            .clone();
        updated.last_usage = used;
        updated.usage_limit = limit;
        updated.last_fetch_timestamp = chrono::Utc::now().timestamp();

        Self::save_usage_cache_to_disk(&self.usage_cache_path, &updated)?;

        let mut cache = self
            .usage_cache
            .lock()
            .map_err(|_| "Internal state error".to_string())?;
        *cache = updated;

        Ok(())
    }

    /// Get usage data (from usage_cache.json)
    pub fn get_usage(&self) -> (f64, u32) {
        let cache = self.usage_cache.lock().unwrap_or_else(|e| e.into_inner());
        (cache.last_usage, cache.usage_limit)
    }

    /// Get last fetch timestamp (from usage_cache.json)
    pub fn get_last_fetch_timestamp(&self) -> i64 {
        self.usage_cache
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .last_fetch_timestamp
    }

    /// Set last update check timestamp (active data - written to usage_cache.json)
    pub fn set_last_update_check_timestamp(&self, timestamp: i64) -> Result<(), String> {
        let _state_operation = self.lock_state_operation()?;
        let mut cache = self
            .usage_cache
            .lock()
            .map_err(|_| "Internal state error".to_string())?;
        cache.last_update_check_timestamp = timestamp;

        // Persist to usage_cache.json
        Self::save_usage_cache_to_disk(&self.usage_cache_path, &cache)?;
        drop(cache);

        Ok(())
    }

    /// Get last update check timestamp (from usage_cache.json)
    pub fn get_last_update_check_timestamp(&self) -> i64 {
        self.usage_cache
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .last_update_check_timestamp
    }

    /// Set launch at login preference
    pub fn set_launch_at_login(&self, enabled: bool) -> Result<(), String> {
        self.update_settings(|s| {
            s.launch_at_login = enabled;
        })
    }

    /// Get launch at login preference
    pub fn get_launch_at_login(&self) -> bool {
        self.settings
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .launch_at_login
    }

    /// Set show notifications preference
    pub fn set_show_notifications(&self, enabled: bool) -> Result<(), String> {
        self.update_settings(|s| {
            s.show_notifications = enabled;
        })
    }

    /// Get show notifications preference
    pub fn get_show_notifications(&self) -> bool {
        self.settings
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .show_notifications
    }

    /// Check if authenticated
    pub fn is_authenticated(&self) -> bool {
        self.settings
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_authenticated
    }

    /// Clear authentication (logout)
    pub fn clear_auth(&self) -> Result<(), String> {
        self.update_settings(|s| {
            s.customer_id = None;
            s.is_authenticated = false;
        })
    }

    pub fn clear_user_session(&self) -> Result<(), String> {
        let _state_operation = self.lock_state_operation()?;
        let previous_usage_state = self.snapshot_usage_state()?;
        self.begin_session_transition();
        let result = (|| {
            self.clear_usage_state_to_empty()?;
            self.clear_auth()
        })();
        if let Err(error) = result {
            if let Err(rollback_error) = self.replace_usage_state(
                previous_usage_state.0,
                previous_usage_state.1,
                previous_usage_state.2,
            ) {
                self.finish_session_transition();
                return Err(format!(
                    "{}; usage rollback failed: {}",
                    error, rollback_error
                ));
            }
            self.finish_session_transition();
            return Err(error);
        }

        self.finish_session_transition();
        Ok(())
    }

    /// Export usage cache for API responses (legacy compatibility)
    pub fn export_usage_cache(&self) -> Result<UsageCache, String> {
        let settings = self
            .settings
            .lock()
            .map_err(|_| "Internal state error".to_string())?;
        let cache = self
            .usage_cache
            .lock()
            .map_err(|_| "Internal state error".to_string())?;

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
        let settings = self.settings.lock().unwrap_or_else(|e| e.into_inner());
        let cache = self.usage_cache.lock().unwrap_or_else(|e| e.into_inner());

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
            let mut cache = self.usage_cache.lock().unwrap_or_else(|e| e.into_inner());
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
        let mut guard = self.usage_history.lock().unwrap_or_else(|e| e.into_inner());
        guard.clear();
        // Also delete the history file
        if self.history_path.exists() {
            let _ = std::fs::remove_file(&self.history_path);
        }
        log::info!("Usage history cleared");
    }

    pub fn clear_quota_history(&self) {
        let mut guard = self.quota_history.lock().unwrap_or_else(|e| e.into_inner());
        guard.clear();
        if self.quota_history_path.exists() {
            let _ = std::fs::remove_file(&self.quota_history_path);
        }
        log::info!("Quota history cleared");
    }

    pub fn set_usage_history(&self, history: Vec<UsageEntry>) {
        if let Err(e) = Self::save_history_to_disk(&self.history_path, &history) {
            log::error!("Failed to save usage history to disk: {}", e);
        } else {
            let mut guard = self.usage_history.lock().unwrap_or_else(|e| e.into_inner());
            *guard = history.clone();
            drop(guard);
            log::info!(
                "Successfully saved {} history entries to disk",
                history.len()
            );
        }
    }

    pub fn get_usage_history(&self) -> Vec<UsageEntry> {
        self.usage_history
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    // -------------------------------------------------------------------------
    // Quota History — records the monthly quota (limit) observed at fetch time.
    // Keyed by "YYYY-MM". Persisted to quota_history.json.
    // This lets the chart show accurate utilization % even when the plan changes.
    // -------------------------------------------------------------------------

    /// Record (or update) the observed quota limit for a given month ("YYYY-MM").
    /// Skips recording if `limit == 0` (unknown/unset).
    pub fn record_quota_for_month(&self, month: &str, limit: u32) {
        if limit == 0 {
            return;
        }
        let updated_snapshot = {
            let h = self.quota_history.lock().unwrap_or_else(|e| e.into_inner());
            if h.get(month) == Some(&limit) {
                return;
            }
            let mut snapshot = h.clone();
            snapshot.insert(month.to_string(), limit);
            snapshot
        };

        match serde_json::to_string_pretty(&updated_snapshot) {
            Ok(json) => {
                if let Err(e) = write_file_atomic(&self.quota_history_path, &json) {
                    log::warn!("Failed to persist quota_history.json: {}", e);
                } else {
                    let mut h = self.quota_history.lock().unwrap_or_else(|e| e.into_inner());
                    *h = updated_snapshot;
                }
            }
            Err(e) => log::warn!("Failed to serialize quota history: {}", e),
        }
    }

    /// Returns a snapshot of the full YYYY-MM → limit map.
    pub fn get_quota_map(&self) -> HashMap<String, u32> {
        self.quota_history
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    pub fn reset_settings(&self) -> Result<AppSettings, String> {
        let _state_operation = self.lock_state_operation()?;
        let previous_settings = self.get_settings();
        let previous_usage_state = self.snapshot_usage_state()?;
        let defaults = AppSettings::default();
        self.begin_session_transition();
        let result = (|| {
            self.replace_settings(defaults.clone())?;
            self.replace_usage_state(Self::empty_usage_cache_data(), Vec::new(), HashMap::new())
        })();
        if let Err(error) = result {
            let rollback_usage_result = self.replace_usage_state(
                previous_usage_state.0,
                previous_usage_state.1,
                previous_usage_state.2,
            );
            if let Err(rollback_error) = self.replace_settings(previous_settings) {
                self.finish_session_transition();
                return Err(format!(
                    "{}; settings rollback failed: {}; usage rollback status: {}",
                    error,
                    rollback_error,
                    rollback_usage_result
                        .err()
                        .unwrap_or_else(|| "ok".to_string())
                ));
            }
            if let Err(rollback_error) = rollback_usage_result {
                self.finish_session_transition();
                return Err(format!(
                    "{}; usage rollback failed: {}",
                    error, rollback_error
                ));
            }
            self.finish_session_transition();
            return Err(error);
        }

        self.finish_session_transition();
        Ok(defaults)
    }

    /// Get the tray icon display format
    pub fn get_tray_icon_format(&self) -> String {
        self.settings
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .tray_icon_format
            .clone()
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
        self.settings
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .widget_enabled
    }

    /// Set widget enabled state
    pub fn set_widget_enabled(&self, enabled: bool) -> Result<(), String> {
        self.update_settings(|s| {
            s.widget_enabled = enabled;
        })
    }

    /// Get widget position
    pub fn get_widget_position(&self) -> WidgetPosition {
        self.settings
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .widget_position
            .clone()
    }

    /// Set widget position
    pub fn set_widget_position(&self, position: WidgetPosition) -> Result<(), String> {
        self.update_settings(|s| {
            s.widget_position = position;
        })
    }

    /// Get widget pinned state
    pub fn get_widget_pinned(&self) -> bool {
        self.settings
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .widget_pinned
    }

    /// Set widget pinned state
    pub fn set_widget_pinned(&self, pinned: bool) -> Result<(), String> {
        self.update_settings(|s| {
            s.widget_pinned = pinned;
        })
    }

    /// Get widget visible state
    pub fn get_widget_visible(&self) -> bool {
        self.settings
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .widget_visible
    }

    /// Set widget visible state
    pub fn set_widget_visible(&self, visible: bool) -> Result<(), String> {
        self.update_settings(|s| {
            s.widget_visible = visible;
        })
    }

    /// Get auto backup enabled state
    pub fn get_auto_backup_enabled(&self) -> bool {
        self.settings
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .auto_backup_enabled
    }

    /// Set auto backup enabled state
    pub fn set_auto_backup_enabled(&self, enabled: bool) -> Result<(), String> {
        self.update_settings(|s| {
            s.auto_backup_enabled = enabled;
        })
    }

    /// Check if an auto-backup should run now based on frequency settings
    pub fn should_auto_backup(&self) -> bool {
        let settings = self.settings.lock().unwrap_or_else(|e| e.into_inner());
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
        self.settings
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .backup_directory
            .clone()
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
            // Reject paths with traversal components
            if custom.contains("..") {
                log::warn!("Path traversal detected in backup_directory, falling back to default.");
                return self
                    .settings_path
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| std::path::PathBuf::from("."))
                    .join("backups");
            }
            let path = PathBuf::from(custom);
            if path.is_absolute() {
                path
            } else {
                // Resolve relative paths against the app directory to ensure
                // consistent behaviour regardless of the process working directory.
                self.settings_path
                    .parent()
                    .map(|p| p.to_path_buf())
                    .unwrap_or_else(|| std::path::PathBuf::from("."))
                    .join(&path)
            }
        } else {
            self.settings_path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| std::path::PathBuf::from("."))
                .join("backups")
        }
    }

    /// Create a backup of usage data (history + cache)
    pub fn create_backup(&self) -> Result<String, String> {
        let _state_operation = self.lock_state_operation()?;
        let _backup_guard = self
            .backup_creation
            .lock()
            .map_err(|_| "Internal state error".to_string())?;
        let settings = self.get_settings();
        let (usage_cache, history, quota_history) = self.snapshot_usage_state()?;
        let backups_dir = self.get_backups_path();

        // Ensure backups directory exists
        std::fs::create_dir_all(&backups_dir)
            .map_err(|e| format!("Failed to create backups directory: {}", e))?;

        // Generate a timestamp-based backup ID and add a numeric suffix if a
        // backup already exists for the same instant. This avoids overwriting an
        // earlier backup when two saves happen in the same clock tick.
        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S_%6f").to_string();
        let (backup_id, temp_dir, final_dir) = {
            let mut suffix = 0;

            loop {
                let backup_id = if suffix == 0 {
                    format!("backup_{}", timestamp)
                } else {
                    format!("backup_{}_{}", timestamp, suffix)
                };
                let temp_dir = backups_dir.join(format!(".temp_{}", backup_id));
                let final_dir = backups_dir.join(&backup_id);

                if !final_dir.exists() {
                    break (backup_id, temp_dir, final_dir);
                }

                suffix += 1;
            }
        };

        // Clean up any existing temp directory from failed attempts
        if temp_dir.exists() {
            std::fs::remove_dir_all(&temp_dir)
                .map_err(|e| format!("Failed to clean up temp directory: {}", e))?;
        }

        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| format!("Failed to create temp backup directory: {}", e))?;

        // Write metadata
        let mut files = Vec::new();
        let history_path = temp_dir.join("usage_history.json");
        write_file_atomic(
            &history_path,
            &serde_json::to_string_pretty(&history)
                .map_err(|e| format!("Failed to serialize history backup: {}", e))?,
        )
        .map_err(|e| format!("Failed to write history backup to temp: {}", e))?;
        files.push("usage_history.json".to_string());

        let cache_path = temp_dir.join("usage_cache.json");
        write_file_atomic(
            &cache_path,
            &serde_json::to_string_pretty(&usage_cache)
                .map_err(|e| format!("Failed to serialize cache backup: {}", e))?,
        )
        .map_err(|e| format!("Failed to write cache backup to temp: {}", e))?;
        files.push("usage_cache.json".to_string());

        let quota_path = temp_dir.join("quota_history.json");
        write_file_atomic(
            &quota_path,
            &serde_json::to_string_pretty(&quota_history)
                .map_err(|e| format!("Failed to serialize quota history backup: {}", e))?,
        )
        .map_err(|e| format!("Failed to write quota history backup to temp: {}", e))?;
        files.push("quota_history.json".to_string());

        let metadata = BackupMetadata {
            backup_id: backup_id.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
            files,
            customer_id: settings.customer_id,
        };

        let metadata_path = temp_dir.join("metadata.json");
        std::fs::write(
            &metadata_path,
            serde_json::to_string_pretty(&metadata).unwrap(),
        )
        .map_err(|e| format!("Failed to write metadata to temp: {}", e))?;

        // Atomic rename from temp to final destination
        std::fs::rename(&temp_dir, &final_dir)
            .map_err(|e| format!("Failed to finalize backup directory: {}", e))?;

        log::info!("Backup created: {}", backup_id);

        // Prune old backups if retention limit is set
        let retention = self
            .settings
            .lock()
            .map_err(|_| "Internal state error".to_string())?
            .backup_retention_count;
        if retention > 0 {
            if let Err(e) = self.prune_backups(retention) {
                log::warn!("Backup pruning failed (non-fatal): {}", e);
            }
        }

        Ok(backup_id)
    }

    /// Restore usage data from a backup
    pub fn restore_backup(&self, backup_id: &str) -> Result<(), String> {
        let _state_operation = self.lock_state_operation()?;
        self.begin_session_transition();
        let result = (|| {
            validate_backup_id(backup_id)?;
            let backups_dir = self.get_backups_path();
            let backup_dir = backups_dir.join(backup_id);

            if !backup_dir.exists() {
                return Err(format!("Backup not found: {}", backup_id));
            }

            // Read metadata to verify — metadata must exist for a valid backup
            let metadata_path = backup_dir.join("metadata.json");
            if !metadata_path.exists() {
                return Err(format!("Backup metadata.json missing: {}", backup_id));
            }
            let metadata: BackupMetadata = serde_json::from_str(
                &std::fs::read_to_string(&metadata_path)
                    .map_err(|e| format!("Failed to read metadata: {}", e))?,
            )
            .map_err(|e| format!("Invalid metadata: {}", e))?;

            match (self.get_customer_id(), metadata.customer_id) {
                (Some(current_customer_id), Some(backup_customer_id))
                    if backup_customer_id != current_customer_id =>
                {
                    return Err(format!(
                        "Backup belongs to a different account (backup={}, current={})",
                        backup_customer_id, current_customer_id
                    ));
                }
                (Some(_), None) => {
                    return Err(
                        "Backup does not include customer identity and cannot be restored while authenticated".to_string(),
                    );
                }
                (None, Some(backup_customer_id)) => {
                    return Err(format!(
                        "Authenticated backup for account {} cannot be restored while logged out",
                        backup_customer_id
                    ));
                }
                _ => {}
            }

            let expected_files: HashSet<String> = metadata.files.iter().cloned().collect();
            let files_listed_in_metadata = !expected_files.is_empty();
            let expects_file = |file_name: &str| {
                if files_listed_in_metadata {
                    expected_files.contains(file_name)
                } else {
                    false
                }
            };

            // Phase 1: Load and validate ALL backup files in memory before writing anything.
            // This prevents partial restores if a later file fails to parse.
            let history_backup = backup_dir.join("usage_history.json");
            if expects_file("usage_history.json") && !history_backup.exists() {
                return Err("usage_history.json is missing from backup".to_string());
            }
            let new_history: Vec<crate::usage::UsageEntry> = if history_backup.exists() {
                serde_json::from_str(
                    &std::fs::read_to_string(&history_backup)
                        .map_err(|e| format!("Failed to read history backup: {}", e))?,
                )
                .map_err(|e| format!("Invalid history backup: {}", e))?
            } else {
                Vec::new()
            };

            let cache_backup = backup_dir.join("usage_cache.json");
            if expects_file("usage_cache.json") && !cache_backup.exists() {
                return Err("usage_cache.json is missing from backup".to_string());
            }
            let new_cache: UsageCacheData = if cache_backup.exists() {
                serde_json::from_str(
                    &std::fs::read_to_string(&cache_backup)
                        .map_err(|e| format!("Failed to read cache backup: {}", e))?,
                )
                .map_err(|e| format!("Invalid cache backup: {}", e))?
            } else {
                Self::empty_usage_cache_data()
            };

            let quota_backup = backup_dir.join("quota_history.json");
            if expects_file("quota_history.json") && !quota_backup.exists() {
                return Err("quota_history.json is missing from backup".to_string());
            }
            let new_quota: HashMap<String, u32> = if quota_backup.exists() {
                serde_json::from_str(
                    &std::fs::read_to_string(&quota_backup)
                        .map_err(|e| format!("Failed to read quota history backup: {}", e))?,
                )
                .map_err(|e| format!("Invalid quota history backup: {}", e))?
            } else {
                HashMap::new()
            };

            // Phase 2: All reads succeeded — safe to apply changes.
            self.replace_usage_state(new_cache, new_history, new_quota)?;

            log::info!("Backup restored: {}", backup_id);
            Ok(())
        })();
        self.finish_session_transition();
        result
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

                // Skip temporary directories
                if backup_id.starts_with(".temp_") {
                    continue;
                }

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

        // Sort by creation date (newest first).
        // Treat empty or "unknown" created_at values as epoch so corrupted/
        // metadata-less backups sort as oldest and are pruned before valid ones.
        backups.sort_by(|a, b| {
            let a_date = if a.created_at.is_empty() || a.created_at == "unknown" {
                "1970-01-01T00:00:00Z"
            } else {
                a.created_at.as_str()
            };
            let b_date = if b.created_at.is_empty() || b.created_at == "unknown" {
                "1970-01-01T00:00:00Z"
            } else {
                b.created_at.as_str()
            };
            b_date.cmp(a_date)
        });

        Ok(backups)
    }

    /// Delete a backup
    pub fn delete_backup(&self, backup_id: &str) -> Result<(), String> {
        validate_backup_id(backup_id)?;
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

        let mut first_error: Option<String> = None;
        for backup in to_delete {
            let backups_dir = self.get_backups_path();
            let backup_dir = backups_dir.join(&backup.backup_id);
            if backup_dir.exists() {
                match std::fs::remove_dir_all(&backup_dir) {
                    Ok(()) => log::info!("Pruned old backup: {}", backup.backup_id),
                    Err(e) => {
                        let msg = format!("Failed to prune backup {}: {}", backup.backup_id, e);
                        log::warn!("{}", msg);
                        if first_error.is_none() {
                            first_error = Some(msg);
                        }
                    }
                }
            }
        }

        if let Some(err) = first_error {
            return Err(err);
        }

        Ok(())
    }
}

/// Reject backup IDs that contain path separators or traversal sequences.
/// This prevents directory traversal attacks where a crafted backup_id could
/// escape the backups directory (e.g., "../../settings.json").
/// Also enforces the "backup_" prefix to filter out Windows reserved device
/// names (COM1, NUL, CON, etc.) that could cause filesystem errors.
/// Null bytes are rejected because some OS path APIs terminate at `\0`, which
/// can split a path unexpectedly on certain platforms.
fn validate_backup_id(backup_id: &str) -> Result<(), String> {
    if !backup_id.starts_with("backup_")
        || backup_id.contains('/')
        || backup_id.contains('\\')
        || backup_id.contains("..")
        || backup_id.contains('\0')
    {
        return Err(format!("Invalid backup ID: {}", backup_id));
    }
    Ok(())
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackupMetadata {
    backup_id: String,
    created_at: String,
    #[serde(default)]
    files: Vec<String>,
    #[serde(default)]
    customer_id: Option<u64>,
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

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use tempfile::TempDir;

    // ─── BackupFrequency enum ───────────────────────────────────────────────

    #[test]
    fn backup_frequency_default_is_daily() {
        assert_eq!(BackupFrequency::default(), BackupFrequency::Daily);
    }

    #[test]
    fn backup_frequency_variants_serialize_to_camel_case() {
        assert_eq!(
            serde_json::to_string(&BackupFrequency::EveryRefresh).unwrap(),
            r#""everyRefresh""#
        );
        assert_eq!(
            serde_json::to_string(&BackupFrequency::Daily).unwrap(),
            r#""daily""#
        );
        assert_eq!(
            serde_json::to_string(&BackupFrequency::Every3Days).unwrap(),
            r#""every3Days""#
        );
        assert_eq!(
            serde_json::to_string(&BackupFrequency::Weekly).unwrap(),
            r#""weekly""#
        );
    }

    #[test]
    fn backup_frequency_deserializes_from_camel_case() {
        let freq: BackupFrequency = serde_json::from_str(r#""everyRefresh""#).unwrap();
        assert_eq!(freq, BackupFrequency::EveryRefresh);

        let freq: BackupFrequency = serde_json::from_str(r#""daily""#).unwrap();
        assert_eq!(freq, BackupFrequency::Daily);

        let freq: BackupFrequency = serde_json::from_str(r#""every3Days""#).unwrap();
        assert_eq!(freq, BackupFrequency::Every3Days);

        let freq: BackupFrequency = serde_json::from_str(r#""weekly""#).unwrap();
        assert_eq!(freq, BackupFrequency::Weekly);
    }

    // ─── AppSettings backup defaults ───────────────────────────────────────

    #[test]
    fn app_settings_default_auto_backup_enabled_is_false() {
        let settings = AppSettings::default();
        assert!(!settings.auto_backup_enabled);
    }

    #[test]
    fn app_settings_default_backup_frequency_is_daily() {
        let settings = AppSettings::default();
        assert_eq!(settings.backup_frequency, BackupFrequency::Daily);
    }

    #[test]
    fn app_settings_default_backup_retention_count_is_ten() {
        let settings = AppSettings::default();
        assert_eq!(settings.backup_retention_count, 10);
    }

    #[test]
    fn app_settings_default_last_auto_backup_at_is_none() {
        let settings = AppSettings::default();
        assert!(settings.last_auto_backup_at.is_none());
    }

    #[test]
    fn app_settings_default_backup_directory_is_none() {
        let settings = AppSettings::default();
        assert!(settings.backup_directory.is_none());
    }

    // ─── should_auto_backup logic ───────────────────────────────────────────

    fn make_store(tmp: &TempDir) -> StoreManager {
        StoreManager::new(tmp.path().to_path_buf()).unwrap()
    }

    fn usage_entry(timestamp: i64, used: f64, limit: u32) -> crate::usage::UsageEntry {
        crate::usage::UsageEntry {
            timestamp,
            used,
            limit,
            included_requests: used,
            billed_requests: 0.0,
            gross_amount: 0.0,
            billed_amount: 0.0,
            models: vec![],
            quota_estimated: false,
        }
    }

    #[test]
    fn clear_user_session_clears_stale_usage_state() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);

        store.set_customer_id(1).unwrap();
        store.set_usage(42.0, 1200).unwrap();
        store.set_usage_history(vec![crate::usage::UsageEntry {
            timestamp: 1_712_448_000,
            used: 42.0,
            limit: 1200,
            included_requests: 42.0,
            billed_requests: 0.0,
            gross_amount: 0.0,
            billed_amount: 0.0,
            models: vec![],
            quota_estimated: false,
        }]);
        store.record_quota_for_month("2026-04", 1200);

        store.clear_user_session().unwrap();

        assert_eq!(store.get_customer_id(), None);
        assert_eq!(store.get_usage(), (0.0, 0));
        assert!(store.get_usage_history().is_empty());
        assert!(store.get_quota_map().is_empty());
    }

    #[test]
    fn clear_user_session_advances_session_generation() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        let generation_before = store.get_session_generation();

        store.clear_user_session().unwrap();

        assert!(store.get_session_generation() > generation_before);
    }

    #[test]
    fn reset_settings_advances_session_generation() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        let generation_before = store.get_session_generation();

        store.reset_settings().unwrap();

        assert!(store.get_session_generation() > generation_before);
    }

    #[test]
    fn set_customer_id_clears_usage_state_when_account_changes() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);

        store.set_customer_id(1).unwrap();
        store.set_usage(42.0, 1200).unwrap();
        store.set_usage_history(vec![crate::usage::UsageEntry {
            timestamp: 1_712_448_000,
            used: 42.0,
            limit: 1200,
            included_requests: 42.0,
            billed_requests: 0.0,
            gross_amount: 0.0,
            billed_amount: 0.0,
            models: vec![],
            quota_estimated: false,
        }]);
        store.record_quota_for_month("2026-04", 1200);

        store.set_customer_id(2).unwrap();

        assert_eq!(store.get_customer_id(), Some(2));
        assert_eq!(store.get_usage(), (0.0, 0));
        assert!(store.get_usage_history().is_empty());
        assert!(store.get_quota_map().is_empty());
    }

    #[test]
    fn set_customer_id_restores_usage_state_when_settings_persist_fails() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);

        store.set_customer_id(1).unwrap();
        store.set_usage(42.0, 1200).unwrap();
        store.set_usage_history(vec![usage_entry(1_712_448_000, 42.0, 1200)]);
        store.record_quota_for_month("2026-04", 1200);

        std::fs::remove_file(&store.settings_path).unwrap();
        std::fs::create_dir(&store.settings_path).unwrap();

        let result = store.set_customer_id(2);

        assert!(result.is_err());
        assert_eq!(store.get_customer_id(), Some(1));
        assert!(store.is_authenticated());
        assert_eq!(store.get_usage(), (42.0, 1200));
        let history = store.get_usage_history();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].timestamp, 1_712_448_000);
        assert_eq!(history[0].used, 42.0);
        assert_eq!(history[0].limit, 1200);
        assert_eq!(
            store.get_quota_map(),
            HashMap::from([(String::from("2026-04"), 1200)])
        );
    }

    #[test]
    fn set_customer_id_clears_stale_sidecars_when_settings_recover_to_defaults() {
        let tmp = TempDir::new().unwrap();
        {
            let seeded = make_store(&tmp);
            seeded.set_customer_id(1).unwrap();
            seeded.set_usage(42.0, 1200).unwrap();
            seeded.set_usage_history(vec![crate::usage::UsageEntry {
                timestamp: 1_712_448_000,
                used: 42.0,
                limit: 1200,
                included_requests: 42.0,
                billed_requests: 0.0,
                gross_amount: 0.0,
                billed_amount: 0.0,
                models: vec![],
                quota_estimated: false,
            }]);
            seeded.record_quota_for_month("2026-04", 1200);
        }

        std::fs::write(tmp.path().join(STORE_FILENAME), "{not valid json").unwrap();

        let recovered = make_store(&tmp);
        assert_eq!(recovered.get_customer_id(), None);
        assert_eq!(recovered.get_usage(), (42.0, 1200));
        assert!(!recovered.get_usage_history().is_empty());
        assert!(!recovered.get_quota_map().is_empty());

        recovered.set_customer_id(2).unwrap();

        assert_eq!(recovered.get_customer_id(), Some(2));
        assert_eq!(recovered.get_usage(), (0.0, 0));
        assert!(recovered.get_usage_history().is_empty());
        assert!(recovered.get_quota_map().is_empty());
    }

    #[test]
    fn reset_settings_clears_quota_history() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);

        store.record_quota_for_month("2026-04", 1200);
        assert_eq!(store.get_quota_map().get("2026-04"), Some(&1200));

        store.reset_settings().unwrap();

        assert!(store.get_quota_map().is_empty());
    }

    #[test]
    fn reset_settings_clears_usage_cache_to_empty_state() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);

        store.set_customer_id(1).unwrap();
        store.set_usage(42.0, 1200).unwrap();

        store.reset_settings().unwrap();

        assert_eq!(store.get_usage(), (0.0, 0));
    }

    #[test]
    fn reset_settings_preserves_existing_settings_when_usage_state_clear_fails() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);

        store.set_customer_id(1).unwrap();
        store
            .update_settings(|settings| {
                settings.refresh_interval = 42;
                settings.prediction_period = 21;
            })
            .unwrap();
        store.set_usage(42.0, 1200).unwrap();

        std::fs::remove_file(&store.usage_cache_path).unwrap();
        std::fs::create_dir(&store.usage_cache_path).unwrap();

        let result = store.reset_settings();

        assert!(result.is_err());
        let settings = store.get_settings();
        assert_eq!(settings.customer_id, Some(1));
        assert!(settings.is_authenticated);
        assert_eq!(settings.refresh_interval, 42);
        assert_eq!(settings.prediction_period, 21);
        assert_eq!(store.get_usage(), (42.0, 1200));
    }

    #[test]
    fn create_backup_metadata_includes_customer_id() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        store.set_customer_id(77).unwrap();

        let backup_id = store.create_backup().unwrap();
        let metadata_path = store
            .get_backups_path()
            .join(&backup_id)
            .join("metadata.json");
        let metadata: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(metadata_path).unwrap()).unwrap();

        assert_eq!(metadata["customer_id"].as_u64(), Some(77));
    }

    #[test]
    fn restore_backup_fails_when_metadata_lists_a_missing_file() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        store.set_customer_id(1).unwrap();
        store.set_usage_history(vec![crate::usage::UsageEntry {
            timestamp: 1_712_448_000,
            used: 42.0,
            limit: 1200,
            included_requests: 42.0,
            billed_requests: 0.0,
            gross_amount: 0.0,
            billed_amount: 0.0,
            models: vec![],
            quota_estimated: false,
        }]);

        let backup_id = store.create_backup().unwrap();
        let backup_dir = store.get_backups_path().join(&backup_id);
        std::fs::remove_file(backup_dir.join("usage_history.json")).unwrap();

        let result = store.restore_backup(&backup_id);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("usage_history.json is missing from backup"));
        assert_eq!(store.get_usage(), (0.0, 0));
        assert_eq!(store.get_customer_id(), Some(1));
        assert_eq!(store.get_usage_history().len(), 1);
    }

    #[test]
    fn restore_backup_rejects_different_authenticated_customer() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        store.set_customer_id(1).unwrap();

        let backup_id = store.create_backup().unwrap();

        store.set_customer_id(2).unwrap();
        let result = store.restore_backup(&backup_id);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("belongs to a different account"));
    }

    #[test]
    fn restore_backup_rejects_unauthenticated_backup_while_authenticated() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);

        let backup_id = store.create_backup().unwrap();

        store.set_customer_id(2).unwrap();
        let result = store.restore_backup(&backup_id);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("does not include customer identity"));
    }

    #[test]
    fn restore_backup_rejects_authenticated_backup_while_logged_out() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);

        store.set_customer_id(77).unwrap();
        store.set_usage(42.0, 1200).unwrap();
        let backup_id = store.create_backup().unwrap();

        store.clear_user_session().unwrap();
        let result = store.restore_backup(&backup_id);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("cannot be restored while logged out"));
        assert_eq!(store.get_customer_id(), None);
        assert!(!store.is_authenticated());
        assert_eq!(store.get_usage(), (0.0, 0));
    }

    #[test]
    fn restore_backup_advances_session_generation() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        let backup_id = store.create_backup().unwrap();
        let generation_before = store.get_session_generation();

        store.restore_backup(&backup_id).unwrap();

        assert!(store.get_session_generation() > generation_before);
    }

    #[test]
    fn restore_backup_clears_missing_history_and_quota_files() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);

        store.set_customer_id(1).unwrap();
        store.set_usage(42.0, 1200).unwrap();
        store.set_usage_history(vec![usage_entry(1_712_448_000, 42.0, 1200)]);
        store.record_quota_for_month("2026-04", 1200);

        store.clear_user_session().unwrap();
        let backup_id = store.create_backup().unwrap();

        store.set_usage(99.0, 2400).unwrap();
        store.set_usage_history(vec![usage_entry(1_712_534_400, 99.0, 2400)]);
        store.record_quota_for_month("2026-05", 2400);

        store.restore_backup(&backup_id).unwrap();

        assert_eq!(store.get_usage(), (0.0, 0));
        assert!(store.get_usage_history().is_empty());
        assert!(store.get_quota_map().is_empty());
    }

    #[test]
    fn restore_backup_keeps_current_in_memory_state_when_apply_fails() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);

        store.set_customer_id(1).unwrap();
        store.set_usage(42.0, 1200).unwrap();
        store.set_usage_history(vec![usage_entry(1_712_448_000, 42.0, 1200)]);
        store.record_quota_for_month("2026-04", 1200);
        let backup_id = store.create_backup().unwrap();

        store.set_usage(99.0, 2400).unwrap();
        store.set_usage_history(vec![usage_entry(1_712_534_400, 99.0, 2400)]);
        store.record_quota_for_month("2026-05", 2400);
        let expected_quota = store.get_quota_map();

        std::fs::remove_file(&store.quota_history_path).unwrap();
        std::fs::create_dir(&store.quota_history_path).unwrap();

        let result = store.restore_backup(&backup_id);

        assert!(result.is_err());
        assert_eq!(store.get_usage(), (99.0, 2400));
        let history = store.get_usage_history();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].timestamp, 1_712_534_400);
        assert_eq!(history[0].used, 99.0);
        assert_eq!(history[0].limit, 2400);
        assert_eq!(store.get_quota_map(), expected_quota);
    }

    #[test]
    fn clear_user_session_keeps_auth_when_usage_state_clear_fails() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);

        store.set_customer_id(1).unwrap();
        store.set_usage(42.0, 1200).unwrap();
        store.set_usage_history(vec![usage_entry(1_712_448_000, 42.0, 1200)]);
        store.record_quota_for_month("2026-04", 1200);

        std::fs::remove_file(&store.usage_cache_path).unwrap();
        std::fs::create_dir(&store.usage_cache_path).unwrap();

        let result = store.clear_user_session();

        assert!(result.is_err());
        assert_eq!(store.get_customer_id(), Some(1));
        assert!(store.is_authenticated());
        assert_eq!(store.get_usage(), (42.0, 1200));
        let history = store.get_usage_history();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].timestamp, 1_712_448_000);
        assert_eq!(history[0].used, 42.0);
        assert_eq!(history[0].limit, 1200);
        assert_eq!(
            store.get_quota_map(),
            HashMap::from([(String::from("2026-04"), 1200)])
        );
    }

    #[test]
    fn clear_user_session_restores_usage_state_when_auth_clear_fails() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);

        store.set_customer_id(1).unwrap();
        store.set_usage(42.0, 1200).unwrap();
        store.set_usage_history(vec![usage_entry(1_712_448_000, 42.0, 1200)]);
        store.record_quota_for_month("2026-04", 1200);

        std::fs::remove_file(&store.settings_path).unwrap();
        std::fs::create_dir(&store.settings_path).unwrap();

        let result = store.clear_user_session();

        assert!(result.is_err());
        assert_eq!(store.get_customer_id(), Some(1));
        assert!(store.is_authenticated());
        assert_eq!(store.get_usage(), (42.0, 1200));
        let history = store.get_usage_history();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].timestamp, 1_712_448_000);
        assert_eq!(history[0].used, 42.0);
        assert_eq!(history[0].limit, 1200);
        assert_eq!(
            store.get_quota_map(),
            HashMap::from([(String::from("2026-04"), 1200)])
        );
    }

    #[test]
    fn create_backup_is_safe_under_parallel_calls() {
        let tmp = TempDir::new().unwrap();
        let store = Arc::new(make_store(&tmp));
        store
            .update_settings(|s| {
                s.backup_retention_count = 0;
            })
            .unwrap();

        const THREADS: usize = 16;
        const ROUNDS: usize = 12;

        for round in 0..ROUNDS {
            let barrier = Arc::new(Barrier::new(THREADS));
            let mut handles = Vec::with_capacity(THREADS);

            for _ in 0..THREADS {
                let store = Arc::clone(&store);
                let barrier = Arc::clone(&barrier);
                handles.push(std::thread::spawn(move || {
                    barrier.wait();
                    store.create_backup()
                }));
            }

            let mut backup_ids = Vec::with_capacity(THREADS);
            for handle in handles {
                backup_ids.push(
                    handle
                        .join()
                        .expect("thread should finish cleanly")
                        .expect("parallel backup should succeed"),
                );
            }

            backup_ids.sort();
            backup_ids.dedup();
            assert_eq!(
                backup_ids.len(),
                THREADS,
                "parallel round {} should produce {} distinct backups",
                round,
                THREADS
            );
        }
    }

    #[test]
    fn should_auto_backup_returns_false_when_disabled() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        // auto_backup_enabled defaults to false
        assert!(!store.should_auto_backup());
    }

    #[test]
    fn should_auto_backup_returns_true_when_enabled_and_no_previous_backup() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        store.set_auto_backup_enabled(true).unwrap();
        // No last_auto_backup_at set, so it should backup
        assert!(store.should_auto_backup());
    }

    #[test]
    fn should_auto_backup_every_refresh_always_returns_true_when_enabled() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        store.set_auto_backup_enabled(true).unwrap();
        store
            .update_settings(|s| {
                s.backup_frequency = BackupFrequency::EveryRefresh;
                // Even with a very recent timestamp, EveryRefresh should backup
                s.last_auto_backup_at = Some(chrono::Utc::now().to_rfc3339());
            })
            .unwrap();
        assert!(store.should_auto_backup());
    }

    #[test]
    fn should_auto_backup_daily_returns_false_when_backup_was_recent() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        store.set_auto_backup_enabled(true).unwrap();
        // Set last backup to 1 hour ago (within 24h threshold)
        let recent = chrono::Utc::now() - chrono::Duration::hours(1);
        store
            .update_settings(|s| {
                s.backup_frequency = BackupFrequency::Daily;
                s.last_auto_backup_at = Some(recent.to_rfc3339());
            })
            .unwrap();
        assert!(!store.should_auto_backup());
    }

    #[test]
    fn should_auto_backup_daily_returns_true_when_backup_was_over_24h_ago() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        store.set_auto_backup_enabled(true).unwrap();
        // Set last backup to 25 hours ago (past 24h threshold)
        let old = chrono::Utc::now() - chrono::Duration::hours(25);
        store
            .update_settings(|s| {
                s.backup_frequency = BackupFrequency::Daily;
                s.last_auto_backup_at = Some(old.to_rfc3339());
            })
            .unwrap();
        assert!(store.should_auto_backup());
    }

    #[test]
    fn should_auto_backup_every3days_returns_false_when_backup_was_recent() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        store.set_auto_backup_enabled(true).unwrap();
        let recent = chrono::Utc::now() - chrono::Duration::hours(48);
        store
            .update_settings(|s| {
                s.backup_frequency = BackupFrequency::Every3Days;
                s.last_auto_backup_at = Some(recent.to_rfc3339());
            })
            .unwrap();
        // 48h < 72h threshold
        assert!(!store.should_auto_backup());
    }

    #[test]
    fn should_auto_backup_every3days_returns_true_when_backup_was_over_72h_ago() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        store.set_auto_backup_enabled(true).unwrap();
        let old = chrono::Utc::now() - chrono::Duration::hours(73);
        store
            .update_settings(|s| {
                s.backup_frequency = BackupFrequency::Every3Days;
                s.last_auto_backup_at = Some(old.to_rfc3339());
            })
            .unwrap();
        assert!(store.should_auto_backup());
    }

    #[test]
    fn should_auto_backup_weekly_returns_false_when_backup_was_recent() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        store.set_auto_backup_enabled(true).unwrap();
        let recent = chrono::Utc::now() - chrono::Duration::hours(100);
        store
            .update_settings(|s| {
                s.backup_frequency = BackupFrequency::Weekly;
                s.last_auto_backup_at = Some(recent.to_rfc3339());
            })
            .unwrap();
        // 100h < 168h threshold
        assert!(!store.should_auto_backup());
    }

    #[test]
    fn should_auto_backup_weekly_returns_true_when_backup_was_over_168h_ago() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        store.set_auto_backup_enabled(true).unwrap();
        let old = chrono::Utc::now() - chrono::Duration::hours(169);
        store
            .update_settings(|s| {
                s.backup_frequency = BackupFrequency::Weekly;
                s.last_auto_backup_at = Some(old.to_rfc3339());
            })
            .unwrap();
        assert!(store.should_auto_backup());
    }

    #[test]
    fn should_auto_backup_ignores_invalid_timestamp() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        store.set_auto_backup_enabled(true).unwrap();
        store
            .update_settings(|s| {
                s.backup_frequency = BackupFrequency::Daily;
                s.last_auto_backup_at = Some("not-a-valid-timestamp".to_string());
            })
            .unwrap();
        // Invalid timestamp → treat as no backup → should backup
        assert!(store.should_auto_backup());
    }

    // ─── record_auto_backup_time ────────────────────────────────────────────

    #[test]
    fn record_auto_backup_time_sets_last_auto_backup_at() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        assert!(store.settings.lock().unwrap().last_auto_backup_at.is_none());
        store.record_auto_backup_time().unwrap();
        let ts = store
            .settings
            .lock()
            .unwrap()
            .last_auto_backup_at
            .clone()
            .unwrap();
        // Should be a valid RFC3339 timestamp
        assert!(chrono::DateTime::parse_from_rfc3339(&ts).is_ok());
    }

    // ─── backup directory ───────────────────────────────────────────────────

    #[test]
    fn get_backups_path_returns_default_when_no_custom_directory() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        let backups_path = store.get_backups_path();
        assert!(backups_path.ends_with("backups"));
    }

    #[test]
    fn get_backups_path_returns_custom_directory_when_set() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        let custom = std::env::temp_dir().join("my-custom-backups");
        store
            .set_backup_directory(Some(custom.to_string_lossy().into_owned()))
            .unwrap();
        let backups_path = store.get_backups_path();
        assert_eq!(backups_path, custom);
    }

    #[test]
    fn get_backups_path_resolves_relative_custom_path_against_app_dir() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        store
            .set_backup_directory(Some("my-backups".to_string()))
            .unwrap();
        let backups_path = store.get_backups_path();
        // A relative path must be resolved against the app data directory,
        // not the process CWD, so the result must start with the tmp dir.
        assert!(
            backups_path.starts_with(tmp.path()),
            "relative custom path must be resolved under the app directory, got: {:?}",
            backups_path
        );
        assert!(backups_path.ends_with("my-backups"));
    }

    #[test]
    fn get_backups_path_rejects_path_traversal() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        store
            .set_backup_directory(Some("../../etc/evil".to_string()))
            .unwrap();
        let backups_path = store.get_backups_path();
        // Path traversal should fall back to default "backups" directory
        assert!(
            backups_path.ends_with("backups"),
            "path traversal must fall back to default, got: {:?}",
            backups_path
        );
    }

    #[test]
    fn set_backup_directory_to_none_resets_to_default() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        store
            .set_backup_directory(Some("/custom".to_string()))
            .unwrap();
        store.set_backup_directory(None).unwrap();
        assert!(store.get_backup_directory().is_none());
        // Path should revert to default backups/
        let backups_path = store.get_backups_path();
        assert!(backups_path.ends_with("backups"));
    }

    // ─── create_backup and list_backups ────────────────────────────────────

    #[test]
    fn create_backup_returns_backup_id_with_prefix() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        let backup_id = store.create_backup().unwrap();
        assert!(
            backup_id.starts_with("backup_"),
            "backup_id must start with 'backup_', got: {}",
            backup_id
        );
    }

    #[test]
    fn create_backup_creates_directory_with_metadata() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        let backup_id = store.create_backup().unwrap();
        let backup_dir = store.get_backups_path().join(&backup_id);
        assert!(backup_dir.exists(), "backup directory must be created");
        let metadata_path = backup_dir.join("metadata.json");
        assert!(metadata_path.exists(), "metadata.json must be created");

        // Verify metadata contains backup_id and created_at
        let content = std::fs::read_to_string(&metadata_path).unwrap();
        let meta: serde_json::Value = serde_json::from_str(&content).unwrap();
        assert_eq!(meta["backup_id"].as_str().unwrap(), backup_id);
        assert!(meta["created_at"].as_str().is_some());
    }

    #[test]
    fn create_backup_serializes_snapshotted_state_when_sidecar_files_are_missing() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        store.set_customer_id(77).unwrap();
        store.set_usage(42.0, 1200).unwrap();
        store.set_usage_history(vec![usage_entry(1_712_448_000, 42.0, 1200)]);
        store.record_quota_for_month("2026-04", 1200);

        std::fs::remove_file(&store.usage_cache_path).unwrap();
        std::fs::remove_file(&store.history_path).unwrap();
        std::fs::remove_file(&store.quota_history_path).unwrap();

        let backup_id = store.create_backup().unwrap();
        let backup_dir = store.get_backups_path().join(&backup_id);
        let metadata: BackupMetadata = serde_json::from_str(
            &std::fs::read_to_string(backup_dir.join("metadata.json")).unwrap(),
        )
        .unwrap();

        assert!(
            metadata.files.contains(&"usage_cache.json".to_string()),
            "in-memory cache snapshot must be backed up even when usage_cache.json is missing"
        );
        assert!(
            metadata.files.contains(&"usage_history.json".to_string()),
            "in-memory history snapshot must be backed up even when usage_history.json is missing"
        );
        assert!(
            metadata.files.contains(&"quota_history.json".to_string()),
            "in-memory quota snapshot must be backed up even when quota_history.json is missing"
        );

        let restored_cache: UsageCacheData = serde_json::from_str(
            &std::fs::read_to_string(backup_dir.join("usage_cache.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(restored_cache.last_usage, 42.0);
        assert_eq!(restored_cache.usage_limit, 1200);

        let restored_history: Vec<crate::usage::UsageEntry> = serde_json::from_str(
            &std::fs::read_to_string(backup_dir.join("usage_history.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(restored_history.len(), 1);
        assert_eq!(restored_history[0].used, 42.0);

        let restored_quota: HashMap<String, u32> = serde_json::from_str(
            &std::fs::read_to_string(backup_dir.join("quota_history.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            restored_quota,
            HashMap::from([(String::from("2026-04"), 1200)])
        );
    }

    #[test]
    fn set_usage_keeps_previous_cache_when_persistence_fails() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        store.set_usage(42.0, 1200).unwrap();

        std::fs::remove_file(&store.usage_cache_path).unwrap();
        std::fs::create_dir(&store.usage_cache_path).unwrap();

        let result = store.set_usage(99.0, 2400);

        assert!(result.is_err());
        assert_eq!(store.get_usage(), (42.0, 1200));
    }

    #[test]
    fn set_usage_history_keeps_previous_state_when_persistence_fails() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        store.set_usage_history(vec![usage_entry(1_712_448_000, 42.0, 1200)]);

        std::fs::remove_file(&store.history_path).unwrap();
        std::fs::create_dir(&store.history_path).unwrap();

        store.set_usage_history(vec![usage_entry(1_712_534_400, 99.0, 2400)]);

        let history = store.get_usage_history();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].timestamp, 1_712_448_000);
        assert_eq!(history[0].used, 42.0);
        assert_eq!(history[0].limit, 1200);
    }

    #[test]
    fn record_quota_for_month_keeps_previous_state_when_persistence_fails() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        store.record_quota_for_month("2026-04", 1200);

        std::fs::remove_file(&store.quota_history_path).unwrap();
        std::fs::create_dir(&store.quota_history_path).unwrap();

        store.record_quota_for_month("2026-05", 2400);

        assert_eq!(
            store.get_quota_map(),
            HashMap::from([(String::from("2026-04"), 1200)])
        );
    }

    #[test]
    fn usage_cache_mutations_are_serialized_under_state_operation_lock() {
        let source = include_str!("store.rs")
            .split("#[cfg(test)]")
            .next()
            .expect("store.rs should contain test module marker");

        let set_usage = source
            .split("pub fn set_usage(&self, used: f64, limit: u32) -> Result<(), String> {")
            .nth(1)
            .expect("set_usage should exist")
            .split("/// Get usage data")
            .next()
            .unwrap();
        assert!(
            set_usage.contains("lock_state_operation"),
            "set_usage should serialize mutations under the state operation lock"
        );

        let set_last_update = source
            .split("pub fn set_last_update_check_timestamp(&self, timestamp: i64) -> Result<(), String> {")
            .nth(1)
            .expect("set_last_update_check_timestamp should exist")
            .split("/// Get last update check timestamp")
            .next()
            .unwrap();
        assert!(
            set_last_update.contains("lock_state_operation"),
            "set_last_update_check_timestamp should serialize mutations under the state operation lock"
        );
    }

    #[test]
    fn list_backups_returns_empty_when_no_backups() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        let backups = store.list_backups().unwrap();
        assert!(backups.is_empty());
    }

    #[test]
    fn list_backups_returns_created_backup() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        let backup_id = store.create_backup().unwrap();
        let backups = store.list_backups().unwrap();
        assert_eq!(backups.len(), 1);
        assert_eq!(backups[0].backup_id, backup_id);
    }

    #[test]
    fn list_backups_sorted_newest_first() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        // Create multiple backups with slight delay would be needed for timestamp ordering.
        // Instead verify the sort direction (newest first) by checking list_backups uses
        // reverse alphabetical sort (which works for backup_YYYYMMDD_HHMMSS format).
        let id1 = store.create_backup().unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        let id2 = store.create_backup().unwrap();
        let backups = store.list_backups().unwrap();
        assert_eq!(backups.len(), 2);
        // Newest (id2) should be first since list is sorted newest-first
        assert_eq!(backups[0].backup_id, id2);
        assert_eq!(backups[1].backup_id, id1);
    }

    // ─── delete_backup ──────────────────────────────────────────────────────

    #[test]
    fn delete_backup_removes_backup_directory() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        let backup_id = store.create_backup().unwrap();
        let backup_dir = store.get_backups_path().join(&backup_id);
        assert!(backup_dir.exists());
        store.delete_backup(&backup_id).unwrap();
        assert!(
            !backup_dir.exists(),
            "backup directory must be removed after deletion"
        );
    }

    #[test]
    fn delete_backup_returns_error_for_nonexistent_backup() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        let result = store.delete_backup("backup_nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Backup not found"));
    }

    // ─── prune_backups ──────────────────────────────────────────────────────

    #[test]
    fn prune_backups_keeps_n_most_recent_backups() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        let id1 = store.create_backup().unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        let _id2 = store.create_backup().unwrap();
        std::thread::sleep(std::time::Duration::from_secs(1));
        let id3 = store.create_backup().unwrap();

        // Keep only 2 most recent
        store.prune_backups(2).unwrap();

        let remaining = store.list_backups().unwrap();
        assert_eq!(remaining.len(), 2, "prune should keep exactly 2 backups");
        // The oldest (id1) should have been pruned
        assert!(
            remaining.iter().all(|b| b.backup_id != id1),
            "oldest backup must be pruned"
        );
        // The newest (id3) must be retained
        assert!(
            remaining.iter().any(|b| b.backup_id == id3),
            "newest backup must be retained"
        );
    }

    #[test]
    fn prune_backups_does_nothing_when_count_within_limit() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        store.create_backup().unwrap();
        store.create_backup().unwrap();
        // Keep 5 but only 2 exist — should be a no-op
        store.prune_backups(5).unwrap();
        let remaining = store.list_backups().unwrap();
        assert_eq!(remaining.len(), 2);
    }

    // ─── restore_backup ─────────────────────────────────────────────────────

    #[test]
    fn restore_backup_returns_error_for_nonexistent_backup() {
        let tmp = TempDir::new().unwrap();
        let store = make_store(&tmp);
        let result = store.restore_backup("backup_nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Backup not found"));
    }

    // ─── validate_backup_id ─────────────────────────────────────────────────

    #[test]
    fn validate_backup_id_accepts_valid_ids() {
        assert!(validate_backup_id("backup_20240101_120000_000").is_ok());
        assert!(validate_backup_id("backup_20240101_120000").is_ok());
        assert!(validate_backup_id("backup_anything_valid").is_ok());
    }

    #[test]
    fn validate_backup_id_rejects_missing_prefix() {
        assert!(
            validate_backup_id("").is_err(),
            "id without 'backup_' prefix must be rejected"
        );
        assert!(
            validate_backup_id("20240101_120000").is_err(),
            "no prefix must be rejected"
        );
        assert!(
            validate_backup_id("COM1").is_err(),
            "Windows reserved name COM1 must be rejected"
        );
        assert!(
            validate_backup_id("NUL").is_err(),
            "Windows reserved name NUL must be rejected"
        );
        assert!(
            validate_backup_id("CON").is_err(),
            "Windows reserved name CON must be rejected"
        );
        assert!(
            validate_backup_id(".").is_err(),
            "single dot must be rejected"
        );
        assert!(
            validate_backup_id("randomname").is_err(),
            "arbitrary name without prefix must be rejected"
        );
    }

    #[test]
    fn validate_backup_id_rejects_path_traversal() {
        assert!(validate_backup_id("backup_../../etc/passwd").is_err());
        assert!(validate_backup_id("backup_foo/bar").is_err());
        assert!(validate_backup_id("backup_foo\\bar").is_err());
        assert!(validate_backup_id("..").is_err());
        assert!(validate_backup_id("../backup_good").is_err());
        // Null bytes can cause path truncation on some platforms
        assert!(validate_backup_id("backup_foo\x00/etc").is_err());
    }

    #[test]
    fn backup_info_serializes_to_camel_case() {
        let info = BackupInfo {
            backup_id: "backup_20240101_120000".to_string(),
            created_at: "2024-01-01T12:00:00Z".to_string(),
            files: vec!["usage_history.json".to_string()],
            size_bytes: 1024,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(
            json.contains("\"backupId\""),
            "backup_id must serialize as backupId"
        );
        assert!(
            json.contains("\"createdAt\""),
            "created_at must serialize as createdAt"
        );
        assert!(
            json.contains("\"sizeBytes\""),
            "size_bytes must serialize as sizeBytes"
        );
    }

    #[test]
    fn backup_info_deserializes_from_camel_case() {
        let json = r#"{
            "backupId": "backup_20240101_120000",
            "createdAt": "2024-01-01T12:00:00Z",
            "files": ["usage_history.json"],
            "sizeBytes": 2048
        }"#;
        let info: BackupInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.backup_id, "backup_20240101_120000");
        assert_eq!(info.created_at, "2024-01-01T12:00:00Z");
        assert_eq!(info.files, vec!["usage_history.json"]);
        assert_eq!(info.size_bytes, 2048);
    }
}
