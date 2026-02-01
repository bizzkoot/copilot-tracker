import { readFileSync, existsSync } from "fs";
import { join } from "path";
import { exit } from "process";

const requiredAssets = [
  "resources/icon.png",
  "resources/icon.icns",
  "resources/tray/tray.png",
  "resources/tray/trayTemplate.png",
];

let hasErrors = false;

console.log("🔍 Validating required assets...\n");

for (const asset of requiredAssets) {
  const assetPath = join(__dirname, "..", asset);

  if (!existsSync(assetPath)) {
    console.error(`❌ Missing: ${asset}`);
    hasErrors = true;
  } else {
    // Check file size
    const stats = readFileSync(assetPath);
    if (stats.length === 0) {
      console.error(`❌ Empty file: ${asset}`);
      hasErrors = true;
    } else {
      console.log(`✅ Found: ${asset} (${stats.length} bytes)`);
    }
  }
}

if (hasErrors) {
  console.error("\n❌ Asset validation failed!");
  console.error("Please ensure all required assets exist before building.\n");
  exit(1);
}

console.log("\n✅ All assets validated successfully!\n");
exit(0);
