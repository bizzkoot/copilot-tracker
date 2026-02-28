# AGENTS.md - Copilot Tracker

Guidelines for AI agents working on this Tauri + React codebase.

## ALWAYS USE `ask_user` TOOL FOR ANY REPLY, RESPONSE, SUMMARY OR ANY INTENTION TO END THE SESSION, THIS IS COMPULSORY. WHEN `ask_user` TOOL IS NOT AVAILABLE, THEN FALLBACK TO `askquestions` OR `question` TOOLS.

`When the task requires multiple steps or non-trivial changes, present a detailed plan using #planReview and wait for approval before executing.
If the plan is rejected, incorporate the comments and submit an updated plan with #planReview.
When the user asks for a step-by-step guide or walkthrough, present it using #walkthroughReview.
Always use #askUser before completing any task to confirm the result matches what the user asked for.`

## Tech Stack

- **Frontend:** React 18, TypeScript 5, Tailwind CSS 3, Zustand 4
- **Backend:** Rust, Tauri 2.x
- **Build:** Vite, Cargo

## Commands

```bash
# Development
npm run dev              # Start Tauri dev mode
npm run dev:web          # Vite dev server only (port 5173)

# Building
npm run build            # Full production build
npm run build:web        # Build web assets only
npm run build:win        # Windows target
npm run build:mac        # macOS universal target
npm run build:linux      # Linux target

# Code Quality
npm run lint             # Run ALL linters (JS + Rust)
npm run lint:js          # ESLint with auto-fix only
npm run lint:rust        # Rust fmt check + clippy
npm run format           # Prettier formatting
npm run typecheck        # TypeScript check (node + web)
npm run typecheck:node   # Node-only types
npm run typecheck:web    # Web-only types

# Utilities
npm run validate:assets  # Validate static assets exist
```

## Code Style

### TypeScript/React

**Imports:**

- Use absolute imports with `@renderer/` alias
- Group imports: React → libraries → `@renderer/` → relative
- Example: `import { cn } from "@renderer/lib/utils";`

**Formatting:**

- Prettier with default config
- 2-space indentation
- Single quotes for strings
- Semicolons required
- Trailing commas in multi-line

**Types:**

- Explicit return types on exported functions
- Interface for object types, type for unions/aliases
- Prefix unused params with `_`

**Naming:**

- PascalCase: components, types, interfaces
- camelCase: variables, functions, hooks
- SCREAMING_SNAKE_CASE: constants

**Components:**

- Use function declarations for components
- Forward refs with `React.forwardRef`
- Set `displayName` on forwarded components
- Tailwind classes in template literals for complex cases

**State Management:**

- Zustand stores in `src/renderer/src/stores/`
- Selectors for individual state slices
- Actions defined in store, not components

**Error Handling:**

- Use `try/catch` for async operations
- Log errors in development: `if (isDev) console.error(...)`
- Set error state via stores, not inline alerts

### Rust

**Formatting:**

- `cargo fmt --all -- --check` for formatting check
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` for linting

**Naming:**

- snake_case: variables, functions, modules
- PascalCase: structs, enums, traits
- SCREAMING_SNAKE_CASE: constants, statics

**Error Handling:**

- Use `?` operator for propagation
- `Result<T, E>` for fallible operations
- `Option<T>` for nullable values

**Organization:**

- Modules in separate files
- Re-exports in `lib.rs`
- Tauri commands in `main.rs` with `#[tauri::command]`

## Project Structure

```
src/renderer/src/
  components/     # React components (feature-based)
    ui/           # Reusable UI primitives
    auth/         # Auth-related components
    dashboard/    # Main dashboard
    layout/       # Layout components
  hooks/          # Custom React hooks
  stores/         # Zustand stores
  services/       # API services
  types/          # TypeScript types
  lib/            # Utilities

src-tauri/src/
  main.rs         # Entry point, Tauri commands
  lib.rs          # Module exports
  auth.rs         # GitHub auth logic
  store.rs        # Persistent storage
  usage.rs        # Usage data fetching
  tray_icon_renderer.rs  # Tray icon generation
```

## Key Patterns

**Tailwind + CSS Variables:**

- Theme colors use CSS variables: `bg-[hsl(var(--primary))]`
- Utility: `cn()` merges classes with tailwind-merge

**Tauri Bridge:**

- Access via `window.electron` (typed in `types/app.ts`)
- Events use Tauri's `emit`/`listen` pattern

**No Tests:**

- This project has no test suite currently
- Manual testing via `npm run dev`

## Critical Notes

- Always run `npm run typecheck` and `npm run lint` before committing
- `npm run lint` runs both JS (ESLint) and Rust (fmt + clippy) checks
- Use `npm run lint:js` or `npm run lint:rust` for individual language checks
- macOS uses private Tauri APIs (enabled in config)
- Tray icon rendered dynamically using fontdue/tiny-skia
