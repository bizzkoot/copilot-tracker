use serde::{Deserialize, Serialize};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tokio::sync::{mpsc, Mutex as TokioMutex};
use tokio::time::Duration;
use url::Url;

use crate::StoreManager;

/// Global channel for hidden extraction webview events.
static HIDDEN_WEBVIEW_EVENTS: TokioMutex<Option<mpsc::Sender<HiddenWebviewEvent>>> =
    TokioMutex::const_new(None);

/// Global channel for hidden update-check webview events.
static UPDATE_CHECK_WEBVIEW_EVENTS: TokioMutex<Option<mpsc::Sender<HiddenWebviewEvent>>> =
    TokioMutex::const_new(None);

/// Serializes concurrent extraction attempts so only one runs at a time.
static EXTRACTION_LOCK: TokioMutex<()> = TokioMutex::const_new(());

/// Serializes concurrent update checks so they cannot clobber each other's
/// event channel.
static UPDATE_CHECK_LOCK: TokioMutex<()> = TokioMutex::const_new(());

#[derive(Debug, Clone)]
pub struct HiddenWebviewEvent {
    pub event: String,
    pub payload: String,
}

const GITHUB_BILLING_URL: &str = "https://github.com/settings/billing";
const GITHUB_LOGIN_URL: &str = "https://github.com/login";
const GITHUB_API_URL: &str =
    "https://api.github.com/repos/bizzkoot/copilot-tracker/releases/latest";
