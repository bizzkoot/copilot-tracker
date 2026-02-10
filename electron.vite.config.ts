import { resolve } from "path";
import { defineConfig, externalizeDepsPlugin } from "electron-vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  main: {
    plugins: [externalizeDepsPlugin()],
  },
  preload: {
    plugins: [externalizeDepsPlugin()],
  },
  renderer: {
    publicDir: resolve("src/renderer/public"),
    build: {
      rollupOptions: {
        input: {
          index: resolve("src/renderer/index.html"),
          widget: resolve("src/renderer/widget.html"),
        },
      },
    },
    resolve: {
      alias: {
        "@renderer": resolve("src/renderer/src"),
      },
    },
    plugins: [react()],
  },
});
