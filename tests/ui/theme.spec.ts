import { expect, launchConverter, test } from "./support/converter";

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
