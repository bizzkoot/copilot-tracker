async function debugCopilot() {
  console.log("Extracting customer ID...");
  let customerId = null;

  // Try to find customer ID from DOM
  try {
    const el = document.querySelector(
      'script[data-target="react-app.embeddedData"]',
    );
    if (el) {
      const data = JSON.parse(el.textContent);
      customerId = data?.payload?.customer?.customerId;
    }
  } catch (e) {
    // Ignore parsing errors
  }

  // Try regex if failed
  if (!customerId) {
    const html = document.body.innerHTML;
    const match =
      html.match(/customerId":(\d+)/) || html.match(/data-customer-id="(\d+)"/);
    if (match) customerId = match[1];
  }

  if (!customerId) {
    console.error(
      "Could not find Customer ID. Please ensure you are on https://github.com/settings/billing",
    );
    return;
  }

  console.log("Found Customer ID:", customerId);
  console.log("Fetching usage data...");

  try {
    const res = await fetch(
      `/settings/billing/copilot_usage_table?customer_id=${customerId}&group=0&period=3&query=&page=1`,
      {
        headers: {
          Accept: "application/json",
          "x-requested-with": "XMLHttpRequest",
        },
      },
    );
    const data = await res.json();
    console.log("--- START DATA ---");
    console.log(JSON.stringify(data, null, 2));
    console.log("--- END DATA ---");
  } catch (e) {
    console.error("Failed to fetch data:", e);
  }
}
debugCopilot();
