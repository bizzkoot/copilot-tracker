# Copilot Tracker - Project Specification

> Cross-platform GitHub Copilot usage monitoring application

## Project Overview

### Vision

Create a modern, cross-platform desktop application that monitors GitHub Copilot usage with a beautiful UI, system tray integration, and smart notifications. The app should work seamlessly on macOS, Windows, and Linux while maintaining the same core functionality as the original macOS-only Swift application.

### Problem Statement

GitHub Copilot users need to track their premium request usage to:
- Avoid unexpected billing charges
- Understand their usage patterns
- Get notified before exceeding their included quota
- Plan their usage based on predictions

The original `copilot-usage-monitor` app solves this for macOS users, but Windows and Linux users have no solution.

### Solution

Build a cross-platform Electron application that:
1. Authenticates via GitHub OAuth (WebView-based, no API tokens needed)
2. Fetches usage data from GitHub's internal billing APIs
3. Displays current usage, predictions, and history
4. Shows in the system tray for quick access
5. Sends notifications when approaching limits

---

## Goals & Non-Goals

### Goals

| Goal | Priority | Description |
|------|----------|-------------|
| Cross-platform | P0 | Must work on macOS, Windows, Linux |
| Same functionality | P0 | All features from original app |
| Modern UI | P1 | Better visual design than original |
| System tray | P1 | Native system tray on all platforms |
| Dark/Light theme | P1 | Theme support with system detection |
| Usage charts | P1 | Trend visualization over time |
| Smart notifications | P1 | Configurable threshold alerts |
| Auto-updates | P2 | Automatic updates via GitHub releases |

### Non-Goals (Out of Scope)

| Non-Goal | Reason |
|----------|--------|
| Mobile apps | Focus on desktop first |
| Web version | Requires different auth approach |
| Multiple accounts | Complexity; single account is sufficient |
| Organization usage | Focus on individual users |
| Historical data export | Can be added later if needed |
| API token auth | WebView auth is more secure and user-friendly |

---

## Target Users

### Primary User

- **Individual GitHub Copilot subscribers** who want to monitor their usage
- Technical users comfortable with desktop applications
- Users on macOS, Windows, or Linux

### User Stories

1. **As a developer**, I want to see my current Copilot usage at a glance so I can avoid unexpected charges.

2. **As a Copilot user**, I want to see usage trends over time so I can understand my patterns.

3. **As a budget-conscious user**, I want notifications before I exceed my quota so I can adjust my usage.

4. **As a desktop user**, I want the app in my system tray so I can quickly check usage without opening a full window.

5. **As a user who works in different lighting**, I want dark/light theme support so the app is comfortable to view.

---

## Success Metrics

| Metric | Target | How to Measure |
|--------|--------|----------------|
| Platform support | 3 platforms | macOS, Windows, Linux builds work |
| Feature parity | 100% | All original features implemented |
| App size | < 100MB | Final installer size |
| Startup time | < 3s | Time from launch to showing data |
| Memory usage | < 150MB | Runtime memory consumption |
| Update rate | Same as original | 10s to 30min configurable |

---

## Competitive Analysis

### Existing Solutions

| Solution | Platforms | Pros | Cons |
|----------|-----------|------|------|
| **copilot-usage-monitor (original)** | macOS only | Native, lightweight, works well | macOS only, basic UI |
| **GitHub Billing Page** | Web | Official, accurate | Manual check, no notifications |
| **Browser extensions** | Chrome/Firefox | Easy to install | Can't run in background |

### Our Differentiators

1. **Cross-platform**: Only solution that works on all desktop platforms
2. **Better UI**: Modern design with charts and visualizations
3. **Smart notifications**: Configurable alerts, not just a static display
4. **System tray**: Background monitoring with quick access

---

## Constraints & Assumptions

### Constraints

1. **No official GitHub API**: Must use internal billing APIs via authenticated WebView
2. **Session-based auth**: Relies on GitHub session cookies, not API tokens
3. **Rate limits unknown**: GitHub's internal APIs may have undocumented rate limits
4. **API stability**: Internal APIs may change without notice

### Assumptions

1. GitHub's billing page structure will remain relatively stable
2. Users have valid GitHub Copilot subscriptions
3. Users are willing to authenticate via GitHub login in the app
4. Electron's Chromium WebView can access all necessary GitHub pages

### Risks & Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| API changes | High | Medium | Monitor for changes, quick patch releases |
| Session expiry | Medium | High | Graceful re-auth flow, clear messaging |
| GitHub blocks automation | High | Low | Use realistic user-agent, respect rate limits |
| Electron security vulnerabilities | High | Medium | Keep Electron updated, follow best practices |

---

## Timeline

### Phase Summary

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| Phase 1: Setup | 1-2 days | Running Electron app skeleton |
| Phase 2: Core | 1-2 days | Types and prediction algorithm |
| Phase 3: Auth | 2-3 days | Working GitHub OAuth |
| Phase 4: Data | 2-3 days | API fetching and caching |
| Phase 5: UI | 3-4 days | Complete dashboard |
| Phase 6: Tray | 2-3 days | System tray integration |
| Phase 7: Settings | 1-2 days | Preferences panel |
| Phase 8: Notifications | 1 day | Alert system |
| Phase 9: Packaging | 2-3 days | Distributable builds |

**Total: ~2-3 weeks of focused development**

### Milestones

1. **M1: Proof of Concept** (End of Phase 3)
   - App can authenticate with GitHub
   - Basic window with login flow

2. **M2: Functional MVP** (End of Phase 5)
   - Full data fetching working
   - Dashboard shows all data
   - No system tray yet

3. **M3: Feature Complete** (End of Phase 8)
   - All features implemented
   - Ready for testing

4. **M4: Release Ready** (End of Phase 9)
   - All platforms built and tested
   - Auto-updater configured

---

## Open Questions

1. **Q: Should we support multiple GitHub accounts?**
   - Current answer: No, single account focus
   - Rationale: Complexity vs. value tradeoff

2. **Q: Should settings sync across devices?**
   - Current answer: No, local settings only
   - Rationale: Adds cloud dependency

3. **Q: Should we cache data for offline viewing?**
   - Current answer: Yes, with cache expiry
   - Rationale: Better UX when offline

4. **Q: What happens if GitHub changes their API?**
   - Current answer: Release patch updates quickly
   - Rationale: Monitor original app for patterns

---

## Approval

This specification defines what we're building and why. The technical details of how we'll build it are in the Research and Implementation Plan documents.

**Status**: Ready for Implementation
