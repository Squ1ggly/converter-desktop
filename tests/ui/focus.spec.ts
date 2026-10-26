import type { Locator, Page } from "@playwright/test";
import { expect, launchConverter, launchWithFiles, test } from "./support/converter";
import { BEACH_PHOTO } from "./support/sample-files";

function pageTitle(page: Page): Locator {
  const title = page.getByRole("banner").getByRole("heading", { level: 1 });
  return title;
}

test("Convert hands focus to Cancel, and the finished run hands it back", async ({ page }) => {
  const backend = await launchWithFiles(page, [BEACH_PHOTO], { conversionMode: "manual" });

  await page.getByRole("button", { name: "Convert 1 file" }).click();
  await expect(page.getByRole("button", { name: "Cancel" })).toBeFocused();

  await backend.completeFile("beach.png", "/Users/sam/Pictures/beach.jpg");
  await expect(page.getByRole("button", { name: "Convert again" })).toBeFocused();
});

test("after cancelling, focus returns to Convert again", async ({ page }) => {
  await launchWithFiles(page, [BEACH_PHOTO], { conversionMode: "manual" });
  await page.getByRole("button", { name: "Convert 1 file" }).click();

  await page.getByRole("button", { name: "Cancel" }).click();

  await expect(page.getByRole("button", { name: "Convert again" })).toBeFocused();
});

test("converting with the shortcut from a number field focuses Cancel", async ({ page }) => {
  await launchWithFiles(page, [BEACH_PHOTO], { conversionMode: "manual" });
  await page.getByRole("radio", { name: "Scale" }).click();
  await page.getByRole("spinbutton", { name: "Scale" }).fill("40");

  await page.keyboard.press("Meta+Enter");

  await expect(page.getByRole("button", { name: "Cancel" })).toBeFocused();
});

test("opening Settings or Help moves focus to the page title", async ({ page }) => {
  await launchConverter(page);

  await page.keyboard.press("Meta+Comma");
  await expect(pageTitle(page)).toHaveText("Settings");
  await expect(pageTitle(page)).toBeFocused();

  await page.keyboard.press("F1");
  await expect(pageTitle(page)).toHaveText("Help");
  await expect(pageTitle(page)).toBeFocused();
});

test("Back moves focus to the converter's title instead of losing it", async ({ page }) => {
  await launchConverter(page);
  await page.getByRole("banner").getByRole("button", { name: "Settings" }).click();

  await page.getByRole("button", { name: "Back" }).click();

  await expect(pageTitle(page)).toHaveText("Converter");
  await expect(pageTitle(page)).toBeFocused();
});
