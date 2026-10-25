import { toast } from "svelte-sonner";
import { inspectPaths } from "$lib/backend";
import { pluralise } from "$lib/formatting";
import type { FileQueue } from "$lib/queue.svelte";

export const WAIT_FOR_CONVERSION_MESSAGE = "Wait for the current conversion to finish.";
const MAX_NAMES_IN_TOAST = 3;

export type AddOutcome =
  | { kind: "nothing" }
  // A conversion is running, so nothing was added.
  | { kind: "busy" }
  // Names of files that aren't images or videos, so the UI can mention them.
  | { kind: "added"; unsupportedNames: string[] };

// Adds dropped or picked paths to the queue. Scanning a large folder can take
// a while, and a conversion can't start until every scan has finished.
export class FileAdder {
  #pendingScanCount = $state(0);
  readonly #queue: FileQueue;

  isAdding = $derived(this.#pendingScanCount > 0);

  constructor(queue: FileQueue) {
    this.#queue = queue;
  }

  async add(paths: string[]): Promise<AddOutcome> {
    if (paths.length === 0) {
      return { kind: "nothing" };
    }
    if (this.#queue.isConverting) {
      return { kind: "busy" };
    }

    this.#pendingScanCount += 1;
    try {
      const inspectedFiles = await inspectPaths(paths);
      // Checked again after the scan, so files never join a running conversion.
      if (this.#queue.isConverting) {
        return { kind: "busy" };
      }

      this.#queue.startNextBatchIfFinished();
      const unsupportedNames = this.#queue.add(inspectedFiles);
      return { kind: "added", unsupportedNames };
    } finally {
      this.#pendingScanCount -= 1;
    }
  }
}

function describeSkippedFiles(skippedNames: string[]): string {
  const listedNames = skippedNames.slice(0, MAX_NAMES_IN_TOAST).join(", ");
  const remainingCount = skippedNames.length - MAX_NAMES_IN_TOAST;
  const remainder = remainingCount > 0 ? ` and ${remainingCount} more` : "";

  const description = `${listedNames}${remainder}`;
  return description;
}

// Tells the user about files that weren't added.
export function reportAddOutcome(outcome: AddOutcome): void {
  if (outcome.kind === "busy") {
    toast.info(WAIT_FOR_CONVERSION_MESSAGE);
    return;
  }
  if (outcome.kind !== "added" || outcome.unsupportedNames.length === 0) {
    return;
  }

  const skippedNames = outcome.unsupportedNames;
  toast.warning(`Skipped ${pluralise(skippedNames.length, "unsupported file")}`, {
    description: describeSkippedFiles(skippedNames),
  });
}
