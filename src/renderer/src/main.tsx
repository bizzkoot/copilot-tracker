import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles/globals.css";
import { initTauriAdapter } from "./tauri-adapter";

// Initialize Tauri Adapter if running in Tauri
// This is now async to wait for Tauri APIs to be available
async function initializeApp() {
  await initTauriAdapter();

  ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
    <React.StrictMode>
      <App />
    </React.StrictMode>,
  );
}

initializeApp();
