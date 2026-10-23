# Settings and data

## What is remembered

Your last-used choices are saved as you change them and restored on the next launch:

| Setting | Default |
|---|---|
| Image format | JPG |
| Image quality | 85 |
| Image size | Original |
| Video format | MP4 |
| Video quality | Balanced |
| Resolution | Original |
| Frame rate | Original |
| Save to | Same folder as original, or the folder you chose |

The file list is not remembered. If the saved image format is HEIC and the app is running on
Windows or Linux, it falls back to JPG.

The app follows your system's light or dark appearance.

## Where files are kept

The app is installed for all users, but each user's data is kept in their own folders.
`com.squ1ggly.converter` is the app's identifier.

| What | macOS | Windows | Linux |
|---|---|---|---|
| Settings (`settings.json`) | `~/Library/Application Support/com.squ1ggly.converter/` | `%APPDATA%\com.squ1ggly.converter\` | `~/.local/share/com.squ1ggly.converter/` |
| Scratch files | `~/Library/Caches/com.squ1ggly.converter/scratch/` | `%LOCALAPPDATA%\com.squ1ggly.converter\scratch\` | `~/.cache/com.squ1ggly.converter/scratch/` |
| Logs | `~/Library/Logs/com.squ1ggly.converter/` | `%LOCALAPPDATA%\com.squ1ggly.converter\logs\` | `~/.local/share/com.squ1ggly.converter/logs/` |

On Linux, `$XDG_DATA_HOME` and `$XDG_CACHE_HOME` replace `~/.local/share` and `~/.cache` when
they are set.

- **Settings**: to go back to the defaults, quit the app and delete `settings.json`.
- **Scratch files** are temporary images used during image conversions. The folder is emptied
  every time the app starts.
- **Logs** record failed conversions and startup problems. Include them when reporting a bug.

Converted files are only written to the [save location](saving.md) you choose. Nothing is
uploaded.
