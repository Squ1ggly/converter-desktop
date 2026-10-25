import type { Page } from "@playwright/test";
import { expect, launchWithFiles, test, type MockBackend } from "./support/converter";
import { BEACH_PHOTO, EXPORT_FOLDER } from "./support/sample-files";

async function convertAndReadOutputDirectory(page: Page, backend: MockBackend): Promise<string | null | undefined> {
  await page.getByRole("button", { name: /^Convert/ }).click();
  await expect(page.getByRole("heading", { name: "1 file converted" })).toBeVisible();

  const { conversionRequests } = await backend.readLog();
  const outputDirectory = conversionRequests.at(-1)?.outputDirectory;
  return outputDirectory;
}

test("files are saved next to their originals by default", async ({ page }) => {
  const backend = await launchWithFiles(page, [BEACH_PHOTO]);

  await expect(page.getByRole("radio", { name: "Same folder as original" })).toBeChecked();
  await expect(page.getByText("Existing files are never overwritten.")).toBeVisible();

  const outputDirectory = await convertAndReadOutputDirectory(page, backend);
  expect(outputDirectory).toBeNull();
});

test("Another folder asks for a folder and saves there", async ({ page }) => {
  const backend = await launchWithFiles(page, [BEACH_PHOTO]);
  await backend.answerNextDialog(EXPORT_FOLDER);

  await page.getByRole("radio", { name: "Another folder" }).click();

  await expect(page.getByRole("radio", { name: "Another folder" })).toBeChecked();
  await expect(page.getByText(EXPORT_FOLDER)).toBeVisible();
  const { dialogs } = await backend.readLog();
  expect(dialogs[0]).toMatchObject({ directory: true, title: "Save converted files to" });

  const outputDirectory = await convertAndReadOutputDirectory(page, backend);
  expect(outputDirectory).toBe(EXPORT_FOLDER);
});

test("cancelling the folder dialog keeps saving next to the originals", async ({ page }) => {
  const backend = await launchWithFiles(page, [BEACH_PHOTO]);
  await backend.answerNextDialog(null);

  await page.getByRole("radio", { name: "Another folder" }).click();

  await expect.poll(async () => (await backend.readLog()).dialogs.length).toBe(1);
  await expect(page.getByRole("radio", { name: "Same folder as original" })).toBeChecked();
  const outputDirectory = await convertAndReadOutputDirectory(page, backend);
  expect(outputDirectory).toBeNull();
});

test("Change picks a different folder", async ({ page }) => {
  const backend = await launchWithFiles(page, [BEACH_PHOTO]);
  await backend.answerNextDialog(EXPORT_FOLDER);
  await page.getByRole("radio", { name: "Another folder" }).click();
  await backend.answerNextDialog("/Users/sam/Desktop/Small");

  await page.getByRole("button", { name: "Change" }).click();

  await expect(page.getByText("/Users/sam/Desktop/Small")).toBeVisible();
  const outputDirectory = await convertAndReadOutputDirectory(page, backend);
  expect(outputDirectory).toBe("/Users/sam/Desktop/Small");
});

test("switching back to the same folder stops using the chosen folder", async ({ page }) => {
  const backend = await launchWithFiles(page, [BEACH_PHOTO]);
  await backend.answerNextDialog(EXPORT_FOLDER);
  await page.getByRole("radio", { name: "Another folder" }).click();
  await expect(page.getByText(EXPORT_FOLDER)).toBeVisible();

  await page.getByRole("radio", { name: "Same folder as original" }).click();

  await expect(page.getByText(EXPORT_FOLDER)).toBeHidden();
  const outputDirectory = await convertAndReadOutputDirectory(page, backend);
  expect(outputDirectory).toBeNull();
});

interface CharacterPositions {
  firstCharacterLeft: number;
  lastCharacterRight: number;
  boxLeft: number;
  boxRight: number;
}

// Where the path's first and last characters are drawn, to check their order on screen.
async function readCharacterPositions(page: Page, folderPath: string): Promise<CharacterPositions> {
  const pathText = page.getByText(folderPath, { exact: true });
  const positions = await pathText.evaluate((element) => {
    const textNode = document.createTreeWalker(element, NodeFilter.SHOW_TEXT).nextNode();
    if (!(textNode instanceof Text)) {
      throw new Error("The folder path has no text");
    }
    const text = textNode.data;
    const firstCharacter = document.createRange();
    firstCharacter.setStart(textNode, text.indexOf("/"));
    firstCharacter.setEnd(textNode, text.indexOf("/") + 1);
    const lastCharacter = document.createRange();
    lastCharacter.setStart(textNode, text.trimEnd().length - 1);
    lastCharacter.setEnd(textNode, text.trimEnd().length);
    const box = element.getBoundingClientRect();
    return {
      firstCharacterLeft: firstCharacter.getBoundingClientRect().left,
      lastCharacterRight: lastCharacter.getBoundingClientRect().right,
      boxLeft: box.left,
      boxRight: box.right,
    };
  });
  return positions;
}

test("the chosen folder reads left to right, starting with its slash", async ({ page }) => {
  const backend = await launchWithFiles(page, [BEACH_PHOTO]);
  await backend.answerNextDialog(EXPORT_FOLDER);
  await page.getByRole("radio", { name: "Another folder" }).click();

  const positions = await readCharacterPositions(page, EXPORT_FOLDER);

  expect(positions.firstCharacterLeft).toBeLessThan(positions.lastCharacterRight);
  expect(Math.abs(positions.firstCharacterLeft - positions.boxLeft)).toBeLessThan(2);
});

test("a long folder path keeps its end in view and shows in full on hover", async ({ page }) => {
  const longFolder = "/Users/sam/Library/Mobile Documents/com~apple~CloudDocs/Projects/2026/Client work/Exports/Converted";
  const backend = await launchWithFiles(page, [BEACH_PHOTO]);
  await backend.answerNextDialog(longFolder);
  await page.getByRole("radio", { name: "Another folder" }).click();

  const positions = await readCharacterPositions(page, longFolder);
  expect(positions.lastCharacterRight).toBeLessThanOrEqual(positions.boxRight + 1);
  expect(positions.lastCharacterRight).toBeGreaterThan(positions.boxLeft);

  await page.getByText(longFolder, { exact: true }).hover();
  await expect(page.locator("[data-slot=tooltip-content]")).toContainText(longFolder);
});
