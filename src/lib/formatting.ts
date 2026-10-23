const BYTES_PER_KILOBYTE = 1024;
const SIZE_UNITS = ["KB", "MB", "GB", "TB"];

export function formatBytes(byteCount: number): string {
  if (byteCount < BYTES_PER_KILOBYTE) {
    return `${byteCount} B`;
  }

  let size = byteCount / BYTES_PER_KILOBYTE;
  let unitIndex = 0;
  while (size >= BYTES_PER_KILOBYTE && unitIndex < SIZE_UNITS.length - 1) {
    size /= BYTES_PER_KILOBYTE;
    unitIndex += 1;
  }

  const decimalPlaces = size < 10 ? 1 : 0;
  const formattedSize = `${size.toFixed(decimalPlaces)} ${SIZE_UNITS[unitIndex]}`;
  return formattedSize;
}

export function pluralise(count: number, singular: string, plural = `${singular}s`): string {
  const noun = count === 1 ? singular : plural;
  const phrase = `${count} ${noun}`;
  return phrase;
}

export function extensionOf(fileName: string): string {
  const dotIndex = fileName.lastIndexOf(".");
  if (dotIndex === -1) {
    return "";
  }

  const extension = fileName.slice(dotIndex + 1).toUpperCase();
  return extension;
}

// Uses each platform's own name for its file manager.
export function revealInFolderLabel(platform: string): string {
  if (platform === "macos") {
    return "Show in Finder";
  }
  if (platform === "windows") {
    return "Show in Explorer";
  }
  return "Show in folder";
}

export type Shortcut = "open" | "convert" | "settings" | "help";

// Key labels as each OS shows them, e.g. ⌘ O on macOS and Ctrl O elsewhere.
export function shortcutKeys(shortcut: Shortcut, platform: string): string[] {
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

export function isPrimaryModifierPressed(event: KeyboardEvent, platform: string): boolean {
  const isPressed = platform === "macos" ? event.metaKey : event.ctrlKey;
  return isPressed;
}
