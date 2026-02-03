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

            // Check for custom protocol redirect
            if url.scheme() == "copilot-tracker" {
                if let Some((_, id_str)) = url.query_pairs().find(|(key, _)| key == "id") {
                    if let Ok(id) = id_str.parse::<u64>() {
                         let store = app_handle.state::<StoreManager>();
                         if store.set_customer_id(id).is_ok() {
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
                         }
                    }
                }
                return false;
            }

            if url_str.contains("/settings/billing") {
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
              // Monitor URL changes for billing page detection
              let currentUrl = location.href;
              
              function checkUrl() {
                const newUrl = location.href;
                if (newUrl !== currentUrl) {
                  currentUrl = newUrl;
                  if (currentUrl.includes('/settings/billing')) {
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
              
              // Check immediately if already on billing page
              if (location.href.includes('/settings/billing')) {
                setTimeout(extractAndSend, 1500);
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
              
              async function extractAndSend() {
                const result = await extractCustomerId();
                if (result.success && result.id) {
                  window.location.href = "copilot-tracker://success?id=" + result.id;
                } else {
                  console.error('Failed to extract customer ID:', result.error);
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
