import { getCurrentWindow } from "@tauri-apps/api/window";
import type { ConversionRun } from "$lib/conversion-run.svelte";
import { showError } from "$lib/errors";
import type { FileQueue } from "$lib/queue.svelte";

// Closing mid-conversion asks first. Quitting then cancels every file, so
// nothing is left half-written, and closes the window.
export class QuitGuard {
  isDialogOpen = $state(false);
  isQuitting = $state(false);
  readonly #queue: FileQueue;
  readonly #conversionRun: ConversionRun;

  constructor(queue: FileQueue, conversionRun: ConversionRun) {
    this.#queue = queue;
    this.#conversionRun = conversionRun;
  }

  // Returns a function that stops listening.
  async listenForClose(): Promise<() => void> {
    const stopListening = await getCurrentWindow().onCloseRequested((event) => {
      if (!this.#queue.isConverting) {
        return;
      }
      event.preventDefault();
      this.isDialogOpen = true;
    });
    return stopListening;
  }

  async quit(): Promise<void> {
    if (this.isQuitting) {
      return;
    }

    this.isQuitting = true;
    try {
      await this.#conversionRun.cancel();
      await this.#conversionRun.waitUntilFinished();
    } catch (error) {
      // The window still closes. The error stays visible if closing fails too.
      showError("Couldn't cancel the conversion", error);
    } finally {
      await this.#destroyWindow();
    }
  }

  async #destroyWindow(): Promise<void> {
    try {
      await getCurrentWindow().destroy();
    } catch (error) {
      this.isQuitting = false;
      showError("Couldn't quit", error);
    }
  }
}
