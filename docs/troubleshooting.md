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
| This image is too large to convert | The image is over 100 megapixels | Make it smaller in another app first |
| This image is too large to convert at that size. Choose a smaller size. | **Scale** or **Fit** would make the image over 100 megapixels | Choose a smaller size |
| Converting produced an empty file | The encoder finished without writing anything. The empty file is removed. | Try another output format, and report it with the log |
| Too many files with this name already exist | Every name from `name.ext` to `name (10000).ext` is taken in the save folder | Choose another save folder, or tidy the folder |
| A line from FFmpeg, for example "Error opening input file clip.mp4." | FFmpeg couldn't read or convert the file. The message is the most relevant line from its output, with file names instead of full paths. The full output is in the log. | Check the file opens in another app. Try another output format. |
| &lt;file&gt; is no longer there. It may have been moved or deleted. | The original was moved, renamed or deleted after it was added | Add the file again |
| Can't read &lt;file&gt;. Check the file's permissions. | Your account isn't allowed to open the original | Change the file's permissions, or copy it somewhere you can read it |
| Can't write to &lt;folder&gt;. Check the folder's permissions. | The save folder is read-only for your account, or on a read-only disk | Choose another folder under **Save to** |
| The folder &lt;folder&gt; no longer exists | The save folder was moved or deleted | Choose another folder under **Save to** |
| There isn't enough space to save in &lt;folder&gt; | The disk is full | Free up space, or choose a folder on another disk |
| Only files with a full path can be converted | The app was given a path it doesn't accept. This shouldn't happen. | Add the file again, and report it with the log |
| The conversion stopped unexpectedly. The log has the details. | A bug in the app | Report it with the log |

## Other problems

| Problem | Cause |
|---|---|
| "Skipped N unsupported files" | The files don't have a supported extension. See [Adding files](adding-files.md#supported-files). |
| Files in a folder were not added | Hidden files, unsupported files and symlinked folders inside folders are skipped without a message |
| "Only the first 10,000 files were added" | One add stops at 10,000 files. Add the rest from smaller folders. |
| "Wait for the current conversion to finish." | Files can't be added during a conversion. Wait, or click **Cancel**. |
| HEIC is missing from the image formats | HEIC output is only available on macOS |
| An animated GIF or WebP became a still image | Only the first frame of an image is converted |
| A video shows a spinner instead of a percentage | FFmpeg couldn't tell the video's length. The conversion still runs. |
| Converting again made `name (2).ext` | Existing files are never overwritten. See [Saving](saving.md#file-names). |
| A large image became smaller as an SVG | SVG tracing works on at most 2000 × 2000 pixels |
| The window shows "Couldn't start" | The app failed to load. The message on screen says why; it isn't written to the log. Quit and open the app again, and report the message if it keeps happening. |
| "Couldn't load your settings" | `settings.json` can't be read. The app uses the defaults and doesn't save changes. Quit and delete the file to start again. See [Settings and data](settings-and-data.md#where-files-are-kept). |
| Opening the app again shows the same window | Only one copy runs at a time. Opening it again brings the open window to the front. |
| The app is blocked on first launch | The builds are unsigned. See [Installing](installing.md). |
| The window is blank on Linux | Some systems with Wayland and an NVIDIA graphics card can't draw it. See [If the window is blank](installing.md#if-the-window-is-blank). |

Closing the window or choosing **Quit Converter** (⌘Q) on macOS during a conversion asks first,
then cancels and removes unfinished files. If the app is made to quit another way, for example
when you log out, it stops every conversion and removes unfinished files before it closes. If
the app crashes mid-conversion, the file being written can be left incomplete. Delete it and
convert again. FFmpeg never keeps running after the app has gone: it is stopped on quit, and on
Windows also when the app crashes.
