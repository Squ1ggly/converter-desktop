# Architecture

Converter is a [Tauri 2](https://v2.tauri.app) app. A Svelte 5 UI runs in the system webview, and
a Rust backend does all file access and conversion. FFmpeg ships inside the app as a Tauri
sidecar.

## How a conversion runs

1. On start, the UI calls `get_catalog`. The backend returns the platform, the input extensions
   and the output formats it offers. Format pickers and file filters are built from this, so a
   format only needs adding in `formats.rs`.
2. Dropped or picked paths go to `inspect_input_paths`, which runs off the main thread. It
   ignores paths that aren't absolute, resolves the rest (`input_files.rs`), expands folders and
   classifies each file as an image, a video or unsupported. Symlinked folders inside a folder
   are skipped, and it stops after 10,000 files, returning `isFileLimitReached: true`. Image
   files are added to the asset protocol scope so the UI can show thumbnails. Nothing else is
   readable by the webview. `inspect_paths` returns the same files without the flag, for the
   UI until it moves to `inspect_input_paths`.
3. **Convert** sends one `convert_files` request per kind (one for images, one for videos), each
   with its own settings. Both run at the same time.
4. Each request is a batch. Images run in parallel, up to four at once or one per core if there are fewer. Videos run one at a time.
   Progress and results come back to the UI through a Tauri channel as `started`, `progress`,
   `completed`, `failed` and `cancelled` events.
5. `cancel_conversion` cancels a batch's token, even if it arrives before the batch has started.
   Waiting files are marked cancelled, and running FFmpeg processes are killed and waited for, so
   their files can be deleted on Windows too.
6. On quit (`RunEvent::Exit`), `quit.rs` cancels every batch and waits up to three seconds for
   them to clean up. On macOS the app menu's **Quit** item is replaced, because the default one
   ends the app without a close request. While converting, it sends `quit-requested` to the main
   window instead, and the UI shows the same dialog as for closing the window. On Windows, FFmpeg
   runs in a job object that the system kills when the app ends, even after a crash.
7. `open_output` and `reveal_output` only accept files the app wrote in the current session.

## Images

Every image goes through decode, resize and encode, so any input can become any output.

| Step | How |
|---|---|
| Decode | The `image` crate (JPG, PNG, WebP, GIF, BMP, TIFF), applying EXIF orientation and converting colours to sRGB with `moxcms` (`color.rs`). `resvg` for SVG, `hayro` for the first page of a PDF. FFmpeg for HEIC, HEIF and AVIF, and as a fallback for misnamed files. |
| Resize | Lanczos3. SVG and PDF are rendered straight at the target size instead. |
| Encode | The `image` crate for JPG, PNG, GIF, BMP and TIFF. FFmpeg for WebP (`libwebp`) and AVIF (`libaom-av1`). `pdf-writer` for PDF. `vtracer` for SVG. macOS `sips` for HEIC. |

Intermediate files go in a temporary folder inside this launch's own folder under the per-user
scratch folder. `scratch.rs` deletes earlier launches' folders at startup. Anything it can't
delete, such as a file another process still has open, is logged and left for a later launch, so
it never stops the app from starting.

## Videos

Videos are converted by one FFmpeg run each. `video_conversion.rs` builds the arguments: the codec
and quality preset per container, a scale filter that only shrinks, the frame rate, and a
per-clip palette for GIF. GIF takes two FFmpeg runs: the first writes the palette to the scratch
folder, the second applies it. Doing both in one run makes FFmpeg hold every frame in memory. `ffmpeg.rs` runs the process, reads progress from `-progress pipe:1`,
and picks the most relevant stderr line as the error message.

## Module map

```
src/
  App.svelte                 Wires the queue, settings, drag and drop, and shortcuts together
  lib/backend.ts             Typed wrappers for the Rust commands
  lib/queue.svelte.ts        The file list and each file's conversion state
  lib/conversion-run.svelte.ts  Starts, cancels and waits for a conversion
  lib/file-adder.svelte.ts   Adds dropped or picked paths, and reports skipped files
  lib/file-pickers.ts        The native file and folder pickers
  lib/quit-guard.svelte.ts   Asks before closing mid-conversion, then cancels and quits
  lib/shortcuts.ts           Keyboard shortcuts and their key labels per OS
  lib/requests.ts            Builds one conversion request per kind from the list and settings
  lib/preferences.ts         Settings, their defaults and the check on saved values
  lib/preference-store.svelte.ts  Remembered settings, stored with the Tauri store plugin
  lib/formatting.ts          Sizes, plurals and the reveal label per OS
  lib/components/            Toolbar, file list, settings panel, format pickers
  lib/components/ui/         shadcn-svelte components
src-tauri/src/
  lib.rs                     App setup: plugins, window, logging, per-user scratch folder
  commands.rs                Commands the UI calls
  input_files.rs             Resolves added paths, expands folders and classifies files
  formats.rs                 The single list of supported inputs and outputs
  options.rs                 Conversion settings sent by the UI, and parallelism limits
  batch.rs                   Runs a batch, reports events, removes failed outputs
  output_path.rs             Picks and reserves a free output name
  scratch.rs                 Clears old scratch files and makes this launch's scratch folder
  image_conversion/          decode.rs, geometry.rs (resize, flatten onto white), encode.rs
  video_conversion.rs        FFmpeg arguments for each video format
  ffmpeg.rs                  Runs the bundled FFmpeg with progress and cancellation
  child_lifetime.rs          Ends child processes with the app (a job object on Windows)
  quit.rs                    Stops conversions on quit, and the macOS Quit menu item
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
  can load only the image previews granted by `inspect_input_paths`: files whose resolved path
  has an image extension. Its permissions are in `src-tauri/capabilities/default.json`.
- Commands accept only absolute paths. Inputs and the output folder are resolved (symlinks and
  `..` followed) before anything is read or written, and output names are always picked inside
  the resolved folder.
- The content security policy in `tauri.conf.json` blocks connections to other hosts.
