#!/bin/bash
set -euo pipefail

INPUT_VERSION="$1"
CURRENT_VERSION="$2"

# Check if running in GitHub Actions
if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
  IN_GITHUB_ACTIONS=true
else
  IN_GITHUB_ACTIONS=false
fi

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to increment semver
increment_version() {
  local version="$1"
  local part="$2"
  
  local major minor patch
  IFS='.' read -r major minor patch <<< "${version%-*}"
  
  case "$part" in
    major)
      echo "$((major + 1)).0.0"
      ;;
    minor)
      echo "${major}.$((minor + 1)).0"
      ;;
    patch)
      echo "${major}.${minor}.$((patch + 1))"
      ;;
    *)
      echo "$version"
      ;;
  esac
}

# Function to add version numbers
add_versions() {
  local v1="$1"  # Current (e.g., 1.5.0)
  local v2="$2"  # Increment (e.g., 0.1.0)
  
  local v1_major v1_minor v1_patch
  local v2_major v2_minor v2_patch
  
  IFS='.' read -r v1_major v1_minor v1_patch <<< "${v1%-*}"
  IFS='.' read -r v2_major v2_minor v2_patch <<< "${v2%-*}"
  
  local new_major=$((v1_major + v2_major))
  local new_minor=$((v1_minor + v2_minor))
  local new_patch=$((v1_patch + v2_patch))
  
  echo "${new_major}.${new_minor}.${new_patch}"
}

# Extract pre-release suffix if present
PRERELEASE_SUFFIX=""
if [[ "$INPUT_VERSION" =~ -(.+)$ ]]; then
  PRERELEASE_SUFFIX="-${BASH_REMATCH[1]}"
fi

# Remove suffix for parsing
VERSION_BASE="${INPUT_VERSION%-*}"

# Detect format and calculate new version
case "$VERSION_BASE" in
  major|minor|patch)
    # Semantic keyword
    NEW_VERSION="$(increment_version "$CURRENT_VERSION" "$VERSION_BASE")"
    INPUT_TYPE="keyword"
    ;;
  [0-9]*)
    # Full semver or increment notation
    if [[ "$VERSION_BASE" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
      VERSION_MAJOR=$(echo "$VERSION_BASE" | cut -d. -f1)
      VERSION_MINOR=$(echo "$VERSION_BASE" | cut -d. -f2)
      VERSION_PATCH=$(echo "$VERSION_BASE" | cut -d. -f3)
      
      # Heuristic for detecting increment vs full version:
      # - Increment: major=0 and minor≤9 and patch≤9 (e.g., 0.1.0, 0.0.1)
      # - Full version: major≥1, OR minor≥10, OR patch≥10
      if [[ "$VERSION_MAJOR" -eq 0 ]] && \
         [[ "$VERSION_MINOR" -le 9 ]] && \
         [[ "$VERSION_PATCH" -le 9 ]]; then
        # Treat as increment (e.g., 0.1.0, 0.0.1)
        NEW_VERSION="$(add_versions "$CURRENT_VERSION" "$VERSION_BASE")"
        INPUT_TYPE="increment"
      else
        # Use as-is (full version like 1.7.0, 2.0.0, 1.6.0-beta)
        NEW_VERSION="$VERSION_BASE"
        INPUT_TYPE="full"
      fi
    else
      echo -e "${RED}❌ Invalid semver format: $VERSION_BASE${NC}"
      echo "Valid formats:"
      echo "  - Full: 1.6.0, 2.0.0-beta.1"
      echo "  - Keyword: major, minor, patch"
      echo "  - Increment: 0.1.0, 0.0.1"
      echo "  - Hybrid: minor-beta.1, patch-rc.2"
      exit 1
    fi
    ;;
  *)
    echo -e "${RED}❌ Unrecognized version format: $INPUT_VERSION${NC}"
    echo "Valid formats:"
    echo "  - Full: 1.6.0, 2.0.0-beta.1"
    echo "  - Keyword: major, minor, patch"
    echo "  - Increment: 0.1.0, 0.0.1"
    echo "  - Hybrid: minor-beta.1, patch-rc.2"
    exit 1
    ;;
esac

# Append pre-release suffix if present
FINAL_VERSION="${NEW_VERSION}${PRERELEASE_SUFFIX}"

# Output for GitHub Actions
if [[ "$IN_GITHUB_ACTIONS" == true ]]; then
  echo "version=$FINAL_VERSION" >> $GITHUB_OUTPUT
  
  # Determine if pre-release
  if [[ -n "$PRERELEASE_SUFFIX" ]]; then
    echo "is_prerelease=true" >> $GITHUB_OUTPUT
  else
    echo "is_prerelease=false" >> $GITHUB_OUTPUT
  fi
fi

# Display type for output
if [[ -n "$PRERELEASE_SUFFIX" ]]; then
  PRERELEASE_TYPE="${GREEN}pre-release${NC}"
else
  PRERELEASE_TYPE="${GREEN}stable${NC}"
fi

# Echo summary
echo ""
echo -e "${BLUE}📦 Version Calculation:${NC}"
echo -e "   Current:  ${BLUE}$CURRENT_VERSION${NC}"
echo -e "   Input:    ${BLUE}$INPUT_VERSION${NC} ${GREEN}($INPUT_TYPE)${NC}"
echo -e "   New:      ${GREEN}$FINAL_VERSION${NC}"
echo -e "   Type:     $PRERELEASE_TYPE"
echo ""
