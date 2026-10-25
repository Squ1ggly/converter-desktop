// The window's minimum size (src-tauri/src/lib.rs), with long names and errors.
import type { Locator, Page } from "@playwright/test";
import { expect, fileRow, launchConverter, launchWithFiles, test } from "./support/converter";
import { BEACH_PHOTO, HOLIDAY_VIDEO } from "./support/sample-files";

const MINIMUM_WINDOW = { width: 760, height: 540 };
const LONG_NAME = "IMG_20260712_183455_holiday_in_the_mountains_with_the_whole_family_final_edit_v3.png";
const LONG_PHOTO = `/Users/sam/Pictures/${LONG_NAME}`;
const LONG_ERROR = "Invalid data found when processing input. The file may be damaged or only partly downloaded.";

test.use({ viewport: MINIMUM_WINDOW });

async function expectNoSidewaysScroll(page: Page): Promise<void> {
  const { scrollWidth, clientWidth } = await page.evaluate(() => ({
    scrollWidth: document.documentElement.scrollWidth,
    clientWidth: document.documentElement.clientWidth,
  }));
  expect(scrollWidth).toBeLessThanOrEqual(clientWidth);
}

async function expectFullyInWindow(control: Locator): Promise<void> {
  const box = await control.boundingBox();
  expect(box).not.toBeNull();
  expect(box?.x ?? -1).toBeGreaterThanOrEqual(0);
  expect(box?.y ?? -1).toBeGreaterThanOrEqual(0);
  expect((box?.x ?? 0) + (box?.width ?? 0)).toBeLessThanOrEqual(MINIMUM_WINDOW.width);
  expect((box?.y ?? 0) + (box?.height ?? 0)).toBeLessThanOrEqual(MINIMUM_WINDOW.height);
}

test("the empty state fits the smallest window", async ({ page }) => {
  await launchConverter(page);

  await expectFullyInWindow(page.getByRole("button", { name: "Add files" }));
  await expectFullyInWindow(page.getByRole("button", { name: "Add folder" }));
  await expectNoSidewaysScroll(page);
});

test("a long file name is shortened and its actions stay in view", async ({ page }) => {
  await launchWithFiles(page, [LONG_PHOTO, HOLIDAY_VIDEO]);

  const title = fileRow(page, LONG_NAME).locator("[data-slot=item-title]");
  const isShortened = await title.evaluate((element) => element.scrollWidth > element.clientWidth);
  expect(isShortened).toBe(true);
  await expect(title).toHaveAttribute("title", LONG_PHOTO);
  await expectFullyInWindow(fileRow(page, LONG_NAME).getByRole("button", { name: "Remove" }));
  await expectFullyInWindow(page.getByRole("button", { name: "Convert 2 files" }));
  await expectNoSidewaysScroll(page);
});

test("a long error wraps inside its row", async ({ page }) => {
  await launchWithFiles(page, [LONG_PHOTO, BEACH_PHOTO], { failingFiles: { [LONG_NAME]: LONG_ERROR } });

  await page.getByRole("button", { name: "Convert 2 files" }).click();

  const error = fileRow(page, LONG_NAME).getByRole("alert");
  await expect(error).toHaveText(LONG_ERROR);
  await expectFullyInWindow(error);
  await expectFullyInWindow(fileRow(page, LONG_NAME).getByRole("button", { name: "Remove" }));
  await expectFullyInWindow(page.getByRole("button", { name: "Convert again" }));
  await expectNoSidewaysScroll(page);
});

test("a running conversion fits the smallest window", async ({ page }) => {
  await launchWithFiles(page, [LONG_PHOTO, HOLIDAY_VIDEO], { conversionMode: "manual" });

  await page.getByRole("button", { name: "Convert 2 files" }).click();

  await expectFullyInWindow(page.getByRole("button", { name: "Cancel" }));
  await expectNoSidewaysScroll(page);
});

for (const pageName of ["Settings", "Help"]) {
  test(`${pageName} fits the smallest window`, async ({ page }) => {
    await launchConverter(page);

    await page.getByRole("banner").getByRole("button", { name: pageName }).click();

    await expectFullyInWindow(page.getByRole("button", { name: "Back" }));
    await expectNoSidewaysScroll(page);
  });
}
