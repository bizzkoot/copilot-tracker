#!/usr/bin/env bash
# parse_version.sh - Parse version bump input and compute new version
#
# Usage: parse_version.sh <version_input> <current_version>
#
# Version input formats:
#   - Full semver:  1.6.0, 2.0.0-beta.1
#   - Keyword:      major, minor, patch
#   - Increment:    0.1.0, 0.0.1 (added to current version; major must be 0)
#   - Hybrid:       minor-beta.1, patch-rc.2 (keyword + pre-release)
#   - Exact pre-1.0: exact:0.9.0 (force exact version when major is 0)
#
# Note: inputs matching ^0\.[0-9]+\.[0-9]+$ are treated as increments.
# Use the "exact:" prefix to target a literal pre-1.0 version instead.
#
# Outputs (GitHub Actions):
#   version=<new_version>  (via GITHUB_OUTPUT)

set -euo pipefail

VERSION_INPUT="${1:?Usage: parse_version.sh <version_input> <current_version>}"
CURRENT_VERSION="${2:?Current version is required}"

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

# Split version into major.minor.patch (strip pre-release)
split_version() {
  local ver="${1%%-*}" # strip pre-release
  IFS='.' read -r major minor patch <<< "$ver"
  echo "$major $minor $patch"
}

bump_major() {
  local parts
  read -r major minor patch <<< "$(split_version "$1")"
  echo "$((10#$major + 1)).0.0"
}

bump_minor() {
  local parts
  read -r major minor patch <<< "$(split_version "$1")"
  echo "$major.$((10#$minor + 1)).0"
}

bump_patch() {
  local parts
  read -r major minor patch <<< "$(split_version "$1")"
  echo "$major.$minor.$((10#$patch + 1))"
}

# Check if string is a valid semver (major.minor.patch with optional pre-release)
is_semver() {
  [[ "$1" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9._]+)?$ ]]
}

# Check if string is an increment pattern (0.x.y where major is 0)
is_increment_pattern() {
  [[ "$1" =~ ^0\.[0-9]+\.[0-9]+$ ]]
}

# ---------------------------------------------------------------------------
# Main logic
# ---------------------------------------------------------------------------

NEW_VERSION=""
PRE_RELEASE=""

# Handle "exact:" prefix — forces literal version targeting (bypasses increment detection).
# Use this for pre-1.0 releases: e.g., "exact:0.9.0" sets version to exactly "0.9.0".
if [[ "$VERSION_INPUT" == exact:* ]]; then
  FINAL_VERSION="${VERSION_INPUT#exact:}"
  echo "version=${FINAL_VERSION}" >> "$GITHUB_OUTPUT"
  exit 0
fi

# Extract pre-release suffix if present (e.g., from "minor-beta.1")
if [[ "$VERSION_INPUT" == *"-"* ]]; then
  # Split on first dash
  KEYWORD_PART="${VERSION_INPUT%%-*}"
  PRE_RELEASE="-${VERSION_INPUT#*-}"
else
  KEYWORD_PART="$VERSION_INPUT"
  PRE_RELEASE=""
fi

case "$KEYWORD_PART" in
  major)
    NEW_VERSION="$(bump_major "$CURRENT_VERSION")"
    ;;
  minor)
    NEW_VERSION="$(bump_minor "$CURRENT_VERSION")"
    ;;
  patch)
    NEW_VERSION="$(bump_patch "$CURRENT_VERSION")"
    ;;
  *.*) 
    # Could be full semver (2.0.0) or increment (0.1.0)
    if is_increment_pattern "$KEYWORD_PART"; then
      # Increment: add to current version
      IFS='.' read -r inc_maj inc_min inc_pat <<< "$KEYWORD_PART"
      read -r cur_maj cur_min cur_pat <<< "$(split_version "$CURRENT_VERSION")"
      NEW_VERSION="$((10#$cur_maj + 10#$inc_maj)).$((10#$cur_min + 10#$inc_min)).$((10#$cur_pat + 10#$inc_pat))"
    else
      # Full semver specified directly
      NEW_VERSION="$KEYWORD_PART"
    fi
    ;;
  *)
    # Unknown format - treat as full version string
    NEW_VERSION="$KEYWORD_PART"
    ;;
esac

# Append pre-release suffix if present
FINAL_VERSION="${NEW_VERSION}${PRE_RELEASE}"

# Validate result looks like semver
if ! [[ "$FINAL_VERSION" =~ ^[0-9]+\.[0-9]+\.[0-9]+ ]]; then
  echo "❌ Error: Computed version '$FINAL_VERSION' is not valid semver"
  exit 1
fi

# Output for GitHub Actions
if [ -n "${GITHUB_OUTPUT:-}" ]; then
  echo "version=$FINAL_VERSION" >> "$GITHUB_OUTPUT"
else
  echo "version=$FINAL_VERSION"
fi

echo "🔄 Version parsed: $CURRENT_VERSION → $FINAL_VERSION"
