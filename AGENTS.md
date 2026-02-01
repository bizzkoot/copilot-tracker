# AGENT NOTES

## Communication Requirement

**CRITICAL: ALWAYS USE ASK_USER TOOL**
**CRITICAL: REPLACE ALL TOOL `question` WITH `ask_user` TOOL**

**MUST use `ask_user` tool for:**

- ANY reply to the user
- ANY response to the user
- ANY summary for the user

**Session ending: ONLY end the session when the user EXPLICITLY requests it through the "ask_user" tool.**

**NO EXCEPTIONS. This rule is MANDATORY and NON-NEGOTIABLE.**

## ask_user Tool Retry Behavior

**When ask_user returns `shouldRetry: true`:**

1. **IMMEDIATELY retry** the ask_user tool call with the SAME parameters
2. Do NOT summarize, end the session, or ask the user what happened
3. This is an automatic UI recovery mechanism - the question failed to display
4. The retry happens within the same session (no premium request cost)

**When ask_user returns `shouldRetry: false` and `timedOut: true`:**

1. The user did not respond within 5 minutes
2. Ask the user if they want to continue or if they missed the question
3. You may re-ask the question if the user confirms they want to continue

**When ask_user returns `shouldRetry: false` with error:**

1. Maximum retry attempts (3) have been exceeded
2. Inform the user: "There was a technical issue displaying the question. Please try again."
3. Ask if they want to continue with the task

## Build & Quality Commands

### Development

- `npm run dev` - Start Electron development server with hot reload

### Code Quality

- `npm run lint` - Run ESLint with auto-fix (run before committing)
- `npm run format` - Format code with Prettier
- `npm run typecheck` - Run TypeScript type checking (both main and renderer)
- `npm run typecheck:node` - Type check main process only
- `npm run typecheck:web` - Type check renderer process only

### Building

- `npm run build` - Full build with type checking
- `npm run build:win` - Build Windows executable
- `npm run build:mac` - Build macOS executable
- `npm run build:linux` - Build Linux executable

### Testing

**No test framework is currently configured.** When adding tests, choose and configure a framework (Jest, Vitest, or Playwright).

## Code Style Guidelines

### File Structure & Organization

- **Barrel exports**: Use `index.ts` files to export from directories
- **Component colocation**: Keep components, hooks, and types in feature folders
- **Renderer code**: `src/renderer/src/` uses `@renderer/` alias for imports
- **Main process**: `src/main/` for Electron backend logic
- **Preload**: `src/preload/` for IPC bridge scripts

### TypeScript & Types

- **Strict typing**: All files must have proper TypeScript types
- **Type definitions**: Centralize types in `types/` directories
- **Interfaces**: Use for object shapes, prefer `type` for unions/intersections
- **Type exports**: Always export types used by other modules
- **No `any`**: Avoid `any`; use `unknown` with type guards if needed

### Import Conventions

```typescript
// Order: 1) External deps, 2) Internal modules, 3) Relative imports
import { useState } from "react";
import { Button } from "@renderer/components/ui/button";
import { useUsage } from "../hooks/useUsage";

// Use @renderer/ alias for cross-directory imports in renderer process
// Use relative imports (../) for same-feature file imports
```

### Component Patterns

```typescript
// Functional components with named exports
export function MyComponent({ prop }: Props) {
  // Hooks at top level
  const [state, setState] = useState(null)

  // Event handlers before return
  const handleClick = () => { }

  // Conditional rendering with fragments
  return state ? <div /> : <Loading />
}
```

### Naming Conventions

- **Components**: PascalCase (`UsageCard`, `Dashboard`)
- **Functions/variables**: camelCase (`fetchUsage`, `isLoading`)
- **Types/interfaces**: PascalCase (`CopilotUsage`, `AuthState`)
- **Constants**: UPPER_SNAKE_CASE for global, PascalCase for exported
- **Files**: PascalCase for components, camelCase for utilities

### State Management (Zustand)

```typescript
// Store structure: state, actions, selectors
interface State {
  data: Type | null;
  setData: (data: Type | null) => void;
}

export const useStore = create<State>((set) => ({
  data: null,
  setData: (data) => set({ data }),
}));

// Export selectors for computed values
export const useData = () => useStore((state) => state.data);
```

### Error Handling

```typescript
// Always handle errors with try-catch
try {
  await operation();
} catch (err) {
  const message = err instanceof Error ? err.message : "Failed to operation";
  setError(message);
}

// Optional chaining for safe access
const value = data?.nested?.property ?? defaultValue;
```

### Styling (Tailwind + shadcn/ui)

- Use shadcn/ui components as base (`Button`, `Card`, `Progress`)
- Utility-first with Tailwind classes
- Responsive: `grid-cols-1 md:grid-cols-2`
- Dark mode: Use semantic tokens (`text-muted-foreground`, not `text-gray-500`)
- Spacing: Tailwind scale (`p-4`, `gap-6`, `space-y-4`)

### Comments & Documentation

```typescript
/**
 * Brief description of what the file/component does
 * Additional context if needed
 */

// Inline comments only for complex logic
```

### IPC Communication (Main ↔ Renderer)

- **Preload scripts**: Define IPC channels in `src/preload/`
- **Type safety**: Use `window.electron` with TypeScript definitions
- **Async/await**: All IPC calls are asynchronous
- **Error propagation**: Return `{ success: boolean, data?, error? }` pattern

### Code Quality Standards

- **No console.log** in production code (use proper error handling)
- **Pure functions** for data transformations
- **Custom hooks** for reusable stateful logic
- **Constants** for magic numbers and strings
- **Early returns** to reduce nesting
