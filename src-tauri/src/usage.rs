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

    /// Fetch and update usage data using hidden webview extraction
    pub async fn fetch_usage(
        &mut self,
        app: &AppHandle,
    ) -> Result<UsageSummary, String> {
        log::info!("Starting usage fetch with hidden webview extraction...");

        // Create auth manager for extraction
        let mut auth_manager = crate::auth::AuthManager::new();
        
        // Perform hidden extraction
        match auth_manager.perform_extraction(app).await {
            Ok(result) => {
                if let Some(error) = result.error {
                    log::warn!("Hidden extraction completed with error: {}", error);
                    // Fall back to cached data on error
                    let summary = Self::get_cached_usage(app)?;
                    log::info!("Fallback: Emitting usage:updated with cached data: used={}, limit={}", summary.used, summary.limit);
                    let _ = app.emit("usage:updated", &summary);
                    return Ok(summary);
                }

                // Process extracted data
                if let Some(customer_id) = result.customer_id {
                    let store = app.state::<crate::store::StoreManager>();
                    let _ = store.set_customer_id(customer_id);

                    if let Some(usage) = result.usage_data {
                        let used = usage.discount_quantity as u32;
                        let limit = usage.user_premium_request_entitlement as u32;
                        
                        log::info!("Extracted usage: {}/{} ({}%)", used, limit, 
                            if limit > 0 { (used as f32 / limit as f32) * 100.0 } else { 0.0 });
                        
                        let _ = store.set_usage(used, limit);

                        // Update cache
                        let cache = crate::store::UsageCache {
                            customer_id,
                            net_quantity: usage.net_quantity,
                            discount_quantity: usage.discount_quantity,
                            user_premium_request_entitlement: usage.user_premium_request_entitlement,
                            filtered_user_premium_request_entitlement: usage.filtered_user_premium_request_entitlement,
                            net_billed_amount: usage.net_billed_amount,
                            timestamp: chrono::Utc::now().timestamp(),
                        };
                        store.set_usage_cache(cache);

                        // Save history if available
                        if let Some(rows) = result.usage_history {
                            let entries = Self::map_history_rows(&rows);
                            store.set_usage_history(entries);
                        }

                        let summary = UsageSummary {
                            used,
                            limit,
                            remaining: limit.saturating_sub(used),
                            percentage: if limit > 0 { (used as f32 / limit as f32) * 100.0 } else { 0.0 },
                            timestamp: chrono::Utc::now().timestamp(),
                        };

                        // Emit full payload
                        let history = Self::get_cached_history(app);
                        let prediction = Self::predict_usage_from_history(&history, used, limit);
                        
                        let payload = UsagePayload {
                            summary: summary.clone(),
                            history,
                            prediction,
                        };
                        
                        log::info!("Emitting usage:data event with used={}, limit={}", used, limit);
                        let _ = app.emit("usage:data", payload);
                        log::info!("Emitting usage:updated event with used={}, limit={} (tray should update)", used, limit);
                        let _ = app.emit("usage:updated", &summary);
                        
                        return Ok(summary);
                    }
                }

                // No data extracted, use cache
                let summary = Self::get_cached_usage(app)?;
                log::info!("No data extracted: Emitting usage:updated with cached data: used={}, limit={}", summary.used, summary.limit);
                let _ = app.emit("usage:updated", &summary);
                Ok(summary)
            }
            Err(e) => {
                log::error!("Hidden extraction failed: {}", e);
                // Fall back to cached data
                let summary = Self::get_cached_usage(app)?;
                log::info!("Extraction failed: Emitting usage:updated with cached data: used={}, limit={}", summary.used, summary.limit);
                let _ = app.emit("usage:updated", &summary);
                Ok(summary)
            }
        }
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
                // Try to parse the date string in multiple formats
                let timestamp = if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&row.date) {
                    dt.timestamp()
                } else {
                    // Try parsing the format from GitHub: "2026-02-01 00:00:00 +0000 UTC"
                    // We can just grab the first part "2026-02-01" since the time is usually 00:00:00
                    let date_part = row.date.split(' ').next().unwrap_or("");
                    if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(date_part, "%Y-%m-%d") {
                        naive_date.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp()
                    } else {
                        log::warn!("Failed to parse date: '{}', using current time", row.date);
                        chrono::Utc::now().timestamp()
                    }
                };
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

    /// Start background usage polling with cancellation support
    /// Returns a channel sender that can be used to cancel the polling task
    pub fn start_polling(app: AppHandle, interval_seconds: u64) -> tokio::sync::mpsc::Sender<()> {
        let (cancel_tx, mut cancel_rx) = tokio::sync::mpsc::channel::<()>(1);
        
        tauri::async_runtime::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(interval_seconds));
            
            // Skip the first tick (immediate fire)
            interval.tick().await;

            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        // Only fetch if authenticated
                        if let Some(store) = app.try_state::<StoreManager>() {
                            if store.is_authenticated() {
                                // Create a new usage manager for this poll
                                let mut usage_manager = UsageManager::new();

                                if let Ok(summary) = usage_manager.fetch_usage(&app).await {
                                    log::info!(
                                        "[Background Polling] Usage updated: {}/{} ({}%)",
                                        summary.used,
                                        summary.limit,
                                        summary.percentage
                                    );
                                } else {
                                    log::warn!("[Background Polling] Failed to fetch usage");
                                }
                            } else {
                                log::debug!("[Background Polling] Skipping - not authenticated");
                            }
                        }
                    }
                    _ = cancel_rx.recv() => {
                        log::info!("[Background Polling] Cancelled");
                        break;
                    }
                }
            }
        });
        
        cancel_tx
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
