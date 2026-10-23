// Downloads FFmpeg for one platform into src-tauri/binaries/ for Tauri to bundle.
//
//   pnpm fetch-ffmpeg
//   pnpm fetch-ffmpeg --target x86_64-pc-windows-msvc
//
// Downloads are pinned and checked against a SHA-256 checksum.
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { chmod, mkdir, writeFile } from "node:fs/promises";
import path from "node:path";
import { unzipSync } from "fflate";

interface FfmpegDownload {
  url: string;
  sha256: string;
  // Path of the ffmpeg executable inside the zip.
  entryPath: string;
}

const FFMPEG_VERSION = "9.0.2";

// macOS and Linux: static builds from https://ffmpeg.martin-riedl.de
// Windows: "essentials" build from https://www.gyan.dev/ffmpeg/builds/
const DOWNLOADS: Record<string, FfmpegDownload> = {
  "aarch64-apple-darwin": {
    url: "https://ffmpeg.martin-riedl.de/download/macos/arm64/1789931890_9.0.2/ffmpeg.zip",
    sha256: "c8ed4c4e6978a03c485edbfe4e0a5dc2380f8a30bba5150531b31b094492d924",
    entryPath: "ffmpeg",
  },
  "x86_64-apple-darwin": {
    url: "https://ffmpeg.martin-riedl.de/download/macos/amd64/1789931006_9.0.2/ffmpeg.zip",
    sha256: "7c6b4125b191cbf773832dc51f424cf2b6bb7da43007d1e066f95909e47cacd4",
    entryPath: "ffmpeg",
  },
  "x86_64-unknown-linux-gnu": {
    url: "https://ffmpeg.martin-riedl.de/download/linux/amd64/1789931100_9.0.2/ffmpeg.zip",
    sha256: "fa8ecf4abbd290d98f7d188b8649cc6b391ae209a98452be955a15aab1909d7f",
    entryPath: "ffmpeg",
  },
  "aarch64-unknown-linux-gnu": {
    url: "https://ffmpeg.martin-riedl.de/download/linux/arm64/1789931697_9.0.2/ffmpeg.zip",
    sha256: "93a76ae90db5474eecdf951a729857c64f3de23567228d6a7d5e6e8e3cd1021b",
    entryPath: "ffmpeg",
  },
  "x86_64-pc-windows-msvc": {
    url: `https://github.com/GyanD/codexffmpeg/releases/download/${FFMPEG_VERSION}/ffmpeg-${FFMPEG_VERSION}-essentials_build.zip`,
    sha256: "60f467265b1e312373dbcd92200c2618a74850f98d3d078e94296bb3fa2047ba",
    entryPath: `ffmpeg-${FFMPEG_VERSION}-essentials_build/bin/ffmpeg.exe`,
  },
};

const BINARIES_DIRECTORY = path.join(import.meta.dirname, "..", "src-tauri", "binaries");

function readTargetArgument(): string | undefined {
  const targetFlagIndex = process.argv.indexOf("--target");
  if (targetFlagIndex === -1) {
    return undefined;
  }

  const target = process.argv[targetFlagIndex + 1];
  return target;
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

async function downloadVerified(download: FfmpegDownload): Promise<Uint8Array> {
  const response = await fetch(download.url);
  if (!response.ok) {
    throw new Error(`Download failed (${response.status}) for ${download.url}`);
  }

  const archiveBytes = new Uint8Array(await response.arrayBuffer());
  const actualSha256 = createHash("sha256").update(archiveBytes).digest("hex");

  if (actualSha256 !== download.sha256) {
    throw new Error(`Checksum mismatch for ${download.url}\n  expected ${download.sha256}\n  got      ${actualSha256}`);
  }

  return archiveBytes;
}

function extractExecutable(archiveBytes: Uint8Array, entryPath: string): Uint8Array {
  const extractedEntries = unzipSync(archiveBytes, {
    filter: (entry) => entry.name === entryPath,
  });
  const executableBytes = extractedEntries[entryPath];

  if (!executableBytes) {
    throw new Error(`${entryPath} was not found inside the downloaded archive`);
  }

  return executableBytes;
}

async function main(): Promise<void> {
  const target = readTargetArgument() ?? detectHostTarget();
  const download = DOWNLOADS[target];

  if (!download) {
    const supportedTargets = Object.keys(DOWNLOADS).join(", ");
    throw new Error(`No FFmpeg build is configured for ${target}. Supported: ${supportedTargets}`);
  }

  console.log(`Downloading FFmpeg ${FFMPEG_VERSION} for ${target}`);
  const archiveBytes = await downloadVerified(download);
  const executableBytes = extractExecutable(archiveBytes, download.entryPath);

  // Tauri expects sidecars named "<name>-<target triple>[.exe]".
  const executableSuffix = target.includes("windows") ? ".exe" : "";
  const outputPath = path.join(BINARIES_DIRECTORY, `ffmpeg-${target}${executableSuffix}`);

  await mkdir(BINARIES_DIRECTORY, { recursive: true });
  await writeFile(outputPath, executableBytes);
  await chmod(outputPath, 0o755);

  console.log(`Saved ${path.relative(process.cwd(), outputPath)}`);
}

await main();
