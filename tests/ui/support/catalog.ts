// A copy of what get_catalog returns, from src-tauri/src/formats.rs.
import type { ConversionCatalog, FormatOption, MediaKind } from "$lib/backend";
import type { Platform } from "./scenario";

const IMAGE_INPUT_EXTENSIONS = [
  "avif",
  "bmp",
  "gif",
  "heic",
  "heif",
  "jpeg",
  "jpg",
  "pdf",
  "png",
  "svg",
  "tif",
  "tiff",
  "webp",
];

const VIDEO_INPUT_EXTENSIONS = [
  "3gp",
  "avi",
  "flv",
  "m2ts",
  "m4v",
  "mkv",
  "mov",
  "mp4",
  "mpeg",
  "mpg",
  "mts",
  "ogv",
  "ts",
  "webm",
  "wmv",
];

const IMAGE_OUTPUTS: FormatOption[] = [
  { id: "jpg", label: "JPG", description: "Photos. Opens anywhere. No transparency.", usesQuality: true },
  { id: "png", label: "PNG", description: "Lossless. Keeps transparency.", usesQuality: false },
  { id: "webp", label: "WebP", description: "Smaller than JPG. Keeps transparency.", usesQuality: true },
  { id: "avif", label: "AVIF", description: "Smallest files. Modern browsers only.", usesQuality: true },
  { id: "heic", label: "HEIC", description: "Apple photo format.", usesQuality: true },
  { id: "gif", label: "GIF", description: "256 colours. Simple graphics.", usesQuality: false },
  { id: "bmp", label: "BMP", description: "Uncompressed. Large files.", usesQuality: false },
  { id: "tiff", label: "TIFF", description: "Lossless. Print and archiving.", usesQuality: false },
  { id: "pdf", label: "PDF", description: "One page per image.", usesQuality: true },
  { id: "svg", label: "SVG", description: "Traced to vector shapes. Logos and icons.", usesQuality: false },
];

const VIDEO_OUTPUTS: FormatOption[] = [
  { id: "mp4", label: "MP4", description: "Plays almost everywhere.", usesQuality: true },
  { id: "mov", label: "MOV", description: "QuickTime. Common on Mac.", usesQuality: true },
  { id: "mkv", label: "MKV", description: "Flexible container.", usesQuality: true },
  { id: "webm", label: "WebM", description: "Open web format. Small files.", usesQuality: true },
  { id: "m4v", label: "M4V", description: "Apple's MP4 variant.", usesQuality: true },
  { id: "avi", label: "AVI", description: "Legacy Windows format.", usesQuality: true },
  { id: "flv", label: "FLV", description: "Legacy Flash video.", usesQuality: true },
  { id: "gif", label: "GIF", description: "Silent animation. Keep clips short.", usesQuality: true },
];

export function buildCatalog(platform: Platform): ConversionCatalog {
  // HEIC output is macOS only.
  const imageOutputs = IMAGE_OUTPUTS.filter((option) => option.id !== "heic" || platform === "macos");

  const catalog: ConversionCatalog = {
    platform,
    imageOutputs,
    videoOutputs: VIDEO_OUTPUTS,
    imageInputExtensions: IMAGE_INPUT_EXTENSIONS,
    videoInputExtensions: VIDEO_INPUT_EXTENSIONS,
  };
  return catalog;
}

export function classifyFileName(fileName: string): MediaKind | null {
  const dotIndex = fileName.lastIndexOf(".");
  if (dotIndex === -1) {
    return null;
  }

  const extension = fileName.slice(dotIndex + 1).toLowerCase();

  if (IMAGE_INPUT_EXTENSIONS.includes(extension)) {
    return "image";
  }
  if (VIDEO_INPUT_EXTENSIONS.includes(extension)) {
    return "video";
  }
  return null;
}
