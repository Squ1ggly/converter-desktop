import type { Locator, Page } from "@playwright/test";
import { expect, fileRow, launchWithFiles, test } from "./support/converter";
import { HOLIDAY_VIDEO } from "./support/sample-files";

function videoSettings(page: Page): Locator {
  const settingsSection = page.getByRole("region", { name: "Videos" });
  return settingsSection;
}

async function chooseOption(page: Page, selectName: string, optionName: string): Promise<void> {
  await videoSettings(page).getByRole("combobox", { name: selectName }).click();
  await page.getByRole("option", { name: optionName }).click();
}

test("the format picker offers the video formats from the catalog", async ({ page }) => {
  await launchWithFiles(page, [HOLIDAY_VIDEO]);
  const formats = videoSettings(page).getByRole("group", { name: "Format" }).getByRole("radio");

  await expect(formats).toHaveText(["MP4", "MOV", "MKV", "WebM", "M4V", "AVI", "FLV", "GIF"]);
  await expect(videoSettings(page).getByRole("radio", { name: "MP4" })).toBeChecked();
});

test("quality defaults to Balanced and offers High and Small", async ({ page }) => {
  await launchWithFiles(page, [HOLIDAY_VIDEO]);
  const quality = videoSettings(page).getByRole("group", { name: "Quality" }).getByRole("radio");

  await expect(quality).toHaveText(["High", "Balanced", "Small"]);
  await expect(videoSettings(page).getByRole("radio", { name: "Balanced" })).toBeChecked();
});

test("GIF hides quality and explains its default frame rate", async ({ page }) => {
  await launchWithFiles(page, [HOLIDAY_VIDEO]);

  await videoSettings(page).getByRole("radio", { name: "GIF" }).click();

  await expect(videoSettings(page).getByRole("group", { name: "Quality" })).toHaveCount(0);
  await expect(videoSettings(page).getByText("GIFs default to 15 fps.")).toBeVisible();
  await expect(fileRow(page, "holiday.mov")).toContainText("GIF");
});

test("choosing a frame rate for a GIF hides the hint", async ({ page }) => {
  await launchWithFiles(page, [HOLIDAY_VIDEO]);
  await videoSettings(page).getByRole("radio", { name: "GIF" }).click();

  await chooseOption(page, "Frame rate", "24 fps");

  await expect(videoSettings(page).getByText("GIFs default to 15 fps.")).toBeHidden();
});

test("resolution, frame rate, quality and format are sent with the conversion", async ({ page }) => {
  const backend = await launchWithFiles(page, [HOLIDAY_VIDEO]);

  await videoSettings(page).getByRole("radio", { name: "WebM" }).click();
  await videoSettings(page).getByRole("radio", { name: "Small" }).click();
  await chooseOption(page, "Resolution", "720p");
  await chooseOption(page, "Frame rate", "30 fps");
  await expect(videoSettings(page).getByRole("combobox", { name: "Resolution" })).toHaveText("720p");
  await expect(videoSettings(page).getByText("Only scales down.")).toBeVisible();
  await page.getByRole("button", { name: "Convert 1 file" }).click();

  await expect(page.getByRole("heading", { name: "1 file converted" })).toBeVisible();
  const { conversionRequests } = await backend.readLog();
  expect(conversionRequests[0]?.settings).toEqual({
    kind: "video",
    format: "webm",
    options: { quality: "small", maxHeight: 720, frameRate: 30 },
  });
});

test("Original resolution and frame rate send no limits", async ({ page }) => {
  const backend = await launchWithFiles(page, [HOLIDAY_VIDEO]);
  await chooseOption(page, "Resolution", "1080p");

  await chooseOption(page, "Resolution", "Original");
  await page.getByRole("button", { name: "Convert 1 file" }).click();

  await expect(page.getByRole("heading", { name: "1 file converted" })).toBeVisible();
  const { conversionRequests } = await backend.readLog();
  expect(conversionRequests[0]?.settings).toMatchObject({ options: { maxHeight: null, frameRate: null } });
});
