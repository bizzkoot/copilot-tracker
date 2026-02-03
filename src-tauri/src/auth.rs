use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Listener, Manager, WebviewUrl, WebviewWindowBuilder};
use tokio::time::{sleep, Duration};
use url::Url;

use crate::StoreManager;

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
                    if fragment.starts_with("payload=") {
                        let encoded = &fragment["payload=".len()..];
                        if let Ok(decoded) = urlencoding::decode(encoded) {
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&decoded) {
                                // Extract ID
                                if let Some(id) = json.get("id").and_then(|v| v.as_u64()) {
                                    extracted_id = Some(id);
                                    
                                    // Extract Usage Data
                                    if let Some(usage_card) = json.get("usageCard").and_then(|v| v.get("data")) {
                                        extracted_usage_data = Some(UsageData {
                                            net_billed_amount: usage_card.get("net_billed_amount").and_then(|v| v.as_f64()).unwrap_or(0.0),
                                            net_quantity: usage_card.get("net_quantity").and_then(|v| v.as_u64()).unwrap_or(0),
                                            discount_quantity: usage_card.get("discount_quantity").and_then(|v| v.as_u64()).unwrap_or(0),
                                            user_premium_request_entitlement: usage_card.get("user_premium_request_entitlement").and_then(|v| v.as_u64()).unwrap_or(0),
                                            filtered_user_premium_request_entitlement: usage_card.get("filtered_user_premium_request_entitlement").and_then(|v| v.as_u64()).unwrap_or(0),
                                        });
                                    }

                                    // Extract Usage History
                                    if let Some(rows) = json.get("usageTable")
                                        .and_then(|v| v.get("data"))
                                        .and_then(|v| v.get("rows"))
                                        .and_then(|v| v.as_array()) 
                                    {
                                        let history: Vec<UsageHistoryRow> = rows.iter().filter_map(|row| {
                                            Some(UsageHistoryRow {
                                                date: row.get("date").and_then(|v| v.as_str())?.to_string(),
                                                included_requests: row.get("included_requests").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                                                billed_requests: row.get("billed_requests").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                                                gross_amount: row.get("gross_amount").and_then(|v| v.as_f64()).unwrap_or(0.0),
                                                billed_amount: row.get("billed_amount").and_then(|v| v.as_f64()).unwrap_or(0.0),
                                            })
                                        }).collect();
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
                         
                         // Save usage data
                         if let Some(usage) = extracted_usage_data {
                             let used = usage.discount_quantity as u32;
                             let limit = usage.user_premium_request_entitlement as u32;
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
                             
                             // Emit summary
                             let remaining = limit.saturating_sub(used);
                             let percentage = if limit > 0 { (used as f32 / limit as f32) * 100.0 } else { 0.0 };
                             let summary = crate::usage::UsageSummary {
                                 used,
                                 limit,
                                 remaining,
                                 percentage,
                                 timestamp: chrono::Utc::now().timestamp(),
                             };
                             let _ = app_handle.emit("usage:updated", &summary);
                         }

                         // Save history
                         if let Some(rows) = extracted_usage_history {
                             let entries = crate::usage::UsageManager::map_history_rows(&rows);
                             store.set_usage_history(entries);
                         }

                         let _ = app_handle.emit("auth:state-changed", "authenticated");
                         
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
              
              async function extractAndSend() {
                console.log('[AuthInjector] Running extractAndSend...');
                const result = await extractCustomerId();
                if (result.success && result.id) {
                  console.log('[AuthInjector] Extraction success, fetching usage data...');
                  
                  const usageCard = await fetchUsageCard(result.id);
                  const usageTable = await fetchUsageTable(result.id);
                  
                  const payload = {
                      id: result.id,
                      usageCard: usageCard,
                      usageTable: usageTable
                  };
                  
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
    pub fn create_hidden_webview(
        &mut self,
        app: &AppHandle,
    ) -> Result<tauri::WebviewWindow, String> {
        let url = Url::parse(GITHUB_BILLING_URL)
            .map_err(|e| format!("Failed to parse URL: {}", e))?;

        let window = WebviewWindowBuilder::new(
            app,
            "hidden-auth",
            WebviewUrl::External(url),
        )
        .title("Hidden Auth")
        .inner_size(900.0, 700.0)
        .visible(false)
        .initialization_script(r#"
            (function() {
              async function sendResult(kind, payload) {
                try {
                  if (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.emit) {
                    await window.__TAURI__.core.emit(kind, payload);
                  }
                } catch (e) {
                  console.error('emit failed', e);
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
                    /data-customer-id="(\d+)"/
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
                const customerResult = await extractCustomerId();
                await sendResult('auth:extraction:customer', customerResult);

                if (!customerResult.success) return;

                const usageCard = await fetchUsageCard(customerResult.id);
                const usageTable = await fetchUsageTable(customerResult.id);
                await sendResult('auth:extraction:usage', { usageCard, usageTable });
              }

              setTimeout(runExtraction, 1500);
            })();
        "#)
        .build()
        .map_err(|e| format!("Failed to create hidden webview: {}", e))?;

        Ok(window)
    }

    /// Extract customer ID from the billing page
    pub async fn extract_customer_id(
        &self,
        window: &tauri::WebviewWindow,
    ) -> Result<Option<u64>, String> {
        let (tx, rx) = tokio::sync::oneshot::channel::<Option<u64>>();
        window.once("auth:extraction:customer", move |event| {
            let payload = event.payload();
            let result: serde_json::Value = match serde_json::from_str(payload) {
                Ok(value) => value,
                Err(_) => return,
            };
            let success = result.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
            if !success {
                let _ = tx.send(None);
                return;
            }
            let id = result.get("id").and_then(|v| v.as_u64());
            let _ = tx.send(id);
        });

        let result = tokio::time::timeout(Duration::from_secs(10), rx)
            .await
            .map_err(|_| "Timed out waiting for customer ID".to_string())?
            .unwrap_or(None);

        Ok(result)
    }

    /// Extract usage data from the billing page
    pub async fn extract_usage_data(
        &self,
        window: &tauri::WebviewWindow,
    ) -> Result<(Option<UsageData>, Option<Vec<UsageHistoryRow>>), String> {
        let (tx, rx) = tokio::sync::oneshot::channel::<(Option<UsageData>, Option<Vec<UsageHistoryRow>>)>();
        window.once("auth:extraction:usage", move |event| {
            let payload = event.payload();
            let result: serde_json::Value = match serde_json::from_str(payload) {
                Ok(value) => value,
                Err(_) => return,
            };

            let usage_card = result.get("usageCard");
            let usage_card_data = usage_card.and_then(|v| v.get("data"));
            let usage = usage_card_data.map(|data| UsageData {
                net_billed_amount: data.get("net_billed_amount").and_then(|v| v.as_f64()).unwrap_or(0.0),
                net_quantity: data.get("net_quantity").and_then(|v| v.as_u64()).unwrap_or(0),
                discount_quantity: data.get("discount_quantity").and_then(|v| v.as_u64()).unwrap_or(0),
                user_premium_request_entitlement: data.get("user_premium_request_entitlement").and_then(|v| v.as_u64()).unwrap_or(0),
                filtered_user_premium_request_entitlement: data.get("filtered_user_premium_request_entitlement").and_then(|v| v.as_u64()).unwrap_or(0),
            });

            let history_rows = result
                .get("usageTable")
                .and_then(|v| v.get("data"))
                .and_then(|v| v.get("rows"))
                .and_then(|v| v.as_array())
                .map(|rows| {
                    rows.iter()
                        .filter_map(|row| {
                            let date = row.get("date").and_then(|v| v.as_str())?.to_string();
                            let included_requests = row
                                .get("included_requests")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0) as u32;
                            let billed_requests = row
                                .get("billed_requests")
                                .and_then(|v| v.as_u64())
                                .unwrap_or(0) as u32;
                            let gross_amount = row
                                .get("gross_amount")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.0);
                            let billed_amount = row
                                .get("billed_amount")
                                .and_then(|v| v.as_f64())
                                .unwrap_or(0.0);

                            Some(UsageHistoryRow {
                                date,
                                included_requests,
                                billed_requests,
                                gross_amount,
                                billed_amount,
                            })
                        })
                        .collect::<Vec<_>>()
                });

            let _ = tx.send((usage, history_rows));
        });

        let result = tokio::time::timeout(Duration::from_secs(10), rx)
            .await
            .map_err(|_| "Timed out waiting for usage data".to_string())?
            .unwrap_or((None, None));

        Ok(result)
    }

    /// Complete extraction flow: get customer ID and usage data
    pub async fn perform_extraction(
        &mut self,
        app: &AppHandle,
    ) -> Result<ExtractionResult, String> {
        // Create hidden webview
        let window = self.create_hidden_webview(app)?;

        // Wait for page load
        sleep(Duration::from_secs(3)).await;

        // Extract customer ID
        let customer_id = match self.extract_customer_id(&window).await {
            Ok(id) => id,
            Err(e) => {
                let _ = window.close();
                return Ok(ExtractionResult {
                    customer_id: None,
                    usage_data: None,
                    usage_history: None,
                    error: Some(format!("Customer ID extraction failed: {}", e)),
                });
            }
        };

        // Extract usage data
        let (usage_data, usage_history) = match self.extract_usage_data(&window).await {
            Ok(data) => data,
            Err(e) => {
                let _ = window.close();
                return Ok(ExtractionResult {
                    customer_id,
                    usage_data: None,
                    usage_history: None,
                    error: Some(format!("Usage extraction failed: {}", e)),
                });
            }
        };

        // Clean up
        let _ = window.close();

        Ok(ExtractionResult {
            customer_id,
            usage_data,
            usage_history,
            error: None,
        })
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
