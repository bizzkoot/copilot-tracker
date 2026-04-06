use super::*;
use crate::auth::UsageHistoryRow;
use crate::store::StoreManager;
use tempfile::TempDir;

fn timestamp_for(date: &str) -> i64 {
    chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc()
        .timestamp()
}

fn history_row(date: &str, included_requests: f64, billed_requests: f64) -> UsageHistoryRow {
    UsageHistoryRow {
        date: date.to_string(),
        included_requests,
        billed_requests,
        gross_amount: 0.0,
        billed_amount: 0.0,
        models: vec![],
    }
}

fn history_entry(date: &str, used: f64, limit: u32, quota_estimated: bool) -> UsageEntry {
    UsageEntry {
        timestamp: timestamp_for(date),
        used,
        limit,
        included_requests: used,
        billed_requests: 0.0,
        gross_amount: 0.0,
        billed_amount: 0.0,
        models: vec![],
        quota_estimated,
    }
}

#[test]
fn extraction_results_are_rejected_after_session_generation_changes() {
    let tmp = TempDir::new().unwrap();
    let store = StoreManager::new(tmp.path().to_path_buf()).unwrap();

    store.set_customer_id(1).unwrap();
    let generation = store.get_session_generation();
    assert!(UsageManager::can_apply_extraction_results(
        &store, generation
    ));

    store.clear_user_session().unwrap();

    assert!(!UsageManager::can_apply_extraction_results(
        &store, generation
    ));
}

#[test]
fn extraction_results_are_revalidated_before_apply_after_session_changes() {
    let tmp = TempDir::new().unwrap();
    let store = StoreManager::new(tmp.path().to_path_buf()).unwrap();

    store.set_customer_id(1).unwrap();
    let generation = store.get_session_generation();

    assert!(UsageManager::can_apply_extraction_results(
        &store, generation
    ));
    assert!(UsageManager::lock_and_validate_extraction_results(&store, generation).is_ok());

    store.clear_user_session().unwrap();

    assert!(UsageManager::lock_and_validate_extraction_results(&store, generation).is_err());
}

#[test]
fn extraction_results_are_rejected_while_a_session_transition_is_active() {
    let tmp = TempDir::new().unwrap();
    let store = StoreManager::new(tmp.path().to_path_buf()).unwrap();

    store.set_customer_id(1).unwrap();
    let generation = store.get_session_generation();
    store.begin_session_transition();

    assert!(!UsageManager::can_apply_extraction_results(
        &store, generation
    ));

    store.finish_session_transition();
}

#[test]
fn stores_history_rows_when_extraction_has_history_but_no_usage_summary() {
    let tmp = TempDir::new().unwrap();
    let store = StoreManager::new(tmp.path().to_path_buf()).unwrap();
    store.set_usage_history(vec![history_entry("2026-03-31", 240.0, 3000, false)]);

    let rows = vec![history_row("2026-04-01", 10.0, 0.0)];

    let merged = UsageManager::persist_history_without_usage_data(&store, &rows);

    assert_eq!(
        merged.len(),
        2,
        "new history rows should be merged into store"
    );

    let stored = store.get_usage_history();
    assert_eq!(
        stored.len(),
        2,
        "merged history should be persisted to store"
    );
    assert_eq!(
        stored
            .iter()
            .find(|entry| entry.timestamp == timestamp_for("2026-03-31"))
            .unwrap()
            .limit,
        3000,
        "existing historical quota should be preserved"
    );

    let april_entry = stored
        .iter()
        .find(|entry| entry.timestamp == timestamp_for("2026-04-01"))
        .unwrap();
    assert_eq!(april_entry.used, 10.0);
    assert_eq!(
        april_entry.limit, 0,
        "history-only persistence should not invent a current quota when usage summary is missing"
    );
    assert!(
        april_entry.quota_estimated,
        "history rows saved without a usage summary must remain estimated"
    );
}

#[test]
fn persists_history_only_refresh_using_cached_summary_for_reconciliation() {
    let today = chrono::Utc::now().date_naive();
    let yesterday = today.pred_opt().unwrap();
    let today_str = today.format("%Y-%m-%d").to_string();
    let yesterday_str = yesterday.format("%Y-%m-%d").to_string();

    let tmp = TempDir::new().unwrap();
    let store = StoreManager::new(tmp.path().to_path_buf()).unwrap();
    store.set_usage_history(vec![
        history_entry(&yesterday_str, 10.0, 1200, false),
        history_entry(&today_str, 20.0, 1200, false),
    ]);

    let rows = vec![history_row(&today_str, 20.0, 0.0)];
    let summary = UsageSummary {
        used: 50.0,
        limit: 1200,
        remaining: 1150.0,
        percentage: 4.2,
        timestamp: 0,
    };

    let merged = UsageManager::persist_history_with_cached_summary(&store, &rows, &summary);

    let month_total: f64 = merged
        .iter()
        .map(|entry| entry.included_requests + entry.billed_requests)
        .sum();

    assert_eq!(
        month_total, 50.0,
        "history-only refresh should reconcile to the cached usage summary"
    );
    assert_eq!(
        merged
            .iter()
            .find(|entry| entry.timestamp == timestamp_for(&today_str))
            .unwrap()
            .used,
        40.0
    );
}

