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
