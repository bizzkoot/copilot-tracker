# GitHub Personal Access Token (PAT) Documentation

## RELEASE_PLEASE_TOKEN

This repository uses a GitHub Personal Access Token (PAT) to enable release-please automation to trigger PR checks on release PRs.

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

The token is used in `.github/workflows/release.yml`:

```yaml
# Line 43: Release-please action
- uses: googleapis/release-please-action@v4
  with:
    token: ${{ secrets.RELEASE_PLEASE_TOKEN }}

# Line 63: Format release PR checkout
- uses: actions/checkout@v4
  with:
    token: ${{ secrets.RELEASE_PLEASE_TOKEN }}

# Line 113: Validate release PR checkout  
- uses: actions/checkout@v4
  with:
    token: ${{ secrets.RELEASE_PLEASE_TOKEN }}
```

### Security Best Practices

1. **Minimal scope:** Only grant required permissions (`repo` + `workflow`)
2. **Repository-only:** Consider using fine-grained tokens scoped to this repository only
3. **Rotation policy:** Rotate token every **90 days** (recommended)
4. **Token monitoring:** Review GitHub security log for unexpected token usage
5. **Audit trail:** Document token creation/rotation in this file

### Token Rotation

**Last rotated:** [To be filled on rotation]  
**Next rotation due:** [To be filled on rotation]

When rotating:
1. Generate new token with same permissions
2. Update `RELEASE_PLEASE_TOKEN` secret in repository settings
3. Verify release workflow runs successfully
4. Revoke old token from GitHub settings
5. Update this document with rotation dates

### Why This Token is Needed

**Problem:** GitHub Actions by default doesn't trigger workflows on PRs created by bots (using `GITHUB_TOKEN`) to prevent infinite workflow loops.

**Solution:** Using a PAT with `workflow` scope allows:
- Release-please to create PRs that **automatically trigger PR Checks**
- No manual close/reopen required for validation
- Fully automated release workflow

### Alternatives Considered

1. **GitHub Apps** - More secure with fine-grained permissions, but requires app setup
2. **Deploy keys** - Cannot trigger workflows, insufficient permissions
3. **GITHUB_TOKEN** - Built-in, but deliberately excludes workflow triggering

**Decision:** PAT is the simplest solution with adequate security for this use case.

---

**Last updated:** 2026-02-10  
**Maintainer:** @bizzkoot
