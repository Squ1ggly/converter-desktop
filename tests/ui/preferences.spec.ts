import { expect, launchWithFiles, test } from "./support/converter";
import { BEACH_PHOTO, EXPORT_FOLDER, HOLIDAY_VIDEO } from "./support/sample-files";

test("saved preferences are restored at launch", async ({ page }) => {
  await launchWithFiles(page, [BEACH_PHOTO, HOLIDAY_VIDEO], {
    savedPreferences: {
      imageFormat: "webp",
      resize: { mode: "fit", maxWidth: 800, maxHeight: null },
      videoFormat: "mkv",
      videoQuality: "high",
      maxHeight: 1080,
      saveLocation: { mode: "folder", path: EXPORT_FOLDER },
    },
  });

  const imageSettings = page.getByRole("region", { name: "Images" });
  const videoSettings = page.getByRole("region", { name: "Videos" });
  await expect(imageSettings.getByRole("radio", { name: "WebP" })).toBeChecked();
  await expect(imageSettings.getByRole("radio", { name: "Fit" })).toBeChecked();
  await expect(imageSettings.getByRole("spinbutton", { name: "Max width" })).toHaveValue("800");
  await expect(videoSettings.getByRole("radio", { name: "MKV" })).toBeChecked();
  await expect(videoSettings.getByRole("radio", { name: "High" })).toBeChecked();
  await expect(videoSettings.getByRole("combobox", { name: "Resolution" })).toHaveText("1080p");
  await expect(page.getByRole("radio", { name: "Another folder" })).toBeChecked();
  await expect(page.getByText(EXPORT_FOLDER)).toBeVisible();
});

test("changed settings are saved to the store", async ({ page }) => {
  const backend = await launchWithFiles(page, [BEACH_PHOTO, HOLIDAY_VIDEO]);

  await page.getByRole("radio", { name: "PNG" }).click();
  await page.getByRole("radio", { name: "Small" }).click();

  await expect
    .poll(async () => (await backend.readLog()).storedPreferences)
    .toMatchObject({ imageFormat: "png", videoQuality: "small" });
});

test("a saved format this platform can't write falls back to the first one", async ({ page }) => {
  await launchWithFiles(page, [BEACH_PHOTO], {
    platform: "windows",
    savedPreferences: { imageFormat: "heic" },
  });

  await expect(page.getByRole("radio", { name: "JPG" })).toBeChecked();
});
