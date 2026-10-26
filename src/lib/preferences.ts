import type { ResizeOption, VideoQuality } from "$lib/backend";
import {
  DEFAULT_SCALE_PERCENT,
  FRAME_RATE_OPTIONS,
  MAX_IMAGE_QUALITY,
  MAX_SCALE_PERCENT,
  MIN_IMAGE_QUALITY,
  MIN_SCALE_PERCENT,
  RESOLUTION_OPTIONS,
  VIDEO_QUALITY_OPTIONS,
} from "$lib/conversion-options";

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

export const DEFAULT_PREFERENCES: Preferences = {
  imageFormat: "jpg",
  imageQuality: 85,
  resize: { mode: "original" },
  videoFormat: "mp4",
  videoQuality: "balanced",
  maxHeight: null,
  frameRate: null,
  saveLocation: { mode: "besideOriginal" },
};

type UnknownRecord = Record<string, unknown>;

function isRecord(value: unknown): value is UnknownRecord {
  const isPlainObject = typeof value === "object" && value !== null && !Array.isArray(value);
  return isPlainObject;
}

function isWholeNumberInRange(value: unknown, minimum: number, maximum: number): value is number {
  const isInRange = typeof value === "number" && Number.isInteger(value) && value >= minimum && value <= maximum;
  return isInRange;
}

function readNonEmptyString(value: unknown, fallback: string): string {
  const isUsable = typeof value === "string" && value.length > 0;
  if (!isUsable) {
    return fallback;
  }
  return value;
}

function readOptionValue<Value>(value: unknown, options: { value: Value }[], fallback: Value): Value {
  const matchingOption = options.find((option) => option.value === value);
  const optionValue = matchingOption ? matchingOption.value : fallback;
  return optionValue;
}

function readSideLimit(value: unknown): number | null {
  if (!isWholeNumberInRange(value, 1, Number.MAX_SAFE_INTEGER)) {
    return null;
  }
  return value;
}

function readScalePercent(value: unknown): number {
  if (!isWholeNumberInRange(value, MIN_SCALE_PERCENT, MAX_SCALE_PERCENT)) {
    return DEFAULT_SCALE_PERCENT;
  }
  return value;
}

function readResize(value: unknown): ResizeOption {
  if (!isRecord(value)) {
    return DEFAULT_PREFERENCES.resize;
  }

  if (value.mode === "percent") {
    const percentResize: ResizeOption = { mode: "percent", percent: readScalePercent(value.percent) };
    return percentResize;
  }
  if (value.mode === "fit") {
    const fitResize: ResizeOption = {
      mode: "fit",
      maxWidth: readSideLimit(value.maxWidth),
      maxHeight: readSideLimit(value.maxHeight),
    };
    return fitResize;
  }
  return DEFAULT_PREFERENCES.resize;
}

function readImageQuality(value: unknown): number {
  if (!isWholeNumberInRange(value, MIN_IMAGE_QUALITY, MAX_IMAGE_QUALITY)) {
    return DEFAULT_PREFERENCES.imageQuality;
  }
  return value;
}

function readSaveLocation(value: unknown): SaveLocation {
  if (!isRecord(value) || value.mode !== "folder") {
    return DEFAULT_PREFERENCES.saveLocation;
  }

  const folderPath = readNonEmptyString(value.path, "");
  if (folderPath === "") {
    return DEFAULT_PREFERENCES.saveLocation;
  }

  const folderLocation: SaveLocation = { mode: "folder", path: folderPath };
  return folderLocation;
}

// Settings come from a file on disk that may be old, edited by hand or
// damaged. Each field that isn't valid falls back to its default on its own.
export function parseSavedPreferences(saved: unknown): Preferences {
  const fields: UnknownRecord = isRecord(saved) ? saved : {};
  const defaults = DEFAULT_PREFERENCES;

  const preferences: Preferences = {
    imageFormat: readNonEmptyString(fields.imageFormat, defaults.imageFormat),
    imageQuality: readImageQuality(fields.imageQuality),
    resize: readResize(fields.resize),
    videoFormat: readNonEmptyString(fields.videoFormat, defaults.videoFormat),
    videoQuality: readOptionValue(fields.videoQuality, VIDEO_QUALITY_OPTIONS, defaults.videoQuality),
    maxHeight: readOptionValue(fields.maxHeight, RESOLUTION_OPTIONS, defaults.maxHeight),
    frameRate: readOptionValue(fields.frameRate, FRAME_RATE_OPTIONS, defaults.frameRate),
    saveLocation: readSaveLocation(fields.saveLocation),
  };
  return preferences;
}
