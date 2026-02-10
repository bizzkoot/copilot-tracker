/**
 * Tauri Window API Type Definitions
 * Provides type-safe access to Tauri window APIs for widget components
 */

// ============================================================================
// Window API Types
// ============================================================================

export interface TauriCurrentWindow {
  setAlwaysOnTop(value: boolean): Promise<void>;
  hide(): Promise<void>;
  show(): Promise<void>;
  getPosition(): Promise<TauriPosition>;
  setPosition(position: TauriPosition): Promise<void>;
  outerPosition(): Promise<TauriPosition>;
}

export interface TauriPosition {
  x: number;
  y: number;
}

// ============================================================================
// Helper Functions
// ============================================================================

/**
 * Get the current Tauri window with type safety
 */
export function getCurrentWindow(): TauriCurrentWindow {
  const tauriWindow = (
    window as unknown as {
      __TAURI__?: {
        window: { getCurrent: () => TauriCurrentWindow };
      };
    }
  ).__TAURI__?.window;
  if (!tauriWindow) {
    throw new Error(
      "Tauri window API is not available. Make sure you are running in a Tauri context.",
    );
  }
  return tauriWindow.getCurrent();
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
