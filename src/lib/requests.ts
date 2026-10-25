import type { ConversionRequest, ConversionSettings, MediaKind } from "$lib/backend";
import type { Preferences } from "$lib/preferences";
import type { QueueItem } from "$lib/queue.svelte";

function buildSettings(kind: MediaKind, preferences: Preferences): ConversionSettings {
  if (kind === "image") {
    const imageSettings: ConversionSettings = {
      kind,
      format: preferences.imageFormat,
      options: { quality: preferences.imageQuality, resize: preferences.resize },
    };
    return imageSettings;
  }

  const videoSettings: ConversionSettings = {
    kind,
    format: preferences.videoFormat,
    options: { quality: preferences.videoQuality, maxHeight: preferences.maxHeight, frameRate: preferences.frameRate },
  };
  return videoSettings;
}

function buildRequestForKind(kind: MediaKind, items: QueueItem[], preferences: Preferences): ConversionRequest | null {
  const itemsOfKind = items.filter((item) => item.kind === kind);
  if (itemsOfKind.length === 0) {
    return null;
  }

  const outputDirectory = preferences.saveLocation.mode === "folder" ? preferences.saveLocation.path : null;
  const request: ConversionRequest = {
    batchId: crypto.randomUUID(),
    files: itemsOfKind.map((item) => ({ fileId: item.id, path: item.path })),
    settings: buildSettings(kind, preferences),
    outputDirectory,
  };
  return request;
}

// Images and videos have different settings, so each kind goes to the
// backend as its own batch. Both batches run at the same time.
export function buildConversionRequests(items: QueueItem[], preferences: Preferences): ConversionRequest[] {
  const kinds: MediaKind[] = ["image", "video"];
  const requests = kinds
    .map((kind) => buildRequestForKind(kind, items, preferences))
    .filter((request): request is ConversionRequest => request !== null);
  return requests;
}
