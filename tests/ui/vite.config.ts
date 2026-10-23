// Serves the real UI for the Playwright tests, with the mock backend loaded
// before the app's own entry point.
import { defineConfig, mergeConfig, type Plugin } from "vite";
import appConfig from "../../vite.config.ts";

const MOCK_BACKEND_PATH = "/tests/ui/support/mock-backend.ts";

function loadMockBackendFirst(): Plugin {
  const plugin: Plugin = {
    name: "load-mock-backend-first",
    transformIndexHtml: () => [
      { tag: "script", attrs: { type: "module", src: MOCK_BACKEND_PATH }, injectTo: "head-prepend" },
    ],
  };
  return plugin;
}

export default mergeConfig(
  appConfig,
  defineConfig({
    plugins: [loadMockBackendFirst()],
    // Pre-bundled up front, so Vite doesn't reload the page when a test first imports it.
    optimizeDeps: { include: ["@tauri-apps/api/mocks", "@tauri-apps/api/event"] },
  }),
);
