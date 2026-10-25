// The FFmpeg builds bundled with the app, one per target, and the source each was built from.
// scripts/fetch-ffmpeg.ts downloads them and scripts/generate-notices.ts lists them in
// THIRD-PARTY-NOTICES.md.

// A released FFmpeg version, published as a tarball on ffmpeg.org.
export interface FfmpegReleaseSource {
  kind: "release";
  version: string;
  url: string;
  sha256: string;
}

// An FFmpeg git commit, for builds made from a release branch rather than a release.
export interface FfmpegCommitSource {
  kind: "commit";
  version: string;
  repository: string;
  commit: string;
}

export type FfmpegSource = FfmpegReleaseSource | FfmpegCommitSource;

export interface FfmpegBuilder {
  name: string;
  // Where the builder publishes its build scripts and configuration.
  buildScriptsUrl: string;
}

export interface FfmpegBuild {
  platformName: string;
  builder: FfmpegBuilder;
  url: string;
  sha256: string;
  // Path of the ffmpeg executable inside the zip.
  entryPath: string;
  source: FfmpegSource;
}

export const FFMPEG_VERSION = "9.0.2";

const RELEASE_SOURCE: FfmpegReleaseSource = {
  kind: "release",
  version: FFMPEG_VERSION,
  url: `https://ffmpeg.org/releases/ffmpeg-${FFMPEG_VERSION}.tar.xz`,
  sha256: "8c3850283eb25fa026482078a04051e0be17347b09ef81a0849bec15a96e002e",
};

// BtbN builds the 9.0 release branch, so its Windows ARM64 build is a few fixes past 9.0.2.
const BTBN_SOURCE: FfmpegCommitSource = {
  kind: "commit",
  version: "n9.0.2-17-g2a571b6068",
  repository: "https://github.com/FFmpeg/FFmpeg",
  commit: "2a571b606854520cf89804d8030c8b328e621689",
};

const MARTIN_RIEDL: FfmpegBuilder = {
  name: "Martin Riedl",
  buildScriptsUrl: "https://git.martin-riedl.de/ffmpeg/build-script",
};

const GYAN: FfmpegBuilder = {
  name: "Gyan Doshi",
  buildScriptsUrl: "https://www.gyan.dev/ffmpeg/builds/",
};

// The last build of each month is kept for two years, so pin one of those, not a daily build.
const BTBN_RELEASE_TAG = "autobuild-2026-09-30-13-08";
const BTBN_ARCHIVE_NAME = `ffmpeg-${BTBN_SOURCE.version}-winarm64-gpl-9.0`;

const BTBN: FfmpegBuilder = {
  name: "BtbN",
  buildScriptsUrl: `https://github.com/BtbN/FFmpeg-Builds/tree/${BTBN_RELEASE_TAG}`,
};

export const FFMPEG_BUILDS: Record<string, FfmpegBuild> = {
  "aarch64-apple-darwin": {
    platformName: "macOS, Apple Silicon",
    builder: MARTIN_RIEDL,
    url: "https://ffmpeg.martin-riedl.de/download/macos/arm64/1789931890_9.0.2/ffmpeg.zip",
    sha256: "c8ed4c4e6978a03c485edbfe4e0a5dc2380f8a30bba5150531b31b094492d924",
    entryPath: "ffmpeg",
    source: RELEASE_SOURCE,
  },
  "x86_64-apple-darwin": {
    platformName: "macOS, Intel",
    builder: MARTIN_RIEDL,
    url: "https://ffmpeg.martin-riedl.de/download/macos/amd64/1789931006_9.0.2/ffmpeg.zip",
    sha256: "7c6b4125b191cbf773832dc51f424cf2b6bb7da43007d1e066f95909e47cacd4",
    entryPath: "ffmpeg",
    source: RELEASE_SOURCE,
  },
  "x86_64-pc-windows-msvc": {
    platformName: "Windows x64",
    builder: GYAN,
    url: `https://github.com/GyanD/codexffmpeg/releases/download/${FFMPEG_VERSION}/ffmpeg-${FFMPEG_VERSION}-essentials_build.zip`,
    sha256: "60f467265b1e312373dbcd92200c2618a74850f98d3d078e94296bb3fa2047ba",
    entryPath: `ffmpeg-${FFMPEG_VERSION}-essentials_build/bin/ffmpeg.exe`,
    source: RELEASE_SOURCE,
  },
  "aarch64-pc-windows-msvc": {
    platformName: "Windows ARM64",
    builder: BTBN,
    url: `https://github.com/BtbN/FFmpeg-Builds/releases/download/${BTBN_RELEASE_TAG}/${BTBN_ARCHIVE_NAME}.zip`,
    sha256: "6dcd0626f9f6d6c7323565e57410e9f37ed14a85946ec9ea9caff294ca32f5c0",
    entryPath: `${BTBN_ARCHIVE_NAME}/bin/ffmpeg.exe`,
    source: BTBN_SOURCE,
  },
  "x86_64-unknown-linux-gnu": {
    platformName: "Linux x64",
    builder: MARTIN_RIEDL,
    url: "https://ffmpeg.martin-riedl.de/download/linux/amd64/1789931100_9.0.2/ffmpeg.zip",
    sha256: "fa8ecf4abbd290d98f7d188b8649cc6b391ae209a98452be955a15aab1909d7f",
    entryPath: "ffmpeg",
    source: RELEASE_SOURCE,
  },
  "aarch64-unknown-linux-gnu": {
    platformName: "Linux ARM64",
    builder: MARTIN_RIEDL,
    url: "https://ffmpeg.martin-riedl.de/download/linux/arm64/1789931697_9.0.2/ffmpeg.zip",
    sha256: "93a76ae90db5474eecdf951a729857c64f3de23567228d6a7d5e6e8e3cd1021b",
    entryPath: "ffmpeg",
    source: RELEASE_SOURCE,
  },
};

// The file name each source is published under on the GitHub release.
export function sourceArchiveName(source: FfmpegSource): string {
  const extension = source.kind === "release" ? "tar.xz" : "tar.gz";
  const archiveName = `ffmpeg-${source.version}.${extension}`;
  return archiveName;
}

export function distinctSources(): FfmpegSource[] {
  const sourcesByVersion = new Map<string, FfmpegSource>();
  for (const build of Object.values(FFMPEG_BUILDS)) {
    sourcesByVersion.set(build.source.version, build.source);
  }

  const sources = [...sourcesByVersion.values()];
  return sources;
}
