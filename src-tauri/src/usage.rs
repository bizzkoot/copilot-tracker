use crate::store::StoreManager;
use crate::auth::UsageHistoryRow;
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
    pub included_requests: u32,
    pub billed_requests: u32,
    pub gross_amount: f64,
    pub billed_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePayload {
    pub summary: UsageSummary,
    pub history: Vec<UsageEntry>,
    pub prediction: Option<UsagePrediction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsagePrediction {
    pub predicted_monthly_requests: u32,
    pub predicted_billed_amount: f64,
    pub confidence_level: String,
    pub days_used_for_prediction: u32,
}

pub struct UsageManager {
}

impl Default for UsageManager {
    fn default() -> Self {
        Self::new()
    }
}

impl UsageManager {
    pub fn new() -> Self {
        Self {}
    }

    /// Fetch and update usage data
    pub async fn fetch_usage(
        &mut self,
        app: &AppHandle,
    ) -> Result<UsageSummary, String> {
        // Extraction skipped (hidden window not supported). Returning cached data.
        log::info!("Extraction skipped (hidden window not supported). Returning cached data.");

        let summary = Self::get_cached_usage(app)?;

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

    pub fn get_cached_history(app: &AppHandle) -> Vec<UsageEntry> {
        if let Some(store) = app.try_state::<StoreManager>() {
            let history = store.get_usage_history();
            if !history.is_empty() {
                return history;
            }
            if let Some(cache) = store.get_usage_cache() {
                return vec![UsageEntry {
                    timestamp: cache.timestamp,
                    used: cache.discount_quantity as u32,
                    limit: cache.user_premium_request_entitlement as u32,
                    included_requests: cache.discount_quantity as u32,
                    billed_requests: cache.net_quantity.saturating_sub(cache.discount_quantity) as u32,
                    gross_amount: cache.net_billed_amount,
                    billed_amount: cache.net_billed_amount,
                }];
            }
        }
        vec![]
    }

    pub fn get_cached_history_from_rows(rows: &[UsageHistoryRow]) -> Vec<UsageEntry> {
        Self::map_history_rows(rows)
    }

    pub fn map_history_rows(rows: &[UsageHistoryRow]) -> Vec<UsageEntry> {
        let mut entries: Vec<UsageEntry> = rows
            .iter()
            .map(|row| {
                let timestamp = chrono::DateTime::parse_from_rfc3339(&row.date)
                    .map(|dt| dt.timestamp())
                    .unwrap_or_else(|_| chrono::Utc::now().timestamp());
                UsageEntry {
                    timestamp,
                    used: row.included_requests + row.billed_requests,
                    limit: 0,
                    included_requests: row.included_requests,
                    billed_requests: row.billed_requests,
                    gross_amount: row.gross_amount,
                    billed_amount: row.billed_amount,
                }
            })
            .collect();
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        entries
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
                        let mut usage_manager = UsageManager::new();

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
            let next_month = chrono::NaiveDate::from_ymd_opt(now.year(), now.month() + 1, 1)
                .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(now.year() + 1, 1, 1).unwrap());
            let current_month =
                chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap();
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

    pub fn predict_usage_from_history(
        history: &[UsageEntry],
        used: u32,
        limit: u32,
    ) -> Option<UsagePrediction> {
        if history.is_empty() {
            return None;
        }

        let now = chrono::Utc::now();
        let current_day = now.day() as f32;
        let days_in_month = if now.month() == 12 {
            31
        } else {
            let next_month = chrono::NaiveDate::from_ymd_opt(now.year(), now.month() + 1, 1)
                .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(now.year() + 1, 1, 1).unwrap());
            let current_month =
                chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap();
            (next_month - current_month).num_days() as u32
        };

        if current_day == 0.0 {
            return None;
        }

        let daily_average = used as f32 / current_day;
        let remaining_days = days_in_month as f32 - current_day;
        let predicted = used as f32 + (daily_average * remaining_days);
        let predicted_monthly_requests = predicted.max(0.0).round() as u32;
        let excess_requests = predicted_monthly_requests.saturating_sub(limit);
        let predicted_billed_amount = (excess_requests as f64) * 0.04;

        let confidence_level = if history.len() < 3 {
            "low"
        } else if history.len() < 7 {
            "medium"
        } else {
            "high"
        };

        Some(UsagePrediction {
            predicted_monthly_requests,
            predicted_billed_amount,
            confidence_level: confidence_level.to_string(),
            days_used_for_prediction: history.len() as u32,
        })
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
