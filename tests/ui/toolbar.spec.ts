import type { Locator, Page } from "@playwright/test";
import { expect, fileRow, launchConverter, test } from "./support/converter";
import type { Platform } from "./support/scenario";
import { BEACH_PHOTO } from "./support/sample-files";

const PLATFORMS: Platform[] = ["macos", "windows", "linux"];
// sr-only text keeps a 1px box: read by screen readers, not seen.
const VISUALLY_HIDDEN_MAX_WIDTH_PX = 1;

function toolbar(page: Page): Locator {
  const banner = page.getByRole("banner");
  return banner;
}

function pageTitle(page: Page): Locator {
  const title = toolbar(page).getByRole("heading", { level: 1 });
  return title;
}

async function titleWidthOf(page: Page): Promise<number> {
  const titleBox = await pageTitle(page).boundingBox();
  const titleWidth = titleBox?.width ?? 0;
  return titleWidth;
}

for (const platform of PLATFORMS) {
  test(`Settings and Help are in the toolbar on ${platform}, even with no files`, async ({ page }) => {
    await launchConverter(page, { platform });

    await expect(toolbar(page).getByRole("button", { name: "Settings" })).toBeVisible();
    await expect(toolbar(page).getByRole("button", { name: "Help" })).toBeVisible();
  });
}

test("Settings opens its page and shows as pressed", async ({ page }) => {
  await launchConverter(page);
  const settingsButton = toolbar(page).getByRole("button", { name: "Settings" });
  await expect(settingsButton).toHaveAttribute("aria-pressed", "false");

  await settingsButton.click();

  await expect(settingsButton).toHaveAttribute("aria-pressed", "true");
  await expect(page.getByRole("heading", { name: "Appearance" })).toBeVisible();
  await expect(page.getByText("Drop images or videos")).toBeHidden();
});

test("pressing Settings again goes back to the converter", async ({ page }) => {
  await launchConverter(page);
  const settingsButton = toolbar(page).getByRole("button", { name: "Settings" });
  await settingsButton.click();

  await settingsButton.click();

  await expect(settingsButton).toHaveAttribute("aria-pressed", "false");
  await expect(page.getByText("Drop images or videos")).toBeVisible();
});

test("Help opens its page, and Settings switches from Help to Settings", async ({ page }) => {
  await launchConverter(page);
  const helpButton = toolbar(page).getByRole("button", { name: "Help" });
  const settingsButton = toolbar(page).getByRole("button", { name: "Settings" });

  await helpButton.click();
  await expect(helpButton).toHaveAttribute("aria-pressed", "true");
  await expect(page.getByRole("navigation", { name: "Help pages" })).toBeVisible();

  await settingsButton.click();
  await expect(settingsButton).toHaveAttribute("aria-pressed", "true");
  await expect(helpButton).toHaveAttribute("aria-pressed", "false");
  await expect(page.getByRole("navigation", { name: "Help pages" })).toBeHidden();
});

test("Back returns to the converter with the files still listed", async ({ page }) => {
  const backend = await launchConverter(page);
  await backend.dropFiles([BEACH_PHOTO]);
  await toolbar(page).getByRole("button", { name: "Help" }).click();
  await expect(fileRow(page, "beach.png")).toBeHidden();

  await toolbar(page).getByRole("button", { name: "Back" }).click();

  await expect(fileRow(page, "beach.png")).toBeVisible();
  await expect(toolbar(page).getByRole("button", { name: "Back" })).toBeHidden();
});

test("on macOS the centred title names the page", async ({ page }) => {
  await launchConverter(page, { platform: "macos" });
  await expect(pageTitle(page)).toHaveText("Converter");

  await toolbar(page).getByRole("button", { name: "Settings" }).click();
  await expect(pageTitle(page)).toHaveText("Settings");

  await toolbar(page).getByRole("button", { name: "Help" }).click();
  await expect(pageTitle(page)).toHaveText("Help");
});

for (const platform of ["windows", "linux"] as const) {
  test(`on ${platform} the title only shows on Settings and Help`, async ({ page }) => {
    await launchConverter(page, { platform });
    await expect(pageTitle(page)).toHaveText("Converter");
    expect(await titleWidthOf(page)).toBeLessThanOrEqual(VISUALLY_HIDDEN_MAX_WIDTH_PX);

    await toolbar(page).getByRole("button", { name: "Settings" }).click();
    await expect(pageTitle(page)).toHaveText("Settings");
    expect(await titleWidthOf(page)).toBeGreaterThan(VISUALLY_HIDDEN_MAX_WIDTH_PX);

    await toolbar(page).getByRole("button", { name: "Help" }).click();
    await expect(pageTitle(page)).toHaveText("Help");
    expect(await titleWidthOf(page)).toBeGreaterThan(VISUALLY_HIDDEN_MAX_WIDTH_PX);
  });
}

for (const pageName of ["Settings", "Help"]) {
  test(`dropping files on ${pageName} switches back to the file list`, async ({ page }) => {
    const backend = await launchConverter(page);
    await toolbar(page).getByRole("button", { name: pageName }).click();

    await backend.dropFiles([BEACH_PHOTO]);

    await expect(fileRow(page, "beach.png")).toBeVisible();
    await expect(toolbar(page).getByRole("button", { name: pageName })).toHaveAttribute("aria-pressed", "false");
  });
}
