use tauri::{AppHandle, Emitter};
use tauri_plugin_updater::UpdaterExt;

#[derive(Debug, serde::Serialize)]
pub struct UpdateStatus {
    pub available: bool,
    pub current_version: String,
    pub latest_version: Option<String>,
    pub body: Option<String>,
    pub date: Option<String>,
}

pub struct UpdateManager;

impl UpdateManager {
    /// Check for updates
    pub async fn check_for_updates(app: &AppHandle) -> Result<UpdateStatus, String> {
        let package_info = app.package_info();
        let current_version = package_info.version.to_string();

        // Use Tauri's built-in updater
        let updater = app.updater()
            .map_err(|e| format!("Failed to get updater: {}", e))?;

        if let Some(update) = updater.check().await
            .map_err(|e| format!("Failed to check for updates: {}", e))? 
        {
            Ok(UpdateStatus {
                available: true,
                current_version,
                latest_version: Some(update.version),
                body: update.body,
                date: update.date.map(|d| d.to_string()),
            })
        } else {
            Ok(UpdateStatus {
                available: false,
                current_version,
                latest_version: None,
                body: None,
                date: None,
            })
        }
    }

    /// Download and install update
    pub async fn install_update(app: &AppHandle) -> Result<(), String> {
        let updater = app.updater()
            .map_err(|e| format!("Failed to get updater: {}", e))?;

        // Check if update is available
        if let Some(update) = updater.check().await
            .map_err(|e| format!("Failed to check for updates: {}", e))? 
        {
            // Download and install
            update.download_and_install(
                |chunk_length, content_length| {
                    let content = content_length.unwrap_or(1) as f32;
                    let progress = (chunk_length as f32 / content) * 100.0;
                    log::info!("Download progress: {:.1}%", progress);

                    // Emit progress event
                    let _ = app.emit("update:download-progress", progress);
                },
                || {
                    log::info!("Download complete");
                },
            ).await
            .map_err(|e| format!("Failed to download update: {}", e))?;

            // Notify that app should restart
            let _ = app.emit("update:ready", true);

            Ok(())
        } else {
            Err("No update available".to_string())
        }
    }

    /// Start automatic update checks
    pub fn start_auto_check(app: AppHandle, interval_hours: u64) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(interval_hours * 3600)
            );

            loop {
                interval.tick().await;

                match Self::check_for_updates(&app).await {
                    Ok(status) => {
                        if status.available {
                            let latest_version = status.latest_version.clone()
                                .unwrap_or_default();

                            log::info!("Update available: {} -> {}",
                                status.current_version,
                                latest_version
                            );

                            // Emit notification event
                            let _ = app.emit("update:available", &status);
                        }
                    }
                    Err(e) => {
                        log::warn!("Failed to check for updates: {}", e);
                    }
                }
            }
        });
    }
}
