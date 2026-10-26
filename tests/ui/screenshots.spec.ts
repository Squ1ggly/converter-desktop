// Saves PNGs of the main states in light and dark mode for design review.
// Run with `pnpm screenshots`; set SCREENSHOT_DIR to choose where they go.
import path from "node:path";
import type { Page } from "@playwright/test";
import { expect, fileRow, launchConverter, launchWithFiles, test } from "./support/converter";
import { BEACH_PHOTO, CITY_PHOTO, DOG_VIDEO, HOLIDAY_VIDEO } from "./support/sample-files";

const SCREENSHOT_DIR = process.env.SCREENSHOT_DIR ?? "test-results/screenshots";
const MIXED_FILES = [BEACH_PHOTO, CITY_PHOTO, HOLIDAY_VIDEO, DOG_VIDEO];
const COLOR_SCHEMES = ["light", "dark"] as const;
const LONG_NAME = "IMG_20260712_183455_holiday_in_the_mountains_with_the_whole_family_final_edit_v3.png";
const LONG_PHOTO = `/Users/sam/Pictures/${LONG_NAME}`;
const LONG_ERROR = "Invalid data found when processing input. The file may be damaged or only partly downloaded.";
const LONG_FOLDER = "/Users/sam/Library/Mobile Documents/com~apple~CloudDocs/Projects/2026/Client work/Exports";

async function saveScreenshot(page: Page, stateName: string, colorScheme: string): Promise<void> {
  // Keeps the last clicked button out of its hover state.
  await page.mouse.move(0, 0);
  await page.evaluate(() => document.fonts.ready);
  await page.screenshot({
    path: path.join(SCREENSHOT_DIR, `${stateName}-${colorScheme}.png`),
    animations: "disabled",
  });
}

for (const colorScheme of COLOR_SCHEMES) {
  test.describe(`${colorScheme} mode`, () => {
    test.use({ colorScheme });

    test("empty state", async ({ page }) => {
      await launchConverter(page);

      await saveScreenshot(page, "1-empty", colorScheme);
    });

    test("files added", async ({ page }) => {
      await launchWithFiles(page, MIXED_FILES);
      await expect(fileRow(page, "dog.mp4")).toBeVisible();

      await saveScreenshot(page, "2-files-added", colorScheme);
    });

    test("image settings with Fit", async ({ page }) => {
      await launchWithFiles(page, [BEACH_PHOTO, CITY_PHOTO]);
      await page.getByRole("radio", { name: "Fit" }).click();
      await page.getByRole("spinbutton", { name: "Max width" }).fill("1920");

      await saveScreenshot(page, "3-image-settings-fit", colorScheme);
    });

    test("converting", async ({ page }) => {
      const backend = await launchWithFiles(page, MIXED_FILES, { conversionMode: "manual" });
      await page.getByRole("button", { name: "Convert 4 files" }).click();
      await backend.completeFile("beach.png", "/Users/sam/Pictures/beach.jpg");
      await backend.startFile("city.jpg");
      await backend.startFile("holiday.mov");
      await backend.reportProgress("holiday.mov", 62);
      await expect(page.getByText("Converting 2 of 4")).toBeVisible();

      await saveScreenshot(page, "4-converting", colorScheme);
    });

    test("finished with one failure", async ({ page }) => {
      await launchWithFiles(page, MIXED_FILES, {
        failingFiles: { "dog.mp4": "Invalid data found when processing input" },
      });
      await page.getByRole("button", { name: "Convert 4 files" }).click();
      await expect(page.getByText("1 failed")).toBeVisible();

      await saveScreenshot(page, "5-finished-with-failure", colorScheme);
    });

    test("drag overlay", async ({ page }) => {
      const backend = await launchWithFiles(page, MIXED_FILES);
      await backend.dragFilesOver(["/Users/sam/Pictures/forest.png"]);
      await expect(page.getByText("Drop to add")).toBeVisible();

      await saveScreenshot(page, "6-drag-overlay", colorScheme);
    });

    test("settings page", async ({ page }) => {
      await launchConverter(page);
      await page.getByRole("banner").getByRole("button", { name: "Settings" }).click();
      await expect(page.getByRole("heading", { name: "Appearance" })).toBeVisible();

      await saveScreenshot(page, "7-settings", colorScheme);
    });

    test("help page", async ({ page }) => {
      await launchConverter(page);
      await page.getByRole("banner").getByRole("button", { name: "Help" }).click();
      await expect(page.getByRole("article").getByRole("heading", { level: 1 })).toBeVisible();

      await saveScreenshot(page, "8-help", colorScheme);
    });

    test("adding a large folder", async ({ page }) => {
      const backend = await launchWithFiles(page, [BEACH_PHOTO], { folders: { "/Users/sam/Pictures": [CITY_PHOTO] } });
      await backend.holdScans();
      await backend.dropFiles(["/Users/sam/Pictures"]);
      await expect(page.getByText("Adding files")).toBeVisible();

      await saveScreenshot(page, "9-adding-files", colorScheme);
    });

    test("saving to a chosen folder", async ({ page }) => {
      const backend = await launchWithFiles(page, [BEACH_PHOTO]);
      await backend.answerNextDialog(LONG_FOLDER);
      await page.getByRole("radio", { name: "Another folder" }).click();
      await page.getByRole("complementary", { name: "Settings" }).locator("[data-slot=scroll-area-viewport]").evaluate((viewport) => {
        viewport.scrollTop = viewport.scrollHeight;
      });

      await saveScreenshot(page, "10-save-to-folder", colorScheme);
    });

    test("smallest window with long names", async ({ page }) => {
      await page.setViewportSize({ width: 760, height: 540 });
      await launchWithFiles(page, [LONG_PHOTO, ...MIXED_FILES], { failingFiles: { [LONG_NAME]: LONG_ERROR } });
      await page.getByRole("button", { name: "Convert 5 files" }).click();
      await expect(page.getByText("1 failed")).toBeVisible();

      await saveScreenshot(page, "11-smallest-window", colorScheme);
    });

    test("quitting while converting", async ({ page }) => {
      const backend = await launchWithFiles(page, MIXED_FILES, { conversionMode: "manual" });
      await page.getByRole("button", { name: "Convert 4 files" }).click();
      await backend.requestWindowClose();
      await expect(page.getByRole("alertdialog")).toBeVisible();

      await saveScreenshot(page, "12-quit-dialog", colorScheme);
    });
  });
}
