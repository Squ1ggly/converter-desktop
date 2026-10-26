import { defineConfig, devices } from "@playwright/test";

const isCi = Boolean(process.env.CI);

// Not the app's own dev port (1420), so `pnpm dev` can keep running alongside. Set
// UI_TEST_PORT to run the tests of two checkouts at once; otherwise one reuses the other's server.
const UI_TEST_PORT = Number(process.env.UI_TEST_PORT ?? 1430);
const baseUrl = `http://localhost:${UI_TEST_PORT}`;

// The same size as the app's main window (src-tauri/src/lib.rs).
const WINDOW_SIZE = { width: 1080, height: 720 };

export default defineConfig({
  testDir: "tests/ui",
  fullyParallel: true,
  forbidOnly: isCi,
  retries: isCi ? 1 : 0,
  reporter: isCi ? [["github"], ["list"]] : "list",
  use: {
    baseURL: baseUrl,
    viewport: WINDOW_SIZE,
    trace: "retain-on-failure",
  },
  // macOS and Linux render the app in WebKit, Windows in WebView2 (Chromium).
  projects: [
    {
      name: "webkit",
      use: { ...devices["Desktop Safari"], viewport: WINDOW_SIZE },
      testIgnore: "screenshots.spec.ts",
    },
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"], viewport: WINDOW_SIZE },
      testIgnore: "screenshots.spec.ts",
    },
    {
      name: "screenshots",
      use: { ...devices["Desktop Safari"], viewport: WINDOW_SIZE, deviceScaleFactor: 2 },
      testMatch: "screenshots.spec.ts",
    },
  ],
  webServer: {
    command: `pnpm exec vite --config tests/ui/vite.config.ts --port ${UI_TEST_PORT}`,
    url: baseUrl,
    reuseExistingServer: !isCi,
  },
});
