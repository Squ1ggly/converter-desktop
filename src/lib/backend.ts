// Typed wrappers around the Rust commands in src-tauri/src/commands.rs.
// The shapes here mirror the Rust structs' serde output (camelCase fields).
import { Channel, invoke } from "@tauri-apps/api/core";

export type MediaKind = "image" | "video";

export interface FormatOption {
  id: string;
  label: string;
  description: string;
  usesQuality: boolean;
}

export interface ConversionCatalog {
  platform: string;
  imageOutputs: FormatOption[];
  videoOutputs: FormatOption[];
  imageInputExtensions: string[];
  videoInputExtensions: string[];
}

export interface InspectedFile {
  path: string;
  name: string;
  sizeBytes: number;
  // null when the app can't convert this file.
  kind: MediaKind | null;
}

export type ResizeOption =
  | { mode: "original" }
  | { mode: "percent"; percent: number }
  | { mode: "fit"; maxWidth: number | null; maxHeight: number | null };

export interface ImageOptions {
  quality: number;
  resize: ResizeOption;
}

export type VideoQuality = "high" | "balanced" | "small";

export interface VideoOptions {
  quality: VideoQuality;
  maxHeight: number | null;
  frameRate: number | null;
}

export type ConversionSettings =
  | { kind: "image"; format: string; options: ImageOptions }
  | { kind: "video"; format: string; options: VideoOptions };

export interface ConversionRequest {
  batchId: string;
  files: { fileId: string; path: string }[];
  settings: ConversionSettings;
  // null saves each result next to its original.
  outputDirectory: string | null;
}

export type ConversionEvent =
  | { type: "started"; fileId: string }
  | { type: "progress"; fileId: string; percent: number }
  | { type: "completed"; fileId: string; outputPath: string }
  | { type: "failed"; fileId: string; message: string }
  | { type: "cancelled"; fileId: string };

export function getCatalog(): Promise<ConversionCatalog> {
  return invoke<ConversionCatalog>("get_catalog");
}

export function inspectPaths(paths: string[]): Promise<InspectedFile[]> {
  return invoke<InspectedFile[]>("inspect_paths", { paths });
}

// Resolves once every file in the request has finished, failed or been
// cancelled. Per-file progress arrives through `onEvent` in the meantime.
export function convertFiles(request: ConversionRequest, onEvent: (event: ConversionEvent) => void): Promise<void> {
  const eventChannel = new Channel<ConversionEvent>();
  eventChannel.onmessage = onEvent;

  return invoke<void>("convert_files", { request, onEvent: eventChannel });
}

export function cancelConversion(batchId: string): Promise<void> {
  return invoke<void>("cancel_conversion", { batchId });
}

export function revealOutput(path: string): Promise<void> {
  return invoke<void>("reveal_output", { path });
}

export function openOutput(path: string): Promise<void> {
  return invoke<void>("open_output", { path });
}
