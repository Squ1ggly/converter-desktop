// Shared between the Playwright tests (Node) and the mock backend (browser).
import type { OpenDialogOptions } from "@tauri-apps/plugin-dialog";
import type { ConversionEvent, ConversionRequest } from "$lib/backend";
import type { Preferences } from "$lib/preferences.svelte";

export type Platform = "macos" | "windows" | "linux";

export type ConversionMode = "automatic" | "manual";

export interface MockScenario {
  platform: Platform;
  // Files inside each folder, returned when the folder itself is added.
  folders: Record<string, string[]>;
  savedPreferences: Partial<Preferences> | null;
  // "automatic" converts every file straight away. "manual" waits for the
  // test to send each file's events.
  conversionMode: ConversionMode;
  // File names that fail in automatic mode, with the error to report.
  failingFiles: Record<string, string>;
}

type WithoutFileId<Event> = Event extends ConversionEvent ? Omit<Event, "fileId"> : never;

// A conversion event for one file, addressed by file name instead of id.
export type FileConversionEvent = WithoutFileId<ConversionEvent>;

export type DragEventType = "enter" | "drop" | "leave";

// Everything the app asked the backend to do, in the order it asked.
export interface BackendLog {
  dialogs: OpenDialogOptions[];
  conversionRequests: ConversionRequest[];
  cancelledBatchIds: string[];
  openedOutputs: string[];
  revealedOutputs: string[];
  storedPreferences: Partial<Preferences> | null;
}

// The browser half of the mock backend, driven by the tests through page.evaluate.
export interface MockBackendControls {
  setNextDialogResult(result: string[] | string | null): void;
  emitDragEvent(type: DragEventType, paths: string[]): Promise<void>;
  sendFileEvent(fileName: string, event: FileConversionEvent): void;
  readLog(): BackendLog;
}

declare global {
  interface Window {
    converterScenario?: MockScenario;
    // Set by the mock backend before the app starts.
    converterMockBackend: MockBackendControls;
  }
}
