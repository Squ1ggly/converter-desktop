import { convertFileSrc } from "@tauri-apps/api/core";
import type { ConversionEvent, InspectedFile, MediaKind } from "./backend";

export type ItemStatus = "ready" | "waiting" | "converting" | "done" | "failed" | "cancelled";

export interface QueueItem {
  id: string;
  path: string;
  name: string;
  sizeBytes: number;
  kind: MediaKind;
  status: ItemStatus;
  // null while a file converts without measurable progress (images, or videos
  // FFmpeg can't measure).
  progressPercent: number | null;
  outputPath: string | null;
  errorMessage: string | null;
  previewUrl: string | null;
}

export interface QueueSummary {
  doneCount: number;
  failedCount: number;
  cancelledCount: number;
  finishedCount: number;
}

const FINISHED_STATUSES: ReadonlySet<ItemStatus> = new Set(["done", "failed", "cancelled"]);
const IN_PROGRESS_STATUSES: ReadonlySet<ItemStatus> = new Set(["waiting", "converting"]);

function createQueueItem(file: InspectedFile, kind: MediaKind): QueueItem {
  // Images get a thumbnail straight from disk; the Rust side has already
  // granted the asset protocol access to exactly this file.
  const previewUrl = kind === "image" ? convertFileSrc(file.path) : null;

  const queueItem: QueueItem = {
    id: crypto.randomUUID(),
    path: file.path,
    name: file.name,
    sizeBytes: file.sizeBytes,
    kind,
    status: "ready",
    progressPercent: null,
    outputPath: null,
    errorMessage: null,
    previewUrl,
  };
  return queueItem;
}

function summarise(items: QueueItem[]): QueueSummary {
  const doneCount = items.filter((item) => item.status === "done").length;
  const failedCount = items.filter((item) => item.status === "failed").length;
  const cancelledCount = items.filter((item) => item.status === "cancelled").length;

  const summary: QueueSummary = {
    doneCount,
    failedCount,
    cancelledCount,
    finishedCount: doneCount + failedCount + cancelledCount,
  };
  return summary;
}

// The list of files the user has added, and each one's conversion state.
export class FileQueue {
  items = $state<QueueItem[]>([]);

  images = $derived(this.items.filter((item) => item.kind === "image"));
  videos = $derived(this.items.filter((item) => item.kind === "video"));
  isEmpty = $derived(this.items.length === 0);
  isConverting = $derived(this.items.some((item) => IN_PROGRESS_STATUSES.has(item.status)));
  hasFinishedRun = $derived(this.items.length > 0 && this.items.every((item) => FINISHED_STATUSES.has(item.status)));
  summary = $derived(summarise(this.items));

  // Adds convertible files, skipping ones already in the list. Returns the
  // names of files that aren't images or videos, so the UI can mention them.
  add(files: InspectedFile[]): string[] {
    const existingPaths = new Set(this.items.map((item) => item.path));
    const unsupportedNames: string[] = [];

    for (const file of files) {
      if (file.kind === null) {
        unsupportedNames.push(file.name);
        continue;
      }
      if (existingPaths.has(file.path)) {
        continue;
      }

      existingPaths.add(file.path);
      this.items.push(createQueueItem(file, file.kind));
    }

    return unsupportedNames;
  }

  // Adding files after a finished conversion starts the next batch: files
  // that converted successfully leave the list, and ones that failed or were
  // cancelled stay so they can be tried again.
  startNextBatchIfFinished(): void {
    if (!this.hasFinishedRun) {
      return;
    }

    this.items = this.items.filter((item) => item.status !== "done");
    for (const item of this.items) {
      item.status = "ready";
      item.errorMessage = null;
      item.progressPercent = null;
    }
  }

  remove(itemId: string): void {
    this.items = this.items.filter((item) => item.id !== itemId);
  }

  clear(): void {
    this.items = [];
  }

  // Re-running a conversion (e.g. after changing the format) converts every
  // file again from scratch.
  prepareForRun(): void {
    for (const item of this.items) {
      item.status = "waiting";
      item.progressPercent = null;
      item.outputPath = null;
      item.errorMessage = null;
    }
  }

  applyEvent(event: ConversionEvent): void {
    const item = this.items.find((queuedItem) => queuedItem.id === event.fileId);
    if (!item) {
      return;
    }

    switch (event.type) {
      case "started":
        item.status = "converting";
        break;
      case "progress":
        item.progressPercent = event.percent;
        break;
      case "completed":
        item.status = "done";
        item.progressPercent = 100;
        item.outputPath = event.outputPath;
        break;
      case "failed":
        item.status = "failed";
        item.errorMessage = event.message;
        break;
      case "cancelled":
        item.status = "cancelled";
        break;
    }
  }

  // Covers files whose batch ended without an event for them, e.g. if the
  // backend returned an error before starting.
  markUnfinishedAsFailed(message: string): void {
    for (const item of this.items) {
      if (!IN_PROGRESS_STATUSES.has(item.status)) {
        continue;
      }
      item.status = "failed";
      item.errorMessage = message;
    }
  }
}
