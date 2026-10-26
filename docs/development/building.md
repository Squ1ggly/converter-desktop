# Building

## Prerequisites

| Tool | Version |
|---|---|
| [Rust](https://rustup.rs) | 1.99.1, pinned in `rust-toolchain.toml`. rustup installs it on first use. |
| [Node.js](https://nodejs.org) | 24 or later |
| [pnpm](https://pnpm.io) | The version in `packageManager` in `package.json` |
| [7-Zip](https://www.7-zip.org) | Only to fetch FFmpeg for Windows ARM64. `7z` must be on the `PATH`. |
| Platform tools | See [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) |

On Debian or Ubuntu, CI installs these system packages:

```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
```

## First run

```bash
pnpm install
pnpm fetch-ffmpeg
pnpm dev
```

`pnpm fetch-ffmpeg` is needed once per machine, and again when the pinned FFmpeg version changes.
Without it, `pnpm dev` and `pnpm build` fail because the sidecar is missing.

## Commands

| Command | What it does |
|---|---|
| `pnpm dev` | Runs the app with hot reload |
| `pnpm build` | Builds the installers for this machine |
| `pnpm check` | Type-checks the UI, scripts and UI tests with svelte-check, and the end-to-end tests with tsc |
| `pnpm ui:dev` | Runs only the UI in Vite, on port 1420 |
| `pnpm ui:build` | Builds only the UI into `dist/` |
| `pnpm fetch-ffmpeg [--target <triple>]` | Downloads and checksum-verifies FFmpeg for this machine or the given target |
| `pnpm notices` | Regenerates `THIRD-PARTY-NOTICES.md` from the bundled FFmpeg, npm packages and Rust crates. CI fails if the committed file is out of date. |
| `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | Checks Rust formatting |
| `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | Lints the Rust code |

For tests, see [Testing](../../CONTRIBUTING.md#testing).

## FFmpeg

`pnpm fetch-ffmpeg` detects this machine's target from `rustc -vV`, downloads the pinned archive,
checks its SHA-256, and saves the executable as
`src-tauri/binaries/converter-ffmpeg-<target triple>` (with `.exe` on Windows). The folder is
gitignored. The Windows ARM64 build comes as a `.7z` archive, so fetching it needs 7-Zip.

Supported targets:

- `aarch64-apple-darwin`
- `x86_64-apple-darwin`
- `x86_64-pc-windows-msvc`
- `aarch64-pc-windows-msvc`
- `x86_64-unknown-linux-gnu`
- `aarch64-unknown-linux-gnu`

To change the FFmpeg version, update `FFMPEG_VERSION`, each URL and each SHA-256 in
`scripts/fetch-ffmpeg.ts`, then run `pnpm fetch-ffmpeg` again.

## Installers

`pnpm build` writes the installers for this machine to `src-tauri/target/release/bundle/`:

| Platform | Installers |
|---|---|
| macOS | `.app` and `.dmg` |
| Windows | NSIS `-setup.exe` (per-machine) and `.msi` |
| Linux | `.deb` and `.rpm` |

Each platform has to be built on that platform. The exception is Intel macOS, which can be built
on Apple Silicon:

```bash
rustup target add x86_64-apple-darwin
pnpm fetch-ffmpeg --target x86_64-apple-darwin
pnpm build --target x86_64-apple-darwin
```

A build with `--target` is written to `src-tauri/target/<triple>/release/bundle/`.

The [Release workflow](releasing.md) builds every platform on GitHub Actions. Builds are not
signed. Signing needs [macOS signing and notarisation](https://v2.tauri.app/distribute/sign/macos/)
and [Windows code signing](https://v2.tauri.app/distribute/sign/windows/) to be set up.
