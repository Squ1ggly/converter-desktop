# Converting

1. [Add files](adding-files.md).
2. Pick the settings for [images](images.md), [videos](videos.md) and [where to save](saving.md).
3. Click **Convert N files** at the bottom of the settings panel, or press ⌘↩ on macOS or
   Ctrl+Enter on Windows and Linux.

## While converting

- The panel shows "Converting N of M" and an overall progress bar.
- Each file shows its own state: **Queued**, then a spinner or a percentage while it converts.
  Videos show a percentage when FFmpeg can tell the video's length. Images show a spinner.
- Images and videos convert at the same time. Up to four images convert in parallel, depending on
  the number of processor cores. Videos convert one at a time, because each video already uses
  every core.
- Settings, **Clear**, **Add files** and **Remove** are locked until the conversion ends.

## Cancelling

Click **Cancel**. The button shows "Cancelling" until every file has stopped.

- Files that haven't started are marked **Cancelled**.
- Video conversions in progress are stopped, and their partly written files are deleted.
- An image that is already being converted may still finish.

## Results

When every file has finished, the heading shows how many converted, plus how many failed or were
cancelled. If none converted, the heading shows how many failed, for example "3 failed".

| Action | Where | What it does |
|---|---|---|
| **Open** | Next to each converted file | Opens the file in its default app |
| Show in Finder (macOS), Show in Explorer (Windows), Show in folder (Linux) | Next to each converted file, and at the top of the list | Shows the file in the file manager. The button at the top shows the first converted file. |

A failed file is highlighted and marked **Failed**, with the error message under its name. See
[Troubleshooting](troubleshooting.md).

## Converting again

After a conversion, the button changes to **Convert again**. It converts every file in the list
again with the current settings, including files that already converted. Existing files are never
overwritten, so the new results get numbered names such as `photo (2).jpg`.

To start with new files instead, add them. Files that converted successfully leave the list, and
files that failed or were cancelled stay so you can retry them.
