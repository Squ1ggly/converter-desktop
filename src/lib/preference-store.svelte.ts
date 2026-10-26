import { load, type Store } from "@tauri-apps/plugin-store";
import type { ConversionCatalog } from "$lib/backend";
import { DEFAULT_PREFERENCES, parseSavedPreferences, type Preferences } from "$lib/preferences";

// The store plugin keeps this file in the current user's app data folder,
// e.g. ~/Library/Application Support/com.squ1ggly.converter/ on macOS.
const STORE_FILE_NAME = "settings.json";
const STORE_KEY = "preferences";
const SAVE_DELAY_MS = 400;

// The user's last-used conversion choices, remembered between launches.
export class PreferenceStore {
  values = $state<Preferences>({ ...DEFAULT_PREFERENCES });
  #store: Store | null = null;

  // Rejects if the settings file can't be read. The defaults stay in use and
  // nothing is saved, so a damaged file is never overwritten.
  async load(): Promise<void> {
    const store = await load(STORE_FILE_NAME, { autoSave: SAVE_DELAY_MS, defaults: {} });
    const savedPreferences: unknown = await store.get(STORE_KEY);
    this.values = parseSavedPreferences(savedPreferences);
    this.#store = store;
  }

  // A saved format may not exist on this machine (HEIC output is macOS only).
  useAvailableFormats(catalog: ConversionCatalog): void {
    const isImageFormatAvailable = catalog.imageOutputs.some((option) => option.id === this.values.imageFormat);
    const isVideoFormatAvailable = catalog.videoOutputs.some((option) => option.id === this.values.videoFormat);

    if (!isImageFormatAvailable && catalog.imageOutputs[0]) {
      this.values.imageFormat = catalog.imageOutputs[0].id;
    }
    if (!isVideoFormatAvailable && catalog.videoOutputs[0]) {
      this.values.videoFormat = catalog.videoOutputs[0].id;
    }
  }

  // Call from an $effect: reading every value here makes the effect re-run
  // (and so re-save) whenever any preference changes.
  persist(): void {
    const snapshot = $state.snapshot(this.values);
    void this.#store?.set(STORE_KEY, snapshot);
  }
}
