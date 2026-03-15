mod auth;
mod store;
mod tray_icon_renderer;
mod usage;

pub use auth::{
    hidden_webview_event, AuthManager, AuthState, ExtractionResult, HiddenWebviewEvent, UsageData,
};
// REMOVED init_store_manager - StoreManager is now initialized in main() before builder
pub use store::{
    AppSettings, BackupFrequency, BackupInfo, StoreManager, UsageCache, WidgetPosition,
};
pub use tray_icon_renderer::{TrayIconRenderer, TrayImage};
pub use usage::{UsageEntry, UsageHistory, UsageManager, UsagePayload, UsageSummary};
