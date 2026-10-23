# Converter

A desktop app for macOS, Windows and Linux that converts images and videos on your own computer.
FFmpeg is bundled, and no files are uploaded.

## Features

- Images: JPG, PNG, WebP, AVIF, HEIC, GIF, BMP, TIFF, SVG and PDF in; JPG, PNG, WebP, AVIF, GIF,
  BMP, TIFF, PDF, traced SVG and (on macOS) HEIC out.
- Videos: MP4, MOV, MKV, WebM, AVI, FLV, M4V, WMV, MPEG, TS, M2TS, 3GP and OGV in; MP4, MOV, MKV,
  WebM, M4V, AVI, FLV and animated GIF out.
- Drop files or whole folders. Images and videos can be mixed in one list, each with their own
  settings.
- Image quality and resizing; video quality presets, maximum resolution and frame rate.
- Saves next to the originals or into a folder you choose, and never overwrites a file.
- Phone photos are turned upright using their EXIF orientation.
- Remembers your last-used settings.

## Install

Download the installer for your platform from the
[releases page](https://github.com/Squ1ggly/converter-desktop/releases). The builds are unsigned;
[Installing](docs/installing.md) explains how to open them on each platform.

## Development

You need Rust 1.92+, Node.js 24+ and pnpm. See [Building](docs/development/building.md) for the
full list.

```bash
pnpm install
pnpm fetch-ffmpeg   # downloads the pinned FFmpeg for this machine
pnpm dev            # runs the app with hot reload
```

## Documentation

- [User guide](docs/README.md): adding files, image and video settings, saving, shortcuts,
  settings and troubleshooting
- [Architecture](docs/development/architecture.md)
- [Building](docs/development/building.md)
- [Testing](docs/development/testing.md)
- [Releasing](docs/development/releasing.md)
- [Contributing](CONTRIBUTING.md)

## Licence

The app is MIT-licensed; see [LICENSE](LICENSE).

The bundled FFmpeg builds are licensed under the GPL. They are downloaded by
`scripts/fetch-ffmpeg.ts` from [ffmpeg.martin-riedl.de](https://ffmpeg.martin-riedl.de) (macOS and
Linux) and [gyan.dev](https://www.gyan.dev/ffmpeg/builds/) (Windows), which publish the source for
their builds. Anyone distributing the installers must include FFmpeg's licence and say where its
source can be obtained.
