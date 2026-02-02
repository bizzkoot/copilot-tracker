use serde::{Deserialize, Serialize};
use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder};
use tokio::time::{sleep, Duration};
use url::Url;

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
    pub error: Option<String>,
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
}

impl AuthManager {
    pub fn new() -> Self {
        Self {
            auth_window: None,
            customer_id: None,
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

        let window = WebviewWindowBuilder::new(
            app,
            "auth",
            WebviewUrl::External(url),
        )
        .title("GitHub Login")
        .inner_size(900.0, 700.0)
        .resizable(true)
        .visible(true)
        .initialization_script(r#"
            // Poll for URL changes to detect successful login
            setInterval(() => {
                try {
                    const url = window.location.href;
                    if (url.includes('/settings/billing')) {
                        if (window.__TAURI__) {
                            window.__TAURI__.core.invoke('handle_auth_redirect');
                        }
                    }
                } catch (e) {
                    console.error('Auth check error:', e);
                }
            }, 500);
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
        self.auth_window = None;
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
        .build()
        .map_err(|e| format!("Failed to create hidden webview: {}", e))?;

        Ok(window)
    }

    /// Extract customer ID from the billing page
    pub async fn extract_customer_id(
        &self,
        _window: &tauri::WebviewWindow,
    ) -> Result<Option<u64>, String> {
        // NOTE: Tauri's eval() doesn't return values like Electron's executeJavaScript
        // For a production implementation, we need to use one of these approaches:
        //
        // 1. Use a custom protocol to communicate from JS to Rust
        // 2. Inject a script that emits events back to Rust via window.__TAURI__.core.emit
        // 3. Use a hidden window with URL-based communication
        // 4. Scrap the page HTML from Rust using webview's get_contents()
        //
        // For now, this is a placeholder that returns a mock customer ID
        // to demonstrate the architecture. In production, you must implement
        // one of the above approaches.

        log::warn!("Customer ID extraction not yet fully implemented - returning mock value");

        // Return a mock customer ID for testing the flow
        // In production, this must be replaced with actual extraction logic
        Ok(Some(12345678))
    }

    /// Extract usage data from the billing page
    pub async fn extract_usage_data(
        &self,
        _window: &tauri::WebviewWindow,
    ) -> Result<Option<UsageData>, String> {
        // NOTE: Same issue as extract_customer_id - Tauri's eval() doesn't return values
        // This is a placeholder for testing the architecture.
        //
        // Production implementation requires one of:
        // 1. Custom protocol for JS->Rust communication
        // 2. Event-based communication via window.__TAURI__
        // 3. HTML scraping from Rust
        // 4. URL-based IPC

        log::warn!("Usage data extraction not yet fully implemented - returning mock value");

        // Return mock usage data for testing
        Ok(Some(UsageData {
            net_billed_amount: 0.0,
            net_quantity: 0,
            discount_quantity: 0,
            user_premium_request_entitlement: 0,
            filtered_user_premium_request_entitlement: 0,
        }))
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
                    error: Some(format!("Customer ID extraction failed: {}", e)),
                });
            }
        };

        // Extract usage data
        let usage_data = match self.extract_usage_data(&window).await {
            Ok(data) => data,
            Err(e) => {
                let _ = window.close();
                return Ok(ExtractionResult {
                    customer_id,
                    usage_data: None,
                    error: Some(format!("Usage extraction failed: {}", e)),
                });
            }
        };

        // Clean up
        let _ = window.close();

        Ok(ExtractionResult {
            customer_id,
            usage_data,
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
