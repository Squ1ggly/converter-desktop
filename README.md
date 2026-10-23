# Converter

A desktop app for converting images and videos on your own computer. It's built
with Tauri: a Rust backend and a Svelte UI. Everything it needs, including FFmpeg,
is bundled into the installer, and no files are uploaded anywhere.

It started as the desktop version of a self-hosted web converter.

## What it converts

| | Inputs | Outputs |
|---|---|---|
| **Images** | JPG, PNG, WebP, AVIF, HEIC/HEIF, GIF, BMP, TIFF, SVG, PDF (first page) | JPG, PNG, WebP, AVIF, HEIC (macOS only), GIF, BMP, TIFF, PDF, SVG (traced) |
| **Videos** | MP4, MOV, MKV, WebM, AVI, FLV, M4V, WMV, MPEG, TS, 3GP, OGV | MP4, MOV, MKV, WebM, M4V, AVI, FLV, animated GIF |

- Add files by dragging them in, or with **Choose files…** (⌘O / Ctrl+O). Dropping a folder adds every image and video inside it.
- A list can mix images and videos. Each type gets its own settings, and both convert at the same time.
- Image options are quality and size: original, a percentage, or fit within a width × height.
- Video options are quality (High / Balanced / Smaller file), a maximum resolution and the frame rate.
- Converted files go next to their originals or into a folder you choose. Existing files are never overwritten: a clash becomes `photo (2).jpg`.
- Phone photos are rotated using their EXIF orientation, and transparency becomes white for formats that can't store it.
- Your last-used settings are remembered.

## Development

You need [Rust](https://rustup.rs) 1.92+ and [Bun](https://bun.sh). On Linux you also need the
[Tauri system packages](https://tauri.app/start/prerequisites/#linux).

```bash
bun install
bun run fetch-ffmpeg     # downloads the pinned FFmpeg for this machine into src-tauri/binaries/
bun run dev              # opens the app with hot reload
```

| Command | What it does |
|---|---|
| `bun run dev` | Runs the app in development mode |
| `bun run build` | Builds installers for this machine into `src-tauri/target/release/bundle/` |
| `bun run check` | Type-checks the Svelte/TypeScript UI |
| `bun run fetch-ffmpeg [--target <triple>]` | Downloads and checksum-verifies FFmpeg for a platform |
| `cargo test --manifest-path src-tauri/Cargo.toml` | Unit tests, plus real conversions of every format through the bundled FFmpeg |

### Building for every platform

Each platform has to be built on that platform. [.github/workflows/build.yml](.github/workflows/build.yml)
builds all of them on GitHub Actions: macOS (Apple Silicon and Intel), Windows x64, and Linux
(x64 and ARM). Run it manually to get the installers as artifacts, or push a `v*` tag to attach
them to a draft release.

Builds are unsigned. Before sharing them widely, set up
[macOS signing and notarisation](https://v2.tauri.app/distribute/sign/macos/) and
[Windows code signing](https://v2.tauri.app/distribute/sign/windows/), or users will see
Gatekeeper and SmartScreen warnings.

## Installing

The installers install the app for all users of the machine:

| Platform | Installer | Installs to |
|---|---|---|
| macOS | `.dmg` (drag to Applications) | `/Applications/Converter.app` |
| Windows | `.exe` (NSIS, per-machine) or `.msi` | `C:\Program Files\Converter` |
| Linux | `.deb` or `.rpm` | `/usr/bin`, with a desktop entry |

Each user's own data is kept in their own folders:

| What | macOS | Windows | Linux |
|---|---|---|---|
| Settings (`settings.json`) | `~/Library/Application Support/com.squ1ggly.converter/` | `%APPDATA%\com.squ1ggly.converter\` | `~/.local/share/com.squ1ggly.converter/` |
| Scratch files (cleared every launch) | `~/Library/Caches/com.squ1ggly.converter/` | `%LOCALAPPDATA%\com.squ1ggly.converter\` | `~/.cache/com.squ1ggly.converter/` |
| Logs | `~/Library/Logs/com.squ1ggly.converter/` | `%LOCALAPPDATA%\com.squ1ggly.converter\logs\` | `~/.local/share/com.squ1ggly.converter/logs/` |

## How it works

```
src/                      Svelte 5 UI
  App.svelte              Wires the queue, settings, drag-and-drop and shortcuts together
  components/             Toolbar, file list, settings panel, format pickers…
  lib/backend.ts          Typed wrappers for the Rust commands
  lib/queue.svelte.ts     The file list and each file's conversion state
  lib/preferences.svelte.ts  Remembered settings (stored with the Tauri store plugin)
src-tauri/src/
  formats.rs              The single list of supported formats; the UI is built from it
  commands.rs             Commands the UI calls
  batch.rs                Runs a batch: parallel images, one video at a time, cancellation
  image_conversion/       decode → resize → encode, for every image format
  video_conversion.rs     FFmpeg arguments for each video format
  ffmpeg.rs               Runs the bundled FFmpeg with progress and cancellation
  output_path.rs          Picks a free output name without overwriting
scripts/fetch-ffmpeg.ts   Downloads the pinned, checksummed FFmpeg for a platform
```

- **Images** are decoded into memory, resized, then written out. Most formats use the
  [`image`](https://crates.io/crates/image) crate. SVG is rendered with `resvg`, PDF with `hayro`,
  and HEIC/AVIF are decoded by FFmpeg. WebP and AVIF are encoded by FFmpeg, PDF output uses
  `pdf-writer`, and SVG output is traced with `vtracer`. HEIC output uses macOS's built-in `sips`,
  since no encoder can be bundled on other platforms.
- **Videos** are encoded by FFmpeg: H.264/AAC for MP4/MOV/MKV/M4V/FLV, VP9/Opus for WebM, MPEG-4
  for AVI, and a per-clip palette for GIF.
- **FFmpeg** is a self-contained 9.0.2 build. The macOS and Linux builds come from
  [martin-riedl.de](https://ffmpeg.martin-riedl.de) and the Windows build from
  [gyan.dev](https://www.gyan.dev/ffmpeg/builds/). Each download is pinned to an exact URL and
  SHA-256 in `scripts/fetch-ffmpeg.ts`, and ships inside the app as a Tauri sidecar.

## Licence

The app is MIT-licensed (see [LICENSE](LICENSE)). The bundled FFmpeg builds are licensed under the
GPL (they include x264 and x265). If you distribute the installers, ship FFmpeg's licence and say
where its source can be obtained. Both download sites publish the build sources.