#[test]
fn cached_summary_reconciliation_requires_current_day_fetch_timestamp() {
    let now = chrono::DateTime::parse_from_rfc3339("2026-04-04T12:00:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);

    assert!(!UsageManager::can_reconcile_with_cached_summary(0, now));
    assert!(!UsageManager::can_reconcile_with_cached_summary(
        timestamp_for("2026-03-31"),
        now,
    ));
    assert!(!UsageManager::can_reconcile_with_cached_summary(
        timestamp_for("2026-04-01"),
        now,
    ));
    assert!(UsageManager::can_reconcile_with_cached_summary(
        chrono::DateTime::parse_from_rfc3339("2026-04-04T08:00:00Z")
            .unwrap()
            .timestamp(),
        now,
    ));
}

#[test]
fn keeps_prior_month_entries_when_new_month_refresh_returns_partial_history() {
    let existing = vec![
        history_entry("2026-04-01", 5.0, 1200, false),
        history_entry("2026-03-31", 240.0, 3000, false),
    ];
    let rows = vec![history_row("2026-04-01", 10.0, 0.0)];
    let quota_map = HashMap::from([(String::from("2026-04"), 1200)]);

    let merged = UsageManager::merge_history_rows_with_persisted_history(
        &existing, &rows, &quota_map, 1200, None,
    );

    assert_eq!(merged.len(), 2, "prior-month entry must be preserved");
    assert_eq!(
        merged
            .iter()
            .find(|entry| entry.timestamp == timestamp_for("2026-03-31"))
            .unwrap()
            .limit,
        3000
    );
    assert_eq!(
        merged
            .iter()
            .find(|entry| entry.timestamp == timestamp_for("2026-04-01"))
            .unwrap()
            .used,
        10.0
    );
}

#[test]
fn reuses_known_month_quota_instead_of_current_month_fallback() {
    let existing = vec![history_entry("2026-03-31", 240.0, 3000, false)];
    let rows = vec![history_row("2026-03-15", 90.0, 0.0)];

    let merged = UsageManager::merge_history_rows_with_persisted_history(
        &existing,
        &rows,
        &HashMap::new(),
        1200,
        None,
    );

    let march_entry = merged
        .iter()
        .find(|entry| entry.timestamp == timestamp_for("2026-03-15"))
        .unwrap();

    assert_eq!(
        march_entry.limit, 3000,
        "stored month quota must win over current-month fallback quota"
    );
    assert!(
        !march_entry.quota_estimated,
        "known month quota should stay non-estimated"
    );
}

#[test]
fn estimated_historical_limit_stays_estimated_when_no_authoritative_quota_exists() {
    let existing = vec![history_entry("2026-03-31", 240.0, 1200, true)];
    let rows = vec![history_row("2026-03-15", 90.0, 0.0)];

    let merged = UsageManager::merge_history_rows_with_persisted_history(
        &existing,
        &rows,
        &HashMap::new(),
        3000,
        None,
    );

    let march_entry = merged
        .iter()
        .find(|entry| entry.timestamp == timestamp_for("2026-03-15"))
        .unwrap();

    assert_eq!(
        march_entry.limit, 1200,
        "estimated historical month limit should be preserved instead of being rewritten to the current month's quota"
    );
    assert!(
        march_entry.quota_estimated,
        "estimated historical month limit must not become authoritative"
    );
}

#[test]
fn partial_current_month_refresh_does_not_double_count_preserved_days() {
    let now = chrono::DateTime::parse_from_rfc3339("2026-04-02T12:00:00Z")
        .unwrap()
        .with_timezone(&chrono::Utc);
    let existing = vec![
        history_entry("2026-04-01", 10.0, 1200, false),
        history_entry("2026-04-02", 20.0, 1200, false),
    ];
    let rows = vec![history_row("2026-04-02", 20.0, 0.0)];
    let quota_map = HashMap::from([(String::from("2026-04"), 1200)]);

    let merged = UsageManager::merge_history_rows_with_persisted_history_at(
        &existing,
        &rows,
        &quota_map,
        1200,
        Some(50.0),
        now,
    );

    let month_total: f64 = merged
        .iter()
        .map(|entry| entry.included_requests + entry.billed_requests)
        .sum();

    assert_eq!(
        month_total, 50.0,
        "merged month total must match current usage"
    );
    assert_eq!(
        merged
            .iter()
            .find(|entry| entry.timestamp == timestamp_for("2026-04-01"))
            .unwrap()
            .used,
        10.0
    );
    assert_eq!(
        merged
            .iter()
            .find(|entry| entry.timestamp == timestamp_for("2026-04-02"))
            .unwrap()
            .used,
        40.0
    );
}
