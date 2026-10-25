import type { Locator, Page } from "@playwright/test";
import type { ConversionSettings } from "$lib/backend";
import { expect, fileRow, launchWithFiles, test, type MockBackend } from "./support/converter";
import { BEACH_PHOTO } from "./support/sample-files";

function imageSettings(page: Page): Locator {
  const settingsSection = page.getByRole("region", { name: "Images" });
  return settingsSection;
}

async function convertAndReadImageSettings(page: Page, backend: MockBackend): Promise<ConversionSettings | undefined> {
  await page.getByRole("button", { name: "Convert 1 file" }).click();
  await expect(page.getByRole("heading", { name: "1 file converted" })).toBeVisible();

  const { conversionRequests } = await backend.readLog();
  const imageRequestSettings = conversionRequests[0]?.settings;
  return imageRequestSettings;
}

test("the format picker offers the formats from the catalog", async ({ page }) => {
  await launchWithFiles(page, [BEACH_PHOTO]);
  const formats = imageSettings(page).getByRole("group", { name: "Format" }).getByRole("radio");

  await expect(formats).toHaveText(["JPG", "PNG", "WebP", "AVIF", "HEIC", "GIF", "BMP", "TIFF", "PDF", "SVG"]);
  await expect(imageSettings(page).getByRole("radio", { name: "JPG" })).toBeChecked();
  await expect(imageSettings(page).getByText("Photos. Opens anywhere. No transparency.")).toBeVisible();
});

test("picking a format updates the description and the file list", async ({ page }) => {
  await launchWithFiles(page, [BEACH_PHOTO]);
  await imageSettings(page).getByRole("radio", { name: "WebP" }).click();

  await expect(imageSettings(page).getByRole("radio", { name: "WebP" })).toBeChecked();
  await expect(imageSettings(page).getByText("Smaller than JPG. Keeps transparency.")).toBeVisible();
  await expect(fileRow(page, "beach.png")).toContainText("WebP");
});

test("the quality slider only shows for formats that use it", async ({ page }) => {
  await launchWithFiles(page, [BEACH_PHOTO]);
  const qualitySlider = imageSettings(page).getByRole("slider", { name: "Quality" });
  await expect(qualitySlider).toBeVisible();

  await imageSettings(page).getByRole("radio", { name: "PNG" }).click();
  await expect(qualitySlider).toBeHidden();

  await imageSettings(page).getByRole("radio", { name: "AVIF" }).click();
  await expect(qualitySlider).toBeVisible();
});

test("the chosen quality is sent with the conversion", async ({ page }) => {
  const backend = await launchWithFiles(page, [BEACH_PHOTO]);
  const qualitySlider = imageSettings(page).getByRole("slider", { name: "Quality" });

  await qualitySlider.focus();
  await page.keyboard.press("ArrowLeft");
  await page.keyboard.press("ArrowLeft");

  await expect(imageSettings(page).getByText("83", { exact: true })).toBeVisible();
  const settings = await convertAndReadImageSettings(page, backend);
  expect(settings).toMatchObject({ kind: "image", format: "jpg", options: { quality: 83 } });
});

test("Size starts at Original", async ({ page }) => {
  const backend = await launchWithFiles(page, [BEACH_PHOTO]);

  await expect(imageSettings(page).getByRole("radio", { name: "Original" })).toBeChecked();

  const settings = await convertAndReadImageSettings(page, backend);
  expect(settings).toMatchObject({ options: { resize: { mode: "original" } } });
});

test("Scale resizes by a percentage", async ({ page }) => {
  const backend = await launchWithFiles(page, [BEACH_PHOTO]);

  await imageSettings(page).getByRole("radio", { name: "Scale" }).click();
  const scaleField = imageSettings(page).getByRole("spinbutton", { name: "Scale" });
  await expect(scaleField).toHaveValue("50");
  await scaleField.fill("25");
  await scaleField.blur();

  const settings = await convertAndReadImageSettings(page, backend);
  expect(settings).toMatchObject({ options: { resize: { mode: "percent", percent: 25 } } });
});

