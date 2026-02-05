import { Layout } from "./components/layout/Layout";
import { useTheme } from "./hooks/useTheme";
import { useSettingsSync } from "./hooks/useSettingsSync";

function App(): JSX.Element {
  // Initialize theme
  useTheme();

  // Global settings sync - ensures settings stay in sync between tray and dashboard
  useSettingsSync();

  return <Layout />;
}

export default App;
