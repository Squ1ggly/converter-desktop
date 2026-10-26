import { expect, launchWithFiles, test } from "./support/converter";
import type { SavedSettings } from "./support/scenario";
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

interface InvalidSettingsCase {
  description: string;
  savedPreferences: SavedSettings;
  expectedRequests: Partial<Record<"image" | "video", Record<string, unknown>>>;
  expectedOutputDirectory?: string | null;
}

// Each invalid field falls back to its default; valid fields next to it are kept.
const INVALID_SETTINGS_CASES: InvalidSettingsCase[] = [
  {
    description: "a quality that isn't a number",
    savedPreferences: { imageFormat: "webp", imageQuality: "high" },
    expectedRequests: { image: { format: "webp", options: { quality: 85 } } },
  },
  {
    description: "a quality out of range",
    savedPreferences: { imageQuality: 250 },
    expectedRequests: { image: { options: { quality: 85 } } },
  },
  {
    description: "Scale without a percentage",
    savedPreferences: { resize: { mode: "percent" } },
    expectedRequests: { image: { options: { resize: { mode: "percent", percent: 50 } } } },
  },
  {
    description: "Fit with sides that aren't sizes",
    savedPreferences: { resize: { mode: "fit", maxWidth: -5, maxHeight: "big" } },
    expectedRequests: { image: { options: { resize: { mode: "fit", maxWidth: null, maxHeight: null } } } },
  },
  {
    description: "a size that isn't an object",
    savedPreferences: { resize: "small" },
    expectedRequests: { image: { options: { resize: { mode: "original" } } } },
  },
  {
    description: "a format that isn't text",
    savedPreferences: { imageFormat: 42, videoFormat: null },
    expectedRequests: { image: { format: "jpg" }, video: { format: "mp4" } },
  },
  {
    description: "video choices the settings don't offer",
    savedPreferences: { videoFormat: "mkv", videoQuality: "ultra", maxHeight: 999, frameRate: "60" },
    expectedRequests: { video: { format: "mkv", options: { quality: "balanced", maxHeight: null, frameRate: null } } },
  },
  {
    description: "a folder save location without a path",
    savedPreferences: { saveLocation: { mode: "folder" } },
    expectedRequests: {},
    expectedOutputDirectory: null,
  },
];

for (const invalidCase of INVALID_SETTINGS_CASES) {
  test(`saved settings with ${invalidCase.description} fall back to the defaults`, async ({ page }) => {
    const backend = await launchWithFiles(page, [BEACH_PHOTO, HOLIDAY_VIDEO], {
      savedPreferences: invalidCase.savedPreferences,
    });

    await page.getByRole("button", { name: "Convert 2 files" }).click();

    await expect(page.getByRole("heading", { name: "2 files converted" })).toBeVisible();
    const { conversionRequests } = await backend.readLog();
    const [imageRequest, videoRequest] = conversionRequests;
    expect(imageRequest?.settings).toMatchObject(invalidCase.expectedRequests.image ?? {});
    expect(videoRequest?.settings).toMatchObject(invalidCase.expectedRequests.video ?? {});
    if (invalidCase.expectedOutputDirectory !== undefined) {
      expect(imageRequest?.outputDirectory).toBe(invalidCase.expectedOutputDirectory);
    }
  });
}

test("a settings file that can't be read starts with the defaults and says so", async ({ page }) => {
  const backend = await launchWithFiles(page, [BEACH_PHOTO], {
    failingCommands: { "plugin:store|load": "expected value at line 1 column 1" },
  });

  await expect(page.getByText("Couldn't load your settings")).toBeVisible();
  await expect(page.getByText("Couldn't start")).toHaveCount(0);
  await expect(page.getByRole("radio", { name: "JPG" })).toBeChecked();
  await page.getByRole("radio", { name: "PNG" }).click();
  await page.getByRole("button", { name: "Convert 1 file" }).click();

  await expect(page.getByRole("heading", { name: "1 file converted" })).toBeVisible();
  const { conversionRequests, storedPreferences } = await backend.readLog();
  expect(conversionRequests[0]?.settings.format).toBe("png");
  expect(storedPreferences).toBeNull();
});
