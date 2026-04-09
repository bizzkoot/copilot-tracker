#!/bin/bash

# =============================================================================
# Manual Testing Script - NOT part of npm run test
# =============================================================================
#
# This script is for MANUAL TESTING ONLY. It does NOT run with `npm run test`.
#
# Purpose:
#   Verify macOS build artifacts locally before releases by simulating the
#   manual-upload.yml workflow for macOS.
#
# Usage:
#   ./scripts/test-macos-build.sh
#
# What it tests:
#   - Web assets build
#   - macOS universal binary compilation (Rust)
#   - .app bundle creation
#   - .app.zip creation with ditto (preserves metadata)
#   - DMG creation
#
# Expected time: ~3 minutes on local machine
#
# Note: This is a long-running integration test, not a unit test.
# =============================================================================


set -e

echo "🧪 Testing macOS Build Workflow (Local Simulation)"
echo "=================================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Step 1: Build web assets
echo -e "${YELLOW}Step 1: Building web assets...${NC}"
npm run build:web

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Web assets built successfully${NC}"
else
    echo -e "${RED}❌ Web assets build failed${NC}"
    exit 1
fi

echo ""

# Step 2: Build macOS universal binary
echo -e "${YELLOW}Step 2: Building macOS universal binary (this will take 5-10 minutes)...${NC}"
TAURI_BUNDLE_SKIP_SIGNING=true npx tauri build --target universal-apple-darwin

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ macOS universal binary built successfully${NC}"
else
    echo -e "${RED}❌ macOS build failed${NC}"
    exit 1
fi

echo ""

# Step 3: Verify .app bundle exists
echo -e "${YELLOW}Step 3: Verifying .app bundle...${NC}"
APP_DIR="src-tauri/target/universal-apple-darwin/release/bundle/macos/Copilot Tracker.app"

if [ -d "$APP_DIR" ]; then
    echo -e "${GREEN}✅ .app bundle found at $APP_DIR${NC}"
    ls -lh "$APP_DIR"
else
    echo -e "${RED}❌ .app bundle not found at $APP_DIR${NC}"
    exit 1
fi

echo ""

# Step 4: Create .app.zip with ditto
echo -e "${YELLOW}Step 4: Creating .app.zip with ditto...${NC}"
ditto -c -k --sequesterRsrc --keepParent "$APP_DIR" "${APP_DIR}.zip"

if [ -f "${APP_DIR}.zip" ]; then
    echo -e "${GREEN}✅ .app.zip created successfully${NC}"
    ls -lh "${APP_DIR}.zip"
else
    echo -e "${RED}❌ .app.zip creation failed${NC}"
    exit 1
fi

echo ""

# Step 5: Verify all artifacts
echo -e "${YELLOW}Step 5: Verifying all artifacts...${NC}"
DMG_PATH=$(find src-tauri/target/universal-apple-darwin/release/bundle/dmg -name "*.dmg" | head -1)
ZIP_PATH="${APP_DIR}.zip"

SUCCESS=true

if [ -f "$DMG_PATH" ]; then
    echo -e "${GREEN}✅ DMG artifact: $DMG_PATH${NC}"
    ls -lh "$DMG_PATH"
else
    echo -e "${RED}❌ DMG artifact not found${NC}"
    SUCCESS=false
fi

echo ""

if [ -f "$ZIP_PATH" ]; then
    echo -e "${GREEN}✅ .app.zip artifact: $ZIP_PATH${NC}"
    ls -lh "$ZIP_PATH"
else
    echo -e "${RED}❌ .app.zip artifact not found${NC}"
    SUCCESS=false
fi

echo ""
echo "=================================================="

if [ "$SUCCESS" = true ]; then
    echo -e "${GREEN}🎉 All tests passed! macOS build workflow verified.${NC}"
    exit 0
else
    echo -e "${RED}❌ Some tests failed. Please review the output above.${NC}"
    exit 1
fi
