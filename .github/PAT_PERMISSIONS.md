# GitHub Personal Access Token (PAT) Documentation

## RELEASE_PLEASE_TOKEN

This repository uses a GitHub Personal Access Token (PAT) to enable release-please automation and manual release workflow operations.

### 🔄 Workflow Update (2025-03-15)

**Changed from automatic to manual release system:**

| Before                                         | After                                      |
| ---------------------------------------------- | ------------------------------------------ |
| Automatic trigger on push to `main`            | Manual trigger via GitHub Actions UI       |
| release-please determined version from commits | Explicit version input with smart parsing  |
| No pre-release support                         | Full pre-release support (beta, rc, alpha) |
| Limited validation                             | Comprehensive validation with dry-run mode |

**New responsibilities (still requires `RELEASE_PLEASE_TOKEN`):**

- ✅ Manual workflow triggering
- ✅ Version bump commits (package.json, Cargo.toml, tauri.conf.json)
- ✅ Pushing version changes to main branch
- ✅ Creating GitHub releases with tags
- ✅ Triggering PR validation workflows
- ✅ Uploading release artifacts

**See:** `.github/workflows/RELEASE_WORKFLOW.md` for usage guide

### Required Permissions

The `RELEASE_PLEASE_TOKEN` must have the following scopes:

- ✅ **`repo`** - Full control of private repositories
  - Required for: Creating/updating release PRs, reading repository content
  - Note: For public repositories, `public_repo` scope is sufficient

- ✅ **`workflow`** - Update GitHub Action workflows
  - Required for: Triggering PR Checks workflow on release PRs created by bot
  - Without this, PRs created by github-actions[bot] won't trigger other workflows

### Creating the Token

1. Go to: https://github.com/settings/tokens/new
2. **Token name:** `RELEASE_PLEASE_TOKEN` (or descriptive name)
3. **Expiration:** Choose 90 days, 1 year, or no expiration (use discretion)
4. **Select scopes:**
   - ✅ `repo` (or `public_repo` for public repos)
   - ✅ `workflow`
5. Click **"Generate token"**
6. **Copy the token** immediately (starts with `ghp_...`)

### Adding Token to Repository

1. Go to repository settings: https://github.com/bizzkoot/copilot-tracker/settings/secrets/actions
2. Click **"New repository secret"**
3. **Name:** `RELEASE_PLEASE_TOKEN`
4. **Secret:** Paste the PAT token
5. Click **"Add secret"**

### Token Usage

The token is used in `.github/workflows/release.yml` for the following operations:

```yaml
# Manual Release Workflow
- name: Check out Git repository
  uses: actions/checkout@v4
  with:
    token: ${{ secrets.RELEASE_PLEASE_TOKEN }}
  # Required for: Version bump commits and pushes

- name: Commit version bump
  run: |
    git push
  # Required for: Pushing version changes to main branch

- uses: googleapis/release-please-action@v4
  with:
    token: ${{ secrets.RELEASE_PLEASE_TOKEN }}
  # Required for: Creating releases and tags

# Format Release PR Workflow
- name: Check out Git repository
  uses: actions/checkout@v4
  with:
    ref: ${{ steps.release-pr.outputs.head_ref }}
    token: ${{ secrets.RELEASE_PLEASE_TOKEN }}
  # Required for: Checkout PR branch for formatting

- name: Commit all changes
  run: |
    git push origin "HEAD:${{ steps.release-pr.outputs.head_ref }}"
  # Required for: Pushing formatted changes to PR

# Validate Release PR Workflow
- name: Check out Git repository
  uses: actions/checkout@v4
  with:
    ref: refs/pull/${{ needs.format-release-pr.outputs.pr_number }}/head
    token: ${{ secrets.RELEASE_PLEASE_TOKEN }}
  # Required for: Checkout PR for validation

# Build Tauri App
- uses: softprops/action-gh-release@v1
  with:
    tag_name: ${{ needs.release-please.outputs.tag_name }}
  # Required for: Uploading release assets
```

**Usage Summary:**

- **7 total occurrences** in the workflow
- Used across 4 job types: release-please, format-release-pr, validate-release-pr, build-tauri
- Enables manual version bumping, release creation, PR management, and artifact uploads

### Security Best Practices

1. **Minimal scope:** Only grant required permissions (`repo` + `workflow`)
2. **Repository-only:** Consider using fine-grained tokens scoped to this repository only
3. **Rotation policy:** Rotate token every **90 days** (recommended)
4. **Token monitoring:** Review GitHub security log for unexpected token usage
5. **Audit trail:** Document token creation/rotation in this file
6. **Manual control:** Token now used for manual operations only (no automatic triggers)

### Token Rotation

**Last rotated:** [To be filled on rotation]  
**Next rotation due:** [To be filled on rotation]

When rotating:

1. Generate new token with same permissions
2. Update `RELEASE_PLEASE_TOKEN` secret in repository settings
3. Test manual release workflow with dry-run mode
4. Verify all operations succeed (version bump, release creation, uploads)
5. Revoke old token from GitHub settings
6. Update this document with rotation dates

### Why This Token is Needed

**Problem:** GitHub Actions by default doesn't trigger workflows on PRs created by bots (using `GITHUB_TOKEN`) to prevent infinite workflow loops. Additionally, manual release operations require write access to the repository.

**Solution:** Using a PAT with `repo` + `workflow` scopes allows:

#### Manual Release Workflow (New - 2025-03-15)

- ✅ Manual workflow triggering via GitHub Actions UI
- ✅ Version bump commits to main branch
- ✅ Pushing version changes (package.json, Cargo.toml, tauri.conf.json)
- ✅ Creating GitHub releases and tags
- ✅ Uploading release artifacts (macOS, Windows, Linux)
- ✅ Triggering PR validation workflows

#### Legacy PR Workflow (Still Supported)

- ✅ Release-please to create PRs that **automatically trigger PR Checks**
- ✅ No manual close/reopen required for validation
- ✅ Fully automated PR formatting and validation

**Without this token:**

- ❌ Cannot manually trigger releases
- ❌ Cannot commit version bumps
- ❌ Cannot push to main branch
- ❌ Cannot create GitHub releases
- ❌ PRs created by github-actions[bot] won't trigger validation workflows

### Alternatives Considered

1. **GitHub Apps** - More secure with fine-grained permissions, but requires app setup
2. **Deploy keys** - Cannot trigger workflows, insufficient permissions
3. **GITHUB_TOKEN** - Built-in, but deliberately excludes workflow triggering

**Decision:** PAT is the simplest solution with adequate security for this use case.

---

**Last updated:** 2026-02-10  
**Maintainer:** @bizzkoot
