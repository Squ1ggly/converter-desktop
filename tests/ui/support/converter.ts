// What the UI tests import: a `test` that fails on uncaught page errors, and
// launchConverter(), which opens the app against the mock backend.
import { test as base, expect, type Locator, type Page } from "@playwright/test";
import type { MediaKind } from "$lib/backend";
import type { BackendLog, FileConversionEvent, MockScenario } from "./scenario";

export { expect };

const DEFAULT_SCENARIO: MockScenario = {
  platform: "macos",
  folders: {},
  savedPreferences: null,
  conversionMode: "automatic",
  failingFiles: {},
  failingCommands: {},
  holdsCatalog: false,
};

export const test = base.extend<{ failOnPageErrors: void }>({
  failOnPageErrors: [
    async ({ page }, use) => {
      const pageErrors: string[] = [];
      page.on("pageerror", (error) => pageErrors.push(error.message));

      await use();

      expect(pageErrors, "uncaught errors in the page").toEqual([]);
    },
    { auto: true },
  ],
});

// Plays the backend's part: native dialogs, file drops and conversion events.
export class MockBackend {
  readonly #page: Page;

  constructor(page: Page) {
    this.#page = page;
  }

  // The next native file or folder dialog returns these paths. null means the user cancelled.
  async answerNextDialog(dialogResult: string[] | string | null): Promise<void> {
    await this.#page.evaluate((result) => window.converterMockBackend.setNextDialogResult(result), dialogResult);
  }

  async dragFilesOver(paths: string[]): Promise<void> {
    await this.#page.evaluate((draggedPaths) => window.converterMockBackend.emitDragEvent("enter", draggedPaths), paths);
  }

  async dragAway(): Promise<void> {
    await this.#page.evaluate(() => window.converterMockBackend.emitDragEvent("leave", []));
  }

  async dropFiles(paths: string[]): Promise<void> {
    await this.dragFilesOver(paths);
    await this.#page.evaluate((droppedPaths) => window.converterMockBackend.emitDragEvent("drop", droppedPaths), paths);
  }

  async startFile(fileName: string): Promise<void> {
    await this.#sendFileEvent(fileName, { type: "started" });
  }

  async reportProgress(fileName: string, percent: number): Promise<void> {
    await this.#sendFileEvent(fileName, { type: "progress", percent });
  }

  async completeFile(fileName: string, outputPath: string): Promise<void> {
    await this.#sendFileEvent(fileName, { type: "completed", outputPath });
  }

  async failFile(fileName: string, message: string): Promise<void> {
    await this.#sendFileEvent(fileName, { type: "failed", message });
  }

  async failBatch(kind: MediaKind, message: string): Promise<void> {
    await this.#page.evaluate(
      ({ batchKind, errorMessage }) => window.converterMockBackend.failBatch(batchKind, errorMessage),
      { batchKind: kind, errorMessage: message },
    );
  }

  async endBatchEarly(kind: MediaKind): Promise<void> {
    await this.#page.evaluate((batchKind) => window.converterMockBackend.endBatchEarly(batchKind), kind);
  }

  async holdScans(): Promise<void> {
    await this.#page.evaluate(() => window.converterMockBackend.holdScans());
  }

  async releaseScans(): Promise<void> {
    await this.#page.evaluate(() => window.converterMockBackend.releaseScans());
  }

  async releaseCatalog(): Promise<void> {
    await this.#page.evaluate(() => window.converterMockBackend.releaseCatalog());
  }

  // Like clicking the window's close button.
  async requestWindowClose(): Promise<void> {
    await this.#page.evaluate(() => window.converterMockBackend.requestWindowClose());
  }

  async requestQuitFromMenu(): Promise<void> {
    await this.#page.evaluate(() => window.converterMockBackend.requestQuitFromMenu());
  }

  async readLog(): Promise<BackendLog> {
    const backendLog = await this.#page.evaluate(() => window.converterMockBackend.readLog());
    return backendLog;
  }

  async #sendFileEvent(fileName: string, event: FileConversionEvent): Promise<void> {
    await this.#page.evaluate(
      ({ name, fileEvent }) => window.converterMockBackend.sendFileEvent(name, fileEvent),
      { name: fileName, fileEvent: event },
    );
  }
}

// Opens the app without waiting for it to finish starting.
export async function openConverter(page: Page, scenarioChanges: Partial<MockScenario> = {}): Promise<MockBackend> {
  const scenario: MockScenario = { ...DEFAULT_SCENARIO, ...scenarioChanges };
  await page.addInitScript((initialScenario) => {
    window.converterScenario = initialScenario;
  }, scenario);

  await page.goto("/");

  const backend = new MockBackend(page);
  return backend;
}

// Opens the app and waits until it has loaded the catalog and preferences.
export async function launchConverter(page: Page, scenarioChanges: Partial<MockScenario> = {}): Promise<MockBackend> {
  const backend = await openConverter(page, scenarioChanges);
  await expect(page.getByText("Drop images or videos")).toBeVisible();
  return backend;
}

export async function launchWithFiles(
  page: Page,
  paths: string[],
  scenarioChanges: Partial<MockScenario> = {},
): Promise<MockBackend> {
  const backend = await launchConverter(page, scenarioChanges);
  await backend.dropFiles(paths);
  await expect(page.getByRole("region", { name: "Files" })).toBeVisible();
  return backend;
}

export function fileRow(page: Page, fileName: string): Locator {
  const row = page.getByRole("region", { name: "Files" }).getByRole("listitem").filter({ hasText: fileName });
  return row;
}
