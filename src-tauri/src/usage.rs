use crate::auth::AuthManager;
use crate::store::StoreManager;
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSummary {
    pub used: u32,
    pub limit: u32,
    pub remaining: u32,
    pub percentage: f32,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageHistory {
    pub entries: Vec<UsageEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEntry {
    pub timestamp: i64,
    pub used: u32,
    pub limit: u32,
}

pub struct UsageManager {
    auth_manager: AuthManager,
}

impl UsageManager {
    pub fn new(auth_manager: AuthManager) -> Self {
        Self { auth_manager }
    }

    /// Fetch and update usage data
    pub async fn fetch_usage(
        &mut self,
        app: &AppHandle,
    ) -> Result<UsageSummary, String> {
        // Perform extraction using auth manager
        let result = self.auth_manager.perform_extraction(app).await?;

        if let Some(error) = result.error {
            return Err(error);
        }

        // Get usage data
        let usage_data = result
            .usage_data
            .ok_or("No usage data available")?;

        // Calculate usage summary
        let used = usage_data.net_quantity as u32;
        let limit: u32 = 1200; // Default Copilot monthly limit
        let remaining = limit.saturating_sub(used);
        let percentage = if limit > 0 {
            (used as f32 / limit as f32) * 100.0
        } else {
            0.0
        };

        let summary = UsageSummary {
            used,
            limit,
            remaining,
            percentage,
            timestamp: chrono::Utc::now().timestamp(),
        };

        // Update store
        if let Some(store) = app.try_state::<StoreManager>() {
            let _ = store.set_usage(used, limit);
        }

        // Emit event to frontend
        let _ = app.emit("usage:updated", &summary);

        Ok(summary)
    }

    /// Get cached usage from store
    pub fn get_cached_usage(app: &AppHandle) -> Result<UsageSummary, String> {
        let store = app.state::<StoreManager>();
        let (used, limit) = store.get_usage();

        let remaining = limit.saturating_sub(used);
        let percentage = if limit > 0 {
            (used as f32 / limit as f32) * 100.0
        } else {
            0.0
        };

        Ok(UsageSummary {
            used,
            limit,
            remaining,
            percentage,
            timestamp: chrono::Utc::now().timestamp(),
        })
    }

    /// Start background usage polling
    pub fn start_polling(app: AppHandle, interval_minutes: u64) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(interval_minutes * 60));

            loop {
                interval.tick().await;

                // Only fetch if authenticated
                if let Some(store) = app.try_state::<StoreManager>() {
                    if store.is_authenticated() {
                        // Create a new auth manager for this poll
                        let auth_manager = AuthManager::new();
                        let mut usage_manager = UsageManager::new(auth_manager);

                        if let Ok(summary) = usage_manager.fetch_usage(&app).await {
            log::info!(
                "Usage updated: {}/{} ({}%)",
                summary.used,
                summary.limit,
                summary.percentage
            );
                        } else {
                            log::warn!("Failed to fetch usage during polling");
                        }
                    }
                }
            }
        });
    }

    /// Predict end-of-month usage based on current trends
    pub fn predict_eom_usage(app: &AppHandle) -> Result<u32, String> {
        let store = app.state::<StoreManager>();
        let (used, _limit) = store.get_usage();

        // Simple prediction: assume linear usage
        let now = chrono::Utc::now();
        let current_day = now.day() as f32;

        // Get days in month using naive date calculation
        let days_in_month = if now.month() == 12 {
            // January next year
            31
        } else {
            // First day of next month minus first day of current month
            let next_month = chrono::NaiveDate::from_ymd_opt(
                now.year() as i32,
                now.month() + 1,
                1,
            )
            .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(now.year() as i32 + 1, 1, 1).unwrap());
            let current_month =
                chrono::NaiveDate::from_ymd_opt(now.year() as i32, now.month(), 1).unwrap();
            (next_month - current_month).num_days() as u32
        };

        if current_day == 0.0 {
            return Ok(used);
        }

        let daily_average = used as f32 / current_day;
        let remaining_days = days_in_month as f32 - current_day;
        let predicted = used as f32 + (daily_average * remaining_days);

        Ok(predicted as u32)
    }

    /// Calculate days until limit is reached
    pub fn days_until_limit(app: &AppHandle) -> Result<Option<i64>, String> {
        let store = app.state::<StoreManager>();
        let (used, limit) = store.get_usage();

        if used >= limit {
            return Ok(Some(0)); // Already at or over limit
        }

        let remaining = (limit - used) as f32;

        // Calculate daily average
        let now = chrono::Utc::now();
        let current_day = now.day() as f32;

        if current_day == 0.0 {
            return Ok(None);
        }

        let daily_average = used as f32 / current_day;

        if daily_average == 0.0 {
            return Ok(None); // Can't predict if no usage yet
        }

        let days_until_limit = (remaining / daily_average).ceil() as i64;
        Ok(Some(days_until_limit))
    }
}
