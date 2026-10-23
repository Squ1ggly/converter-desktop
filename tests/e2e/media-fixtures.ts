// Small sample files made at test time with the FFmpeg that `pnpm fetch-ffmpeg`
// downloaded, plus checks that a converted file is valid.
import { execFile } from "node:child_process";
import { mkdtemp, open, readdir, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { promisify } from "node:util";

const runExecutable = promisify(execFile);

const BINARIES_DIRECTORY = path.resolve(import.meta.dirname, "..", "..", "src-tauri", "binaries");

export interface SampleMedia {
  folder: string;
  photoPath: string;
  videoPath: string;
  textPath: string;
  brokenVideoPath: string;
}

// Bytes that identify a file format, found `offset` bytes into the file.
export interface FileSignature {
  offset: number;
  text: string;
}

export const WEBP_SIGNATURE: FileSignature = { offset: 8, text: "WEBP" };
export const MP4_SIGNATURE: FileSignature = { offset: 4, text: "ftyp" };

async function findDownloadedFfmpeg(): Promise<string> {
  const fileNames = await readdir(BINARIES_DIRECTORY).catch(() => []);
  const ffmpegFileName = fileNames.find((fileName) => fileName.startsWith("ffmpeg-"));

  if (!ffmpegFileName) {
    throw new Error("No FFmpeg in src-tauri/binaries. Run `pnpm fetch-ffmpeg` first.");
  }

  const ffmpegPath = path.join(BINARIES_DIRECTORY, ffmpegFileName);
  return ffmpegPath;
}

// Resolves with FFmpeg's error output, which is empty when nothing went wrong.
async function runFfmpeg(ffmpegArguments: string[]): Promise<string> {
  const ffmpegPath = await findDownloadedFfmpeg();
  const { stderr } = await runExecutable(ffmpegPath, ["-hide_banner", "-loglevel", "error", "-y", ...ffmpegArguments]);
  return stderr;
}

export async function createSampleMedia(): Promise<SampleMedia> {
  const folder = await mkdtemp(path.join(os.tmpdir(), "converter-e2e-"));
  const sampleMedia: SampleMedia = {
    folder,
    photoPath: path.join(folder, "photo.png"),
    videoPath: path.join(folder, "clip.mov"),
    textPath: path.join(folder, "notes.txt"),
    brokenVideoPath: path.join(folder, "broken.mp4"),
  };

  await runFfmpeg(["-f", "lavfi", "-i", "testsrc2=size=320x240", "-frames:v", "1", sampleMedia.photoPath]);
  await runFfmpeg([
    ...["-f", "lavfi", "-i", "testsrc2=size=320x240:rate=15:duration=1"],
    ...["-f", "lavfi", "-i", "sine=frequency=440:duration=1"],
    ...["-c:v", "mpeg4", "-c:a", "aac", "-shortest", sampleMedia.videoPath],
  ]);
  await writeFile(sampleMedia.textPath, "Not an image or a video.");
  await writeFile(sampleMedia.brokenVideoPath, "Not really a video.");

  return sampleMedia;
}

// Decoding every frame is the strictest check that a file is valid.
export async function decodesWithoutErrors(filePath: string): Promise<boolean> {
  try {
    const errorOutput = await runFfmpeg(["-xerror", "-i", filePath, "-f", "null", "-"]);
    const hasNoErrors = errorOutput.trim() === "";
    return hasNoErrors;
  } catch {
    return false;
  }
}

export async function hasSignature(filePath: string, signature: FileSignature): Promise<boolean> {
  const file = await open(filePath);
  const signatureBytes = Buffer.alloc(signature.text.length);
  await file.read(signatureBytes, 0, signatureBytes.length, signature.offset);
  await file.close();

  const matchesSignature = signatureBytes.toString("latin1") === signature.text;
  return matchesSignature;
}
