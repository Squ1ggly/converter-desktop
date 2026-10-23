import { expect, fileRow, launchWithFiles, test } from "./support/converter";
import { BEACH_PHOTO, CITY_PHOTO, DOG_VIDEO, HOLIDAY_VIDEO } from "./support/sample-files";

const BEACH_OUTPUT = "/Users/sam/Pictures/beach.jpg";
const HOLIDAY_OUTPUT = "/Users/sam/Movies/holiday.mp4";

test("images and videos are converted as separate batches", async ({ page }) => {
  const backend = await launchWithFiles(page, [BEACH_PHOTO, HOLIDAY_VIDEO, DOG_VIDEO]);

  await page.getByRole("button", { name: "Convert 3 files" }).click();

  await expect(page.getByRole("heading", { name: "3 files converted" })).toBeVisible();
  const { conversionRequests } = await backend.readLog();
  const batchKinds = conversionRequests.map((request) => request.settings.kind);
  const batchSizes = conversionRequests.map((request) => request.files.length);
  expect(batchKinds).toEqual(["image", "video"]);
  expect(batchSizes).toEqual([1, 2]);
});

test("each file shows its progress and the footer counts finished files", async ({ page }) => {
  const backend = await launchWithFiles(page, [BEACH_PHOTO, HOLIDAY_VIDEO, DOG_VIDEO], { conversionMode: "manual" });

  await page.getByRole("button", { name: "Convert 3 files" }).click();
  await backend.startFile("holiday.mov");
  await backend.reportProgress("holiday.mov", 40);

  const holidayRow = fileRow(page, "holiday.mov");
  await expect(holidayRow.getByRole("progressbar", { name: "Converting" })).toBeVisible();
  await expect(holidayRow).toContainText("40%");
  await expect(fileRow(page, "dog.mp4")).toContainText("Queued");
  await expect(page.getByText("Converting 1 of 3")).toBeVisible();

  await backend.completeFile("holiday.mov", HOLIDAY_OUTPUT);

  await expect(page.getByText("Converting 2 of 3")).toBeVisible();
  await expect(holidayRow.getByRole("button", { name: "Open" })).toBeVisible();
});

test("an image without progress shows a spinner while it converts", async ({ page }) => {
  const backend = await launchWithFiles(page, [BEACH_PHOTO], { conversionMode: "manual" });

  await page.getByRole("button", { name: "Convert 1 file" }).click();
  await backend.startFile("beach.png");

  await expect(fileRow(page, "beach.png").getByRole("status", { name: "Converting" })).toBeVisible();
});

test("settings and the file list are locked while converting", async ({ page }) => {
  await launchWithFiles(page, [BEACH_PHOTO, HOLIDAY_VIDEO], { conversionMode: "manual" });

  await page.getByRole("button", { name: "Convert 2 files" }).click();

  await expect(page.getByRole("radio", { name: "PNG" })).toBeDisabled();
  await expect(page.getByRole("combobox", { name: "Resolution" })).toBeDisabled();
  await expect(page.getByRole("radio", { name: "Another folder" })).toBeDisabled();
  await expect(page.getByRole("button", { name: "Clear" })).toBeDisabled();
  await expect(page.getByRole("button", { name: "Add files" })).toBeDisabled();
});

test("Cancel stops every batch and marks unfinished files as cancelled", async ({ page }) => {
  const backend = await launchWithFiles(page, [BEACH_PHOTO, HOLIDAY_VIDEO, CITY_PHOTO], { conversionMode: "manual" });
  await page.getByRole("button", { name: "Convert 3 files" }).click();
  await backend.completeFile("beach.png", BEACH_OUTPUT);

  await page.getByRole("button", { name: "Cancel" }).click();

  await expect(fileRow(page, "holiday.mov")).toContainText("Cancelled");
  await expect(fileRow(page, "city.jpg")).toContainText("Cancelled");
  await expect(page.getByRole("heading", { name: "1 file converted" })).toBeVisible();
  await expect(page.getByText("2 cancelled")).toBeVisible();
  const { conversionRequests, cancelledBatchIds } = await backend.readLog();
  const batchIds = conversionRequests.map((request) => request.batchId);
  expect(new Set(cancelledBatchIds)).toEqual(new Set(batchIds));
});

test("converted files can be opened and shown in Finder", async ({ page }) => {
  const backend = await launchWithFiles(page, [BEACH_PHOTO, HOLIDAY_VIDEO]);

  await page.getByRole("button", { name: "Convert 2 files" }).click();
  await expect(page.getByRole("heading", { name: "2 files converted" })).toBeVisible();
  await fileRow(page, "beach.png").getByRole("button", { name: "Open" }).click();
  await fileRow(page, "holiday.mov").getByRole("button", { name: "Show in Finder" }).click();

  await expect.poll(async () => (await backend.readLog()).openedOutputs).toEqual([BEACH_OUTPUT]);
  await expect.poll(async () => (await backend.readLog()).revealedOutputs).toEqual([HOLIDAY_OUTPUT]);
});

test("the list header shows the first converted file in Finder", async ({ page }) => {
  const backend = await launchWithFiles(page, [BEACH_PHOTO, HOLIDAY_VIDEO]);

  await page.getByRole("button", { name: "Convert 2 files" }).click();
  await page.getByRole("region", { name: "Files" }).getByRole("button", { name: "Show in Finder" }).first().click();

  await expect.poll(async () => (await backend.readLog()).revealedOutputs).toEqual([BEACH_OUTPUT]);
});

test("a failed file shows its error and the others still convert", async ({ page }) => {
  await launchWithFiles(page, [BEACH_PHOTO, HOLIDAY_VIDEO, CITY_PHOTO], {
    failingFiles: { "holiday.mov": "Invalid data found when processing input" },
  });

  await page.getByRole("button", { name: "Convert 3 files" }).click();

  const holidayRow = fileRow(page, "holiday.mov");
  await expect(holidayRow.getByRole("alert")).toHaveText("Invalid data found when processing input");
  await expect(holidayRow).toContainText("Failed");
  await expect(holidayRow.getByRole("button", { name: "Open" })).toHaveCount(0);
  await expect(page.getByRole("heading", { name: "2 files converted" })).toBeVisible();
  await expect(page.getByText("1 failed")).toBeVisible();
});

test("Convert again converts every file a second time", async ({ page }) => {
  const backend = await launchWithFiles(page, [BEACH_PHOTO, CITY_PHOTO], {
    failingFiles: { "city.jpg": "Permission denied" },
  });
  await page.getByRole("button", { name: "Convert 2 files" }).click();
  await expect(page.getByText("1 failed")).toBeVisible();

  await page.getByRole("radio", { name: "WebP" }).click();
  await page.getByRole("button", { name: "Convert again" }).click();

  await expect.poll(async () => (await backend.readLog()).conversionRequests.length).toBe(2);
  const { conversionRequests } = await backend.readLog();
  const secondRun = conversionRequests[1];
  expect(secondRun?.settings.format).toBe("webp");
  expect(secondRun?.files.map((file) => file.path)).toEqual([BEACH_PHOTO, CITY_PHOTO]);
});
