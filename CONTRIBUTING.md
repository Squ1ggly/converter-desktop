# Contributing

Read [AGENTS.md](AGENTS.md) for the coding rules and the checks to run before a pull request.
Everyone taking part follows the [code of conduct](CODE_OF_CONDUCT.md).

## Reporting bugs

Open an issue with the [bug report form](https://github.com/Squ1ggly/converter-desktop/issues/new/choose).
It asks for your OS, the app version and the app's logs. Report security problems privately
instead, as described in [SECURITY.md](SECURITY.md).

## Commit messages

Commits follow [Conventional Commits](https://www.conventionalcommits.org): `type(scope): summary`,
for example `fix(video): keep the audio track when changing the frame rate`. Keep the subject
short and on one line.

Pull requests are squash merged, and the PR title becomes the commit subject on `main`, so the
title has to follow the same format.

| Type | Use for | In the changelog |
|---|---|---|
| `feat` | A new feature | Yes, bumps the minor version |
| `fix` | A bug fix | Yes, bumps the patch version |
| `perf` | A speed or memory improvement | Yes, bumps the patch version |
| `refactor` | A code change that keeps the same behaviour | No |
| `docs` | Documentation only | No |
| `test` | Adding or fixing tests | No |
| `build` | Dependencies, build scripts, the bundled FFmpeg | No |
| `ci` | GitHub Actions workflows | No |
| `style` | Formatting only | No |
| `chore` | Anything else | No |

Mark a breaking change with `!` after the type, for example `feat!: rename the settings file`.
Before 1.0 it bumps the minor version.

## Releases

[release-please](https://github.com/googleapis/release-please) keeps a release PR open on `main`,
titled `chore(main): release x.y.z`. It bumps the version in `package.json`,
`src-tauri/Cargo.toml`, `src-tauri/Cargo.lock` and `src-tauri/tauri.conf.json`, and adds the
changes to `CHANGELOG.md`. Don't change these versions by hand.

To release, merge the release PR. That tags the commit and creates the GitHub release, then the
Release workflow builds the installers for every platform and attaches them to it.

To build installers without releasing, run the Release workflow from the Actions tab. The
installers are attached to the run as artifacts.

## Checks

CI runs these on Linux, macOS and Windows for every PR and every push to `main`. To run them
locally:

```bash
pnpm install
pnpm fetch-ffmpeg
pnpm check
pnpm test:ui
pnpm ui:build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

## Testing

There are three suites.

| Suite | Command | What it covers |
|---|---|---|
| Rust | `cargo test --manifest-path src-tauri/Cargo.toml` | Unit tests, and every conversion run through the downloaded FFmpeg |
| UI | `pnpm test:ui` | The Svelte UI in WebKit and Chromium, with the backend mocked |
| End-to-end | `pnpm test:e2e` | The built app converting real files, Linux and Windows only |

### UI tests

`tests/ui/` holds Playwright tests that run the real UI from the Vite dev server.
`tests/ui/support/mock-backend.ts` stands in for the Rust side: it answers every Tauri command,
plays native dialogs and file drops, and streams conversion events. Tests start the app with
`launchConverter()` and drive the backend through the `MockBackend` it returns. WebKit is what
macOS and Linux use, Chromium is what WebView2 on Windows uses.

Install the browsers once with `pnpm exec playwright install chromium webkit`.

`pnpm screenshots` saves PNGs of the main states in light and dark mode to
`test-results/screenshots/`, or to `SCREENSHOT_DIR` if set.

### End-to-end tests

`tests/e2e/` holds WebdriverIO tests that build a debug copy of the app, start it through
[tauri-driver](https://v2.tauri.app/develop/tests/webdriver/) and convert sample files made with
the FFmpeg from `pnpm fetch-ffmpeg`. They check the converted files on disk.

tauri-driver has no macOS support, because macOS has no WebDriver for its webview. CI runs these
tests on Linux and Windows. To run them locally on Linux, install the WebKitGTK WebDriver
(`webkitgtk-webdriver` on Ubuntu 26.04, `webkit2gtk-driver` on older releases) and
`cargo install tauri-driver --locked`. On Windows, also put a `msedgedriver` that matches the
installed WebView2 on your `PATH`, and run the tests from a terminal that isn't elevated. WebView2
150 and later ignores the driver's settings for an app running as administrator, so the session
never starts.
