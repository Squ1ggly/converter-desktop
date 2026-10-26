import { expect, openConverter, test } from "./support/converter";

test("the window shows it's loading until the app knows the platform", async ({ page }) => {
  const backend = await openConverter(page, { platform: "macos", holdsCatalog: true });

  await expect(page.getByRole("main").getByText("Loading")).toBeVisible();
  await expect(page.locator("kbd")).toHaveCount(0);
  await backend.releaseCatalog();

  await expect(page.getByText("Drop images or videos")).toBeVisible();
  await expect(page.getByRole("button", { name: "Add files" }).locator("kbd > kbd")).toHaveText(["⌘", "O"]);
});

test("shortcuts do nothing until the app knows the platform", async ({ page }) => {
  const backend = await openConverter(page, { platform: "macos", holdsCatalog: true });
  await expect(page.getByRole("main").getByText("Loading")).toBeVisible();

  await page.keyboard.press("Control+O");
  await page.keyboard.press("Meta+O");
  await page.keyboard.press("Control+Comma");

  const { dialogs } = await backend.readLog();
  expect(dialogs).toEqual([]);
  await expect(page.getByRole("banner").getByRole("button", { name: "Settings" })).toHaveAttribute("aria-pressed", "false");
});

test("Settings shows it's loading instead of a blank page", async ({ page }) => {
  const backend = await openConverter(page, { holdsCatalog: true });

  await page.getByRole("banner").getByRole("button", { name: "Settings" }).click();

  await expect(page.getByRole("main").getByText("Loading")).toBeVisible();
  await backend.releaseCatalog();
  await expect(page.getByRole("heading", { name: "Appearance" })).toBeVisible();
});

test("an app that can't start says why", async ({ page }) => {
  await openConverter(page, { failingCommands: { get_catalog: "The bundled FFmpeg is missing" } });

  await expect(page.getByRole("alert")).toContainText("The bundled FFmpeg is missing");
});
