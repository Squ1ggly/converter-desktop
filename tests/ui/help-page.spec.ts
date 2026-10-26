import { readFile } from "node:fs/promises";
import path from "node:path";
import type { Locator, Page } from "@playwright/test";
import { expect, launchConverter, test, type MockBackend } from "./support/converter";

const REPOSITORY_URL = "https://github.com/Squ1ggly/converter-desktop/blob/main";

function helpPages(page: Page): Locator {
  const pageButtons = page.getByRole("navigation", { name: "Help pages" }).getByRole("button");
  return pageButtons;
}

function guideArticle(page: Page): Locator {
  const article = page.getByRole("article");
  return article;
}

function pageHeading(page: Page): Locator {
  const heading = guideArticle(page).getByRole("heading", { level: 1 });
  return heading;
}

async function openHelp(page: Page): Promise<MockBackend> {
  const backend = await launchConverter(page);
  await page.getByRole("banner").getByRole("button", { name: "Help" }).click();
  await expect(guideArticle(page)).toBeVisible();
  return backend;
}

// No guide page links outside the guide with a relative link yet, so these
// tests add one the way the Markdown would render it.
async function addLinkToArticle(page: Page, href: string, text: string): Promise<void> {
  await guideArticle(page).evaluate(
    (article, link) => article.insertAdjacentHTML("beforeend", `<p><a href="${link.href}">${link.text}</a></p>`),
    { href, text },
  );
}

test("lists the guide pages and opens the first one", async ({ page }) => {
  await openHelp(page);

  const firstPage = helpPages(page).first();
  const firstTitle = (await firstPage.textContent())?.trim() ?? "";
  expect(await helpPages(page).count()).toBeGreaterThan(1);
  await expect(firstPage).toHaveAttribute("aria-current", "page");
  await expect(pageHeading(page)).toHaveText(firstTitle);
});

test("choosing a page in the sidebar shows it", async ({ page }) => {
  await openHelp(page);
  const thirdPage = helpPages(page).nth(2);
  const thirdTitle = (await thirdPage.textContent())?.trim() ?? "";

  await thirdPage.click();

  await expect(thirdPage).toHaveAttribute("aria-current", "page");
  await expect(helpPages(page).first()).not.toHaveAttribute("aria-current", "page");
  await expect(pageHeading(page)).toHaveText(thirdTitle);
});

test("a link to another guide page opens it at the linked heading", async ({ page }) => {
  await openHelp(page);
  await helpPages(page).filter({ hasText: "Troubleshooting" }).click();

  await guideArticle(page).locator('a[href="saving.md#file-names"]').click();

  await expect(helpPages(page).filter({ hasText: "Saving" })).toHaveAttribute("aria-current", "page");
  await expect(guideArticle(page).getByRole("heading", { name: "File names" })).toBeInViewport();
});

test("a link outside the guide opens on GitHub in the browser", async ({ page }) => {
  const backend = await openHelp(page);
  const shownTitle = (await pageHeading(page).textContent()) ?? "";
  await addLinkToArticle(page, "development/building.md", "Building the app");
  await addLinkToArticle(page, "../CONTRIBUTING.md#testing", "How to test");

  await guideArticle(page).getByRole("link", { name: "Building the app" }).click();
  await guideArticle(page).getByRole("link", { name: "How to test" }).click();

  await expect
    .poll(async () => (await backend.readLog()).openedUrls)
    .toEqual([`${REPOSITORY_URL}/docs/development/building.md`, `${REPOSITORY_URL}/CONTRIBUTING.md#testing`]);
  await expect(pageHeading(page)).toHaveText(shownTitle);
  expect(new URL(page.url()).pathname).toBe("/");
});

test("the guide's link to Installing opens on GitHub, since Installing isn't in the app", async ({ page }) => {
  const backend = await openHelp(page);
  await helpPages(page).filter({ hasText: "Troubleshooting" }).click();

  await guideArticle(page).getByRole("link", { name: "Installing" }).click();

  await expect
    .poll(async () => (await backend.readLog()).openedUrls)
    .toEqual([`${REPOSITORY_URL}/docs/installing.md`]);
});

test("the pages follow the guide's index, without Installing", async ({ page }) => {
  const indexMarkdown = await readFile(path.join(import.meta.dirname, "../../docs/README.md"), "utf8");
  const usingTheApp = indexMarkdown.slice(indexMarkdown.indexOf("## Using the app"), indexMarkdown.indexOf("## Development"));
  const indexTitles = [...usingTheApp.matchAll(/\| \[([^\]]+)\]\(/g)].map((match) => match[1] ?? "");
  await openHelp(page);

  await expect(helpPages(page)).toHaveText(indexTitles.filter((title) => title !== "Installing"));
});

test("a link to another guide page also opens with the keyboard", async ({ page }) => {
  await openHelp(page);
  await helpPages(page).filter({ hasText: "Troubleshooting" }).click();

  await guideArticle(page).locator('a[href="saving.md#file-names"]').focus();
  await page.keyboard.press("Enter");

  await expect(helpPages(page).filter({ hasText: "Saving" })).toHaveAttribute("aria-current", "page");
});

test("the blank window tip opens at its heading on GitHub", async ({ page }) => {
  const backend = await openHelp(page);
  await helpPages(page).filter({ hasText: "Troubleshooting" }).click();

  await guideArticle(page).getByRole("link", { name: "If the window is blank" }).click();

  await expect
    .poll(async () => (await backend.readLog()).openedUrls)
    .toEqual([`${REPOSITORY_URL}/docs/installing.md#if-the-window-is-blank`]);
});
