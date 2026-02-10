import React from "react";
import ReactDOM from "react-dom/client";
import { Widget } from "./components/widget";
import "./styles/globals.css";
import { initTauriAdapter } from "./tauri-adapter";

// Initialize Tauri Adapter if running in Tauri
initTauriAdapter();

ReactDOM.createRoot(
  document.getElementById("widget-root") as HTMLElement,
).render(
  <React.StrictMode>
    <Widget />
  </React.StrictMode>,
);
