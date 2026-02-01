/**
 * API Service
 * JavaScript injection scripts for GitHub billing API
 * Ported from StatusBarController.swift
 */

// GitHub billing page URLs
export const GITHUB_BILLING_URL =
  "https://github.com/settings/billing/premium_requests_usage";
export const GITHUB_LOGIN_URL = "https://github.com/login";

/**
 * JavaScript scripts to be injected into GitHub WebView
 * These are executed in the context of the authenticated GitHub session
 */
export const API_SCRIPTS = {
  /**
   * Method 1: Get user ID from GitHub API
   * Most reliable method when API is accessible
   */
  getUserId: `
    return await (async function() {
      try {
        const response = await fetch('/api/v3/user', {
          headers: { 'Accept': 'application/json' }
        });
        if (!response.ok) {
          return JSON.stringify({ success: false, error: 'API request failed: ' + response.status });
        }
        const data = await response.json();
        return JSON.stringify({ success: true, id: data.id });
      } catch (error) {
        return JSON.stringify({ success: false, error: error.message });
      }
    })()
  `,

  /**
   * Method 2: Extract customer ID from embedded React data
   * Fallback when API is not accessible
   */
  getCustomerIdFromDOM: `
    return (function() {
      try {
        const el = document.querySelector('script[data-target="react-app.embeddedData"]');
        if (!el) {
          return JSON.stringify({ success: false, error: 'Embedded data element not found' });
        }
        const data = JSON.parse(el.textContent);
        const customerId = data?.payload?.customer?.customerId;
        if (!customerId) {
          return JSON.stringify({ success: false, error: 'Customer ID not found in embedded data' });
        }
        return JSON.stringify({ success: true, id: customerId });
      } catch (error) {
        return JSON.stringify({ success: false, error: error.message });
      }
    })()
  `,

  /**
   * Method 3: Extract customer ID from HTML via regex patterns
   * Last resort fallback
   */
  getCustomerIdFromHTML: `
    return (function() {
      try {
        const html = document.body.innerHTML;
        const patterns = [
          /customerId":(\\d+)/,
          /customerId&quot;:(\\d+)/,
          /customer_id=(\\d+)/,
          /"customerId":(\\d+)/,
          /data-customer-id="(\\d+)"/
        ];
        for (const pattern of patterns) {
          const match = html.match(pattern);
          if (match && match[1]) {
            return JSON.stringify({ success: true, id: parseInt(match[1]) });
          }
        }
        return JSON.stringify({ success: false, error: 'No customer ID pattern matched' });
      } catch (error) {
        return JSON.stringify({ success: false, error: error.message });
      }
    })()
  `,

  /**
   * Fetch current usage from billing card API
   * @param customerId - The GitHub customer ID
   */
  getUsageCard: (customerId: number): string => `
    return await (async function() {
      try {
        const res = await fetch('/settings/billing/copilot_usage_card?customer_id=${customerId}&period=3', {
          headers: {
            'Accept': 'application/json',
            'x-requested-with': 'XMLHttpRequest'
          }
        });
        if (!res.ok) {
          return JSON.stringify({ success: false, error: 'Usage card request failed: ' + res.status });
        }
        const data = await res.json();
        return JSON.stringify({ success: true, data });
      } catch (error) {
        return JSON.stringify({ success: false, error: error.message });
      }
    })()
  `,

  /**
   * Fetch usage history from billing table API
   * @param customerId - The GitHub customer ID
   */
  getUsageTable: (customerId: number): string => `
    return await (async function() {
      try {
        const res = await fetch('/settings/billing/copilot_usage_table?customer_id=${customerId}&group=0&period=3&query=&page=1', {
          headers: {
            'Accept': 'application/json',
            'x-requested-with': 'XMLHttpRequest'
          }
        });
        if (!res.ok) {
          return JSON.stringify({ success: false, error: 'Usage table request failed: ' + res.status });
        }
        const data = await res.json();
        return JSON.stringify({ success: true, data });
      } catch (error) {
        return JSON.stringify({ success: false, error: error.message });
      }
    })()
  `,

  /**
   * Check if user is authenticated by looking for login elements
   */
  checkAuthState: `
    return (function() {
      try {
        // Check for login form
        const loginForm = document.querySelector('form[action="/session"]');
        if (loginForm) {
          return JSON.stringify({ authenticated: false, reason: 'Login form present' });
        }
        
        // Check for user avatar/menu (indicates logged in)
        const userMenu = document.querySelector('[data-target="user-navigation-frame"]') ||
                        document.querySelector('.Header-link--profile') ||
                        document.querySelector('[aria-label="Open user navigation menu"]');
        if (userMenu) {
          return JSON.stringify({ authenticated: true });
        }
        
        // Check URL
        if (window.location.pathname.includes('/login') || 
            window.location.pathname.includes('/session')) {
          return JSON.stringify({ authenticated: false, reason: 'On login page' });
        }
        
        // Default to assuming authenticated if on billing page
        if (window.location.pathname.includes('/settings/billing')) {
          return JSON.stringify({ authenticated: true });
        }
        
        return JSON.stringify({ authenticated: false, reason: 'Unknown state' });
      } catch (error) {
        return JSON.stringify({ authenticated: false, error: error.message });
      }
    })()
  `,
};

/**
 * Parse usage card response into CopilotUsage type
 */
export function parseUsageCardResponse(data: {
  net_billed_amount?: number;
  net_quantity?: number;
  discount_quantity?: number;
  user_premium_request_entitlement?: number;
  filtered_user_premium_request_entitlement?: number;
}) {
  return {
    netBilledAmount: data.net_billed_amount ?? 0,
    netQuantity: data.net_quantity ?? 0,
    discountQuantity: data.discount_quantity ?? 0,
    userPremiumRequestEntitlement: data.user_premium_request_entitlement ?? 0,
    filteredUserPremiumRequestEntitlement:
      data.filtered_user_premium_request_entitlement ?? 0,
  };
}

/**
 * Parse usage table response into UsageHistory type
 */
export function parseUsageTableResponse(data: {
  rows?: Array<{
    date: string;
    included_requests?: number;
    billed_requests?: number;
    gross_amount?: number;
    billed_amount?: number;
  }>;
}) {
  const days = (data.rows ?? []).map((row) => ({
    date: new Date(row.date),
    includedRequests: row.included_requests ?? 0,
    billedRequests: row.billed_requests ?? 0,
    grossAmount: row.gross_amount ?? 0,
    billedAmount: row.billed_amount ?? 0,
  }));

  return {
    fetchedAt: new Date(),
    days,
  };
}
