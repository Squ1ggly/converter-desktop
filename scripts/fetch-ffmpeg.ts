// Downloads FFmpeg for one platform into src-tauri/binaries/ for Tauri to bundle, or downloads
// the FFmpeg source code that every bundled build was made from.
// The Windows ARM64 build is a .7z archive, which needs 7-Zip (`7z`) on the PATH.
//
//   pnpm fetch-ffmpeg
//   pnpm fetch-ffmpeg --target x86_64-pc-windows-msvc
//   pnpm fetch-ffmpeg --source <directory>
//
// Downloads are pinned and checked against a SHA-256 checksum.
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { chmod, mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { unzipSync } from "fflate";
import {
  FFMPEG_BUILDS,
  FFMPEG_SOURCE,
  FFMPEG_SOURCE_ARCHIVE_NAME,
  FFMPEG_VERSION,
  type FfmpegBuild,
} from "./ffmpeg-builds.ts";

const BINARIES_DIRECTORY = path.join(import.meta.dirname, "..", "src-tauri", "binaries");

function readFlagValue(flag: string): string | undefined {
  const flagIndex = process.argv.indexOf(flag);
  if (flagIndex === -1) {
    return undefined;
  }

  const value = process.argv[flagIndex + 1];
  return value;
}

// The Rust target triple of this machine, e.g. "aarch64-apple-darwin".
function detectHostTarget(): string {
  const rustcOutput = execFileSync("rustc", ["-vV"], { encoding: "utf8" });
  const hostLine = rustcOutput.split("\n").find((line) => line.startsWith("host: "));

  if (!hostLine) {
    throw new Error("Could not detect this machine's platform from `rustc -vV`");
  }

  const hostTarget = hostLine.slice("host: ".length).trim();
  return hostTarget;
}

async function downloadVerified(url: string, expectedSha256: string): Promise<Uint8Array> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Download failed (${response.status}) for ${url}`);
  }

  const downloadedBytes = new Uint8Array(await response.arrayBuffer());
  const actualSha256 = createHash("sha256").update(downloadedBytes).digest("hex");

  if (actualSha256 !== expectedSha256) {
    throw new Error(`Checksum mismatch for ${url}\n  expected ${expectedSha256}\n  got      ${actualSha256}`);
  }

  return downloadedBytes;
}

function extractFromZip(archiveBytes: Uint8Array, entryPath: string): Uint8Array | undefined {
  const extractedEntries = unzipSync(archiveBytes, {
    filter: (entry) => entry.name === entryPath,
  });
  const executableBytes = extractedEntries[entryPath];
  return executableBytes;
}

async function extractFrom7z(archiveBytes: Uint8Array, entryPath: string): Promise<Uint8Array> {
  const workDirectory = await mkdtemp(path.join(tmpdir(), "ffmpeg-archive-"));
  const archivePath = path.join(workDirectory, "ffmpeg.7z");

  try {
    await writeFile(archivePath, archiveBytes);
    const executableBytes = execFileSync("7z", ["e", "-so", archivePath, entryPath], {
      maxBuffer: 1024 * 1024 * 1024,
      stdio: ["ignore", "pipe", "inherit"],
    });
    return new Uint8Array(executableBytes);
  } finally {
    await rm(workDirectory, { recursive: true, force: true });
  }
}

async function extractExecutable(build: FfmpegBuild, archiveBytes: Uint8Array): Promise<Uint8Array> {
  const executableBytes = build.url.endsWith(".7z")
    ? await extractFrom7z(archiveBytes, build.entryPath)
    : extractFromZip(archiveBytes, build.entryPath);

  if (!executableBytes || executableBytes.length === 0) {
    throw new Error(`${build.entryPath} was not found inside the downloaded archive`);
  }

  return executableBytes;
}

function findBuild(target: string): FfmpegBuild {
  const build = FFMPEG_BUILDS[target];

  if (!build) {
    const supportedTargets = Object.keys(FFMPEG_BUILDS).join(", ");
    throw new Error(`No FFmpeg build is configured for ${target}. Supported: ${supportedTargets}`);
  }

  return build;
}

async function fetchExecutable(target: string): Promise<void> {
  const build = findBuild(target);

  console.log(`Downloading FFmpeg ${FFMPEG_VERSION} for ${target}`);
  const archiveBytes = await downloadVerified(build.url, build.sha256);
  const executableBytes = await extractExecutable(build, archiveBytes);

  // Tauri expects sidecars named "<name>-<target triple>[.exe]".
  const executableSuffix = target.includes("windows") ? ".exe" : "";
  const outputPath = path.join(BINARIES_DIRECTORY, `converter-ffmpeg-${target}${executableSuffix}`);

  await mkdir(BINARIES_DIRECTORY, { recursive: true });
  await writeFile(outputPath, executableBytes);
  await chmod(outputPath, 0o755);

  console.log(`Saved ${path.relative(process.cwd(), outputPath)}`);
}

async function fetchSource(outputDirectory: string): Promise<void> {
  console.log(`Downloading the FFmpeg ${FFMPEG_VERSION} source`);
  const tarballBytes = await downloadVerified(FFMPEG_SOURCE.url, FFMPEG_SOURCE.sha256);
  const outputPath = path.resolve(outputDirectory, FFMPEG_SOURCE_ARCHIVE_NAME);

  await mkdir(outputDirectory, { recursive: true });
  await writeFile(outputPath, tarballBytes);

  console.log(`Saved ${path.relative(process.cwd(), outputPath)}`);
}

async function main(): Promise<void> {
  const sourceDirectory = readFlagValue("--source");

  if (sourceDirectory) {
    await fetchSource(sourceDirectory);
    return;
  }

  const target = readFlagValue("--target") ?? detectHostTarget();
  await fetchExecutable(target);
}

await main();
