# Architecture

Converter is a [Tauri 2](https://v2.tauri.app) app. A Svelte 5 UI runs in the system webview, and
a Rust backend does all file access and conversion. FFmpeg ships inside the app as a Tauri
sidecar.

## How a conversion runs

1. On start, the UI calls `get_catalog`. The backend returns the platform, the input extensions
   and the output formats it offers. Format pickers and file filters are built from this, so a
   format only needs adding in `formats.rs`.
2. Dropped or picked paths go to `inspect_paths`, which expands folders and classifies each file
   as an image, a video or unsupported. Image files are added to the asset protocol scope so the
   UI can show thumbnails. Nothing else is readable by the webview.
3. **Convert** sends one `convert_files` request per kind (one for images, one for videos), each
   with its own settings. Both run at the same time.
4. Each request is a batch. Images run in parallel, up to four at once or one per core if there are fewer. Videos run one at a time.
   Progress and results come back to the UI through a Tauri channel as `started`, `progress`,
   `completed`, `failed` and `cancelled` events.
5. `cancel_conversion` cancels a batch's token. Waiting files are marked cancelled, and running
   FFmpeg processes are killed.
6. `open_output` and `reveal_output` only accept files the app wrote in the current session.

## Images

Every image goes through decode, resize and encode, so any input can become any output.

| Step | How |
|---|---|
| Decode | The `image` crate (JPG, PNG, WebP, GIF, BMP, TIFF), applying EXIF orientation. `resvg` for SVG, `hayro` for the first page of a PDF. FFmpeg for HEIC, HEIF and AVIF, and as a fallback for misnamed files. |
| Resize | Lanczos3. SVG and PDF are rendered straight at the target size instead. |
| Encode | The `image` crate for JPG, PNG, GIF, BMP and TIFF. FFmpeg for WebP (`libwebp`) and AVIF (`libaom-av1`). `pdf-writer` for PDF. `vtracer` for SVG. macOS `sips` for HEIC. |

Intermediate files go in a temporary folder under the per-user scratch folder, which is emptied
on every launch.

## Videos

Videos are converted by one FFmpeg run each. `video_conversion.rs` builds the arguments: the codec
and quality preset per container, a scale filter that only shrinks, the frame rate, and a
per-clip palette for GIF. `ffmpeg.rs` runs the process, reads progress from `-progress pipe:1`,
and picks the most relevant stderr line as the error message.

## Module map

```
src/
  App.svelte                 Wires the queue, settings, drag and drop, and shortcuts together
  lib/backend.ts             Typed wrappers for the Rust commands
  lib/queue.svelte.ts        The file list and each file's conversion state
  lib/conversion-run.svelte.ts  Starts, cancels and waits for a conversion
  lib/shortcuts.ts           Keyboard shortcuts and their key labels per OS
  lib/requests.ts            Builds one conversion request per kind from the list and settings
  lib/preferences.svelte.ts  Remembered settings, stored with the Tauri store plugin
  lib/formatting.ts          Sizes, plurals and the reveal label per OS
  lib/components/            Toolbar, file list, settings panel, format pickers
  lib/components/ui/         shadcn-svelte components
src-tauri/src/
  lib.rs                     App setup: plugins, window, logging, per-user scratch folder
  commands.rs                Commands the UI calls, and folder expansion
  formats.rs                 The single list of supported inputs and outputs
  options.rs                 Conversion settings sent by the UI, and parallelism limits
  batch.rs                   Runs a batch, reports events, removes failed outputs
  output_path.rs             Picks and reserves a free output name
  image_conversion/          decode.rs, geometry.rs (resize, flatten onto white), encode.rs
  video_conversion.rs        FFmpeg arguments for each video format
  ffmpeg.rs                  Runs the bundled FFmpeg with progress and cancellation
  state.rs                   Active batches and the outputs written this session
  error.rs                   Error messages shown to the user
  conversion_tests.rs        Real conversions through the bundled FFmpeg
scripts/fetch-ffmpeg.ts      Downloads the pinned, checksummed FFmpeg for a platform
```

## FFmpeg

`scripts/fetch-ffmpeg.ts` downloads FFmpeg 9.0.2 for one target into
`src-tauri/binaries/converter-ffmpeg-<target triple>`. Each download is pinned to an exact URL and SHA-256.

| Platform | Source |
|---|---|
| macOS and Linux | Static builds from [ffmpeg.martin-riedl.de](https://ffmpeg.martin-riedl.de) |
| Windows | The "essentials" build from [gyan.dev](https://www.gyan.dev/ffmpeg/builds/) |

Tauri copies the sidecar next to the app's executable, which is where `ffmpeg.rs` looks for it.
On Windows, FFmpeg is started without a console window.

## Security

- The UI can't read files itself. It gets paths from the native file dialog or drag and drop, and
  can load only the image previews granted by `inspect_paths`. Its permissions are in
  `src-tauri/capabilities/default.json`.
- The content security policy in `tauri.conf.json` blocks connections to other hosts.
