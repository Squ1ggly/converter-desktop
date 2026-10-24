import type { Page } from "@playwright/test";
import { expect, launchConverter, launchWithFiles, test } from "./support/converter";
import { BEACH_PHOTO, EXPORT_FOLDER, HOLIDAY_VIDEO } from "./support/sample-files";

async function openSettings(page: Page): Promise<void> {
  await page.getByRole("banner").getByRole("button", { name: "Settings" }).click();
  await expect(page.getByRole("heading", { name: "Appearance" })).toBeVisible();
}

test("Theme switches between light and dark", async ({ page }) => {
  await launchConverter(page);
  await openSettings(page);
  const html = page.locator("html");
  await expect(page.getByRole("radio", { name: "System" })).toBeChecked();

  await page.getByRole("radio", { name: "Dark" }).click();
  await expect(html).toHaveClass(/\bdark\b/);

  await page.getByRole("radio", { name: "Light" }).click();
  await expect(html).not.toHaveClass(/\bdark\b/);
});

test("System theme follows the operating system", async ({ page }) => {
  await launchConverter(page);
  await openSettings(page);
  await page.getByRole("radio", { name: "Light" }).click();

  await page.emulateMedia({ colorScheme: "dark" });
  await page.getByRole("radio", { name: "System" }).click();

  await expect(page.locator("html")).toHaveClass(/\bdark\b/);
});

test("defaults set on Settings are used by the side panel and the conversion", async ({ page }) => {
  const backend = await launchConverter(page);
  await openSettings(page);
  await page.getByRole("region", { name: "Images" }).getByRole("radio", { name: "PNG" }).click();
  await page.getByRole("region", { name: "Videos" }).getByRole("radio", { name: "Small" }).click();
  await backend.answerNextDialog(EXPORT_FOLDER);
  await page.getByRole("radio", { name: "Another folder" }).click();
  await expect(page.getByText(EXPORT_FOLDER)).toBeVisible();

  await page.getByRole("button", { name: "Back" }).click();
  await backend.dropFiles([BEACH_PHOTO, HOLIDAY_VIDEO]);

  const sidePanel = page.getByRole("complementary", { name: "Settings" });
  await expect(sidePanel.getByRole("radio", { name: "PNG" })).toBeChecked();
  await expect(sidePanel.getByRole("radio", { name: "Small" })).toBeChecked();
  await expect(sidePanel.getByRole("radio", { name: "Another folder" })).toBeChecked();
  await sidePanel.getByRole("button", { name: "Convert 2 files" }).click();
  await expect(page.getByRole("heading", { name: "2 files converted" })).toBeVisible();
  const { conversionRequests } = await backend.readLog();
  expect(conversionRequests.map((request) => request.settings.format)).toEqual(["png", "mp4"]);
  expect(conversionRequests[1]?.settings.options).toMatchObject({ quality: "small" });
  expect(conversionRequests.map((request) => request.outputDirectory)).toEqual([EXPORT_FOLDER, EXPORT_FOLDER]);
});

test("changes in the side panel show on Settings", async ({ page }) => {
  await launchWithFiles(page, [BEACH_PHOTO]);
  await page.getByRole("complementary", { name: "Settings" }).getByRole("radio", { name: "Fit" }).click();

  await openSettings(page);

  await expect(page.getByRole("region", { name: "Images" }).getByRole("radio", { name: "Fit" })).toBeChecked();
});

test("conversion settings are locked while converting, the theme is not", async ({ page }) => {
  await launchWithFiles(page, [BEACH_PHOTO, HOLIDAY_VIDEO], { conversionMode: "manual" });
  await page.getByRole("button", { name: "Convert 2 files" }).click();

  await openSettings(page);

  await expect(page.getByRole("region", { name: "Images" }).getByRole("radio", { name: "PNG" })).toBeDisabled();
  await expect(page.getByRole("region", { name: "Videos" }).getByRole("combobox", { name: "Resolution" })).toBeDisabled();
  await expect(page.getByRole("radio", { name: "Another folder" })).toBeDisabled();
  await expect(page.getByRole("radio", { name: "Dark" })).toBeEnabled();
});
