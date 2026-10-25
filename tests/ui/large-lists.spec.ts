import type { Locator, Page } from "@playwright/test";
import { expect, fileRow, launchConverter, test, type MockBackend } from "./support/converter";

const TRIP_FOLDER = "/Users/sam/Pictures/Trip";
// Far more rows than fit on screen, so only the visible ones may be drawn.
const MAX_RENDERED_ROWS = 40;

function photoPaths(count: number): string[] {
  const paths = Array.from({ length: count }, (_, index) => `${TRIP_FOLDER}/photo-${String(index + 1).padStart(4, "0")}.jpg`);
  return paths;
}

async function countOverlappingRows(list: Locator): Promise<number> {
  const rowBoxes = await list
    .getByRole("listitem")
    .evaluateAll((rows) => rows.map((row) => row.getBoundingClientRect()).map((box) => ({ top: box.top, bottom: box.bottom })));
  const sortedBoxes = [...rowBoxes].sort((first, second) => first.top - second.top);
  const overlaps = sortedBoxes.filter((box, index) => index > 0 && box.top < (sortedBoxes[index - 1]?.bottom ?? 0) - 1);
  return overlaps.length;
}

function fileList(page: Page): Locator {
  const list = page.getByRole("region", { name: "Files" }).getByRole("list");
  return list;
}

// `listedCount` is how many the list shows, when the backend's file limit cuts the folder short.
async function addFolderOf(page: Page, fileCount: number, listedCount = fileCount): Promise<MockBackend> {
  const backend = await launchConverter(page, { folders: { [TRIP_FOLDER]: photoPaths(fileCount) } });
  await backend.dropFiles([TRIP_FOLDER]);
  await expect(page.getByRole("heading", { name: `${listedCount.toLocaleString("en-US")} files` })).toBeVisible();
  return backend;
}

// Rows are measured as they're drawn, so the list can grow while scrolling, as it does for a user.
async function scrollListTo(page: Page, fileName: string): Promise<void> {
  const viewport = page.getByRole("region", { name: "Files" }).locator("[data-slot=scroll-area-viewport]");
  await expect(async () => {
    await viewport.evaluate((element) => {
      element.scrollTop = element.scrollHeight;
    });
    await expect(fileRow(page, fileName)).toBeInViewport({ timeout: 500 });
  }).toPass();
}

test("a thousand files only draw the rows in view", async ({ page }) => {
  await addFolderOf(page, 1000);
  await expect(page.getByRole("button", { name: "Convert 1,000 files" })).toBeVisible();

  const rows = fileList(page).getByRole("listitem");
  expect(await rows.count()).toBeLessThan(MAX_RENDERED_ROWS);
  await expect(rows.first()).toHaveAttribute("aria-setsize", "1000");
  await expect(page.locator("img")).toHaveCount(await rows.count());
});

test("scrolling a long list reaches the last file", async ({ page }) => {
  await addFolderOf(page, 500);

  await scrollListTo(page, "photo-0500.jpg");

  await expect(fileRow(page, "photo-0500.jpg")).toHaveAttribute("aria-posinset", "500");
});

test("a file far down a long list can be removed", async ({ page }) => {
  await addFolderOf(page, 300);
  await scrollListTo(page, "photo-0300.jpg");

  await fileRow(page, "photo-0299.jpg").getByRole("button", { name: "Remove" }).click();

  await expect(page.getByRole("heading", { name: "299 files" })).toBeVisible();
  await expect(fileRow(page, "photo-0299.jpg")).toHaveCount(0);
  await expect(fileRow(page, "photo-0300.jpg")).toBeVisible();
});

test("hundreds of files convert and every result is counted", async ({ page }) => {
  const backend = await addFolderOf(page, 300);

  await page.getByRole("button", { name: "Convert 300 files" }).click();

  await expect(page.getByRole("heading", { name: "300 files converted" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Convert again" })).toBeVisible();
  await scrollListTo(page, "photo-0300.jpg");
  await expect(fileRow(page, "photo-0300.jpg").getByRole("button", { name: "Open" })).toBeVisible();
  const { conversionRequests } = await backend.readLog();
  expect(conversionRequests[0]?.files).toHaveLength(300);
});

test("rows with long errors are measured, so they never overlap", async ({ page }) => {
  const longError = "Invalid data found when processing input. ".repeat(6).trim();
  const paths = photoPaths(30);
  const failingFiles = Object.fromEntries(paths.slice(0, 10).map((filePath) => [filePath.split("/").at(-1) ?? "", longError]));
  const backend = await launchConverter(page, { folders: { [TRIP_FOLDER]: paths }, failingFiles });
  await backend.dropFiles([TRIP_FOLDER]);
  await page.getByRole("button", { name: "Convert 30 files" }).click();
  await expect(page.getByRole("heading", { name: "20 files converted" })).toBeVisible();

  await expect.poll(async () => countOverlappingRows(fileList(page))).toBe(0);
  await expect(fileRow(page, "photo-0001.jpg").getByRole("alert")).toHaveText(longError);
});

test("a folder over the file limit adds the first 10,000 files and says so", async ({ page }) => {
  await addFolderOf(page, 10_001, 10_000);

  const toast = page.getByRole("listitem").filter({ hasText: "Only the first 10,000 files were added" });
  await expect(toast).toBeVisible();
  await expect(toast).toContainText("Add the rest from smaller folders.");
});

test("a folder under the file limit adds every file without a warning", async ({ page }) => {
  await addFolderOf(page, 1000);

  await expect(page.getByText("Only the first 10,000 files were added")).toHaveCount(0);
});
