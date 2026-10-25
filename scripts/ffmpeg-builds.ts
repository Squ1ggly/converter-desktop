// The FFmpeg builds bundled with the app, one per target, and the source they were built from.
// scripts/fetch-ffmpeg.ts downloads them and scripts/generate-notices.ts lists them in
// THIRD-PARTY-NOTICES.md.

export interface FfmpegSource {
  url: string;
  sha256: string;
}

export interface FfmpegBuilder {
  name: string;
  // Where the builder publishes its build scripts and configuration.
  buildScriptsUrl: string;
}

export interface FfmpegBuild {
  platformName: string;
  builder: FfmpegBuilder;
  // A .zip or .7z archive.
  url: string;
  sha256: string;
  // Path of the ffmpeg executable inside the archive.
  entryPath: string;
}

export const FFMPEG_VERSION = "9.0.2";

// Every build below is made from this release.
export const FFMPEG_SOURCE: FfmpegSource = {
  url: `https://ffmpeg.org/releases/ffmpeg-${FFMPEG_VERSION}.tar.xz`,
  sha256: "8c3850283eb25fa026482078a04051e0be17347b09ef81a0849bec15a96e002e",
};

const MARTIN_RIEDL: FfmpegBuilder = {
  name: "Martin Riedl",
  buildScriptsUrl: "https://git.martin-riedl.de/ffmpeg/build-script",
};

const GYAN: FfmpegBuilder = {
  name: "Gyan Doshi",
  buildScriptsUrl: "https://www.gyan.dev/ffmpeg/builds/",
};

// BtbN's Windows ARM64 builds leave out libvpx and libaom, which WebM and AVIF need.
const TORDONA: FfmpegBuilder = {
  name: "tordona",
  buildScriptsUrl: `https://github.com/tordona/ffmpeg-win-arm64/tree/${FFMPEG_VERSION}`,
};

const TORDONA_ARCHIVE_NAME = `ffmpeg-${FFMPEG_VERSION}-essentials-static-win-arm64`;

export const FFMPEG_BUILDS: Record<string, FfmpegBuild> = {
  "aarch64-apple-darwin": {
    platformName: "macOS, Apple Silicon",
    builder: MARTIN_RIEDL,
    url: "https://ffmpeg.martin-riedl.de/download/macos/arm64/1789931890_9.0.2/ffmpeg.zip",
    sha256: "c8ed4c4e6978a03c485edbfe4e0a5dc2380f8a30bba5150531b31b094492d924",
    entryPath: "ffmpeg",
  },
  "x86_64-apple-darwin": {
    platformName: "macOS, Intel",
    builder: MARTIN_RIEDL,
    url: "https://ffmpeg.martin-riedl.de/download/macos/amd64/1789931006_9.0.2/ffmpeg.zip",
    sha256: "7c6b4125b191cbf773832dc51f424cf2b6bb7da43007d1e066f95909e47cacd4",
    entryPath: "ffmpeg",
  },
  "x86_64-pc-windows-msvc": {
    platformName: "Windows x64",
    builder: GYAN,
    url: `https://github.com/GyanD/codexffmpeg/releases/download/${FFMPEG_VERSION}/ffmpeg-${FFMPEG_VERSION}-essentials_build.zip`,
    sha256: "60f467265b1e312373dbcd92200c2618a74850f98d3d078e94296bb3fa2047ba",
    entryPath: `ffmpeg-${FFMPEG_VERSION}-essentials_build/bin/ffmpeg.exe`,
  },
  "aarch64-pc-windows-msvc": {
    platformName: "Windows ARM64",
    builder: TORDONA,
    url: `https://github.com/tordona/ffmpeg-win-arm64/releases/download/${FFMPEG_VERSION}/${TORDONA_ARCHIVE_NAME}.7z`,
    sha256: "5319165d85ecee87360166dc0198b6de7747be6ce7a97a9f59b3920eddb6a3c3",
    entryPath: `${TORDONA_ARCHIVE_NAME}/bin/ffmpeg.exe`,
  },
  "x86_64-unknown-linux-gnu": {
    platformName: "Linux x64",
    builder: MARTIN_RIEDL,
    url: "https://ffmpeg.martin-riedl.de/download/linux/amd64/1789931100_9.0.2/ffmpeg.zip",
    sha256: "fa8ecf4abbd290d98f7d188b8649cc6b391ae209a98452be955a15aab1909d7f",
    entryPath: "ffmpeg",
  },
  "aarch64-unknown-linux-gnu": {
    platformName: "Linux ARM64",
    builder: MARTIN_RIEDL,
    url: "https://ffmpeg.martin-riedl.de/download/linux/arm64/1789931697_9.0.2/ffmpeg.zip",
    sha256: "93a76ae90db5474eecdf951a729857c64f3de23567228d6a7d5e6e8e3cd1021b",
    entryPath: "ffmpeg",
  },
};

// The file name the source is published under on each GitHub release.
export const FFMPEG_SOURCE_ARCHIVE_NAME = `ffmpeg-${FFMPEG_VERSION}.tar.xz`;
