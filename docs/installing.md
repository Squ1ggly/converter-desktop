# Installing

Download the installer for your platform from the
[releases page](https://github.com/Squ1ggly/converter-desktop/releases). FFmpeg is included, so
there is nothing else to install.

| Platform | Download | Installs to |
|---|---|---|
| macOS 11 or later, Apple Silicon | `.dmg` ending in `aarch64` | `/Applications/Converter.app` |
| macOS 11 or later, Intel | `.dmg` ending in `x64` | `/Applications/Converter.app` |
| Windows x64 | `-setup.exe` (recommended) or `.msi` | `C:\Program Files\Converter` |
| Linux x64 or ARM64 | `.deb` (Debian, Ubuntu) or `.rpm` (Fedora, openSUSE) | `/usr/bin`, with an entry in the applications menu |

The app is installed once for every user of the machine. Each user's settings and logs are kept
separately; see [Settings and data](settings-and-data.md).

## macOS

1. Open the `.dmg` and drag Converter into Applications.
2. Open Converter from Applications.

The builds are not signed or notarised, so the first launch is blocked. To open it anyway:

1. Try to open Converter once and dismiss the warning.
2. Open System Settings, then Privacy & Security.
3. Scroll to the message about Converter and click **Open Anyway**, then confirm.

If macOS says the app "is damaged and can't be opened", remove the download quarantine flag in
Terminal and open it again:

```bash
xattr -dr com.apple.quarantine /Applications/Converter.app
```

## Windows

Run the `-setup.exe` (or the `.msi`). Both install for all users, so Windows asks for administrator
approval.

The installers are not code-signed, so SmartScreen may show "Windows protected your PC". Click
**More info**, then **Run anyway**.

## Linux

Install the package with your package manager, for example:

```bash
sudo apt install ./Converter_<version>_amd64.deb
sudo dnf install ./Converter-<version>-1.x86_64.rpm
```

Converter then appears in the applications menu.

## Uninstalling

| Platform | How |
|---|---|
| macOS | Delete `/Applications/Converter.app` |
| Windows | Settings, Apps, Installed apps, Converter, Uninstall |
| Linux | Remove the package with the package manager you installed it with |

Uninstalling leaves each user's settings and logs in place. Their locations are listed in
[Settings and data](settings-and-data.md).
