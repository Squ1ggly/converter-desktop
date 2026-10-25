// Automated accessibility checks of each main state, in light and dark mode.
import AxeBuilder from "@axe-core/playwright";
import type { Page } from "@playwright/test";
import { expect, launchConverter, launchWithFiles, test } from "./support/converter";
import { BEACH_PHOTO, CITY_PHOTO, DOG_VIDEO, HOLIDAY_VIDEO } from "./support/sample-files";

const MIXED_FILES = [BEACH_PHOTO, CITY_PHOTO, HOLIDAY_VIDEO, DOG_VIDEO];
const COLOR_SCHEMES = ["light", "dark"] as const;

// `scope` limits the check to part of the page, e.g. a dialog over content checked elsewhere.
async function expectNoViolations(page: Page, scope?: string): Promise<void> {
  // Pause transitions so colours are checked at their final values.
  await page.addStyleTag({ content: "*, *::before, *::after { transition: none !important; animation: none !important; }" });
  const axe = scope ? new AxeBuilder({ page }).include(scope) : new AxeBuilder({ page });
  const results = await axe.analyze();
  const violations = results.violations.map((violation) => ({
    rule: violation.id,
    targets: violation.nodes.map((node) => node.target.join(" ")),
  }));
  expect(violations).toEqual([]);
}

for (const colorScheme of COLOR_SCHEMES) {
  test.describe(`${colorScheme} mode`, () => {
    test.use({ colorScheme });

    test("the empty state has no accessibility violations", async ({ page }) => {
      await launchConverter(page);

      await expectNoViolations(page);
    });

    test("the file list and settings have no accessibility violations", async ({ page }) => {
      await launchWithFiles(page, MIXED_FILES);

      await expectNoViolations(page);
    });

    test("a running conversion has no accessibility violations", async ({ page }) => {
      const backend = await launchWithFiles(page, MIXED_FILES, { conversionMode: "manual" });
      await page.getByRole("button", { name: "Convert 4 files" }).click();
      await backend.startFile("holiday.mov");
      await backend.reportProgress("holiday.mov", 40);
      await backend.startFile("beach.png");

      await expectNoViolations(page);
    });

    test("finished results have no accessibility violations", async ({ page }) => {
      await launchWithFiles(page, MIXED_FILES, { failingFiles: { "dog.mp4": "Invalid data found when processing input" } });
      await page.getByRole("button", { name: "Convert 4 files" }).click();
      await expect(page.getByText("1 failed")).toBeVisible();

      await expectNoViolations(page);
    });

    test("the Settings page has no accessibility violations", async ({ page }) => {
      await launchConverter(page);
      await page.getByRole("banner").getByRole("button", { name: "Settings" }).click();
      await expect(page.getByRole("heading", { name: "Appearance" })).toBeVisible();

      await expectNoViolations(page);
    });

    test("the Help page has no accessibility violations", async ({ page }) => {
      await launchConverter(page);
      await page.getByRole("banner").getByRole("button", { name: "Help" }).click();
      await expect(page.getByRole("article")).toBeVisible();

      await expectNoViolations(page);
    });

    test("the quit dialog has no accessibility violations", async ({ page }) => {
      const backend = await launchWithFiles(page, [BEACH_PHOTO], { conversionMode: "manual" });
      await page.getByRole("button", { name: "Convert 1 file" }).click();
      await backend.requestWindowClose();
      await expect(page.getByRole("alertdialog")).toBeVisible();

      await expectNoViolations(page, "[role=alertdialog]");
    });
  });
}