const EXTRACTION_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthState {
    pub is_authenticated: bool,
    pub customer_id: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExtractionResult {
    pub customer_id: Option<u64>,
    pub usage_data: Option<UsageData>,
    pub usage_history: Option<Vec<UsageHistoryRow>>,
    pub raw_usage_payload: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageHistoryRow {
    pub date: String,
    pub included_requests: f64,
    pub billed_requests: f64,
    pub gross_amount: f64,
    pub billed_amount: f64,
    pub models: Vec<UsageModelRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageModelRow {
    pub name: String,
    pub included_requests: f64,
    pub billed_requests: f64,
    pub gross_amount: f64,
    pub billed_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageData {
    pub net_billed_amount: f64,
    pub net_quantity: f64,
    pub discount_quantity: f64,
    pub user_premium_request_entitlement: u32,
    pub filtered_user_premium_request_entitlement: u32,
}

fn round_request_count(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

fn parse_json_number(value: &serde_json::Value) -> Option<f64> {
    match value {
        serde_json::Value::Number(n) => n.as_f64(),
        serde_json::Value::String(s) => s.trim().replace(',', "").parse::<f64>().ok(),
        _ => None,
    }
}

fn parse_json_u32(value: &serde_json::Value) -> Option<u32> {
    parse_json_number(value).and_then(|v| {
        if v.is_finite() && v >= 0.0 {
            Some(v.round() as u32)
        } else {
            None
        }
    })
}

fn parse_usage_card_data(usage_card: &serde_json::Value) -> Option<UsageData> {
    let used_opt = usage_card
        .get("discountQuantity")
        .and_then(parse_json_number)
        .map(round_request_count);
    let limit_opt = usage_card
        .get("userPremiumRequestEntitlement")
        .and_then(parse_json_u32);

    if used_opt.is_none() && limit_opt.is_none() {
        return None;
    }

    Some(UsageData {
        net_billed_amount: usage_card
            .get("netBilledAmount")
            .and_then(parse_json_number)
            .unwrap_or(0.0),
        net_quantity: usage_card
            .get("netQuantity")
            .and_then(parse_json_number)
            .map(round_request_count)
            .unwrap_or_else(|| used_opt.unwrap_or(0.0)),
        discount_quantity: used_opt.unwrap_or(0.0),
        user_premium_request_entitlement: limit_opt.unwrap_or(0),
        filtered_user_premium_request_entitlement: usage_card
            .get("filteredUserPremiumRequestEntitlement")
            .and_then(parse_json_u32)
            .unwrap_or_else(|| limit_opt.unwrap_or(0)),
    })
}

fn merge_entitlement_usage(
    usage_data: Option<UsageData>,
    entitlement: Option<&serde_json::Value>,
) -> Option<UsageData> {
    let mut merged = usage_data;

    let Some(ent) = entitlement else {
        log::warn!("No entitlement field in extraction payload");
        return merged;
    };

    if !ent
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        let err = ent
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        log::warn!("Entitlement fetch failed: {}", err);
        return merged;
    }

    let entitlement_limit = ent.get("limit").and_then(parse_json_u32).unwrap_or(0);
    let entitlement_used = ent
        .get("used")
        .and_then(parse_json_number)
        .map(round_request_count)
        .unwrap_or(0.0);

    if let Some(existing) = merged.as_mut() {
        // usageCard count is authoritative for display when present.
        if entitlement_limit > 0 {
            existing.user_premium_request_entitlement = entitlement_limit;
            existing.filtered_user_premium_request_entitlement = entitlement_limit;
        }
        if existing.discount_quantity <= 0.0 && entitlement_used > 0.0 {
            existing.discount_quantity = entitlement_used;
        }
        return merged;
    }

    if entitlement_limit > 0 || entitlement_used > 0.0 {
        return Some(UsageData {
            net_billed_amount: 0.0,
            net_quantity: entitlement_used,
            discount_quantity: entitlement_used,
            user_premium_request_entitlement: entitlement_limit,
            filtered_user_premium_request_entitlement: entitlement_limit,
        });
    }

    merged
}

/// Parses a single usage history row from JSON data
/// Extracts date, request counts, amounts, and model breakdowns
fn parse_usage_history_row(row: &serde_json::Value) -> Option<UsageHistoryRow> {
    let id = row.get("id").and_then(|v| v.as_str())?.to_string();
    let cells = row.get("cells").and_then(|v| v.as_array())?;

    if cells.len() < 5 {
        return None;
    }

    let included_requests = parse_request_count(cells.get(1)?.get("value")?)?;

    let billed_requests = parse_request_count(cells.get(2)?.get("value")?)?;

    let gross_amount = cells
        .get(3)?
        .get("value")?
        .as_str()?
        .trim_start_matches('$')
        .parse::<f64>()
        .ok()?;

    let billed_amount = cells
        .get(4)?
        .get("value")?
        .as_str()?
        .trim_start_matches('$')
        .parse::<f64>()
        .ok()?;

    let models = if let Some(subtable) = row.get("subtable") {
        if let Some(sub_rows) = subtable.get("rows").and_then(|v| v.as_array()) {
            sub_rows.iter().filter_map(parse_usage_model_row).collect()
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    Some(UsageHistoryRow {
        date: id,
        included_requests,
        billed_requests,
        gross_amount,
        billed_amount,
        models,
    })
}

/// Parses a single usage model row from JSON data
/// Extracts model name, request counts, and amounts
fn parse_usage_model_row(sub_row: &serde_json::Value) -> Option<UsageModelRow> {
    let sub_cells = sub_row.get("cells").and_then(|v| v.as_array())?;
    if sub_cells.len() < 5 {
        return None;
    }

    let name = sub_cells.first()?.get("value")?.as_str()?.to_string();
    let included_requests = parse_request_count(sub_cells.get(1)?.get("value")?)?;
    let billed_requests = parse_request_count(sub_cells.get(2)?.get("value")?)?;
    let gross_amount = sub_cells
        .get(3)?
        .get("value")?
        .as_str()?
        .trim_start_matches('$')
        .parse::<f64>()
        .ok()?;
    let billed_amount = sub_cells
        .get(4)?
        .get("value")?
        .as_str()?
        .trim_start_matches('$')
        .parse::<f64>()
        .ok()?;

    Some(UsageModelRow {
        name,
        included_requests,
        billed_requests,
        gross_amount,
        billed_amount,
    })
}

/// Parse GitHub request counts that may be integer or decimal strings (e.g. "6.90").
fn parse_request_count(value: &serde_json::Value) -> Option<f64> {
    match value {
        serde_json::Value::Number(n) => n.as_f64().map(|v| (v * 10.0).round() / 10.0),
        serde_json::Value::String(s) => {
            let cleaned = s.trim().replace(',', "");
            cleaned
                .parse::<f64>()
                .ok()
                .map(|v| (v * 10.0).round() / 10.0)
        }
        _ => None,
    }
}

#[derive(Clone)]
pub struct AuthManager {
    auth_window: Option<tauri::WebviewWindow>,
    customer_id: Option<u64>,
    extraction_in_progress: bool,
    auth_window_listener_attached: bool,
    visible_auth_generation: Arc<AtomicU64>,
}

impl AuthManager {
    pub fn new() -> Self {
        Self {
            auth_window: None,
            customer_id: None,
            extraction_in_progress: false,
            auth_window_listener_attached: false,
            visible_auth_generation: Arc::new(AtomicU64::new(0)),
        }
    }

    fn can_apply_visible_auth_result(store: &StoreManager, expected_generation: u64) -> bool {
        expected_generation > 0
            && !store.is_session_transition_in_progress()
            && store.get_session_generation() == expected_generation
    }

    fn apply_customer_extraction_result(
        result: &serde_json::Value,
        customer_id: &mut Option<u64>,
        error: &mut Option<String>,
    ) {
        if result
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            if let Some(id) = result.get("id").and_then(|v| v.as_u64()) {
                *customer_id = Some(id);
                *error = None;
            } else if customer_id.is_none() {
                *error = Some("Customer extraction succeeded without an id".to_string());
            }
        } else if customer_id.is_none() {
            *error = result
                .get("error")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
        }
    }

    fn apply_usage_extraction_result(
        result: &serde_json::Value,
        customer_id: &mut Option<u64>,
        usage_data: &mut Option<UsageData>,
        usage_history: &mut Option<Vec<UsageHistoryRow>>,
        raw_usage_payload: &mut Option<serde_json::Value>,
        error: &mut Option<String>,
    ) {
        *raw_usage_payload = Some(result.clone());

        if customer_id.is_none() {
            *customer_id = result.get("customerId").and_then(|v| v.as_u64());
        }

        let parsed_usage = result
            .get("usageCard")
            .and_then(|v| v.get("data"))
            .and_then(parse_usage_card_data);
        *usage_data = merge_entitlement_usage(parsed_usage, result.get("entitlement"));

        *usage_history = result
            .get("usageTable")
            .and_then(|v| v.get("data"))
            .and_then(|v| v.get("table"))
            .and_then(|v| v.get("rows"))
            .and_then(|v| v.as_array())
            .map(|rows| rows.iter().filter_map(parse_usage_history_row).collect());

        if customer_id.is_some() && (usage_data.is_some() || usage_history.is_some()) {
            *error = None;
        }
    }

    /// Create or show the auth webview window
    pub fn show_auth_window(&mut self, app: &AppHandle) -> Result<(), String> {
        let current_generation = {
            let store = app.state::<StoreManager>();
            store.get_session_generation()
        };
        self.visible_auth_generation
            .store(current_generation, Ordering::SeqCst);

        // If window exists, just show it
        if let Some(window) = &self.auth_window {
            if window.is_visible().unwrap_or(false) {
                window
                    .show()
                    .map_err(|e| format!("Failed to show window: {}", e))?;
                window
                    .set_focus()
                    .map_err(|e| format!("Failed to focus window: {}", e))?;
                let url = Url::parse(GITHUB_LOGIN_URL)
                    .map_err(|e| format!("Failed to parse URL: {}", e))?;
                window
                    .navigate(url)
                    .map_err(|e| format!("Failed to navigate: {}", e))?;
                return Ok(());
            }
        }

        // Create new auth window
        let url =
            Url::parse(GITHUB_LOGIN_URL).map_err(|e| format!("Failed to parse URL: {}", e))?;

        let app_handle = app.clone();
        let visible_auth_generation = Arc::clone(&self.visible_auth_generation);
        let window = WebviewWindowBuilder::new(app, "auth", WebviewUrl::External(url))
        .on_navigation(move |url| {
            let url_str = url.as_str();

            // Check for HTTPS interception redirect
            if url_str.contains("copilot-auth-success.local") {
                log::info!("Intercepted auth success URL: {}", url_str);

                let mut extracted_id = None;
                let mut extracted_usage_data = None;
                let mut extracted_usage_history = None;

                // Try to parse from hash payload first (new method)
                if let Some(fragment) = url.fragment() {
                    if let Some(encoded) = fragment.strip_prefix("payload=") {
                        if let Ok(decoded) = urlencoding::decode(encoded) {
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&decoded) {
                                // Extract ID
                                if let Some(id) = json.get("id").and_then(|v| v.as_u64()) {
                                    extracted_id = Some(id);

                                    // Extract Usage Data
                                    extracted_usage_data = json
                                      .get("usageCard")
                                      .and_then(|v| v.get("data"))
                                      .and_then(parse_usage_card_data);
                                    extracted_usage_data =
                                      merge_entitlement_usage(extracted_usage_data, json.get("entitlement"));

                                    // Extract Usage History
                                    if let Some(rows) = json.get("usageTable")
                                        .and_then(|v| v.get("data"))
                                        .and_then(|v| v.get("table"))
                                        .and_then(|v| v.get("rows"))
                                        .and_then(|v| v.as_array())
                                    {
                                        log::info!("Parsing usage history, found {} rows", rows.len());
                                        let history: Vec<UsageHistoryRow> = rows.iter().filter_map(parse_usage_history_row).collect();

                                        log::info!("Successfully parsed {} history rows", history.len());
                                        extracted_usage_history = Some(history);
                                    }
                                }
                            }
                        }
                    }
                }

                // Fallback to query param
                if extracted_id.is_none() {
                    if let Some((_, id_str)) = url.query_pairs().find(|(key, _)| key == "id") {
                        if let Ok(id) = id_str.parse::<u64>() {
                            extracted_id = Some(id);
                        }
                    }
                }

                if let Some(id) = extracted_id {
                      let expected_generation = visible_auth_generation.load(Ordering::SeqCst);
                      let store = app_handle.state::<StoreManager>();
                      let _state_operation = match store.lock_state_operation() {
                          Ok(guard) => guard,
                          Err(error) => {
                              log::error!(
                                  "Failed to lock state for auth window success handling: {}",
                                  error
                              );
                              return false;
                          }
                      };
                      if !AuthManager::can_apply_visible_auth_result(&store, expected_generation) {
                          log::warn!(
                              "Discarding auth window success because the session changed while login was in progress"
                          );
                          return false;
                      }

                      if store.set_customer_id_locked(id).is_ok() {
                          log::info!("Successfully authenticated with Customer ID: {}", id);

                          // Process usage data and emit events via shared helper
                           if let Some(ref usage) = extracted_usage_data {
                               if let Err(error) = crate::usage::UsageManager::process_and_emit_usage(
                                   &app_handle,
                                   id,
                                   usage,
                                   extracted_usage_history.take(),
                               ) {
                                   log::error!("Failed to persist usage after authentication: {}", error);
                               }
                           } else if let Some(rows) = extracted_usage_history.take() {
                               log::warn!(
                                   "No usage data was extracted from GitHub API; persisting {} history rows only",
                                   rows.len()
                               );
                               if let Err(error) =
                                   crate::usage::UsageManager::persist_history_without_usage_data(
                                       &store, &rows,
                                   )
                               {
                                   log::error!(
                                       "Failed to persist history-only usage data after authentication: {}",
                                       error
                                   );
                               }
                           } else {
                               log::warn!("No usage data was extracted from GitHub API");
                           }

                         let _ = app_handle.emit("auth:state-changed", "authenticated");

                         // Trigger refresh to get fresh usage data (same as tray menu refresh)
                         let app_handle_refresh = app_handle.clone();
                         tauri::async_runtime::spawn(async move {
                             log::info!("Auto-refreshing usage data after authentication...");
                             let mut usage_manager = crate::usage::UsageManager::new();
                             match usage_manager.fetch_usage(&app_handle_refresh).await {
                                 Ok(summary) => {
                                     log::info!("Auto-refresh after auth succeeded: {}/{} (tray should update via usage:updated event)",
                                         summary.used, summary.limit);
                                 }
                                 Err(e) => {
                                     log::error!("Auto-refresh after auth failed: {}", e);
                                 }
                             }
                         });

                         // Close auth window
                         if let Some(auth_window) = app_handle.get_webview_window("auth") {
                             let _ = auth_window.close();
                         }

                         // Show main window
                         if let Some(main_window) = app_handle.get_webview_window("main") {
                             let _ = main_window.show();
                             let _ = main_window.set_focus();
                         }
                     } else {
                         log::error!("Failed to save customer ID to store");
                     }
                } else {
                    log::error!("No customer ID found in URL: {}", url_str);
                }
                return false;
            }

            if url_str.contains("/settings/billing") {
                log::info!("Billing page detected: {}", url_str);
                let _ = app_handle.emit("auth:redirect-detected", url_str);
            }
            true
        })
        .title("GitHub Login")
        .inner_size(900.0, 700.0)
        .resizable(true)
        .visible(true)
        .initialization_script(r#"
            (function() {
              console.log('[AuthInjector] Script loaded');

              // Monitor URL changes for billing page detection
              let currentUrl = location.href;
              console.log('[AuthInjector] Initial URL:', currentUrl);

              function checkUrl() {
                const newUrl = location.href;
                if (newUrl === 'https://github.com/' || newUrl === 'https://github.com') {
                  console.log('[AuthInjector] Detected homepage, redirecting to billing...');
                  window.location.href = 'https://github.com/settings/billing';
                }

                if (newUrl !== currentUrl) {
                  currentUrl = newUrl;
                  console.log('[AuthInjector] URL changed to:', currentUrl);
                  if (currentUrl.includes('/settings/billing')) {
                    console.log('[AuthInjector] Billing page detected, starting extraction in 1.5s');
                    // Page changed to billing - start extraction
                    setTimeout(extractAndSend, 1500);
                  }
                }
              }

              // Monitor URL changes using MutationObserver
              const urlObserver = new MutationObserver(function() {
                checkUrl();
              });

              // Observe changes to the document
              urlObserver.observe(document, { subtree: true, childList: true });

              // Also check on popstate events
              window.addEventListener('popstate', checkUrl);
              window.addEventListener('hashchange', checkUrl);

              if (location.href === 'https://github.com/' || location.href === 'https://github.com') {
                console.log('[AuthInjector] Detected homepage, redirecting to billing...');
                window.location.href = 'https://github.com/settings/billing';
              }

              // Check immediately if already on billing page
              if (location.href.includes('/settings/billing')) {
                console.log('[AuthInjector] Already on billing page, starting extraction in 1.5s');
                setTimeout(extractAndSend, 1500);
              }

              async function getUserId() {
                console.log('[AuthInjector] Attempting to get User ID via API...');
                try {
                  const response = await fetch('/api/v3/user', {
                    headers: { 'Accept': 'application/json' }
                  });
                  console.log('[AuthInjector] API Response Status:', response.status);
                  if (!response.ok) {
                    console.error('[AuthInjector] API request failed:', response.status);
                    return { success: false, error: 'API request failed: ' + response.status };
                  }
                  const data = await response.json();
                  console.log('[AuthInjector] User ID retrieved:', data.id);
                  return { success: true, id: data.id };
                } catch (error) {
                  console.error('[AuthInjector] API request error:', error);
                  return { success: false, error: error.message };
                }
              }

              function getCustomerIdFromDOM() {
                console.log('[AuthInjector] Attempting to get Customer ID from DOM...');
                try {
                  const el = document.querySelector('script[data-target="react-app.embeddedData"]');
                  if (!el) {
                    console.log('[AuthInjector] Embedded data element not found');
                    return { success: false, error: 'Embedded data element not found' };
                  }
                  const data = JSON.parse(el.textContent);
                  const customerId = data?.payload?.customer?.customerId;
                  if (!customerId) {
                    console.log('[AuthInjector] Customer ID not found in embedded data');
                    return { success: false, error: 'Customer ID not found in embedded data' };
                  }
                  console.log('[AuthInjector] Customer ID found in DOM:', customerId);
                  return { success: true, id: customerId };
                } catch (error) {
                  console.error('[AuthInjector] DOM extraction error:', error);
                  return { success: false, error: error.message };
                }
              }

              function getCustomerIdFromHTML() {
                console.log('[AuthInjector] Attempting to get Customer ID from HTML regex...');
                try {
                  const html = document.body.innerHTML;
                  const patterns = [
                    /customerId":(\d+)/,
                    /customerId&quot;:(\d+)/,
                    /customer_id=(\d+)/,
                    /"customerId":(\d+)/,
                    /data-customer-id="(\d+)"/
                  ];
                  for (const pattern of patterns) {
                    const match = html.match(pattern);
                    if (match && match[1]) {
                      console.log('[AuthInjector] Customer ID matched pattern:', pattern);
                      return { success: true, id: parseInt(match[1]) };
                    }
                  }
                  console.log('[AuthInjector] No customer ID pattern matched');
                  return { success: false, error: 'No customer ID pattern matched' };
                } catch (error) {
                  console.error('[AuthInjector] HTML extraction error:', error);
                  return { success: false, error: error.message };
                }
              }

              async function extractCustomerId() {
                console.log('[AuthInjector] Starting extraction chain...');
                let result = await getUserId();
                if (!result.success) {
                  result = getCustomerIdFromDOM();
                }
                if (!result.success) {
                  result = getCustomerIdFromHTML();
                }
                return result;
              }

              async function fetchUsageCard(customerId) {
                try {
                  console.log('[AuthInjector] Fetching usage card for customer:', customerId);
                  const res = await fetch(`/settings/billing/copilot_usage_card?customer_id=${customerId}&period=3`, {
                    headers: {
                      'Accept': 'application/json',
                      'x-requested-with': 'XMLHttpRequest'
                    }
                  });
                  console.log('[AuthInjector] Usage card response status:', res.status);
                  if (!res.ok) {
                    console.error('[AuthInjector] Usage card request failed:', res.status);
                    return { success: false, error: 'Usage card request failed: ' + res.status };
                  }
                  const data = await res.json();
                  console.log('[AuthInjector] Usage card data received:', data ? 'YES' : 'NO', 'Keys:', data ? Object.keys(data) : []);
                  return { success: true, data };
                } catch (error) {
                  console.error('[AuthInjector] Usage card fetch error:', error);
                  return { success: false, error: error.message };
                }
              }

              async function fetchUsageTablePage(customerId, page) {
                const res = await fetch(`/settings/billing/copilot_usage_table?customer_id=${customerId}&group=0&period=3&query=&page=${page}`, {
                  headers: {
                    'Accept': 'application/json',
                    'x-requested-with': 'XMLHttpRequest'
                  }
                });

                if (!res.ok) {
                  return { success: false, error: 'Usage table request failed: ' + res.status };
                }

                const data = await res.json();
                return { success: true, data };
              }

              function getUsageTableTotalPages(data) {
                const candidates = [
                  data?.table?.pagination?.totalPages,
                  data?.table?.pagination?.total_pages,
                  data?.pagination?.totalPages,
                  data?.pagination?.total_pages,
                  data?.table?.totalPages,
                  data?.table?.total_pages,
                  data?.totalPages,
                  data?.total_pages
                ];

                for (const value of candidates) {
                  const parsed = Number(value);
                  if (Number.isFinite(parsed) && parsed > 0) {
                    return parsed;
                  }
                }

                return null;
              }

              async function fetchUsageTable(customerId) {
                try {
                  console.log('[AuthInjector] Fetching usage table for customer:', customerId);

                  // Wait for billing page hydration.
                  await new Promise(resolve => setTimeout(resolve, 3000));

                  const MAX_PAGES = 10;
                  const seenRowIds = new Set();
                  const mergedRows = [];
                  let totalPages = null;
                  let page = 1;
                  let baseData = null;

                  while (page <= (totalPages ?? MAX_PAGES)) {
                    const pageResult = await fetchUsageTablePage(customerId, page);
                    if (!pageResult.success) {
                      if (page === 1) {
                        return pageResult;
                      }
                      break;
                    }

                    if (!baseData) {
                      baseData = pageResult.data;
                    }

                    if (totalPages === null) {
                      totalPages = getUsageTableTotalPages(pageResult.data);
                    }

                    const pageRows = pageResult.data?.table?.rows || [];
                    if (pageRows.length === 0) {
                      break;
                    }

                    let added = 0;
                    for (const row of pageRows) {
                      const key = row?.id || JSON.stringify(row?.cells?.[0]?.value || row);
                      if (!seenRowIds.has(key)) {
                        seenRowIds.add(key);
                        mergedRows.push(row);
                        added += 1;
                      }
                    }

                    if (added === 0) {
                      break;
                    }

                    if (totalPages !== null && page >= totalPages) {
                      break;
                    }

                    page += 1;
                  }

                  const data = {
                    ...(baseData || {}),
                    table: {
                      ...((baseData && baseData.table) ? baseData.table : {}),
                      rows: mergedRows,
                    },
                  };

                  console.log('[AuthInjector] Usage table merged rows:', mergedRows.length);
                  return { success: true, data };
                } catch (error) {
                  console.error('[AuthInjector] Usage table fetch error:', error);
                  return { success: false, error: error.message };
                }
              }

              async function fetchEntitlement() {
                try {
                  console.log('[AuthInjector] Fetching copilot entitlement...');
                  const res = await fetch('/github-copilot/chat/entitlement', {
                    headers: { 'Accept': 'application/json' }
                  });
                  console.log('[AuthInjector] Entitlement response status:', res.status);
                  if (!res.ok) {
                    console.error('[AuthInjector] Entitlement request failed:', res.status);
                    return { success: false, error: 'Entitlement request failed: ' + res.status };
                  }
                  const data = await res.json();
                  const limit = data?.quotas?.limits?.premiumInteractions ?? 0;
                  const remaining = data?.quotas?.remaining?.premiumInteractions ?? 0;
                  const used = limit - remaining;
                  const resetDate = data?.quotas?.resetDate ?? null;
                  console.log('[AuthInjector] Entitlement: used=' + used + ', limit=' + limit + ', resetDate=' + resetDate);
                  return { success: true, used, limit, resetDate };
                } catch (error) {
                  console.error('[AuthInjector] Entitlement fetch error:', error);
                  return { success: false, error: error.message };
                }
              }

              async function extractAndSend() {
                console.log('[AuthInjector] Running extractAndSend...');
                const result = await extractCustomerId();
                if (result.success && result.id) {
                  console.log('[AuthInjector] Extraction success, ID:', result.id, 'fetching usage data...');

                  const usageCard = await fetchUsageCard(result.id);
                  const usageTable = await fetchUsageTable(result.id);
                  const entitlement = await fetchEntitlement();

                  console.log('[AuthInjector] Creating payload...');
                  const payload = {
                      id: result.id,
                      usageCard: usageCard,
                      usageTable: usageTable,
                      entitlement: entitlement
                  };

                  console.log('[AuthInjector] Redirecting with payload...');
                  const hash = encodeURIComponent(JSON.stringify(payload));
                  window.location.href = "https://copilot-auth-success.local/success#payload=" + hash;
                } else {
                  console.error('[AuthInjector] Failed to extract customer ID:', result.error);
                  // Emit event to notify renderer of extraction failure
                  if (window.__TAURI__?.event) {
                    window.__TAURI__.event.emit('auth:extraction-failed', {
                      error: result.error || 'Unknown extraction error'
                    });
                  }
                }
              }
            })();
        "#)
        .build()
        .map_err(|e| format!("Failed to create auth window: {}", e))?;

        self.auth_window = Some(window);
        Ok(())
    }

    /// Hide the auth window
    pub fn hide_auth_window(&mut self) {
        if let Some(window) = &self.auth_window {
            if window.is_visible().unwrap_or(false) {
                let _ = window.close();
            }
        }
        self.clear_auth_window();
    }

    pub fn clear_auth_window(&mut self) {
        self.auth_window = None;
        self.auth_window_listener_attached = false;
        self.visible_auth_generation.store(0, Ordering::SeqCst);
    }

    pub fn mark_auth_window_listener_attached(&mut self) -> bool {
        if self.auth_window_listener_attached {
            false
        } else {
            self.auth_window_listener_attached = true;
            true
        }
    }

    pub fn start_extraction(&mut self) -> bool {
        if self.extraction_in_progress {
            false
        } else {
            self.extraction_in_progress = true;
            true
        }
    }

    pub fn finish_extraction(&mut self) {
        self.extraction_in_progress = false;
    }

    /// Create a hidden webview for data extraction
    /// Uses an off-screen visible window to avoid macOS throttling
    /// On Windows, uses a tiny transparent window since off-screen positioning may not work
    pub fn create_hidden_webview(
        &mut self,
        app: &AppHandle,
    ) -> Result<tauri::WebviewWindow, String> {
        let url =
            Url::parse(GITHUB_BILLING_URL).map_err(|e| format!("Failed to parse URL: {}", e))?;

        let builder = WebviewWindowBuilder::new(app, "hidden-auth", WebviewUrl::External(url))
            .title("Hidden Auth");

        // Platform-specific configuration
        #[cfg(target_os = "windows")]
        let builder = builder
            .skip_taskbar(true)
            .inner_size(1.0, 1.0)
            // Position far off-screen (-32000, -32000) to ensure window is completely hidden
            // Windows coordinates at 0,0 would still be visible on screen, so we use extreme negative values
            .position(-32000.0, -32000.0)
            .transparent(true)
            .decorations(false)
            .visible(true);

        #[cfg(target_os = "macos")]
        let builder = builder
            .skip_taskbar(true)
            .inner_size(10.0, 10.0)
            .position(-100.0, -100.0)
            .visible(true);

        #[cfg(target_os = "linux")]
        let builder = builder
            .inner_size(10.0, 10.0)
            .position(-100.0, -100.0)
            .visible(true);

        let window = builder
        .initialization_script(r#"
            (function() {
              console.log('[HiddenAuth] Script initialized');

              async function sendResult(kind, payload) {
                try {
                  // Tauri v2 event emission via core invoke
                  if (window.__TAURI__ && window.__TAURI__.core) {
                    await window.__TAURI__.core.invoke('hidden_webview_event', {
                      event: kind,
                      payload: JSON.stringify(payload)
                    });
                    console.log('[HiddenAuth] Sent event:', kind);
                  } else {
                    console.error('[HiddenAuth] Tauri not available');
                    // Fallback: store in localStorage for parent window to pick up
                    localStorage.setItem('tauri_hidden_webview_' + kind, JSON.stringify(payload));
                  }
                } catch (e) {
                  console.error('[HiddenAuth] Failed to send:', e);
                }
              }

              async function getUserId() {
                try {
                  const response = await fetch('/api/v3/user', {
                    headers: { 'Accept': 'application/json' }
                  });
                  if (!response.ok) {
                    return { success: false, error: 'API request failed: ' + response.status };
                  }
                  const data = await response.json();
                  return { success: true, id: data.id };
                } catch (error) {
                  return { success: false, error: error.message };
                }
              }

              function getCustomerIdFromDOM() {
                try {
                  const el = document.querySelector('script[data-target="react-app.embeddedData"]');
                  if (!el) {
                    return { success: false, error: 'Embedded data element not found' };
                  }
                  const data = JSON.parse(el.textContent);
                  const customerId = data?.payload?.customer?.customerId;
                  if (!customerId) {
                    return { success: false, error: 'Customer ID not found in embedded data' };
                  }
                  return { success: true, id: customerId };
                } catch (error) {
                  return { success: false, error: error.message };
                }
              }

              function getCustomerIdFromHTML() {
                try {
                  const html = document.body.innerHTML;
                  const patterns = [
                    /customerId":(\d+)/,
                    /customerId&quot;:(\d+)/,
                    /customer_id=(\d+)/,
                    /"customerId":(\d+)/,
                    /data-customer-id=\"(\d+)\"/
                  ];
                  for (const pattern of patterns) {
                    const match = html.match(pattern);
                    if (match && match[1]) {
                      return { success: true, id: parseInt(match[1]) };
                    }
                  }
                  return { success: false, error: 'No customer ID pattern matched' };
                } catch (error) {
                  return { success: false, error: error.message };
                }
              }

              async function extractCustomerId() {
                let result = await getUserId();
                if (!result.success) {
                  result = getCustomerIdFromDOM();
                }
                if (!result.success) {
                  result = getCustomerIdFromHTML();
                }
                return result;
              }

              async function fetchUsageCard(customerId) {
                try {
                  const res = await fetch(`/settings/billing/copilot_usage_card?customer_id=${customerId}&period=3`, {
                    headers: {
                      'Accept': 'application/json',
                      'x-requested-with': 'XMLHttpRequest'
                    }
                  });
                  if (!res.ok) {
                    return { success: false, error: 'Usage card request failed: ' + res.status };
                  }
                  const data = await res.json();
                  return { success: true, data };
                } catch (error) {
                  return { success: false, error: error.message };
                }
              }

              async function fetchUsageTablePage(customerId, page) {
                const res = await fetch(`/settings/billing/copilot_usage_table?customer_id=${customerId}&group=0&period=3&query=&page=${page}`, {
                  headers: {
                    'Accept': 'application/json',
                    'x-requested-with': 'XMLHttpRequest'
                  }
                });

                if (!res.ok) {
                  return { success: false, error: 'Usage table request failed: ' + res.status };
                }

                const data = await res.json();
                return { success: true, data };
              }

              function getUsageTableTotalPages(data) {
                const candidates = [
                  data?.table?.pagination?.totalPages,
                  data?.table?.pagination?.total_pages,
                  data?.pagination?.totalPages,
                  data?.pagination?.total_pages,
                  data?.table?.totalPages,
                  data?.table?.total_pages,
                  data?.totalPages,
                  data?.total_pages
                ];

                for (const value of candidates) {
                  const parsed = Number(value);
                  if (Number.isFinite(parsed) && parsed > 0) {
                    return parsed;
                  }
                }

                return null;
              }

              async function fetchUsageTable(customerId) {
                try {
                  // Wait for billing page hydration.
                  await new Promise(resolve => setTimeout(resolve, 3000));

                  const MAX_PAGES = 10;
                  const seenRowIds = new Set();
                  const mergedRows = [];
                  let totalPages = null;
                  let page = 1;
                  let baseData = null;

                  while (page <= (totalPages ?? MAX_PAGES)) {
                    const pageResult = await fetchUsageTablePage(customerId, page);
                    if (!pageResult.success) {
                      if (page === 1) {
                        return pageResult;
                      }
                      break;
                    }

                    if (!baseData) {
                      baseData = pageResult.data;
                    }

                    if (totalPages === null) {
                      totalPages = getUsageTableTotalPages(pageResult.data);
                    }

                    const pageRows = pageResult.data?.table?.rows || [];
                    if (pageRows.length === 0) {
                      break;
                    }

                    let added = 0;
                    for (const row of pageRows) {
                      const key = row?.id || JSON.stringify(row?.cells?.[0]?.value || row);
                      if (!seenRowIds.has(key)) {
                        seenRowIds.add(key);
                        mergedRows.push(row);
                        added += 1;
                      }
                    }

                    if (added === 0) {
                      break;
                    }

                    if (totalPages !== null && page >= totalPages) {
                      break;
                    }

                    page += 1;
                  }

                  const data = {
                    ...(baseData || {}),
                    table: {
                      ...((baseData && baseData.table) ? baseData.table : {}),
                      rows: mergedRows,
                    },
                  };

                  console.log('[HiddenAuth] Usage table merged rows:', mergedRows.length);
                  return { success: true, data };
                } catch (error) {
                  return { success: false, error: error.message };
                }
              }

              async function fetchEntitlement() {
                try {
                  const res = await fetch('/github-copilot/chat/entitlement', {
                    headers: { 'Accept': 'application/json' }
                  });
                  if (!res.ok) {
                    const body = await res.text().catch(() => '');
                    console.error('[HiddenAuth] Entitlement request failed:', res.status, body);
                    return { success: false, error: 'Entitlement request failed: ' + res.status + ' ' + body };
                  }
                  const data = await res.json();
                  console.log('[HiddenAuth] Entitlement raw response:', JSON.stringify(data));
                  const limit = data?.quotas?.limits?.premiumInteractions ?? 0;
                  const remaining = data?.quotas?.remaining?.premiumInteractions ?? 0;
                  const used = limit - remaining;
                  const resetDate = data?.quotas?.resetDate ?? null;
                  console.log('[HiddenAuth] Entitlement: used=' + used + ', limit=' + limit);
                  return { success: true, used, limit, resetDate };
                } catch (error) {
                  console.error('[HiddenAuth] Entitlement fetch error:', error);
                  return { success: false, error: error.message };
                }
              }

              async function runExtraction() {
                console.log('[HiddenAuth] Starting extraction...');

                // Always fetch entitlement first - works for all plan types
                // including enterprise where billing page yields no data
                const entitlement = await fetchEntitlement();
                console.log('[HiddenAuth] Entitlement result:', JSON.stringify(entitlement));

                const customerResult = await extractCustomerId();
                console.log('[HiddenAuth] Customer result:', customerResult);
                await sendResult('auth:extraction:customer', customerResult);

                if (!customerResult.success) {
                  // For enterprise users: customer ID may not be on billing page,
                  // but we can still report entitlement data using the GitHub user ID as a fallback key
                  console.log('[HiddenAuth] Customer ID extraction failed, attempting user ID fallback...');
                  const userResult = await getUserId();
                  if (userResult.success && entitlement.success && entitlement.limit > 0) {
                    console.log('[HiddenAuth] Using user ID as fallback with entitlement data');
                    await sendResult('auth:extraction:customer', { success: true, id: userResult.id });
                    await sendResult('auth:extraction:usage', {
                      customerId: userResult.id,
                      usageCard: { success: false, error: 'No billing page data' },
                      usageTable: { success: false, error: 'No billing page data' },
                      entitlement
                    });
                    await sendResult('auth:extraction:complete', { success: true });
                    return;
                  }
                  await sendResult('auth:extraction:complete', { success: false });
                  return;
                }

                console.log('[HiddenAuth] Fetching usage data...');
                const usageCard = await fetchUsageCard(customerResult.id);
                const usageTable = await fetchUsageTable(customerResult.id);

                await sendResult('auth:extraction:usage', {
                  customerId: customerResult.id,
                  usageCard,
                  usageTable,
                  entitlement
                });

                await sendResult('auth:extraction:complete', { success: true });
                console.log('[HiddenAuth] Extraction complete');
              }

              // Run extraction when page is ready
              if (document.readyState === 'complete') {
                setTimeout(runExtraction, 1500);
              } else {
                window.addEventListener('load', () => setTimeout(runExtraction, 1500));
              }
            })();
        "#)
        .build()
        .map_err(|e| format!("Failed to create hidden webview: {}", e))?;

        Ok(window)
    }

    /// Complete extraction flow using channel-based communication
    pub async fn perform_extraction(
        &mut self,
        app: &AppHandle,
    ) -> Result<ExtractionResult, String> {
        // Serialize concurrent extractions — second caller waits until first finishes
        let _extraction_guard = EXTRACTION_LOCK.lock().await;

        // Create event channel
        let (tx, mut rx) = mpsc::channel::<HiddenWebviewEvent>(10);

        // Store channel for command handler to use
        {
            let mut global_tx = HIDDEN_WEBVIEW_EVENTS.lock().await;
            *global_tx = Some(tx);
        }

        // Create hidden webview
        let window = self.create_hidden_webview(app)?;

        // Wait for extraction events
        let timeout = tokio::time::timeout(Duration::from_secs(EXTRACTION_TIMEOUT_SECS), async {
            let mut customer_id: Option<u64> = None;
            let mut usage_data: Option<UsageData> = None;
            let mut usage_history: Option<Vec<UsageHistoryRow>> = None;
            let mut raw_usage_payload: Option<serde_json::Value> = None;
            let mut error: Option<String> = None;

            while let Some(event) = rx.recv().await {
                log::info!("Received hidden webview event: {}", event.event);

                match event.event.as_str() {
                    "auth:extraction:customer" => {
                        if let Ok(result) =
                            serde_json::from_str::<serde_json::Value>(&event.payload)
                        {
                            Self::apply_customer_extraction_result(
                                &result,
                                &mut customer_id,
                                &mut error,
                            );
                        }
                    }
                    "auth:extraction:usage" => {
                        if let Ok(result) =
                            serde_json::from_str::<serde_json::Value>(&event.payload)
                        {
                            Self::apply_usage_extraction_result(
                                &result,
                                &mut customer_id,
                                &mut usage_data,
                                &mut usage_history,
                                &mut raw_usage_payload,
                                &mut error,
                            );
                        }
                    }
                    "auth:extraction:complete" => {
                        // Extraction is complete, break the loop
                        break;
                    }
                    _ => {}
                }
            }

            ExtractionResult {
                customer_id,
                usage_data,
                usage_history,
                raw_usage_payload,
                error,
            }
        })
        .await;

        // Clean up
        let _ = window.close();

        // Clear the global channel
        {
            let mut global_tx = HIDDEN_WEBVIEW_EVENTS.lock().await;
            *global_tx = None;
        }

        match timeout {
            Ok(result) => Ok(result),
            Err(_) => Ok(ExtractionResult {
                customer_id: None,
                usage_data: None,
                usage_history: None,
                raw_usage_payload: None,
                error: Some("Extraction timed out".to_string()),
            }),
        }
    }

    pub fn get_customer_id(&self) -> Option<u64> {
        self.customer_id
    }

    pub fn set_customer_id(&mut self, id: u64) {
        self.customer_id = Some(id);
    }

    pub fn is_authenticated(&self) -> bool {
        self.customer_id.is_some()
    }

    /// Fetch updates via webview using a simpler approach
    /// This creates a temporary webview that navigates and injects JavaScript
    pub async fn fetch_github_releases(
        &mut self,
        app: &AppHandle,
    ) -> Result<serde_json::Value, String> {
        let _update_check_guard = UPDATE_CHECK_LOCK.lock().await;

        // Create event channel
        let (tx, mut rx) = mpsc::channel::<HiddenWebviewEvent>(10);

        // Store channel for command handler to use
        {
            let mut global_tx = UPDATE_CHECK_WEBVIEW_EVENTS.lock().await;
            *global_tx = Some(tx);
        }

        // Create temporary hidden webview
        let url = Url::parse("https://api.github.com")
            .map_err(|e| format!("Failed to parse URL: {}", e))?;

        let js_code = format!(
            r#"
            (async function() {{
                async function sendResult(kind, payload) {{
                    try {{
                        // Tauri v2 event emission via core invoke
                        if (window.__TAURI__ && window.__TAURI__.core) {{
                            await window.__TAURI__.core.invoke('hidden_webview_event', {{
                                event: kind,
                                payload: JSON.stringify(payload)
                            }});
                            console.log('[UpdateCheck] Sent event:', kind);
                        }} else {{
                            console.error('[UpdateCheck] Tauri not available');
                        }}
                    }} catch (e) {{
                        console.error('[UpdateCheck] Failed to send:', e);
                    }}
                }}

                try {{
                    // Do NOT set a manual User-Agent header in the injected JS — browsers forbid it.
                    // Let the WebView provide a normal User-Agent and request the GitHub API.
                    const response = await fetch('{}', {{
                        headers: {{
                            'Accept': 'application/vnd.github.v3+json'
                        }},
                        cache: 'no-store'
                    }});

                    if (!response.ok) {{
                        console.error('Update check failed: HTTP ' + response.status);
                        await sendResult('update_check:error', {{ success: false, error: 'HTTP ' + response.status }});
                        return;
                    }}

                    const data = await response.json();
                    console.log('Update check success:', data);

                    // Send result back via Tauri event
                    await sendResult('update_check:complete', {{ success: true, data: data }});
                }} catch (error) {{
                    // Include stack where available to aid Windows debugging
                    console.error('Update check error:', error);
                    await sendResult('update_check:error', {{ success: false, error: error.message || error.toString(), stack: error.stack || null }});
                }}
            }})()
        "#,
            GITHUB_API_URL
        );

        // Create minimal hidden webview
        let builder =
            WebviewWindowBuilder::new(app, "update-check-temp", WebviewUrl::External(url))
                .title("Update Check Temp")
                .skip_taskbar(true)
                .inner_size(1.0, 1.0)
                .initialization_script(js_code);

        // On Windows make the tiny webview visible (1x1 transparent off-screen) so JS will run reliably.
        #[cfg(target_os = "windows")]
        let window = builder
            .position(-32000.0, -32000.0)
            .transparent(true)
            .decorations(false)
            .visible(true)
            .build()
            .map_err(|e| format!("Failed to create update check webview: {}", e))?;

        // Non-Windows platforms can use a non-visible off-screen webview
        #[cfg(not(target_os = "windows"))]
        let window = builder
            .position(-100.0, -100.0)
            .visible(false)
            .build()
            .map_err(|e| format!("Failed to create update check webview: {}", e))?;

        // Wait for update_check:complete event with timeout
        let timeout_duration = Duration::from_secs(15);
        let result = tokio::time::timeout(timeout_duration, async {
            while let Some(event) = rx.recv().await {
                log::info!("Received update check event: {}", event.event);
                log::info!("Received update check event payload: {}", event.payload);

                if event.event == "update_check:complete" {
                    return serde_json::from_str::<serde_json::Value>(&event.payload)
                        .map_err(|e| format!("Failed to parse update check result: {}", e));
                } else if event.event == "update_check:error" {
                    // Parse error from payload
                    let error_payload =
                        serde_json::from_str::<serde_json::Value>(&event.payload)
                            .map_err(|e| format!("Failed to parse error payload: {}", e))?;

                    let error_msg = error_payload
                        .get("error")
                        .and_then(|e| e.as_str())
                        .unwrap_or("Unknown error")
                        .to_string();

                    return Err::<serde_json::Value, String>(error_msg);
                }
            }

            // If loop completes without result event, it timed out
            Err::<serde_json::Value, String>("Update check timed out".to_string())
        })
        .await;

        // Clean up window
        let _ = window.close();
        {
            let mut global_tx = UPDATE_CHECK_WEBVIEW_EVENTS.lock().await;
            *global_tx = None;
        }

        match result {
            Ok(result) => result,
            Err(_) => Err("Update check timed out".to_string()),
        }
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Command handler for hidden webview events
/// This receives data from the injected JavaScript in the hidden webview
#[tauri::command]
pub async fn hidden_webview_event(event: String, payload: String) -> Result<(), String> {
    let channel = if event.starts_with("update_check:") {
        &UPDATE_CHECK_WEBVIEW_EVENTS
    } else {
        &HIDDEN_WEBVIEW_EVENTS
    };
    let sender = channel.lock().await;
    if let Some(tx) = sender.as_ref() {
        let _ = tx.send(HiddenWebviewEvent { event, payload }).await;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::time::timeout;

    #[test]
    fn customer_success_clears_stale_extraction_error() {
        let mut customer_id = None;
        let mut error = None;

        AuthManager::apply_customer_extraction_result(
            &serde_json::json!({
                "success": false,
                "error": "customer id not found",
            }),
            &mut customer_id,
            &mut error,
        );

        assert_eq!(customer_id, None);
        assert_eq!(error.as_deref(), Some("customer id not found"));

        AuthManager::apply_customer_extraction_result(
            &serde_json::json!({
                "success": true,
                "id": 42,
            }),
            &mut customer_id,
            &mut error,
        );

        assert_eq!(customer_id, Some(42));
        assert_eq!(error, None);
    }

    #[test]
    fn visible_auth_results_are_rejected_after_session_generation_changes() {
        let tmp = TempDir::new().unwrap();
        let store = StoreManager::new(tmp.path().to_path_buf()).unwrap();
        let generation = store.get_session_generation();

        assert!(AuthManager::can_apply_visible_auth_result(
            &store, generation
        ));

        store.begin_session_transition();
        assert!(!AuthManager::can_apply_visible_auth_result(
            &store, generation
        ));

        store.finish_session_transition();
        assert!(!AuthManager::can_apply_visible_auth_result(
            &store, generation,
        ));
    }

    #[test]
    fn usage_payload_backfills_customer_id_when_customer_event_is_missing() {
        let mut customer_id = None;
        let mut error = None;
        let mut usage_data = None;
        let mut usage_history = None;
        let mut raw_usage_payload = None;

        AuthManager::apply_usage_extraction_result(
            &serde_json::json!({
                "customerId": 77,
                "usageCard": {
                    "data": {
                        "discountQuantity": 12,
                        "userPremiumRequestEntitlement": 1200,
                    }
                },
                "usageTable": {
                    "data": {
                        "table": {
                            "rows": []
                        }
                    }
                },
                "entitlement": {
                    "success": true,
                    "used": 12,
                    "limit": 1200
                }
            }),
            &mut customer_id,
            &mut usage_data,
            &mut usage_history,
            &mut raw_usage_payload,
            &mut error,
        );

        assert_eq!(customer_id, Some(77));
        assert!(usage_data.is_some());
        assert_eq!(usage_history.as_ref().map(Vec::len), Some(0));
        assert!(raw_usage_payload.is_some());
    }

    #[test]
    fn usage_payload_clears_stale_error_when_it_backfills_customer_id() {
        let mut customer_id = None;
        let mut error = None;
        let mut usage_data = None;
        let mut usage_history = None;
        let mut raw_usage_payload = None;

        AuthManager::apply_customer_extraction_result(
            &serde_json::json!({
                "success": false,
                "error": "customer id not found",
            }),
            &mut customer_id,
            &mut error,
        );

        AuthManager::apply_usage_extraction_result(
            &serde_json::json!({
                "customerId": 77,
                "usageCard": {
                    "data": {
                        "discountQuantity": 12,
                        "userPremiumRequestEntitlement": 1200,
                    }
                },
                "usageTable": {
                    "data": {
                        "table": {
                            "rows": []
                        }
                    }
                },
                "entitlement": {
                    "success": true,
                    "used": 12,
                    "limit": 1200
                }
            }),
            &mut customer_id,
            &mut usage_data,
            &mut usage_history,
            &mut raw_usage_payload,
            &mut error,
        );

        assert_eq!(customer_id, Some(77));
        assert!(usage_data.is_some());
        assert_eq!(error, None);
    }

    #[tokio::test]
    async fn hidden_webview_event_routes_update_events_to_update_channel() {
        let (extraction_tx, mut extraction_rx) = mpsc::channel::<HiddenWebviewEvent>(1);
        let (update_tx, mut update_rx) = mpsc::channel::<HiddenWebviewEvent>(1);
        {
            let mut sender = HIDDEN_WEBVIEW_EVENTS.lock().await;
            *sender = Some(extraction_tx);
        }
        {
            let mut sender = UPDATE_CHECK_WEBVIEW_EVENTS.lock().await;
            *sender = Some(update_tx);
        }

        hidden_webview_event("update_check:complete".to_string(), "{}".to_string())
            .await
            .unwrap();

        let received = timeout(Duration::from_millis(50), extraction_rx.recv()).await;
        assert!(
            received.is_err() || received.unwrap().is_none(),
            "update events must not be delivered to the extraction channel"
        );
        let update_event = timeout(Duration::from_millis(50), update_rx.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(update_event.event, "update_check:complete");

        let mut sender = HIDDEN_WEBVIEW_EVENTS.lock().await;
        *sender = None;
        let mut sender = UPDATE_CHECK_WEBVIEW_EVENTS.lock().await;
        *sender = None;
    }
}
