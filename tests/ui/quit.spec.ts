import type { Page } from "@playwright/test";
import { expect, launchWithFiles, test, type MockBackend } from "./support/converter";
import { BEACH_PHOTO, HOLIDAY_VIDEO } from "./support/sample-files";

async function startConvertingTwoBatches(page: Page): Promise<MockBackend> {
  const backend = await launchWithFiles(page, [BEACH_PHOTO, HOLIDAY_VIDEO], { conversionMode: "manual" });
  await page.getByRole("button", { name: "Convert 2 files" }).click();
  await expect(page.getByText("Converting 1 of 2")).toBeVisible();
  return backend;
}

test("closing the window while converting asks first", async ({ page }) => {
  const backend = await startConvertingTwoBatches(page);

  await backend.requestWindowClose();

  const quitDialog = page.getByRole("alertdialog", { name: "Quit while converting?" });
  await expect(quitDialog).toBeVisible();
  const { windowDestroys } = await backend.readLog();
  expect(windowDestroys).toEqual([]);
});

test("Keep converting closes the dialog and carries on", async ({ page }) => {
  const backend = await startConvertingTwoBatches(page);
  await backend.requestWindowClose();

  await page.getByRole("button", { name: "Keep converting" }).click();

  await expect(page.getByRole("alertdialog")).toBeHidden();
  await expect(page.getByText("Converting 1 of 2")).toBeVisible();
  await backend.completeFile("beach.png", "/Users/sam/Pictures/beach.jpg");
  await expect(page.getByText("Converting 2 of 2")).toBeVisible();
  const { windowDestroys, cancelledBatchIds } = await backend.readLog();
  expect(windowDestroys).toEqual([]);
  expect(cancelledBatchIds).toEqual([]);
});

test("Quit cancels every batch, waits for them, then closes the window", async ({ page }) => {
  const backend = await startConvertingTwoBatches(page);
  await backend.requestWindowClose();

  await page.getByRole("button", { name: "Quit", exact: true }).click();

  await expect.poll(async () => (await backend.readLog()).windowDestroys).toEqual([{ runningBatchCount: 0 }]);
  const { conversionRequests, cancelledBatchIds } = await backend.readLog();
  const batchIds = conversionRequests.map((request) => request.batchId);
  expect(new Set(cancelledBatchIds)).toEqual(new Set(batchIds));
  expect(cancelledBatchIds).toHaveLength(2);
});

test("closing the window when nothing is converting just closes it", async ({ page }) => {
  const backend = await launchWithFiles(page, [BEACH_PHOTO]);

  await backend.requestWindowClose();

  await expect.poll(async () => (await backend.readLog()).windowDestroys).toEqual([{ runningBatchCount: 0 }]);
  await expect(page.getByRole("alertdialog")).toBeHidden();
});

test("closing the window after a finished run just closes it", async ({ page }) => {
  const backend = await launchWithFiles(page, [BEACH_PHOTO]);
  await page.getByRole("button", { name: "Convert 1 file" }).click();
  await expect(page.getByRole("heading", { name: "1 file converted" })).toBeVisible();

  await backend.requestWindowClose();

  await expect.poll(async () => (await backend.readLog()).windowDestroys).toHaveLength(1);
  await expect(page.getByRole("alertdialog")).toBeHidden();
});
