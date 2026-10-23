import path from "node:path";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";

// Windows builds render in WebView2 (Chromium); macOS and Linux use WebKit.
const isWindowsBuild = process.env.TAURI_ENV_PLATFORM === "windows";

export default defineConfig({
  plugins: [tailwindcss(), svelte()],
  resolve: {
    alias: { $lib: path.resolve(import.meta.dirname, "src/lib") },
  },
  clearScreen: false,
  server: {
    // Must match build.devUrl in src-tauri/tauri.conf.json.
    port: 1420,
    strictPort: true,
    watch: { ignored: ["**/src-tauri/**"] },
  },
  build: {
    target: isWindowsBuild ? "chrome111" : "safari15",
    outDir: "dist",
    emptyOutDir: true,
  },
});
