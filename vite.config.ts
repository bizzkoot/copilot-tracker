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
  build: {
    outDir: "../../out/renderer",
    rollupOptions: {
      input: {
        index: resolve(__dirname, "src/renderer/index.html"),
        widget: resolve(__dirname, "src/renderer/widget.html"),
      },
    },
  },
  plugins: [react()],
  server: {
    port: 5173,
  },
});
