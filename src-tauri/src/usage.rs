use crate::auth::UsageHistoryRow;
use crate::store::StoreManager;
use chrono::Datelike;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::{AppHandle, Emitter, Manager};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageSummary {
    pub used: f64,
    pub limit: u32,
    pub remaining: f64,
    pub percentage: f32,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageHistory {
    pub entries: Vec<UsageEntry>,
}

fn default_quota_estimated() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEntry {
    pub timestamp: i64,
    pub used: f64,
    pub limit: u32,
    pub included_requests: f64,
    pub billed_requests: f64,
    pub gross_amount: f64,
    pub billed_amount: f64,
    #[serde(default)]
    pub models: Vec<UsageModel>,
    /// True when `limit` is an estimate (current quota used as fallback because
    /// no historical quota was recorded for this month in quota_history.json).
    #[serde(default = "default_quota_estimated")]
    pub quota_estimated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageModel {
    pub name: String,
    pub included_requests: f64,
    pub billed_requests: f64,
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
    pub predicted_monthly_requests: f64,
    pub predicted_billed_amount: f64,
    pub confidence_level: String,
    pub days_used_for_prediction: u32,
}

fn round_request_count(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

pub struct UsageManager {}

impl Default for UsageManager {
    fn default() -> Self {
        Self::new()
    }
}

impl UsageManager {
    pub fn new() -> Self {
        Self {}
    }

    /// Process extracted usage data and history rows, persisting everything to
    /// the store and emitting the standard suite of Tauri events
    /// (`usage:data`, `usage:updated`).
    ///
    /// Returns `Some(UsageSummary)` when usage data was present and processed,
    /// or `None` when there was nothing to save.
    ///
    /// This is the single source of truth for the "save & emit" pipeline used
    /// by both the auth callback (visible login) and the hidden-webview
    /// extraction path.
    pub fn process_and_emit_usage(
        app: &AppHandle,
        _customer_id: u64,
        usage_data: &crate::auth::UsageData,
        history_rows: Option<Vec<crate::auth::UsageHistoryRow>>,
    ) -> Result<Option<UsageSummary>, String> {
        let store = app.state::<crate::store::StoreManager>();

        let used = round_request_count(usage_data.discount_quantity);
        let limit = usage_data.user_premium_request_entitlement;

        log::info!(
            "Processing usage data: {}/{} (net_quantity={}, discount_quantity={})",
            used,
            limit,
            usage_data.net_quantity,
            usage_data.discount_quantity
        );

        if used == 0.0 && limit == 0 {
            log::warn!("Usage data shows 0/0 - API may have returned empty data");
        }

        // Always record the current month's quota so new-month history entries
        // are paired with the observed limit immediately rather than being
        // marked as estimated.
        let current_month = chrono::Utc::now().format("%Y-%m").to_string();
        let mut quota_map = store.get_quota_map();
        if limit > 0 {
            quota_map.insert(current_month, limit);
        }

        // Merge history rows with persisted data if available
        let existing_history = store.get_usage_history();
        let usage_entries = if let Some(rows) = history_rows {
            log::info!("Processing {} usage history rows", rows.len());
            Self::merge_history_rows_with_persisted_history(
                &existing_history,
                &rows,
                &quota_map,
                limit,
                Some(used),
            )
        } else {
            log::warn!("No usage history rows to process");
            existing_history
        };

        store.persist_usage_snapshot(used, limit, usage_entries.clone(), quota_map)?;

        // Build summary
        let summary = UsageSummary {
            used,
            limit,
            remaining: (limit as f64 - used).max(0.0),
            percentage: if limit > 0 {
                (used / limit as f64 * 100.0) as f32
            } else {
                0.0
            },
            timestamp: chrono::Utc::now().timestamp(),
        };

        // Emit full payload with prediction
        let history = if !usage_entries.is_empty() {
            usage_entries
        } else {
            Self::get_cached_history(app)
        };

        let settings = store.get_settings();
        let prediction = Self::predict_usage_from_history(
            &history,
            summary.used,
            summary.limit,
            settings.prediction_period,
        );

        log::info!(
            "Emitting usage:data event - used: {}, limit: {}, history entries: {}",
            summary.used,
            summary.limit,
            history.len()
        );

        let payload = UsagePayload {
            summary: summary.clone(),
            history,
            prediction,
        };

        let _ = app.emit("usage:data", payload);
        log::info!(
            "Emitting usage:updated event - used: {}, limit: {} (tray should update)",
            summary.used,
            summary.limit
        );
        let _ = app.emit("usage:updated", &summary);

        Ok(Some(summary))
    }

    pub fn merge_and_store_history_rows(
        store: &StoreManager,
        rows: &[UsageHistoryRow],
        current_quota: u32,
        current_used: Option<f64>,
    ) -> Result<Vec<UsageEntry>, String> {
        let quota_map = store.get_quota_map();
        let existing_history = store.get_usage_history();
        let merged_entries = Self::merge_history_rows_with_persisted_history(
            &existing_history,
            rows,
            &quota_map,
            current_quota,
            current_used,
        );
        store.persist_history_snapshot(merged_entries.clone())?;
        Ok(merged_entries)
    }

    pub fn persist_history_without_usage_data(
        store: &StoreManager,
        rows: &[UsageHistoryRow],
    ) -> Result<Vec<UsageEntry>, String> {
        Self::merge_and_store_history_rows(store, rows, 0, None)
    }

    pub fn persist_history_with_cached_summary(
        store: &StoreManager,
        rows: &[UsageHistoryRow],
        summary: &UsageSummary,
    ) -> Result<Vec<UsageEntry>, String> {
        Self::merge_and_store_history_rows(store, rows, summary.limit, Some(summary.used))
    }

    fn can_reconcile_with_cached_summary(
        last_fetch_timestamp: i64,
        now: chrono::DateTime<chrono::Utc>,
    ) -> bool {
        if last_fetch_timestamp <= 0 {
            return false;
        }

        chrono::DateTime::from_timestamp(last_fetch_timestamp, 0)
            .map(|last_fetch| {
                last_fetch.year() == now.year()
                    && last_fetch.month() == now.month()
                    && last_fetch.day() == now.day()
            })
            .unwrap_or(false)
    }

    pub fn can_apply_extraction_results(store: &StoreManager, expected_generation: u64) -> bool {
        store.is_authenticated()
            && !store.is_session_transition_in_progress()
            && store.get_session_generation() == expected_generation
    }

    fn lock_and_validate_extraction_results<'a>(
        store: &'a StoreManager,
        expected_generation: u64,
    ) -> Result<std::sync::MutexGuard<'a, ()>, String> {
        let state_operation = store.lock_state_operation()?;
        if Self::can_apply_extraction_results(store, expected_generation) {
            Ok(state_operation)
        } else {
            Err("Session changed during extraction".to_string())
        }
    }

    /// Fetch and update usage data using hidden webview extraction
    pub async fn fetch_usage(&mut self, app: &AppHandle) -> Result<UsageSummary, String> {
        log::info!("Starting usage fetch with hidden webview extraction...");

        // Create auth manager for extraction
        let mut auth_manager = crate::auth::AuthManager::new();
        let extraction_generation = {
            let store = app.state::<StoreManager>();
            store.get_session_generation()
        };

        // Perform hidden extraction
        match auth_manager.perform_extraction(app).await {
            Ok(mut result) => {
                let store = app.state::<StoreManager>();
                let _state_operation = match Self::lock_and_validate_extraction_results(
                    &store,
                    extraction_generation,
                ) {
                    Ok(guard) => guard,
                    Err(error) => {
                        log::warn!(
                        "Discarding hidden extraction results because the authenticated session changed"
                    );
                        return Err(error);
                    }
                };

                if let Some(error) = result.error {
                    log::warn!("Hidden extraction completed with error: {}", error);
                    // Fall back to cached data on error
                    let summary = Self::get_cached_usage(app)?;
                    log::info!(
                        "Fallback: Emitting usage:updated with cached data: used={}, limit={}",
                        summary.used,
                        summary.limit
                    );
                    let _ = app.emit("usage:updated", &summary);
                    return Ok(summary);
                }

                // Process extracted data
                if let Some(customer_id) = result.customer_id {
                    if let Err(error) = store.set_customer_id_locked(customer_id) {
                        log::error!(
                            "Failed to persist customer ID {} before processing hidden extraction: {}",
                            customer_id,
                            error
                        );
                        return Err(format!("Failed to persist customer ID: {}", error));
                    }

                    if let Some(ref usage) = result.usage_data {
                        if let Some(summary) = Self::process_and_emit_usage(
                            app,
                            customer_id,
                            usage,
                            result.usage_history.take(),
                        )? {
                            return Ok(summary);
                        }
                    } else if let Some(rows) = result.usage_history.take() {
                        log::warn!(
                            "Hidden extraction returned no usage summary; persisting {} history rows with cached summary",
                            rows.len()
                        );
                        let summary = Self::get_cached_usage(app)?;
                        let history = {
                            if Self::can_reconcile_with_cached_summary(
                                store.get_last_fetch_timestamp(),
                                chrono::Utc::now(),
                            ) {
                                Self::persist_history_with_cached_summary(&store, &rows, &summary)?
                            } else {
                                Self::persist_history_without_usage_data(&store, &rows)?
                            }
                        };
                        let store = app.state::<crate::store::StoreManager>();
                        let settings = store.get_settings();
                        let prediction = Self::predict_usage_from_history(
                            &history,
                            summary.used,
                            summary.limit,
                            settings.prediction_period,
                        );
                        let payload = UsagePayload {
                            summary: summary.clone(),
                            history,
                            prediction,
                        };
                        let _ = app.emit("usage:data", payload);
                        let _ = app.emit("usage:updated", &summary);
                        return Ok(summary);
                    }
                }

                // No data extracted, use cache
                let summary = Self::get_cached_usage(app)?;
                log::info!(
                    "No data extracted: Emitting usage:updated with cached data: used={}, limit={}",
                    summary.used,
                    summary.limit
                );
                let _ = app.emit("usage:updated", &summary);
                Ok(summary)
            }
            Err(e) => {
                log::error!("Hidden extraction failed: {}", e);
                // Fall back to cached data
                let summary = Self::get_cached_usage(app)?;
                log::info!(
                    "Extraction failed: Emitting usage:updated with cached data: used={}, limit={}",
                    summary.used,
                    summary.limit
                );
                let _ = app.emit("usage:updated", &summary);
                Ok(summary)
            }
        }
    }

    /// Get cached usage from store
    pub fn get_cached_usage(app: &AppHandle) -> Result<UsageSummary, String> {
        let store = app.state::<StoreManager>();
        let (used, limit) = store.get_usage();

        let remaining = (limit as f64 - used).max(0.0);
        let percentage = if limit > 0 {
            (used / limit as f64 * 100.0) as f32
        } else {
            0.0
        };

        // Use the stored fetch timestamp so callers (and the dashboard) always
        // show when the data was actually fetched, not when this function ran.
        let last_fetch = store.get_last_fetch_timestamp();
        let timestamp = if last_fetch > 0 {
            last_fetch
        } else {
            chrono::Utc::now().timestamp()
        };

        Ok(UsageSummary {
            used,
            limit,
            remaining,
            percentage,
            timestamp,
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
                    used: round_request_count(cache.discount_quantity),
                    limit: cache.user_premium_request_entitlement,
                    included_requests: cache.discount_quantity,
                    billed_requests: (cache.net_quantity - cache.discount_quantity).max(0.0),
                    gross_amount: cache.net_billed_amount,
                    billed_amount: cache.net_billed_amount,
                    models: vec![],
                    quota_estimated: false,
                }];
            }
        }
        vec![]
    }

    pub fn get_cached_history_from_rows(rows: &[UsageHistoryRow]) -> Vec<UsageEntry> {
        Self::map_history_rows(rows)
    }

    /// Map history rows with current quota (for backward compatibility)
    pub fn map_history_rows(rows: &[UsageHistoryRow]) -> Vec<UsageEntry> {
        Self::map_history_rows_with_quota(rows, &HashMap::new(), 0)
    }

    /// Map history rows with a per-month quota lookup table.
    ///
    /// For each row, the YYYY-MM of its timestamp is looked up in `quota_map`.
    /// If a stored limit exists for that month, it is used and `quota_estimated = false`.
    /// Otherwise, `current_quota` is used as a fallback and `quota_estimated = true`
    /// (so the frontend can signal to users that the utilization % is approximate).
    ///
    /// Note: GitHub's API does not expose the quota that was active for past months,
    /// so this tracking is done at record-time (each successful fetch records the
    /// current quota for the current month into quota_history.json).
    pub fn map_history_rows_with_quota(
        rows: &[UsageHistoryRow],
        quota_map: &HashMap<String, u32>,
        current_quota: u32,
    ) -> Vec<UsageEntry> {
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
                    if let Ok(naive_date) = chrono::NaiveDate::parse_from_str(date_part, "%Y-%m-%d")
                    {
                        naive_date
                            .and_hms_opt(0, 0, 0)
                            .unwrap()
                            .and_utc()
                            .timestamp()
                    } else {
                        log::warn!("Failed to parse date: '{}', using current time", row.date);
                        chrono::Utc::now().timestamp()
                    }
                };

                let models = row
                    .models
                    .iter()
                    .map(|m| UsageModel {
                        name: m.name.clone(),
                        included_requests: m.included_requests,
                        billed_requests: m.billed_requests,
                        gross_amount: m.gross_amount,
                        billed_amount: m.billed_amount,
                    })
                    .collect();

                // Derive the "YYYY-MM" key for this row so we can look up the
                // quota that was recorded for that specific month.
                let month_key = chrono::DateTime::from_timestamp(timestamp, 0)
                    .map(|dt| dt.format("%Y-%m").to_string())
                    .unwrap_or_else(|| {
                        log::warn!("[Usage] Invalid timestamp {} in usage history entry; quota lookup for this entry will use current quota", timestamp);
                        String::new()
                    });

                let (limit, quota_estimated) = if let Some(&stored) = quota_map.get(&month_key) {
                    (stored, false)
                } else {
                    // No quota recorded for this month yet; fall back to current_quota.
                    // Always mark as estimated — we don't know what the plan quota was
                    // at that time, even if current_quota is 0 (no data at all).
                    (current_quota, true)
                };

                UsageEntry {
                    timestamp,
                    used: round_request_count(row.included_requests + row.billed_requests),
                    limit,
                    quota_estimated,
                    included_requests: row.included_requests,
                    billed_requests: row.billed_requests,
                    gross_amount: row.gross_amount,
                    billed_amount: row.billed_amount,
                    models,
                }
            })
            .collect();
        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        entries
    }

    fn month_key_for_timestamp(timestamp: i64) -> Option<String> {
        chrono::DateTime::from_timestamp(timestamp, 0).map(|dt| dt.format("%Y-%m").to_string())
    }

    fn merge_history_rows_with_persisted_history_at(
        existing_history: &[UsageEntry],
        rows: &[UsageHistoryRow],
        quota_map: &HashMap<String, u32>,
        current_quota: u32,
        current_used: Option<f64>,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Vec<UsageEntry> {
        let mut merged_quota_map = quota_map.clone();
        let mut estimated_limit_map: HashMap<String, u32> = HashMap::new();

        // Preserve month-specific quotas already embedded in persisted history.
        // This protects older months even when quota_history.json is missing an
        // entry for that month (for example, after upgrading from an older build).
        for entry in existing_history {
            if entry.limit == 0 {
                continue;
            }

            if let Some(month_key) = Self::month_key_for_timestamp(entry.timestamp) {
                if entry.quota_estimated {
                    estimated_limit_map.entry(month_key).or_insert(entry.limit);
                } else {
                    merged_quota_map.entry(month_key).or_insert(entry.limit);
                }
            }
        }

        let mut entries_by_timestamp: HashMap<i64, UsageEntry> = existing_history
            .iter()
            .cloned()
            .map(|mut entry| {
                if let Some(month_key) = Self::month_key_for_timestamp(entry.timestamp) {
                    if let Some(&authoritative_limit) = merged_quota_map.get(&month_key) {
                        entry.limit = authoritative_limit;
                        entry.quota_estimated = false;
                    } else if entry.quota_estimated {
                        if let Some(&estimated_limit) = estimated_limit_map.get(&month_key) {
                            entry.limit = estimated_limit;
                        }
                    }
                }
                (entry.timestamp, entry)
            })
            .collect();

        let mut mapped_entries =
            Self::map_history_rows_with_quota(rows, &merged_quota_map, current_quota);
        let refreshed_timestamps: std::collections::HashSet<i64> =
            mapped_entries.iter().map(|entry| entry.timestamp).collect();

        for entry in &mut mapped_entries {
            if !entry.quota_estimated {
                continue;
            }

            if let Some(month_key) = Self::month_key_for_timestamp(entry.timestamp) {
                if let Some(&estimated_limit) = estimated_limit_map.get(&month_key) {
                    entry.limit = estimated_limit;
                }
            }
        }

        for entry in mapped_entries {
            entries_by_timestamp.insert(entry.timestamp, entry);
        }

        let mut merged_entries: Vec<UsageEntry> = entries_by_timestamp.into_values().collect();

        if let Some(used) = current_used {
            Self::reconcile_history_with_usage_total_at(
                &mut merged_entries,
                used,
                current_quota,
                now,
                Some(&refreshed_timestamps),
            );
        }

        merged_entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        merged_entries
    }

    pub fn merge_history_rows_with_persisted_history(
        existing_history: &[UsageEntry],
        rows: &[UsageHistoryRow],
        quota_map: &HashMap<String, u32>,
        current_quota: u32,
        current_used: Option<f64>,
    ) -> Vec<UsageEntry> {
        Self::merge_history_rows_with_persisted_history_at(
            existing_history,
            rows,
            quota_map,
            current_quota,
            current_used,
            chrono::Utc::now(),
        )
    }

    /// If current usage total is newer than billing history rows, inject the missing delta.
    pub fn reconcile_history_with_usage_total(
        entries: &mut Vec<UsageEntry>,
        used: f64,
        limit: u32,
    ) {
        Self::reconcile_history_with_usage_total_at(entries, used, limit, chrono::Utc::now(), None);
    }

    fn reconcile_history_with_usage_total_at(
        entries: &mut Vec<UsageEntry>,
        used: f64,
        limit: u32,
        now: chrono::DateTime<chrono::Utc>,
        refreshed_timestamps: Option<&std::collections::HashSet<i64>>,
    ) {
        let current_year = now.year();
        let current_month = now.month();

        let current_month_indices: Vec<usize> = entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                chrono::DateTime::from_timestamp(entry.timestamp, 0).and_then(|dt| {
                    if dt.year() == current_year && dt.month() == current_month {
                        Some(index)
                    } else {
                        None
                    }
                })
            })
            .collect();

        let month_total: f64 = current_month_indices
            .iter()
            .map(|index| entries[*index].included_requests + entries[*index].billed_requests)
            .sum();

        let difference = round_request_count(used - month_total);

        if difference.abs() < 0.5 {
            return;
        }

        if difference < 0.0 {
            let refreshed = refreshed_timestamps.cloned().unwrap_or_default();
            let mut excess = round_request_count(-difference);
            let mut candidates: Vec<usize> = current_month_indices;
            candidates.sort_by_key(|index| {
                (
                    refreshed.contains(&entries[*index].timestamp),
                    entries[*index].timestamp,
                )
            });

            let mut remove_indices = Vec::new();
            for index in candidates {
                if excess < 0.5 {
                    break;
                }

                let entry_total = entries[index].included_requests + entries[index].billed_requests;
                if entry_total <= excess + 0.0001 {
                    excess = round_request_count((excess - entry_total).max(0.0));
                    remove_indices.push(index);
                    continue;
                }

                let billed_reduction = entries[index].billed_requests.min(excess);
                entries[index].billed_requests = round_request_count(
                    (entries[index].billed_requests - billed_reduction).max(0.0),
                );
                excess = round_request_count((excess - billed_reduction).max(0.0));

                if excess > 0.0 {
                    entries[index].included_requests =
                        round_request_count((entries[index].included_requests - excess).max(0.0));
                    excess = 0.0;
                }

                entries[index].used = round_request_count(
                    entries[index].included_requests + entries[index].billed_requests,
                );
                entries[index].limit = limit;

                if entries[index].used < 0.5 {
                    remove_indices.push(index);
                }
            }

            if excess >= 0.5 {
                log::warn!(
                    "[Usage] Unable to fully prune stale current-month history; {} requests remain above the authoritative total",
                    excess
                );
            }

            remove_indices.sort_unstable_by(|a, b| b.cmp(a));
            remove_indices.dedup();
            for index in remove_indices {
                entries.remove(index);
            }

            entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            return;
        }

        let delta = difference;
        let today_timestamp = now
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
            .timestamp();

        if let Some(today_entry) = entries
            .iter_mut()
            .find(|entry| entry.timestamp == today_timestamp)
        {
            today_entry.included_requests += delta;
            today_entry.used =
                round_request_count(today_entry.included_requests + today_entry.billed_requests);
            today_entry.limit = limit;
        } else {
            entries.push(UsageEntry {
                timestamp: today_timestamp,
                used: round_request_count(delta),
                limit,
                included_requests: delta,
                billed_requests: 0.0,
                gross_amount: 0.0,
                billed_amount: 0.0,
                models: vec![],
                quota_estimated: false,
            });
        }

        entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    }

    /// Start background usage polling with cancellation support
    /// Uses wall-clock time to handle system sleep/hibernation correctly
    /// Returns a channel sender that can be used to cancel the polling task
    pub fn start_polling(app: AppHandle, interval_seconds: u64) -> tokio::sync::mpsc::Sender<()> {
        let (cancel_tx, mut cancel_rx) = tokio::sync::mpsc::channel::<()>(1);

        let max_tick_secs: u64 = interval_seconds.clamp(1, 30);
        log::info!(
            "[Background Polling] Starting polling task with interval: {}s",
            interval_seconds
        );

        tauri::async_runtime::spawn(async move {
            // Use wall-clock time for reliable timing across system sleep/hibernation
            let mut last_tick = chrono::Utc::now();
            let interval_duration = chrono::Duration::seconds(interval_seconds as i64);

            loop {
                // Dynamic tick: sleep for remaining interval time, capped at max_tick_secs
                // so the task remains cancellable even for large intervals.
                // Also resets last_tick if the system clock jumps backward (C3 fix).
                let tick_secs = {
                    let now = chrono::Utc::now();
                    let elapsed = now.signed_duration_since(last_tick);
                    if elapsed < chrono::Duration::zero() {
                        log::warn!(
                            "[Background Polling] System clock moved backward. Resetting timer."
                        );
                        last_tick = now;
                    }
                    let effective_elapsed = elapsed.max(chrono::Duration::zero());
                    let remaining = if interval_duration > effective_elapsed {
                        interval_duration - effective_elapsed
                    } else {
                        chrono::Duration::zero()
                    };
                    (remaining.num_seconds() as u64).clamp(1, max_tick_secs)
                };

                tokio::select! {
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(tick_secs)) => {
                        let now = chrono::Utc::now();
                        let elapsed = now.signed_duration_since(last_tick);

                        // Check if enough wall-clock time has passed
                        if elapsed >= interval_duration {
                            log::info!(
                                "[Background Polling] Timer tick fired (elapsed: {}s, configured: {}s)",
                                elapsed.num_seconds(),
                                interval_seconds
                            );

                            // SAFETY: Only access StoreManager if it's available
                            // Use try_state to avoid panicking if state is not yet managed
                            match app.try_state::<StoreManager>() {
                                Some(store) => {
                                    if store.is_authenticated() {
                                        // Create a new usage manager for this poll
                                        let mut usage_manager = UsageManager::new();

                                        log::debug!("[Background Polling] Fetching usage data...");
                                        if let Ok(summary) = usage_manager.fetch_usage(&app).await {
                                            log::info!(
                                                "[Background Polling] Usage updated: {}/{} ({}%)",
                                                summary.used,
                                                summary.limit,
                                                summary.percentage
                                            );
                                            // Auto backup after successful poll.
                                            // Use spawn_blocking to avoid blocking the async
                                            // executor with file I/O (create_backup performs
                                            // several fs::copy / fs::rename / fs::write calls).
                                            // record_auto_backup_time is inside the blocking
                                            // closure to prevent a race where the next poll
                                            // could fire before the timestamp is persisted.
                                            if store.should_auto_backup() {
                                                let app_for_backup = app.clone();
                                                tauri::async_runtime::spawn(async move {
                                                    match tokio::task::spawn_blocking(move || {
                                                        let Some(s) = app_for_backup
                                                            .try_state::<StoreManager>()
                                                        else {
                                                            return Err(
                                                                "StoreManager not available"
                                                                    .to_string(),
                                                            );
                                                        };
                                                        match s.create_backup() {
                                                            Ok(backup_id) => {
                                                                log::info!(
                                                                    "[Background Polling] Auto backup created: {}",
                                                                    backup_id
                                                                );
                                                                if let Err(e) = s.record_auto_backup_time() {
                                                                    log::warn!("[Background Polling] Failed to record auto backup time (next backup timing may be affected): {}", e);
                                                                }
                                                                Ok(())
                                                            }
                                                            Err(e) => Err(e),
                                                        }
                                                    })
                                                    .await
                                                    {
                                                        Ok(Ok(())) => {}
                                                        Ok(Err(e)) => log::warn!(
                                                            "[Background Polling] Auto backup failed (non-fatal): {}",
                                                            e
                                                        ),
                                                        Err(e) => log::warn!(
                                                            "[Background Polling] Auto backup task panicked: {}",
                                                            e
                                                        ),
                                                    }
                                                });
                                            }
                                        } else {
                                            log::warn!("[Background Polling] Failed to fetch usage");
                                        }
                                    } else {
                                        log::debug!("[Background Polling] Skipping - not authenticated");
                                    }
                                }
                                None => {
                                    // StoreManager not yet available - skip this tick
                                    log::warn!("[Background Polling] StoreManager not available, skipping tick");
                                }
                            }

                            // Update last_tick to current wall-clock time
                            last_tick = now;
                        }
                    }
                    _ = cancel_rx.recv() => {
                        log::info!("[Background Polling] Received cancel signal, stopping polling task");
                        break;
                    }
                }
            }

            log::info!("[Background Polling] Polling task stopped");
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
            return Ok(used.round() as u32);
        }

        let daily_average = used as f32 / current_day;
        let remaining_days = days_in_month as f32 - current_day;
        let predicted = used as f32 + (daily_average * remaining_days);

        Ok(predicted as u32)
    }

    pub fn predict_usage_from_history(
        history: &[UsageEntry],
        used: f64,
        limit: u32,
        prediction_period: u32,
    ) -> Option<UsagePrediction> {
        if history.is_empty() {
            return None;
        }

        // Get weights based on period
        let weights = match prediction_period {
            7 => vec![1.5, 1.5, 1.2, 1.2, 1.2, 1.0, 1.0],
            14 => vec![
                2.0, 1.8, 1.6, 1.4, 1.2, 1.2, 1.0, 1.0, 1.0, 1.0, 0.8, 0.8, 0.6, 0.6,
            ],
            21 => vec![
                2.5, 2.3, 2.1, 1.9, 1.7, 1.5, 1.3, 1.3, 1.2, 1.2, 1.1, 1.1, 1.0, 1.0, 0.9, 0.9,
                0.8, 0.8, 0.7, 0.7, 0.6,
            ],
            _ => vec![1.0], // Fallback
        };

        // Take only the number of days specified by prediction_period
        let daily_data = history
            .iter()
            .take(prediction_period as usize)
            .collect::<Vec<_>>();

        if daily_data.is_empty() {
            return None;
        }

        // 1. Calculate weighted average daily usage
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;

        for (i, entry) in daily_data.iter().enumerate() {
            let weight = if i < weights.len() { weights[i] } else { 1.0 };
            weighted_sum += entry.used * weight;
            total_weight += weight;
        }

        let weighted_avg_daily = if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        };

        // 2. Calculate weekend/weekday ratio
        let mut weekend_sum = 0.0;
        let mut weekend_count = 0;
        let mut weekday_sum = 0.0;
        let mut weekday_count = 0;

        for entry in &daily_data {
            let dt = chrono::DateTime::from_timestamp(entry.timestamp, 0)
                .map(|dt| dt.date_naive())
                .unwrap_or_else(|| chrono::Utc::now().date_naive());

            let is_weekend = matches!(dt.weekday(), chrono::Weekday::Sat | chrono::Weekday::Sun);

            if is_weekend {
                weekend_sum += entry.used;
                weekend_count += 1;
            } else {
                weekday_sum += entry.used;
                weekday_count += 1;
            }
        }

        let avg_weekend = if weekend_count > 0 {
            weekend_sum / weekend_count as f64
        } else {
            0.0
        };
        let avg_weekday = if weekday_count > 0 {
            weekday_sum / weekday_count as f64
        } else {
            0.0
        };

        // If no weekday data, ratio is 1.0
        let weekend_ratio = if avg_weekday > 0.0 {
            avg_weekend / avg_weekday
        } else {
            1.0
        };

        // 3. Calculate remaining days
        let now = chrono::Utc::now();
        let today = now.date_naive();
        let current_day = now.day();

        let days_in_month = if now.month() == 12 {
            31
        } else {
            let next_month = chrono::NaiveDate::from_ymd_opt(now.year(), now.month() + 1, 1)
                .unwrap_or_else(|| chrono::NaiveDate::from_ymd_opt(now.year() + 1, 1, 1).unwrap());
            let current_month =
                chrono::NaiveDate::from_ymd_opt(now.year(), now.month(), 1).unwrap();
            (next_month - current_month).num_days() as u32
        };

        let remaining_days = days_in_month.saturating_sub(current_day);

        // Count remaining weekdays and weekends
        let mut remaining_weekdays = 0;
        let mut remaining_weekends = 0;

        for i in 1..=remaining_days {
            // Need to handle month overflow if we just add days?
            // Actually simpler: iterate and check.
            // Note: `today` is current day. We want remaining days effectively from tomorrow?
            // predictor.ts: date.setDate(today.getDate() + i)
            // So yes, starting from tomorrow.

            if let Some(date) = today.checked_add_days(chrono::Days::new(i as u64)) {
                if matches!(date.weekday(), chrono::Weekday::Sat | chrono::Weekday::Sun) {
                    remaining_weekends += 1;
                } else {
                    remaining_weekdays += 1;
                }
            }
        }

        // 4. Predict total monthly usage
        // usage = current + (avg * weekdays) + (avg * ratio * weekends)
        let predicted_remaining = (weighted_avg_daily * remaining_weekdays as f64)
            + (weighted_avg_daily * weekend_ratio * remaining_weekends as f64);

        let predicted_monthly_requests = round_request_count(used + predicted_remaining);
        let excess_requests = (predicted_monthly_requests - limit as f64).max(0.0);
        let predicted_billed_amount = excess_requests * 0.04;

        let confidence_level = if daily_data.len() < 3 {
            "low"
        } else if daily_data.len() < 7 {
            "medium"
        } else {
            "high"
        };

        Some(UsagePrediction {
            predicted_monthly_requests,
            predicted_billed_amount,
            confidence_level: confidence_level.to_string(),
            days_used_for_prediction: daily_data.len() as u32,
        })
    }

    /// Calculate days until limit is reached
    pub fn days_until_limit(app: &AppHandle) -> Result<Option<i64>, String> {
        let store = app.state::<StoreManager>();
        let (used, limit) = store.get_usage();

        if used >= limit as f64 {
            return Ok(Some(0)); // Already at or over limit
        }

        let remaining = (limit as f64 - used) as f32;

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

#[cfg(test)]
#[path = "usage_history_persistence_tests.rs"]
mod usage_history_persistence_tests;
