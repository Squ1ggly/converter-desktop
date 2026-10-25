import type { Locator, Page } from "@playwright/test";
import { expect, fileRow, launchConverter, launchWithFiles, test } from "./support/converter";
import type { Platform } from "./support/scenario";
import { BEACH_PHOTO } from "./support/sample-files";

interface PlatformExpectations {
  platform: Platform;
  modifierKey: "Meta" | "Control";
  otherModifierKey: "Meta" | "Control";
  openHint: string[];
  convertHint: string[];
  revealLabel: string;
}

const PLATFORMS: PlatformExpectations[] = [
  {
    platform: "macos",
    modifierKey: "Meta",
    otherModifierKey: "Control",
    openHint: ["⌘", "O"],
    convertHint: ["⌘", "↩"],
    revealLabel: "Show in Finder",
  },
  {
    platform: "windows",
    modifierKey: "Control",
    otherModifierKey: "Meta",
    openHint: ["Ctrl", "O"],
    convertHint: ["Ctrl", "Enter"],
    revealLabel: "Show in Explorer",
  },
  {
    platform: "linux",
    modifierKey: "Control",
    otherModifierKey: "Meta",
    openHint: ["Ctrl", "O"],
    convertHint: ["Ctrl", "Enter"],
    revealLabel: "Show in folder",
  },
];

// Each key is a <kbd> inside the hint's own <kbd> group.
function shortcutKeysOf(button: Locator): Locator {
  const keys = button.locator("kbd > kbd");
  return keys;
}

function convertButton(page: Page): Locator {
  const button = page.getByRole("complementary", { name: "Settings" }).getByRole("button", { name: /^Convert/ });
  return button;
}

for (const expected of PLATFORMS) {
  test.describe(expected.platform, () => {
    test("shortcut hints use this platform's keys", async ({ page }) => {
      const backend = await launchConverter(page, { platform: expected.platform });

      await expect(shortcutKeysOf(page.getByRole("button", { name: "Add files" }))).toHaveText(expected.openHint);
      await backend.dropFiles([BEACH_PHOTO]);
      await expect(shortcutKeysOf(convertButton(page))).toHaveText(expected.convertHint);
    });

    test(`${expected.modifierKey}+O opens the file dialog`, async ({ page }) => {
      const backend = await launchConverter(page, { platform: expected.platform });
      await backend.answerNextDialog([BEACH_PHOTO]);

      await page.keyboard.press(`${expected.modifierKey}+O`);

      await expect(fileRow(page, "beach.png")).toBeVisible();
    });

    test(`${expected.otherModifierKey}+O does nothing`, async ({ page }) => {
      const backend = await launchConverter(page, { platform: expected.platform });

      await page.keyboard.press(`${expected.otherModifierKey}+O`);

      const { dialogs } = await backend.readLog();
      expect(dialogs).toEqual([]);
    });

    test(`${expected.modifierKey}+Enter starts the conversion`, async ({ page }) => {
      const backend = await launchWithFiles(page, [BEACH_PHOTO], { platform: expected.platform });

      await page.keyboard.press(`${expected.modifierKey}+Enter`);

      await expect(page.getByRole("heading", { name: "1 file converted" })).toBeVisible();
      const { conversionRequests } = await backend.readLog();
      expect(conversionRequests).toHaveLength(1);
    });

    test(`the file manager is called ${expected.revealLabel}`, async ({ page }) => {
      await launchWithFiles(page, [BEACH_PHOTO], { platform: expected.platform });

      await convertButton(page).click();

      await expect(fileRow(page, "beach.png").getByRole("button", { name: expected.revealLabel })).toBeVisible();
    });

    test(`${expected.modifierKey}+, opens and closes Settings`, async ({ page }) => {
      await launchConverter(page, { platform: expected.platform });
      const settingsButton = page.getByRole("banner").getByRole("button", { name: "Settings" });

      await page.keyboard.press(`${expected.modifierKey}+Comma`);
      await expect(settingsButton).toHaveAttribute("aria-pressed", "true");

      await page.keyboard.press(`${expected.modifierKey}+Comma`);
      await expect(settingsButton).toHaveAttribute("aria-pressed", "false");
    });

    test("F1 opens and closes Help", async ({ page }) => {
      await launchConverter(page, { platform: expected.platform });
      const helpButton = page.getByRole("banner").getByRole("button", { name: "Help" });

      await page.keyboard.press("F1");
      await expect(helpButton).toHaveAttribute("aria-pressed", "true");

      await page.keyboard.press("F1");
      await expect(helpButton).toHaveAttribute("aria-pressed", "false");
    });

    test(`${expected.modifierKey}+Enter on Settings goes back to the list and converts`, async ({ page }) => {
      const backend = await launchWithFiles(page, [BEACH_PHOTO], { platform: expected.platform });
      await page.keyboard.press(`${expected.modifierKey}+Comma`);
      await expect(page.getByRole("heading", { name: "Appearance" })).toBeVisible();

      await page.keyboard.press(`${expected.modifierKey}+Enter`);

      await expect(page.getByRole("heading", { name: "1 file converted" })).toBeVisible();
      await expect(page.getByRole("heading", { name: "Appearance" })).toBeHidden();
      const { conversionRequests } = await backend.readLog();
      expect(conversionRequests).toHaveLength(1);
    });
  });
}

test("HEIC output is only offered on macOS", async ({ page }) => {
  await launchWithFiles(page, [BEACH_PHOTO], { platform: "windows" });

  await expect(page.getByRole("radio", { name: "JPG" })).toBeVisible();
  await expect(page.getByRole("radio", { name: "HEIC" })).toHaveCount(0);
});
