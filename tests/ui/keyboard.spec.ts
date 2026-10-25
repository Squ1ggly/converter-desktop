import type { Page } from "@playwright/test";
import { expect, fileRow, launchConverter, launchWithFiles, test } from "./support/converter";
import { BEACH_PHOTO, HOLIDAY_VIDEO } from "./support/sample-files";

const MAX_TAB_PRESSES = 60;

async function focusedName(page: Page): Promise<string> {
  const name = await page.evaluate(() => {
    const focused = document.activeElement;
    if (!focused || focused === document.body) {
      return "";
    }
    const label = focused.getAttribute("aria-label") ?? focused.textContent ?? "";
    return label.replace(/\s+/g, " ").trim();
  });
  return name;
}

// Presses Tab until `lastName` has focus, returning every control it passed.
async function tabUntil(page: Page, lastName: string): Promise<string[]> {
  const visitedNames: string[] = [];
  for (let pressCount = 0; pressCount < MAX_TAB_PRESSES; pressCount += 1) {
    await page.keyboard.press("Tab");
    const name = await focusedName(page);
    visitedNames.push(name);
    if (name.startsWith(lastName)) {
      return visitedNames;
    }
  }
  throw new Error(`Never reached ${lastName}. Visited: ${visitedNames.join(", ")}`);
}

test("Tab moves through the toolbar, the files, the settings, then Convert", async ({ page }, testInfo) => {
  // Playwright's WebKit follows macOS, which only tabs to text fields and lists unless
  // Keyboard navigation is turned on in System Settings. WebView2 and WebKitGTK tab to every control.
  test.skip(testInfo.project.name === "webkit", "macOS skips buttons on Tab by default");
  await launchWithFiles(page, [BEACH_PHOTO, HOLIDAY_VIDEO]);

  const visitedNames = await tabUntil(page, "Convert 2 files");

  const milestones = ["Clear", "Add files", "Settings", "Help", "Remove", "JPG", "Original", "MP4", "Convert 2 files"];
  const milestonePositions = milestones.map((milestone) => visitedNames.findIndex((name) => name.startsWith(milestone)));
  expect(milestonePositions.every((position) => position >= 0), visitedNames.join(", ")).toBe(true);
  expect([...milestonePositions].sort((first, second) => first - second)).toEqual(milestonePositions);
});

test("a whole conversion can be done with the keyboard alone", async ({ page }) => {
  const backend = await launchConverter(page);
  await backend.answerNextDialog([BEACH_PHOTO]);

  await page.keyboard.press("Meta+O");
  await expect(fileRow(page, "beach.png")).toBeVisible();
  await tabUntil(page, "JPG");
  await page.keyboard.press("ArrowRight");
  await page.keyboard.press("Space");
  await expect(page.getByRole("radio", { name: "PNG" })).toBeChecked();
  await page.keyboard.press("Meta+Enter");
  await expect(page.getByRole("heading", { name: "1 file converted" })).toBeVisible();
  const visitedNames = await tabUntil(page, "Open");
  await page.keyboard.press("Enter");

  await expect.poll(async () => (await backend.readLog()).openedOutputs).toEqual(["/Users/sam/Pictures/beach.png"]);
  expect(visitedNames.at(-1)).toBe("Open");
});
