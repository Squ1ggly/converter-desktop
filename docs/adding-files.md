# Adding files

There are three ways to add files:

- Drag files or folders onto the window. "Drop to add" appears while you drag.
- Click **Add files** (⌘O on macOS, Ctrl+O on Windows and Linux). The picker can filter to
  images, videos or both.
- Click **Add folder** to add one or more folders.

Once the list has files in it, **Add files** moves to the toolbar, next to **Clear**, which empties
the list. Dragging and dropping keeps working at any time.

One list can hold images and videos together. They are shown in separate groups, each with its
own settings. See [Images](images.md) and [Videos](videos.md).

## Supported files

Files are recognised by their extension, in any letter case.

| Kind | Extensions |
|---|---|
| Images | `.avif`, `.bmp`, `.gif`, `.heic`, `.heif`, `.jpg`, `.jpeg`, `.pdf`, `.png`, `.svg`, `.tif`, `.tiff`, `.webp` |
| Videos | `.3gp`, `.avi`, `.flv`, `.m2ts`, `.m4v`, `.mkv`, `.mov`, `.mp4`, `.mpeg`, `.mpg`, `.mts`, `.ogv`, `.ts`, `.webm`, `.wmv` |

GIFs are treated as images, so an animated GIF can't be turned into a video.

## Folders

Folders are searched recursively, including every subfolder. Inside a folder:

- Images and videos are added.
- Other files are skipped without a message.
- Hidden files and folders (names starting with `.`) are skipped.

A large folder can take a moment to search. The list shows "Adding files" until it's done, and
converting waits until then.

## Skipped files

- **Unsupported files** you drop or pick directly are skipped, and a "Skipped N unsupported files"
  message names up to three of them.
- **Duplicates**: a file that is already in the list is not added again.
- **During a conversion** nothing can be added. Dragging files over the window shows "Wait for
  the conversion to finish" instead of "Drop to add".

## Removing files

Click the × button (**Remove**) next to a file to take it out of the list. It's there before and
after a conversion, for converted, failed and cancelled files too, but not while files convert.

Adding files after a conversion has finished starts a new list: files that converted
successfully are removed, and files that failed or were cancelled stay so you can try them again.
