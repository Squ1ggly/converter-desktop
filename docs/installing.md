# Installing

Download the installer for your computer from the
[releases page](https://github.com/Squ1ggly/converter-desktop/releases). FFmpeg is included, so
there is nothing else to install.

## Supported systems

| System | Versions | Processors |
|---|---|---|
| macOS | 12 Monterey or later | Apple Silicon and Intel |
| Windows | 10 (22H2) and 11 | x64 and ARM64 |
| Ubuntu | 22.04 or later | x64 and ARM64 |
| Debian | 12 or later | x64 and ARM64 |
| Fedora | 40 or later | x64 and ARM64 |
| openSUSE Leap | 15.6 or later | x64 and ARM64 |

On macOS 12, install the latest Safari update from Software Update first. The app's window uses
the same engine as Safari and needs Safari 16.4 or later.

Debian 11, RHEL 9 and other Linux systems with a glibc older than 2.35 are not supported. The
bundled FFmpeg needs glibc 2.35.

## Which file to download

`<version>` is the release number, for example `0.1.0`.

| Computer | Download |
|---|---|
| Mac with Apple Silicon (M1 or later) | `Converter_<version>_aarch64.dmg` |
| Mac with an Intel processor | `Converter_<version>_x64.dmg` |
| Windows PC with an Intel or AMD processor | `Converter_<version>_x64-setup.exe`, or `Converter_<version>_x64_en-US.msi` for managed installs |
| Windows PC with an ARM processor (Snapdragon) | `Converter_<version>_arm64-setup.exe`, or `Converter_<version>_arm64_en-US.msi` |
| Ubuntu or Debian, x64 | `Converter_<version>_amd64.deb` |
| Ubuntu or Debian, ARM64 | `Converter_<version>_arm64.deb` |
| Fedora or openSUSE, x64 | `Converter-<version>-1.x86_64.rpm` |
| Fedora or openSUSE, ARM64 | `Converter-<version>-1.aarch64.rpm` |

To check a Mac, open the Apple menu, then **About This Mac**: **Chip** means Apple Silicon,
**Processor** means Intel. To check a Windows PC, open Settings, System, About and read
**System type**.

The app is installed once for every user of the machine:

| Platform | Installs to |
|---|---|
| macOS | `/Applications/Converter.app` |
| Windows | `C:\Program Files\Converter` |
| Linux | `/usr/bin/converter`, with an entry in the applications menu |

Each user's settings and logs are kept separately; see [Settings and data](settings-and-data.md).

## macOS

1. Open the `.dmg` and click **Agree** to accept the licence.
2. Drag Converter into Applications.
3. Open Converter from Applications.

Today's builds are not notarised by Apple, so macOS blocks the first launch. What to do depends on
the macOS version.

### macOS 15 Sequoia and later

1. Open Converter. macOS says it can't verify the app. Click **Done**.
2. Open System Settings, then **Privacy & Security**.
3. Scroll down to **Security**. Next to the message about Converter, click **Open Anyway**. The
   button is only there for about an hour after you tried to open the app; if it's gone, open the
   app again first.
4. Enter your password, then click **Open Anyway** once more.

### macOS 12 to 14

1. In Finder, open Applications.
2. Control-click Converter and choose **Open**.
3. Click **Open** in the message that appears.

### If macOS says the app is damaged

Remove the download quarantine flag in Terminal, then open the app again:

```bash
xattr -dr com.apple.quarantine /Applications/Converter.app
```

You only need to do this once. Later launches open normally.

## Windows

Run the `-setup.exe` (or the `.msi`). Both install for all users, so Windows asks for
administrator approval.

Today's installers are not code-signed, so Microsoft Defender SmartScreen may show "Windows
protected your PC". Click **More info**, then **Run anyway**.

If Smart App Control is on (Windows 11), Windows blocks unsigned apps completely and offers no way
to run them. Wait for a signed release, or turn Smart App Control off in Windows Security, under
**App & browser control**. Smart App Control is a security feature, so only turn it off if you
are comfortable doing so.

Converter's window uses Microsoft Edge WebView2, which Windows 11 and up-to-date Windows 10
already have. If it's missing, the installer downloads and installs it, so the first install
needs an internet connection.

## Linux

Install the package with your package manager:

```bash
# Ubuntu and Debian
sudo apt install ./Converter_<version>_amd64.deb

# Fedora
sudo dnf install ./Converter-<version>-1.x86_64.rpm

# openSUSE
sudo zypper install --allow-unsigned-rpm ./Converter-<version>-1.x86_64.rpm
```

The package manager installs the libraries Converter needs, such as WebKitGTK. Converter then
appears in the applications menu.

### If the window is blank

On some systems with Wayland and an NVIDIA graphics card, the window opens but stays blank. Start
Converter with WebKit's DMA-BUF renderer turned off:

```bash
WEBKIT_DISABLE_DMABUF_RENDERER=1 converter
```

If that fixes it, add `WEBKIT_DISABLE_DMABUF_RENDERER=1` to your environment, for example in
`~/.profile`, so the applications menu entry works too.

## Signed builds

When releases are signed and notarised, macOS opens Converter without the extra steps above, and
SmartScreen and Smart App Control no longer block the Windows installers.

## Licences

Converter is MIT licensed. It includes FFmpeg, which is licensed under the GNU GPL version 3 or
later. The installers include `LICENSE`, `THIRD-PARTY-NOTICES.md` and the FFmpeg licence in a
`licenses` folder: inside `Converter.app/Contents/Resources` on macOS, in the install folder on
Windows, and in `/usr/lib/Converter` on Linux. The FFmpeg source code is attached to every release.

## Uninstalling

| Platform | How |
|---|---|
| macOS | Delete `/Applications/Converter.app` |
| Windows | Settings, Apps, Installed apps, Converter, Uninstall |
| Linux | Remove the package with the package manager you installed it with |

Uninstalling leaves each user's settings and logs in place. Their locations are listed in
[Settings and data](settings-and-data.md).
