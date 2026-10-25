// The choices the settings offer, shared by the controls and by the check on
// saved settings, so a saved value is always one the controls can show.
import type { ResizeOption, VideoQuality } from "$lib/backend";

export type ResizeMode = ResizeOption["mode"];

export const RESIZE_MODES: { value: ResizeMode; label: string }[] = [
  { value: "original", label: "Original" },
  { value: "percent", label: "Scale" },
  { value: "fit", label: "Fit" },
];

export const MIN_IMAGE_QUALITY = 1;
export const MAX_IMAGE_QUALITY = 100;

export const DEFAULT_SCALE_PERCENT = 50;
export const MIN_SCALE_PERCENT = 1;
export const MAX_SCALE_PERCENT = 1000;

export const VIDEO_QUALITY_OPTIONS: { value: VideoQuality; label: string }[] = [
  { value: "high", label: "High" },
  { value: "balanced", label: "Balanced" },
  { value: "small", label: "Small" },
];

// null keeps the original.
export const RESOLUTION_OPTIONS: { value: number | null; label: string }[] = [
  { value: null, label: "Original" },
  { value: 2160, label: "2160p (4K)" },
  { value: 1440, label: "1440p" },
  { value: 1080, label: "1080p" },
  { value: 720, label: "720p" },
  { value: 480, label: "480p" },
];

export const FRAME_RATE_OPTIONS: { value: number | null; label: string }[] = [
  { value: null, label: "Original" },
  { value: 60, label: "60 fps" },
  { value: 30, label: "30 fps" },
  { value: 24, label: "24 fps" },
  { value: 15, label: "15 fps" },
];

export function defaultResizeFor(mode: ResizeMode): ResizeOption {
  if (mode === "percent") {
    return { mode, percent: DEFAULT_SCALE_PERCENT };
  }
  if (mode === "fit") {
    return { mode, maxWidth: null, maxHeight: null };
  }
  return { mode: "original" };
}
