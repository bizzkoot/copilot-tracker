import { resolve } from "path";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

// Vite config for Tauri dev mode
export default defineConfig({
  root: "src/renderer",
  publicDir: "public",
  resolve: {
    alias: {
      "@renderer": resolve(__dirname, "src/renderer/src"),
    },
  },
  plugins: [react()],
  server: {
    port: 5173,
  },
});
