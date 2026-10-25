import { expect, fileRow, launchConverter, test } from "./support/converter";
import { BEACH_PHOTO, CITY_PHOTO, HOLIDAY_VIDEO } from "./support/sample-files";

test("files are grouped into images and videos", async ({ page }) => {
  const backend = await launchConverter(page);

  await backend.dropFiles([BEACH_PHOTO, HOLIDAY_VIDEO, CITY_PHOTO]);

  const files = page.getByRole("region", { name: "Files" });
  await expect(files.getByRole("heading", { name: "3 files" })).toBeVisible();
  await expect(files.getByText("2 images · 1 video")).toBeVisible();
  await expect(files.getByRole("heading", { name: "Images · 2" })).toBeVisible();
  await expect(files.getByRole("heading", { name: "Videos · 1" })).toBeVisible();
  await expect(files.getByRole("group", { name: "Images" }).getByRole("listitem")).toHaveCount(2);
  await expect(files.getByRole("group", { name: "Videos" }).getByRole("listitem")).toHaveText([/holiday\.mov/]);
});

test("a list of one kind has no group headings", async ({ page }) => {
  const backend = await launchConverter(page);

  await backend.dropFiles([BEACH_PHOTO, CITY_PHOTO]);

  await expect(page.getByRole("heading", { name: "2 files" })).toBeVisible();
  await expect(page.getByRole("heading", { name: /Images ·/ })).toHaveCount(0);
});

test("each file shows its size and the format it will become", async ({ page }) => {
  const backend = await launchConverter(page);

  await backend.dropFiles([BEACH_PHOTO, HOLIDAY_VIDEO]);

  await expect(fileRow(page, "beach.png")).toContainText("2.3 MB");
  await expect(fileRow(page, "beach.png")).toContainText("PNG");
  await expect(fileRow(page, "beach.png")).toContainText("JPG");
  await expect(fileRow(page, "holiday.mov")).toContainText("MOV");
  await expect(fileRow(page, "holiday.mov")).toContainText("MP4");
});

test("Remove takes a file out of the list", async ({ page }) => {
  const backend = await launchConverter(page);
  await backend.dropFiles([BEACH_PHOTO, CITY_PHOTO]);

  await fileRow(page, "beach.png").getByRole("button", { name: "Remove" }).click();

  await expect(fileRow(page, "beach.png")).toHaveCount(0);
  await expect(page.getByRole("heading", { name: "1 file", exact: true })).toBeVisible();
});

test("removing the last file goes back to the empty state", async ({ page }) => {
  const backend = await launchConverter(page);
  await backend.dropFiles([BEACH_PHOTO]);

  await fileRow(page, "beach.png").getByRole("button", { name: "Remove" }).click();

  await expect(page.getByText("Drop images or videos")).toBeVisible();
});

test("Clear empties the list", async ({ page }) => {
  const backend = await launchConverter(page);
  await backend.dropFiles([BEACH_PHOTO, HOLIDAY_VIDEO]);

  await page.getByRole("button", { name: "Clear" }).click();

  await expect(page.getByText("Drop images or videos")).toBeVisible();
  await expect(page.getByRole("button", { name: "Clear" })).toBeHidden();
});

test("Add files in the toolbar opens the file dialog", async ({ page }) => {
  const backend = await launchConverter(page);
  await backend.dropFiles([BEACH_PHOTO]);
  await backend.answerNextDialog([CITY_PHOTO]);

  await page.getByRole("button", { name: "Add files" }).click();

  await expect(fileRow(page, "city.jpg")).toBeVisible();
  await expect(page.getByRole("heading", { name: "2 files" })).toBeVisible();
});

test("finished, failed and cancelled files can be removed one at a time", async ({ page }) => {
  const backend = await launchConverter(page, { conversionMode: "manual" });
  await backend.dropFiles([BEACH_PHOTO, CITY_PHOTO, HOLIDAY_VIDEO]);
  await page.getByRole("button", { name: "Convert 3 files" }).click();
  await backend.completeFile("beach.png", "/Users/sam/Pictures/beach.jpg");
  await backend.failFile("city.jpg", "Permission denied");
  await expect(page.getByRole("region", { name: "Files" }).getByRole("button", { name: "Remove" })).toHaveCount(0);
  await page.getByRole("button", { name: "Cancel" }).click();
  await expect(fileRow(page, "holiday.mov")).toContainText("Cancelled");

  await fileRow(page, "holiday.mov").getByRole("button", { name: "Remove" }).click();
  await fileRow(page, "city.jpg").getByRole("button", { name: "Remove" }).click();
  await fileRow(page, "beach.png").getByRole("button", { name: "Remove" }).click();

  await expect(page.getByText("Drop images or videos")).toBeVisible();
});
