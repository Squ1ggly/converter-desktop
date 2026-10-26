// Failures from the backend and plugins show a toast with a title and the details.
import type { Page } from "@playwright/test";
import { expect, fileRow, launchConverter, launchWithFiles, test } from "./support/converter";
import { BEACH_PHOTO, EXPORT_FOLDER } from "./support/sample-files";

const BEACH_OUTPUT = "/Users/sam/Pictures/beach.jpg";

async function expectErrorToast(page: Page, title: string, details: string): Promise<void> {
  const toast = page.getByRole("listitem").filter({ hasText: title });
  await expect(toast).toBeVisible();
  await expect(toast).toContainText(details);
}

test("a file picker that can't open says so", async ({ page }) => {
  await launchConverter(page, { failingCommands: { "plugin:dialog|open": "No display" } });

  await page.getByRole("button", { name: "Add files" }).click();

  await expectErrorToast(page, "Couldn't open the file picker", "No display");
  await expect(page.getByText("Drop images or videos")).toBeVisible();
});

test("a folder picker that can't open says so", async ({ page }) => {
  await launchConverter(page, { failingCommands: { "plugin:dialog|open": "No display" } });

  await page.getByRole("button", { name: "Add folder" }).click();

  await expectErrorToast(page, "Couldn't open the folder picker", "No display");
});

test("a save folder picker that can't open keeps the old choice", async ({ page }) => {
  await launchWithFiles(page, [BEACH_PHOTO], { failingCommands: { "plugin:dialog|open": "No display" } });

  await page.getByRole("radio", { name: "Another folder" }).click();

  await expectErrorToast(page, "Couldn't open the folder picker", "No display");
  await expect(page.getByRole("radio", { name: "Same folder as original" })).toBeChecked();
  await expect(page.getByText(EXPORT_FOLDER)).toHaveCount(0);
});

test("a converted file that can't be opened says so", async ({ page }) => {
  await launchWithFiles(page, [BEACH_PHOTO], { failingCommands: { open_output: "The file no longer exists" } });
  await page.getByRole("button", { name: "Convert 1 file" }).click();

  await fileRow(page, "beach.png").getByRole("button", { name: "Open" }).click();

  await expectErrorToast(page, "Couldn't open the file", "The file no longer exists");
});

test("a converted file that can't be shown in Finder says so", async ({ page }) => {
  await launchWithFiles(page, [BEACH_PHOTO], { failingCommands: { reveal_output: "The file no longer exists" } });
  await page.getByRole("button", { name: "Convert 1 file" }).click();

  await fileRow(page, "beach.png").getByRole("button", { name: "Show in Finder" }).click();

  await expectErrorToast(page, "Couldn't show the file", "The file no longer exists");
});

test("a conversion that can't be cancelled says so and offers Cancel again", async ({ page }) => {
  const backend = await launchWithFiles(page, [BEACH_PHOTO], {
    conversionMode: "manual",
    failingCommands: { cancel_conversion: "The conversion has already ended" },
  });
  await page.getByRole("button", { name: "Convert 1 file" }).click();

  await page.getByRole("button", { name: "Cancel" }).click();

  await expectErrorToast(page, "Couldn't cancel the conversion", "The conversion has already ended");
  await expect(page.getByRole("button", { name: "Cancel" })).toBeEnabled();
  await backend.completeFile("beach.png", BEACH_OUTPUT);
  await expect(page.getByRole("heading", { name: "1 file converted" })).toBeVisible();
});

test("a Help link that can't open in the browser says so", async ({ page }) => {
  await launchConverter(page, { failingCommands: { "plugin:opener|open_url": "No browser" } });
  await page.getByRole("banner").getByRole("button", { name: "Help" }).click();
  await page.getByRole("navigation", { name: "Help pages" }).getByRole("button", { name: "Saving" }).click();
  await page.getByRole("article").evaluate((article) => {
    article.insertAdjacentHTML("beforeend", '<p><a href="https://example.com/">Example</a></p>');
  });

  await page.getByRole("article").getByRole("link", { name: "Example" }).click();

  await expectErrorToast(page, "Couldn't open the link", "No browser");
});
