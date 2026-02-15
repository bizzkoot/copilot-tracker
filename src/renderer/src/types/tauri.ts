/**
 * Tauri Window API Type Definitions
 * Provides type-safe access to Tauri window APIs for widget components
 */

// ============================================================================
// Window API Types
// ============================================================================

import { PhysicalPosition } from "@tauri-apps/api/window";

export interface TauriCurrentWindow {
  setAlwaysOnTop(value: boolean): Promise<void>;
  hide(): Promise<void>;
  show(): Promise<void>;
  outerPosition(): Promise<PhysicalPosition>;
  setPosition(position: PhysicalPosition): Promise<void>;
}

interface TauriInternals {
  metadata?: {
    currentWindow?: TauriCurrentWindow;
  };
}

function getTauriInternals(): TauriInternals | null {
  const internals = (window as unknown as { __TAURI_INTERNALS__?: unknown })
    .__TAURI_INTERNALS__;
  if (!internals || typeof internals !== "object") {
    return null;
  }
  return internals as TauriInternals;
}

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Get the current Tauri window with type safety
 * Tauri 2.x uses a different API than v1
 */
export function getCurrentWindow(): TauriCurrentWindow {
  // Tauri 2.x: Try the new API first (from @tauri-apps/api/window)
  const internals = getTauriInternals();
  if (internals?.metadata?.currentWindow) {
    return internals.metadata.currentWindow;
  }

  // Fallback: Try to access via Tauri v1 API for backwards compatibility
  const tauriWindow = (
    window as unknown as {
      __TAURI__?: {
        window?: { getCurrent?: () => TauriCurrentWindow };
      };
    }
  ).__TAURI__?.window;

  if (tauriWindow && typeof tauriWindow.getCurrent === "function") {
    return tauriWindow.getCurrent();
  }

  // Last resort: Check if we're in a mock/testing environment
  const hasElectron = Boolean(
    (window as unknown as { electron?: unknown }).electron,
  );
  if (process.env.NODE_ENV === "development" || hasElectron) {
    console.warn("Tauri window API not available, using mock");
    // Return a mock implementation for development
    return {
      setAlwaysOnTop: async () => {},
      hide: async () => {},
      show: async () => {},
      outerPosition: async () => new PhysicalPosition({ x: 100, y: 100 }),
      setPosition: async () => {},
    };
  }

  throw new Error(
    "Tauri window API is not available. Make sure you are running in a Tauri context.",
  );
}

/**
 * Invoke a Tauri command with type safety
 */
export async function invoke<T>(cmd: string, args?: unknown): Promise<T> {
  const tauriCore = (
    window as unknown as {
      __TAURI__?: {
        core: { invoke: <U>(cmd: string, args?: unknown) => Promise<U> };
      };
    }
  ).__TAURI__?.core;
  if (!tauriCore) {
    throw new Error(
      "Tauri core API is not available. Make sure you are running in a Tauri context.",
    );
  }
  return await tauriCore.invoke<T>(cmd, args);
}

/**
 * Listen to a Tauri event with type safety
 */
export async function listen<T>(
  event: string,
  handler: (event: { payload: T }) => void,
): Promise<() => void> {
  const tauriEvent = (
    window as unknown as {
      __TAURI__?: {
        event: {
          listen: <U>(
            event: string,
            handler: (event: { payload: U }) => void,
          ) => Promise<() => void>;
        };
      };
    }
  ).__TAURI__?.event;
  if (!tauriEvent) {
    throw new Error(
      "Tauri event API is not available. Make sure you are running in a Tauri context.",
    );
  }
  return await tauriEvent.listen<T>(event, handler);
}

/**
 * Emit a Tauri event with type safety
 */
export async function emit(event: string, payload?: unknown): Promise<void> {
  const tauriEvent = (
    window as unknown as {
      __TAURI__?: {
        event: { emit: (event: string, payload?: unknown) => Promise<void> };
      };
    }
  ).__TAURI__?.event;
  if (!tauriEvent) {
    throw new Error(
      "Tauri event API is not available. Make sure you are running in a Tauri context.",
    );
  }
  await tauriEvent.emit(event, payload);
}
