// Keyboard shortcuts: which keys trigger each one, and how each OS labels them.
export type Shortcut = "open" | "convert" | "settings" | "help";

const KEYS_WITH_PRIMARY_MODIFIER: Record<string, Shortcut> = {
  o: "open",
  enter: "convert",
  ",": "settings",
};

// Until the backend reports the platform, no keys are shown or handled, so
// macOS never briefly shows Ctrl or reacts to it.
export const UNKNOWN_PLATFORM = "";

// Key labels as each OS shows them, e.g. ⌘ O on macOS and Ctrl O elsewhere.
export function shortcutKeys(shortcut: Shortcut, platform: string): string[] {
  if (platform === UNKNOWN_PLATFORM) {
    return [];
  }
  const isMac = platform === "macos";
  const modifierKey = isMac ? "⌘" : "Ctrl";
  const enterKey = isMac ? "↩" : "Enter";

  const keysByShortcut: Record<Shortcut, string[]> = {
    open: [modifierKey, "O"],
    convert: [modifierKey, enterKey],
    settings: [modifierKey, ","],
    help: ["F1"],
  };
  const keys = keysByShortcut[shortcut];
  return keys;
}

function isPrimaryModifierPressed(event: KeyboardEvent, platform: string): boolean {
  const isPressed = platform === "macos" ? event.metaKey : event.ctrlKey;
  return isPressed;
}

export function shortcutFor(event: KeyboardEvent, platform: string): Shortcut | null {
  if (platform === UNKNOWN_PLATFORM) {
    return null;
  }
  if (event.key === "F1") {
    return "help";
  }
  if (!isPrimaryModifierPressed(event, platform)) {
    return null;
  }

  const shortcut = KEYS_WITH_PRIMARY_MODIFIER[event.key.toLowerCase()] ?? null;
  return shortcut;
}
