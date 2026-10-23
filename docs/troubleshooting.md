# Troubleshooting

A failed file is marked **Failed** with its error message under the name. Failures are also
written to the [log](settings-and-data.md#where-files-are-kept).

## Error messages

| Message | Cause | What to do |
|---|---|---|
| The bundled FFmpeg is missing (looked in &lt;folder&gt;). Reinstall the app. | FFmpeg was not found next to the app, so nothing can be converted, including images | Reinstall the app |
| This file could not be read as an image: &lt;details&gt; | The file is damaged, or isn't really the format its extension says, and FFmpeg couldn't read it either | Open the file in another app to check it |
| macOS could not write the HEIC file: &lt;details&gt; | The macOS HEIC encoder (`sips`) failed | Try another output format |
| Could not trace this image into an SVG: &lt;details&gt; | Tracing failed | Try PNG instead, or a simpler image |
| Too many files with this name already exist | Every name from `name.ext` to `name (10000).ext` is taken in the save folder | Choose another save folder, or tidy the folder |
| A line from FFmpeg, for example "Error opening input file" | FFmpeg couldn't read or convert the file. The message is the most relevant line from its output. | Check the file opens in another app. Try another output format. |
| An operating system error, such as permission denied or no such file | The save folder is read-only, no longer exists, or the original was moved | Choose another save folder under **Save to**, or add the file again |

## Other problems

| Problem | Cause |
|---|---|
| "Skipped N unsupported files" | The files don't have a supported extension. See [Adding files](adding-files.md#supported-files). |
| Files in a folder were not added | Hidden files and unsupported files inside folders are skipped without a message |
| "Wait for the current conversion to finish." | Files can't be added during a conversion. Wait, or click **Cancel**. |
| HEIC is missing from the image formats | HEIC output is only available on macOS |
| An animated GIF or WebP became a still image | Only the first frame of an image is converted |
| A video shows a spinner instead of a percentage | FFmpeg couldn't tell the video's length. The conversion still runs. |
| Converting again made `name (2).ext` | Existing files are never overwritten. See [Saving](saving.md#file-names). |
| A large image became smaller as an SVG | SVG tracing works on at most 2000 × 2000 pixels |
| The window shows "Couldn't start" | The app failed to load. The message and the log say why. |
| The app is blocked on first launch | The builds are unsigned. See [Installing](installing.md). |

Closing the window during a conversion asks first, then cancels and removes unfinished files. If
the app crashes mid-conversion, the file being written can be left incomplete. Delete it and
convert again.
