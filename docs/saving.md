# Saving

## Save location

Under **Save to** in the settings panel:

| Option | Where converted files go |
|---|---|
| Same folder as original | Next to each original file. This is the default. |
| Another folder | Into one folder you choose. Selecting this opens a folder picker; **Change** picks a different folder. |

If you cancel the folder picker, the previous choice stays. The chosen folder is remembered
between launches.

## File names

A converted file keeps the original name with the new extension: `holiday.png` becomes
`holiday.jpg`.

**Existing files are never overwritten.** If the name is taken, a number is added:

| Already in the folder | New file |
|---|---|
| `holiday.jpg` | `holiday (2).jpg` |
| `holiday.jpg`, `holiday (2).jpg` | `holiday (3).jpg` |

This also applies when:

- The output format is the same as the input, for example `clip.mp4` to MP4 in the same folder
  becomes `clip (2).mp4`. The original is never touched.
- Two files in the list would get the same name, for example `photo.png` and `photo.webp` from
  one folder both converted to JPG. One becomes `photo.jpg` and the other `photo (2).jpg`.
- You click **Convert again**. Every file is converted again into new numbered files.

## Failed and cancelled files

If a conversion fails or is cancelled, its partly written output is deleted, so you are not left
with a broken file. The original is never changed.
