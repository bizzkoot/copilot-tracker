import { ElectronAPI } from '@electron-toolkit/preload'

interface CustomElectronAPI extends ElectronAPI {
  setSettings: (settings: { launchAtLogin?: boolean; refreshInterval?: number; predictionPeriod?: number; theme?: 'light' | 'dark' | 'system'; notifications?: { enabled?: boolean; thresholds?: number[] } }) => Promise<void>
}

declare global {
  interface Window {
    electron: CustomElectronAPI
    api: unknown
  }
}
