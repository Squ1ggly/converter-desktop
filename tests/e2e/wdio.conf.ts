// Drives the built app through tauri-driver: https://v2.tauri.app/develop/tests/webdriver/
// tauri-driver works on Linux (WebKitWebDriver) and Windows (msedgedriver),
// not on macOS, which has no WebDriver for its webview.
import { spawn, spawnSync, type ChildProcess } from "node:child_process";
import os from "node:os";
import path from "node:path";
import { setTimeout as sleep } from "node:timers/promises";
import { SevereServiceError } from "webdriverio";

const executableSuffix = process.platform === "win32" ? ".exe" : "";
const PROJECT_ROOT = path.resolve(import.meta.dirname, "..", "..");

// A debug build without installers, made in onPrepare.
const APP_PATH = path.join(PROJECT_ROOT, "src-tauri", "target", "debug", `converter${executableSuffix}`);
const TAURI_DRIVER_PATH = path.join(os.homedir(), ".cargo", "bin", `tauri-driver${executableSuffix}`);
const TAURI_DRIVER_PORT = 4444;
const DRIVER_START_TIMEOUT_MS = 15_000;
const DRIVER_POLL_INTERVAL_MS = 200;

// tauri-driver's own capability, which WebdriverIO's types don't know about.
declare global {
  namespace WebdriverIO {
    interface Capabilities {
      "tauri:options"?: { application: string };
    }
  }
}

let tauriDriver: ChildProcess | null = null;

async function isDriverReady(): Promise<boolean> {
  try {
    const response = await fetch(`http://127.0.0.1:${TAURI_DRIVER_PORT}/status`);
    return response.ok;
  } catch {
    return false;
  }
}

async function waitForDriver(): Promise<void> {
  const deadline = Date.now() + DRIVER_START_TIMEOUT_MS;

  while (Date.now() < deadline) {
    if (await isDriverReady()) {
      return;
    }
    await sleep(DRIVER_POLL_INTERVAL_MS);
  }
  throw new Error(`tauri-driver did not start within ${DRIVER_START_TIMEOUT_MS} ms`);
}

function buildApp(): void {
  const buildResult = spawnSync("pnpm", ["tauri", "build", "--debug", "--no-bundle"], {
    cwd: PROJECT_ROOT,
    stdio: "inherit",
    // Finds pnpm.cmd on Windows.
    shell: true,
  });

  if (buildResult.status !== 0) {
    throw new SevereServiceError("Building the app failed");
  }
}

function stopTauriDriver(): void {
  tauriDriver?.kill();
  tauriDriver = null;
}

export const config: WebdriverIO.Config = {
  runner: "local",
  hostname: "127.0.0.1",
  port: TAURI_DRIVER_PORT,
  specs: ["./**/*.e2e.ts"],
  maxInstances: 1,
  capabilities: [
    {
      "tauri:options": { application: APP_PATH },
    },
  ],
  logLevel: "warn",
  reporters: ["spec"],
  framework: "mocha",
  mochaOpts: { ui: "bdd", timeout: 120_000 },

  onPrepare: () => {
    if (process.platform === "darwin") {
      throw new SevereServiceError("tauri-driver doesn't support macOS. Run these tests on Linux or Windows.");
    }
    buildApp();
  },

  beforeSession: async () => {
    tauriDriver = spawn(TAURI_DRIVER_PATH, [], { stdio: ["ignore", "inherit", "inherit"] });
    tauriDriver.on("error", (error) => {
      console.error(`Couldn't start tauri-driver (cargo install tauri-driver --locked): ${error.message}`);
    });
    await waitForDriver();
  },

  afterSession: () => {
    stopTauriDriver();
  },

  // afterSession doesn't run if the session fails to start.
  onComplete: () => {
    stopTauriDriver();
  },
};
