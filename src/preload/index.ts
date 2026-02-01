import { contextBridge, ipcRenderer } from "electron";

// Electron API exposed to renderer
const electronAPI = {
  // Platform info
  platform: process.platform,

  // Auth
  login: (): void => ipcRenderer.send("auth:login"),
  logout: (): void => ipcRenderer.send("auth:logout"),
  checkAuth: (): void => ipcRenderer.send("auth:check"),
  onAuthStateChanged: (callback: (state: string) => void): (() => void) => {
    const listener = (_event: Electron.IpcRendererEvent, state: string): void =>
      callback(state);
    ipcRenderer.on("auth:state-changed", listener);
    return () => ipcRenderer.removeListener("auth:state-changed", listener);
  },
  onSessionExpired: (callback: () => void): (() => void) => {
    const listener = (): void => callback();
    ipcRenderer.on("auth:session-expired", listener);
    return () => ipcRenderer.removeListener("auth:session-expired", listener);
  },

  // Usage
  fetchUsage: (): void => ipcRenderer.send("usage:fetch"),
  refreshUsage: (): void => ipcRenderer.send("usage:refresh"),
  onUsageData: (callback: (data: unknown) => void): (() => void) => {
    const listener = (_event: Electron.IpcRendererEvent, data: unknown): void =>
      callback(data);
    ipcRenderer.on("usage:data", listener);
    return () => ipcRenderer.removeListener("usage:data", listener);
  },
  onUsageLoading: (callback: (loading: boolean) => void): (() => void) => {
    const listener = (
      _event: Electron.IpcRendererEvent,
      loading: boolean,
    ): void => callback(loading);
    ipcRenderer.on("usage:loading", listener);
    return () => ipcRenderer.removeListener("usage:loading", listener);
  },

  // Settings
  getSettings: (): Promise<unknown> => ipcRenderer.invoke("settings:get"),
  setSettings: (settings: unknown): Promise<void> =>
    ipcRenderer.invoke("settings:set", settings),
  resetSettings: (): Promise<void> => ipcRenderer.invoke("settings:reset"),
  onSettingsChanged: (callback: (settings: unknown) => void): (() => void) => {
    const listener = (
      _event: Electron.IpcRendererEvent,
      settings: unknown,
    ): void => callback(settings);
    ipcRenderer.on("settings:changed", listener);
    return () => ipcRenderer.removeListener("settings:changed", listener);
  },

  // App
  quit: (): void => ipcRenderer.send("app:quit"),
  showWindow: (): void => ipcRenderer.send("window:show"),
  hideWindow: (): void => ipcRenderer.send("window:hide"),
  onNavigate: (callback: (route: string) => void): (() => void) => {
    const listener = (_event: Electron.IpcRendererEvent, route: string): void =>
      callback(route);
    ipcRenderer.on("navigate", listener);
    return () => ipcRenderer.removeListener("navigate", listener);
  },
};

// Expose APIs to renderer only if context isolation is enabled
if (process.contextIsolated) {
  try {
    contextBridge.exposeInMainWorld("electron", electronAPI);
  } catch (error) {
    console.error("Failed to expose electron API:", error);
  }
} else {
  // @ts-ignore (define in dts)
  window.electron = electronAPI;
}