test("Scale keeps the percentage within limits", async ({ page }) => {
  await launchWithFiles(page, [BEACH_PHOTO]);
  await imageSettings(page).getByRole("radio", { name: "Scale" }).click();
  const scaleField = imageSettings(page).getByRole("spinbutton", { name: "Scale" });

  await scaleField.fill("5000");
  await scaleField.blur();

  await expect(scaleField).toHaveValue("1000");
});

test("Fit limits the width and height", async ({ page }) => {
  const backend = await launchWithFiles(page, [BEACH_PHOTO]);

  await imageSettings(page).getByRole("radio", { name: "Fit" }).click();
  await expect(imageSettings(page).getByText("Keeps proportions. Leave a side blank to scale by the other.")).toBeVisible();
  const widthField = imageSettings(page).getByRole("spinbutton", { name: "Max width" });
  await widthField.fill("1920");
  await widthField.blur();

  await expect(imageSettings(page).getByRole("spinbutton", { name: "Max height" })).toHaveValue("");
  const settings = await convertAndReadImageSettings(page, backend);
  expect(settings).toMatchObject({ options: { resize: { mode: "fit", maxWidth: 1920, maxHeight: null } } });
});

test("the selected size and the selected format look the same", async ({ page }) => {
  await launchWithFiles(page, [BEACH_PHOTO]);
  const selectedFormat = imageSettings(page).getByRole("radio", { name: "JPG" });
  const selectedSize = imageSettings(page).getByRole("radio", { name: "Fit" });

  await selectedSize.click();

  await expect(selectedFormat).toHaveAttribute("data-state", "on");
  await expect(selectedSize).toHaveAttribute("data-state", "on");
  const formatBackground = await selectedFormat.evaluate((element) => getComputedStyle(element).backgroundColor);
  const sizeBackground = await selectedSize.evaluate((element) => getComputedStyle(element).backgroundColor);
  expect(sizeBackground).toBe(formatBackground);
});

test("the convert shortcut uses a Scale value that is still being typed", async ({ page }) => {
  const backend = await launchWithFiles(page, [BEACH_PHOTO]);
  await imageSettings(page).getByRole("radio", { name: "Scale" }).click();

  await imageSettings(page).getByRole("spinbutton", { name: "Scale" }).fill("25");
  await page.keyboard.press("Meta+Enter");

  await expect(page.getByRole("heading", { name: "1 file converted" })).toBeVisible();
  const { conversionRequests } = await backend.readLog();
  expect(conversionRequests[0]?.settings).toMatchObject({ options: { resize: { mode: "percent", percent: 25 } } });
});

test("the convert shortcut uses a Fit width that is still being typed", async ({ page }) => {
  const backend = await launchWithFiles(page, [BEACH_PHOTO]);
  await imageSettings(page).getByRole("radio", { name: "Fit" }).click();

  await imageSettings(page).getByRole("spinbutton", { name: "Max width" }).fill("640");
  await page.keyboard.press("Meta+Enter");

  await expect(page.getByRole("heading", { name: "1 file converted" })).toBeVisible();
  const { conversionRequests } = await backend.readLog();
  expect(conversionRequests[0]?.settings).toMatchObject({ options: { resize: { mode: "fit", maxWidth: 640 } } });
});

test("the format and size toggles share one style", async ({ page }) => {
  await launchWithFiles(page, [BEACH_PHOTO]);

  const formatBox = await imageSettings(page).getByRole("radio", { name: "PNG" }).boundingBox();
  const sizeBox = await imageSettings(page).getByRole("radio", { name: "Scale" }).boundingBox();
  const spacings = await imageSettings(page)
    .locator("[data-slot=toggle-group]")
    .evaluateAll((groups) => groups.map((group) => group.getAttribute("data-spacing")));

  expect(sizeBox?.height).toBe(formatBox?.height);
  expect(spacings).toHaveLength(2);
  expect(new Set(spacings).size).toBe(1);
});

test("a one-line format description leaves no gap above Quality", async ({ page }) => {
  await launchWithFiles(page, [BEACH_PHOTO]);

  const description = imageSettings(page).getByText("Photos. Opens anywhere. No transparency.");
  const { height, lineHeight } = await description.evaluate((element) => ({
    height: element.getBoundingClientRect().height,
    lineHeight: Number.parseFloat(getComputedStyle(element).lineHeight),
  }));

  expect(height).toBeLessThan(lineHeight * 1.5);
});
