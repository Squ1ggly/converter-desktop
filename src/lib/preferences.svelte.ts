import { load, type Store } from "@tauri-apps/plugin-store";
import type { ResizeOption, VideoQuality } from "$lib/backend";

export type SaveLocation = { mode: "besideOriginal" } | { mode: "folder"; path: string };

export interface Preferences {
  imageFormat: string;
  imageQuality: number;
  resize: ResizeOption;
  videoFormat: string;
  videoQuality: VideoQuality;
  maxHeight: number | null;
  frameRate: number | null;
  saveLocation: SaveLocation;
}

const DEFAULT_PREFERENCES: Preferences = {
  imageFormat: "jpg",
  imageQuality: 85,
  resize: { mode: "original" },
  videoFormat: "mp4",
  videoQuality: "balanced",
  maxHeight: null,
  frameRate: null,
  saveLocation: { mode: "besideOriginal" },
};

// The store plugin keeps this file in the current user's app data folder,
// e.g. ~/Library/Application Support/com.squ1ggly.converter/ on macOS.
const STORE_FILE_NAME = "settings.json";
const STORE_KEY = "preferences";
const SAVE_DELAY_MS = 400;

// The user's last-used conversion choices, remembered between launches.
export class PreferenceStore {
  values = $state<Preferences>({ ...DEFAULT_PREFERENCES });
  #store: Store | null = null;

  async load(): Promise<void> {
    this.#store = await load(STORE_FILE_NAME, { autoSave: SAVE_DELAY_MS, defaults: {} });
    const savedPreferences = await this.#store.get<Partial<Preferences>>(STORE_KEY);
    this.values = { ...DEFAULT_PREFERENCES, ...savedPreferences };
  }

  // Call from an $effect: reading every value here makes the effect re-run
  // (and so re-save) whenever any preference changes.
  persist(): void {
    const snapshot = $state.snapshot(this.values);
    void this.#store?.set(STORE_KEY, snapshot);
  }
}
