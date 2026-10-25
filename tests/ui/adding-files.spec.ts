import { expect, fileRow, launchConverter, test } from "./support/converter";
import {
  BEACH_PHOTO,
  BUDGET,
  CITY_PHOTO,
  HOLIDAY_VIDEO,
  MEETING_NOTES,
  PICTURES_FOLDER,
  SHOPPING_LIST,
  TAX_RETURN,
} from "./support/sample-files";

test("the empty state lists the formats the app accepts", async ({ page }) => {
  await launchConverter(page);

  await expect(page.getByRole("button", { name: "Add files" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Add folder" })).toBeVisible();
  await expect(page.getByRole("definition").first()).toContainText("PNG");
  await expect(page.getByRole("definition").last()).toContainText("MOV");
});

test("Add files adds the files picked in the dialog", async ({ page }) => {
  const backend = await launchConverter(page);
  await backend.answerNextDialog([BEACH_PHOTO, HOLIDAY_VIDEO]);

  await page.getByRole("button", { name: "Add files" }).click();

  await expect(fileRow(page, "beach.png")).toBeVisible();
  await expect(fileRow(page, "holiday.mov")).toBeVisible();
  const { dialogs } = await backend.readLog();
  expect(dialogs[0]).toMatchObject({ multiple: true, title: "Choose images or videos" });
  expect(dialogs[0]?.filters?.[0]?.extensions).toEqual(expect.arrayContaining(["png", "jpg", "mov", "mp4"]));
});

test("cancelling the file dialog adds nothing", async ({ page }) => {
  const backend = await launchConverter(page);
  await backend.answerNextDialog(null);

  await page.getByRole("button", { name: "Add files" }).click();

  await expect.poll(async () => (await backend.readLog()).dialogs.length).toBe(1);
  await expect(page.getByText("Drop images or videos")).toBeVisible();
});

test("Add folder adds the images and videos inside the folder", async ({ page }) => {
  const backend = await launchConverter(page, {
    folders: { [PICTURES_FOLDER]: [BEACH_PHOTO, CITY_PHOTO, `${PICTURES_FOLDER}/captions.txt`] },
  });
  await backend.answerNextDialog([PICTURES_FOLDER]);

  await page.getByRole("button", { name: "Add folder" }).click();

  await expect(page.getByRole("heading", { name: "2 files" })).toBeVisible();
  await expect(fileRow(page, "beach.png")).toBeVisible();
  await expect(fileRow(page, "city.jpg")).toBeVisible();
  const { dialogs } = await backend.readLog();
  expect(dialogs[0]).toMatchObject({ directory: true, multiple: true });
});

test("dragging files over the window shows the drop overlay", async ({ page }) => {
  const backend = await launchConverter(page);
  const overlay = page.getByText("Drop to add");

  await backend.dragFilesOver([BEACH_PHOTO]);
  await expect(overlay).toBeVisible();

  await backend.dragAway();
  await expect(overlay).toBeHidden();
});

test("dropping files adds them and hides the overlay", async ({ page }) => {
  const backend = await launchConverter(page);

  await backend.dropFiles([BEACH_PHOTO, HOLIDAY_VIDEO]);

  await expect(page.getByText("Drop to add")).toBeHidden();
  await expect(page.getByRole("heading", { name: "2 files" })).toBeVisible();
});

test("unsupported files are skipped with a message naming them", async ({ page }) => {
  const backend = await launchConverter(page);

  await backend.dropFiles([BEACH_PHOTO, SHOPPING_LIST]);

  await expect(page.getByText("Skipped 1 unsupported file")).toBeVisible();
  await expect(page.getByText("shopping.txt")).toBeVisible();
  await expect(page.getByRole("heading", { name: "1 file", exact: true })).toBeVisible();
});

test("a long list of skipped files is shortened", async ({ page }) => {
  const backend = await launchConverter(page);

  await backend.dropFiles([BEACH_PHOTO, SHOPPING_LIST, TAX_RETURN, MEETING_NOTES, BUDGET]);

  await expect(page.getByText("Skipped 4 unsupported files")).toBeVisible();
  await expect(page.getByText("shopping.txt, tax.xlsx, meeting.docx and 1 more")).toBeVisible();
});

test("a file that is already in the list is not added twice", async ({ page }) => {
  const backend = await launchConverter(page);
  await backend.dropFiles([BEACH_PHOTO]);
  await expect(fileRow(page, "beach.png")).toBeVisible();

  await backend.dropFiles([BEACH_PHOTO, CITY_PHOTO]);

  await expect(page.getByRole("heading", { name: "2 files" })).toBeVisible();
  await expect(fileRow(page, "beach.png")).toHaveCount(1);
});

test("files can't be added while a conversion is running", async ({ page }) => {
  const backend = await launchConverter(page, { conversionMode: "manual" });
  await backend.dropFiles([BEACH_PHOTO]);
  await page.getByRole("button", { name: "Convert 1 file" }).click();

  await backend.dropFiles([CITY_PHOTO]);

  await expect(page.getByText("Wait for the current conversion to finish.")).toBeVisible();
  await expect(fileRow(page, "city.jpg")).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Add files" })).toBeDisabled();
});

test("adding files after a run clears converted files and keeps failed ones", async ({ page }) => {
  const backend = await launchConverter(page, { failingFiles: { "holiday.mov": "Invalid data found" } });
  await backend.dropFiles([BEACH_PHOTO, HOLIDAY_VIDEO]);
  await page.getByRole("button", { name: "Convert 2 files" }).click();
  await expect(page.getByRole("heading", { name: "1 file converted" })).toBeVisible();

  await backend.dropFiles([CITY_PHOTO]);

  await expect(fileRow(page, "beach.png")).toHaveCount(0);
  await expect(fileRow(page, "holiday.mov").getByRole("button", { name: "Remove" })).toBeVisible();
  await expect(fileRow(page, "city.jpg")).toBeVisible();
  await expect(page.getByRole("button", { name: "Convert 2 files" })).toBeVisible();
});

test("adding files after a cancelled run keeps the cancelled ones", async ({ page }) => {
  const backend = await launchConverter(page, { conversionMode: "manual" });
  await backend.dropFiles([BEACH_PHOTO, CITY_PHOTO]);
  await page.getByRole("button", { name: "Convert 2 files" }).click();
  await backend.completeFile("beach.png", "/Users/sam/Pictures/beach.jpg");
  await page.getByRole("button", { name: "Cancel" }).click();
  await expect(fileRow(page, "city.jpg")).toContainText("Cancelled");

  await backend.dropFiles([HOLIDAY_VIDEO]);

  await expect(fileRow(page, "beach.png")).toHaveCount(0);
  await expect(fileRow(page, "city.jpg")).toBeVisible();
  await expect(fileRow(page, "holiday.mov")).toBeVisible();
});

test("files from a slow folder scan never join a running conversion", async ({ page }) => {
  const backend = await launchConverter(page, {
    conversionMode: "manual",
    folders: { [PICTURES_FOLDER]: [CITY_PHOTO] },
  });
  await backend.dropFiles([BEACH_PHOTO]);
  await expect(fileRow(page, "beach.png")).toBeVisible();
  await backend.holdScans();
  await backend.dropFiles([PICTURES_FOLDER]);

  await expect(page.getByText("Adding files")).toBeVisible();
  await expect(page.getByRole("button", { name: "Convert 1 file" })).toBeDisabled();
  await page.keyboard.press("Meta+Enter");
  await backend.releaseScans();

  await expect(fileRow(page, "city.jpg")).toBeVisible();
  await expect(page.getByText("Adding files")).toBeHidden();
  await expect(page.getByRole("button", { name: "Convert 2 files" })).toBeEnabled();
  const { conversionRequests } = await backend.readLog();
  expect(conversionRequests).toEqual([]);
});

test("an empty list shows progress while a folder is scanned", async ({ page }) => {
  const backend = await launchConverter(page, { folders: { [PICTURES_FOLDER]: [BEACH_PHOTO, CITY_PHOTO] } });
  await backend.holdScans();

  await backend.dropFiles([PICTURES_FOLDER]);

  await expect(page.getByRole("main").getByText("Adding files")).toBeVisible();
  await expect(page.getByText("Drop images or videos")).toBeHidden();
  await backend.releaseScans();
  await expect(page.getByRole("heading", { name: "2 files" })).toBeVisible();
});

test("while converting, the drop overlay says files can't be added", async ({ page }) => {
  const backend = await launchConverter(page, { conversionMode: "manual" });
  await backend.dropFiles([BEACH_PHOTO]);
  await page.getByRole("button", { name: "Convert 1 file" }).click();

  await backend.dragFilesOver([CITY_PHOTO]);

  await expect(page.getByText("Wait for the conversion to finish")).toBeVisible();
  await expect(page.getByText("Drop to add")).toHaveCount(0);
  await backend.dragAway();
  await expect(page.getByText("Wait for the conversion to finish")).toBeHidden();
});
