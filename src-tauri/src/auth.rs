use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};
use tokio::sync::{mpsc, Mutex as TokioMutex};
use tokio::time::Duration;
use url::Url;

use crate::StoreManager;

/// Global channel for hidden webview events
static HIDDEN_WEBVIEW_EVENTS: TokioMutex<Option<mpsc::Sender<HiddenWebviewEvent>>> = TokioMutex::const_new(None);

#[derive(Debug, Clone)]
pub struct HiddenWebviewEvent {
    pub event: String,
    pub payload: String,
}

const GITHUB_BILLING_URL: &str = "https://github.com/settings/billing";
const GITHUB_LOGIN_URL: &str = "https://github.com/login";

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
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageHistoryRow {
    pub date: String,
    pub included_requests: u32,
    pub billed_requests: u32,
    pub gross_amount: f64,
    pub billed_amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageData {
    pub net_billed_amount: f64,
    pub net_quantity: u64,
    pub discount_quantity: u64,
    pub user_premium_request_entitlement: u64,
    pub filtered_user_premium_request_entitlement: u64,
}

#[derive(Clone)]
pub struct AuthManager {
    auth_window: Option<tauri::WebviewWindow>,
    customer_id: Option<u64>,
    extraction_in_progress: bool,
    auth_window_listener_attached: bool,
}

impl AuthManager {
    pub fn new() -> Self {
        Self {
            auth_window: None,
            customer_id: None,
            extraction_in_progress: false,
            auth_window_listener_attached: false,
        }
    }

    /// Create or show the auth webview window
    pub fn show_auth_window(&mut self, app: &AppHandle) -> Result<(), String> {
        // If window exists, just show it
        if let Some(window) = &self.auth_window {
            if window.is_visible().unwrap_or(false) {
                window.show()
                    .map_err(|e| format!("Failed to show window: {}", e))?;
                window.set_focus()
                    .map_err(|e| format!("Failed to focus window: {}", e))?;
                let url = Url::parse(GITHUB_LOGIN_URL)
                    .map_err(|e| format!("Failed to parse URL: {}", e))?;
                window.navigate(url)
                    .map_err(|e| format!("Failed to navigate: {}", e))?;
                return Ok(());
            }
        }

        // Create new auth window
        let url = Url::parse(GITHUB_LOGIN_URL)
            .map_err(|e| format!("Failed to parse URL: {}", e))?;

        let app_handle = app.clone();
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
                                    if let Some(usage_card) = json.get("usageCard").and_then(|v| v.get("data")) {
                                        log::info!("Raw usage card data: {:?}", usage_card);
                                        extracted_usage_data = Some(UsageData {
                                            net_billed_amount: usage_card.get("netBilledAmount").and_then(|v| v.as_f64()).unwrap_or(0.0),
                                            net_quantity: usage_card.get("netQuantity").and_then(|v| v.as_u64()).unwrap_or(0),
                                            discount_quantity: usage_card.get("discountQuantity").and_then(|v| v.as_u64()).unwrap_or(0),
                                            user_premium_request_entitlement: usage_card.get("userPremiumRequestEntitlement").and_then(|v| v.as_u64()).unwrap_or(0),
                                            filtered_user_premium_request_entitlement: usage_card.get("filteredUserPremiumRequestEntitlement").and_then(|v| v.as_u64()).unwrap_or(0),
                                        });
                                    }

                                    // Extract Usage History
                                    if let Some(rows) = json.get("usageTable")
                                        .and_then(|v| v.get("data"))
                                        .and_then(|v| v.get("table"))
                                        .and_then(|v| v.get("rows"))
                                        .and_then(|v| v.as_array()) 
                                    {
                                        log::info!("Parsing usage history, found {} rows", rows.len());
                                        let history: Vec<UsageHistoryRow> = rows.iter().filter_map(|row| {
                                            let id = row.get("id").and_then(|v| v.as_str())?.to_string();
                                            let cells = row.get("cells").and_then(|v| v.as_array())?;
                                            
                                            // Parse cells: [date, included_requests, billed_requests, gross_amount, billed_amount]
                                            if cells.len() < 5 {
                                                return None;
                                            }
                                            
                                            let included_requests = cells.get(1)?
                                                .get("value")?
                                                .as_str()?
                                                .parse::<u32>()
                                                .ok()?;
                                            
                                            let billed_requests = cells.get(2)?
                                                .get("value")?
                                                .as_str()?
                                                .parse::<u32>()
                                                .ok()?;
                                            
                                            let gross_amount = cells.get(3)?
                                                .get("value")?
                                                .as_str()?
                                                .trim_start_matches('$')
                                                .parse::<f64>()
                                                .ok()?;
                                            
                                            let billed_amount = cells.get(4)?
                                                .get("value")?
                                                .as_str()?
                                                .trim_start_matches('$')
                                                .parse::<f64>()
                                                .ok()?;
                                            
                                            Some(UsageHistoryRow {
                                                date: id,
                                                included_requests,
                                                billed_requests,
                                                gross_amount,
                                                billed_amount,
                                            })
                                        }).collect();
                                        
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
                     let store = app_handle.state::<StoreManager>();
                     if store.set_customer_id(id).is_ok() {
                         log::info!("Successfully authenticated with Customer ID: {}", id);
                         
                         // Save usage data and history
                          let mut usage_summary = None;
                          let mut usage_entries = vec![];

                          if let Some(usage) = extracted_usage_data {
                              log::info!("Extracted usage data: net_quantity={}, discount_quantity={}, entitlement={}", 
                                  usage.net_quantity, usage.discount_quantity, usage.user_premium_request_entitlement);
                              
                              let used = usage.discount_quantity as u32;
                              let limit = usage.user_premium_request_entitlement as u32;
                              
                              if used == 0 && limit == 0 {
                                  log::warn!("Usage data shows 0/0 - API may have returned empty data");
                              }
                              
                              let _ = store.set_usage(used, limit);

                              // Update cache
                              let cache = crate::store::UsageCache {
                                  customer_id: id,
                                  net_quantity: usage.net_quantity,
                                  discount_quantity: usage.discount_quantity,
                                  user_premium_request_entitlement: usage.user_premium_request_entitlement,
                                  filtered_user_premium_request_entitlement: usage.filtered_user_premium_request_entitlement,
                                  net_billed_amount: usage.net_billed_amount,
                                  timestamp: chrono::Utc::now().timestamp(),
                              };
                              store.set_usage_cache(cache);
                              
                              // Create summary
                              let remaining = limit.saturating_sub(used);
                              let percentage = if limit > 0 { (used as f32 / limit as f32) * 100.0 } else { 0.0 };
                              usage_summary = Some(crate::usage::UsageSummary {
                                  used,
                                  limit,
                                  remaining,
                                  percentage,
                                  timestamp: chrono::Utc::now().timestamp(),
                              });
                          } else {
                              log::warn!("No usage data was extracted from GitHub API");
                          }

                          // Save history
                          if let Some(rows) = extracted_usage_history {
                              log::info!("Extracted {} usage history rows", rows.len());
                              usage_entries = crate::usage::UsageManager::map_history_rows(&rows);
                              store.set_usage_history(usage_entries.clone());
                          } else {
                              log::warn!("No usage history was extracted from GitHub API");
                          }

                          // Emit full usage:data payload with prediction
                          if let Some(summary) = usage_summary {
                              let history = if !usage_entries.is_empty() {
                                  usage_entries
                              } else {
                                  crate::usage::UsageManager::get_cached_history(&app_handle)
                              };
                              
                              let prediction = crate::usage::UsageManager::predict_usage_from_history(
                                  &history,
                                  summary.used,
                                  summary.limit,
                              );
                              
                              log::info!("Emitting usage:data event - used: {}, limit: {}, history entries: {}", 
                                  summary.used, summary.limit, history.len());
                              
                              let payload = crate::usage::UsagePayload {
                                  summary: summary.clone(),
                                  history,
                                  prediction,
                              };
                              
                              let _ = app_handle.emit("usage:data", payload);
                              let _ = app_handle.emit("usage:updated", &summary);
                          } else {
                              log::warn!("No usage summary to emit - authentication succeeded but no usage data available");
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

              async function fetchUsageTable(customerId) {
                try {
                  console.log('[AuthInjector] Fetching usage table for customer:', customerId);
                  const res = await fetch(`/settings/billing/copilot_usage_table?customer_id=${customerId}&group=0&period=3&query=&page=1`, {
                    headers: {
                      'Accept': 'application/json',
                      'x-requested-with': 'XMLHttpRequest'
                    }
                  });
                  console.log('[AuthInjector] Usage table response status:', res.status);
                  if (!res.ok) {
                    console.error('[AuthInjector] Usage table request failed:', res.status);
                    return { success: false, error: 'Usage table request failed: ' + res.status };
                  }
                  const data = await res.json();
                  console.log('[AuthInjector] Usage table data received:', data ? 'YES' : 'NO', 'Rows:', data?.data?.rows?.length || 0);
                  return { success: true, data };
                } catch (error) {
                  console.error('[AuthInjector] Usage table fetch error:', error);
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
                  
                  console.log('[AuthInjector] Creating payload...');
                  const payload = {
                      id: result.id,
                      usageCard: usageCard,
                      usageTable: usageTable
                  };
                  
                  console.log('[AuthInjector] Redirecting with payload...');
                  const hash = encodeURIComponent(JSON.stringify(payload));
                  window.location.href = "https://copilot-auth-success.local/success#payload=" + hash;
                } else {
                  console.error('[AuthInjector] Failed to extract customer ID:', result.error);
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
        let url = Url::parse(GITHUB_BILLING_URL)
            .map_err(|e| format!("Failed to parse URL: {}", e))?;

        let builder = WebviewWindowBuilder::new(
            app,
            "hidden-auth",
            WebviewUrl::External(url),
        )
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

              async function fetchUsageTable(customerId) {
                try {
                  const res = await fetch(`/settings/billing/copilot_usage_table?customer_id=${customerId}&group=0&period=3&query=&page=1`, {
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
                } catch (error) {
                  return { success: false, error: error.message };
                }
              }

              async function runExtraction() {
                console.log('[HiddenAuth] Starting extraction...');
                const customerResult = await extractCustomerId();
                console.log('[HiddenAuth] Customer result:', customerResult);
                await sendResult('auth:extraction:customer', customerResult);

                if (!customerResult.success) {
                  await sendResult('auth:extraction:complete', { success: false });
                  return;
                }

                console.log('[HiddenAuth] Fetching usage data...');
                const usageCard = await fetchUsageCard(customerResult.id);
                const usageTable = await fetchUsageTable(customerResult.id);
                
                await sendResult('auth:extraction:usage', { 
                  customerId: customerResult.id,
                  usageCard, 
                  usageTable 
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
        let timeout = tokio::time::timeout(Duration::from_secs(30), async {
            let mut customer_id: Option<u64> = None;
            let mut usage_data: Option<UsageData> = None;
            let mut usage_history: Option<Vec<UsageHistoryRow>> = None;
            let mut error: Option<String> = None;

            while let Some(event) = rx.recv().await {
                log::info!("Received hidden webview event: {}", event.event);
                
                match event.event.as_str() {
                    "auth:extraction:customer" => {
                        if let Ok(result) = serde_json::from_str::<serde_json::Value>(&event.payload) {
                            if result.get("success").and_then(|v| v.as_bool()).unwrap_or(false) {
                                customer_id = result.get("id").and_then(|v| v.as_u64());
                            } else {
                                error = result.get("error").and_then(|v| v.as_str()).map(|s| s.to_string());
                            }
                        }
                    }
                    "auth:extraction:usage" => {
                        if let Ok(result) = serde_json::from_str::<serde_json::Value>(&event.payload) {
                            // Parse usage card
                            if let Some(usage_card) = result.get("usageCard").and_then(|v| v.get("data")) {
                                usage_data = Some(UsageData {
                                    net_billed_amount: usage_card.get("netBilledAmount").and_then(|v| v.as_f64()).unwrap_or(0.0),
                                    net_quantity: usage_card.get("netQuantity").and_then(|v| v.as_u64()).unwrap_or(0),
                                    discount_quantity: usage_card.get("discountQuantity").and_then(|v| v.as_u64()).unwrap_or(0),
                                    user_premium_request_entitlement: usage_card.get("userPremiumRequestEntitlement").and_then(|v| v.as_u64()).unwrap_or(0),
                                    filtered_user_premium_request_entitlement: usage_card.get("filteredUserPremiumRequestEntitlement").and_then(|v| v.as_u64()).unwrap_or(0),
                                });
                            }

                            // Parse usage table
                            if let Some(rows) = result
                                .get("usageTable")
                                .and_then(|v| v.get("data"))
                                .and_then(|v| v.get("table"))
                                .and_then(|v| v.get("rows"))
                                .and_then(|v| v.as_array()) 
                            {
                                let history: Vec<UsageHistoryRow> = rows.iter().filter_map(|row| {
                                    let id = row.get("id").and_then(|v| v.as_str())?.to_string();
                                    let cells = row.get("cells").and_then(|v| v.as_array())?;
                                    
                                    if cells.len() < 5 {
                                        return None;
                                    }
                                    
                                    let included_requests = cells.get(1)?
                                        .get("value")?
                                        .as_str()?
                                        .parse::<u32>()
                                        .ok()?;
                                    
                                    let billed_requests = cells.get(2)?
                                        .get("value")?
                                        .as_str()?
                                        .parse::<u32>()
                                        .ok()?;
                                    
                                    let gross_amount = cells.get(3)?
                                        .get("value")?
                                        .as_str()?
                                        .trim_start_matches('$')
                                        .parse::<f64>()
                                        .ok()?;
                                    
                                    let billed_amount = cells.get(4)?
                                        .get("value")?
                                        .as_str()?
                                        .trim_start_matches('$')
                                        .parse::<f64>()
                                        .ok()?;
                                    
                                    Some(UsageHistoryRow {
                                        date: id,
                                        included_requests,
                                        billed_requests,
                                        gross_amount,
                                        billed_amount,
                                    })
                                }).collect();
                                
                                usage_history = Some(history);
                            }
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
                error,
            }
        }).await;

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
    let sender = HIDDEN_WEBVIEW_EVENTS.lock().await;
    if let Some(tx) = sender.as_ref() {
        let _ = tx.send(HiddenWebviewEvent { event, payload }).await;
    }
    Ok(())
}
