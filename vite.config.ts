import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";

// Tauri sets TAURI_ENV_PLATFORM while building. Windows renders with
// Chromium-based WebView2; macOS and Linux use WebKit.
const isWindowsBuild = process.env.TAURI_ENV_PLATFORM === "windows";

export default defineConfig({
  plugins: [svelte()],
  // Keep Rust compiler errors visible in the terminal during `tauri dev`.
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
