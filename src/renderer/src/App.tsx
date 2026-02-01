import { Layout } from "./components/layout/Layout";
import { useTheme } from "./hooks/useTheme";

function App(): JSX.Element {
  // Initialize theme
  useTheme();

  return <Layout />;
}

export default App;
