// Downloads FFmpeg for one platform into src-tauri/binaries/ for Tauri to bundle, or downloads
// the FFmpeg source code the bundled builds were made from.
//
//   pnpm fetch-ffmpeg
//   pnpm fetch-ffmpeg --target x86_64-pc-windows-msvc
//   pnpm fetch-ffmpeg --sources <directory>
//
// Downloads are pinned and checked against a SHA-256 checksum.
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { chmod, mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { unzipSync } from "fflate";
import {
  distinctSources,
  FFMPEG_BUILDS,
  type FfmpegBuild,
  type FfmpegCommitSource,
  type FfmpegSource,
  sourceArchiveName,
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

  console.log(`Downloading FFmpeg ${build.source.version} for ${target}`);
  const archiveBytes = await downloadVerified(build.url, build.sha256);
  const executableBytes = extractExecutable(archiveBytes, build.entryPath);

  // Tauri expects sidecars named "<name>-<target triple>[.exe]".
  const executableSuffix = target.includes("windows") ? ".exe" : "";
  const outputPath = path.join(BINARIES_DIRECTORY, `converter-ffmpeg-${target}${executableSuffix}`);

  await mkdir(BINARIES_DIRECTORY, { recursive: true });
  await writeFile(outputPath, executableBytes);
  await chmod(outputPath, 0o755);

  console.log(`Saved ${path.relative(process.cwd(), outputPath)}`);
}

// Git checks the commit hash, so the snapshot needs no separate checksum.
async function archiveCommit(source: FfmpegCommitSource, outputPath: string): Promise<void> {
  const repositoryDirectory = await mkdtemp(path.join(tmpdir(), "ffmpeg-source-"));
  const runGit = (gitArguments: string[]) => {
    execFileSync("git", gitArguments, { cwd: repositoryDirectory, stdio: "inherit" });
  };

  try {
    runGit(["init", "--quiet"]);
    runGit(["fetch", "--quiet", "--depth", "1", source.repository, source.commit]);
    runGit(["archive", "--format", "tar.gz", "--prefix", `ffmpeg-${source.version}/`, "--output", outputPath, source.commit]);
  } finally {
    await rm(repositoryDirectory, { recursive: true, force: true });
  }
}

async function fetchSource(source: FfmpegSource, outputDirectory: string): Promise<void> {
  const outputPath = path.resolve(outputDirectory, sourceArchiveName(source));
  console.log(`Downloading the FFmpeg ${source.version} source`);

  if (source.kind === "commit") {
    await archiveCommit(source, outputPath);
  } else {
    const tarballBytes = await downloadVerified(source.url, source.sha256);
    await writeFile(outputPath, tarballBytes);
  }

  console.log(`Saved ${path.relative(process.cwd(), outputPath)}`);
}

async function fetchSources(outputDirectory: string): Promise<void> {
  await mkdir(outputDirectory, { recursive: true });

  for (const source of distinctSources()) {
    await fetchSource(source, outputDirectory);
  }
}

async function main(): Promise<void> {
  const sourcesDirectory = readFlagValue("--sources");

  if (sourcesDirectory) {
    await fetchSources(sourcesDirectory);
    return;
  }

  const target = readFlagValue("--target") ?? detectHostTarget();
  await fetchExecutable(target);
}

await main();
