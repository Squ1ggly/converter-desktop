import { expect, launchConverter, openConverter, test } from "./support/converter";

// Only errors may use colour; every other token is a shade of grey.
const COLOURED_TOKENS = new Set(["--destructive"]);

test("every theme colour token is neutral in light and dark", async ({ page }) => {
  await launchConverter(page);

  const tokens = await page.evaluate(() => {
    const themeSelectors = new Set([":root", ".dark"]);
    const themeRules = [...document.styleSheets]
      .flatMap((sheet) => [...sheet.cssRules])
      .flatMap((rule) => (rule instanceof CSSLayerBlockRule ? [...rule.cssRules] : [rule]))
      .filter((rule): rule is CSSStyleRule => rule instanceof CSSStyleRule && themeSelectors.has(rule.selectorText));

    const themeTokens = themeRules.flatMap((rule) =>
      [...rule.style]
        .filter((name) => name.startsWith("--"))
        .map((name) => ({ selector: rule.selectorText, name, value: rule.style.getPropertyValue(name) })),
    );
    return themeTokens;
  });

  const colourTokens = tokens.filter((token) => token.value.includes("oklch(") && !COLOURED_TOKENS.has(token.name));
  expect(colourTokens.some((token) => token.selector === ".dark")).toBe(true);
  const nonNeutralTokens = colourTokens.filter((token) => {
    const chroma = Number(/oklch\(\s*[\d.]+%?\s+([\d.]+)/.exec(token.value)?.[1]);
    return chroma !== 0;
  });
  expect(nonNeutralTokens).toEqual([]);
});

interface EarlyThemeCase {
  savedMode: string | null;
  systemScheme: "light" | "dark";
  isDark: boolean;
}

const EARLY_THEME_CASES: EarlyThemeCase[] = [
  { savedMode: null, systemScheme: "dark", isDark: true },
  { savedMode: null, systemScheme: "light", isDark: false },
  { savedMode: "system", systemScheme: "dark", isDark: true },
  { savedMode: "dark", systemScheme: "light", isDark: true },
  { savedMode: "light", systemScheme: "dark", isDark: false },
];

// Runs only index.html's own script, so the theme is known before the app's code loads.
for (const themeCase of EARLY_THEME_CASES) {
  test(`the ${themeCase.savedMode ?? "default"} theme with a ${themeCase.systemScheme} system is set before the app loads`, async ({
    page,
  }) => {
    await page.emulateMedia({ colorScheme: themeCase.systemScheme });
    await page.addInitScript((savedMode) => {
      if (savedMode !== null) {
        localStorage.setItem("mode-watcher-mode", savedMode);
      }
    }, themeCase.savedMode);
    await page.route("**/src/main.ts", (route) => route.abort());

    await openConverter(page);

    const html = page.locator("html");
    const expectedClass = themeCase.isDark ? /\bdark\b/ : /^(?!.*\bdark\b)/;
    await expect(html).toHaveClass(expectedClass);
  });
}
