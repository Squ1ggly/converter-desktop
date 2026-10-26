// One look for every toggle group (formats, size, quality, theme): spaced
// outline buttons, so a grid of formats and a row of three options match.
export const TOGGLE_GROUP_STYLE = { variant: "outline", size: "sm", spacing: 1 } as const;

export const SELECTED_TOGGLE_CLASS =
  "data-[state=on]:bg-primary data-[state=on]:text-primary-foreground data-[state=on]:border-primary";
