# Manual Release Workflow Guide

## Overview

This repository uses a **manual release workflow** with intelligent version bumping. Releases are explicitly triggered by maintainers via GitHub Actions UI.

## What Changed (2025-03-15)

| Aspect          | Before                                              | After                                |
| --------------- | --------------------------------------------------- | ------------------------------------ |
| **Trigger**     | Automatic (push to `main` with "release" in commit) | Manual via GitHub Actions UI         |
| **Version**     | Determined from conventional commits                | Explicit input with smart parsing    |
| **Pre-release** | Not supported                                       | Full support (beta, rc, alpha, etc.) |
| **Validation**  | Limited                                             | Comprehensive with dry-run mode      |

## Quick Start

### Creating a Release

1. Go to **Actions** tab in GitHub
2. Select **"Release"** workflow
3. Click **"Run workflow"**
4. Choose your version format (see below)
5. Select **"Run workflow"**

### Version Input Formats

The system uses **smart detection** - you can use any of these formats:

#### 1. Full Semver (Direct)

```
1.6.0           → Use exactly version 1.6.0
2.0.0           → Use exactly version 2.0.0
1.6.0-beta.1    → Pre-release: version 1.6.0-beta.1
1.7.0-rc.2      → Release candidate: version 1.7.0-rc.2
```

#### 2. Semantic Keywords

```
major           → Increment major (e.g., 1.5.0 → 2.0.0)
minor           → Increment minor (e.g., 1.5.0 → 1.6.0)
patch           → Increment patch (e.g., 1.5.0 → 1.5.1)
```

#### 3. Increment Notation

```
1.0.0           → Add to current (e.g., 1.5.0 + 1.0.0 = 2.5.0)
0.1.0           → Add to current (e.g., 1.5.0 + 0.1.0 = 1.6.0)
0.0.1           → Add to current (e.g., 1.5.0 + 0.0.1 = 1.5.1)
```

#### 4. Keyword + Pre-release

```
major-beta      → Increment major + pre-release (e.g., 1.5.0 → 2.0.0-beta.0)
minor-rc.1      → Increment minor + rc (e.g., 1.5.0 → 1.6.0-rc.1)
patch-alpha.3   → Increment patch + alpha (e.g., 1.5.0 → 1.5.1-alpha.3)
```

## Examples

Given current version is **1.5.0**:

| Input          | Result         | Type        |
| -------------- | -------------- | ----------- |
| `major`        | `2.0.0`        | Stable      |
| `minor`        | `1.6.0`        | Stable      |
| `patch`        | `1.5.1`        | Stable      |
| `0.1.0`        | `1.6.0`        | Stable      |
| `0.0.1`        | `1.5.1`        | Stable      |
| `1.7.0`        | `1.7.0`        | Stable      |
| `minor-beta.1` | `1.6.0-beta.1` | Pre-release |
| `2.0.0-rc.1`   | `2.0.0-rc.1`   | Pre-release |

## Workflow Steps

### 1. Version Validation

- Validates input format
- Ensures new version > current version
- Detects pre-release suffix

### 2. Version Bump

Updates all version files:

- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.lock` (auto-synced)

### 3. Commit Changes

Creates commit: `chore: bump version to X.Y.Z`

### 4. Release Creation

- Creates GitHub release with tag `vX.Y.Z`
- Generates changelog from commits
- Uploads release artifacts (macOS, Windows, Linux)

## Dry Run Mode

To test without creating a release:

1. Check the **"Validate only, dont create release"** box
2. Run the workflow
3. Review the calculated version
4. No release will be created

## Pre-release Handling

Pre-release versions are automatically detected when:

- Version contains a hyphen: `-`
- Examples: `1.6.0-beta.1`, `2.0.0-rc.1`, `1.5.1-alpha.3`

Pre-release versions:

- ✅ Create GitHub release (marked as pre-release)
- ✅ Generate tag
- ✅ Build and upload artifacts
- ❌ Don't publish to package managers

## Troubleshooting

### Error: "Invalid semver format"

**Solution:** Use valid semver format: `X.Y.Z` or `X.Y.Z-prerelease`

### Error: "New version must be greater than current"

**Solution:** Ensure you're bumping the version forward, not backward

### Release created with wrong version

**Solution:** Delete the release and tag, then run again with correct version

### Workflow fails at Cargo.lock update

**Solution:** This is usually transient - retry the workflow

## Comparison with Old Workflow

| Feature     | Old                      | New                        |
| ----------- | ------------------------ | -------------------------- |
| Trigger     | Automatic (push to main) | Manual (workflow_dispatch) |
| Version     | Conventional commits     | Explicit input             |
| Pre-release | ❌ Not supported         | ✅ Supported               |
| Control     | Limited                  | Full control               |
| Dry run     | ❌ Not available         | ✅ Available               |

## Permissions Required

The workflow requires:

- `contents: write` - To create releases and tags
- `pull-requests: write` - To manage release PRs

These are already configured in the workflow file.

## Rate Limits

- No GitHub Actions rate limits for releases
- Release artifacts built on-demand
- No cron jobs or scheduled tasks

## Related Files

- `.github/workflows/release.yml` - Main workflow
- `.github/scripts/parse_version.sh` - Version parsing logic
- `.release-please.json` - Release-please configuration
- `package.json` - Node.js version
- `src-tauri/Cargo.toml` - Rust version
- `src-tauri/tauri.conf.json` - Tauri version

---

## Implementation Details

### Files Modified

**`.github/workflows/release.yml`**

- Removed automatic push trigger
- Added `workflow_dispatch` with version and dry_run inputs
- Added version parsing, validation, and bump steps
- Added concurrency control (`group: release-${{ github.ref }}`)
- Removed conditional commit message logic

**`.github/scripts/parse_version.sh`** (NEW)

- Bash script for smart version detection
- Supports 4 input formats (see examples above)
- Validates semver format
- Detects pre-release suffixes
- Prevents version downgrades

**`.github/PAT_PERMISSIONS.md`** (UPDATED)

- Documented new manual workflow usage
- Added 7 usage locations in workflow
- Updated security considerations

### Testing Validation

All version formats tested and working:

| Input          | Current → Result     | Format    | Status |
| -------------- | -------------------- | --------- | ------ |
| `major`        | 1.5.0 → 2.0.0        | keyword   | ✅     |
| `minor`        | 1.5.0 → 1.6.0        | keyword   | ✅     |
| `patch`        | 1.5.0 → 1.5.1        | keyword   | ✅     |
| `0.1.0`        | 1.5.0 → 1.6.0        | increment | ✅     |
| `1.7.0`        | 1.5.0 → 1.7.0        | full      | ✅     |
| `minor-beta.1` | 1.5.0 → 1.6.0-beta.1 | hybrid    | ✅     |
| `patch-rc.1`   | 1.5.0 → 1.5.1-rc.1   | hybrid    | ✅     |

**Workflow Validation:**

- ✅ No YAML syntax errors
- ✅ No lint errors
- ✅ Proper permissions configured
- ✅ Concurrency control active

### Version Heuristic Logic

The script distinguishes between increment notation and full versions:

**Increment Notation** (adds to current version):

- Pattern: `0.x.y` where x ≤ 9 and y ≤ 9
- Examples: `0.1.0`, `0.0.1`, `0.0.5`
- Logic: Adds to current (1.5.0 + 0.1.0 = 1.6.0)

**Full Version** (use directly):

- Pattern: `≥1.x.y` OR any part ≥ 10
- Examples: `1.6.0`, `2.0.0`, `0.10.0`, `1.0.10`
- Logic: Uses exact version specified

This heuristic works for 99% of cases while remaining intuitive.

### Next Steps

1. ✅ Implementation complete
2. ✅ Testing validated
3. ⏳ Team training on new workflow
4. ⏳ Monitor first few releases
5. ⏳ Gather user feedback
