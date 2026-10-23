# Releasing

Releases are automated with [release-please](https://github.com/googleapis/release-please) and
the Release workflow in `.github/workflows/release.yml`.

## 1. Write conventional commits

Commits and squash-merged PR titles follow [Conventional Commits](https://www.conventionalcommits.org).
The type decides the next version:

| Commit | Version bump before 1.0 |
|---|---|
| `fix:`, `perf:` | Patch |
| `feat:` | Minor |
| `feat!:` or any other breaking change | Minor |
| `refactor:`, `docs:`, `test:`, `build:`, `ci:`, `chore:` | None |

[CONTRIBUTING.md](../../CONTRIBUTING.md) has the full list.

## 2. Review the release PR

On every push to `main`, release-please updates an open PR titled `chore(main): release x.y.z`.
It:

- bumps the version in `package.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock` and
  `src-tauri/tauri.conf.json`
- adds the changes to `CHANGELOG.md`

Don't change these versions by hand.

## 3. Merge it

Merging the release PR tags the commit as `vx.y.z` and creates the GitHub release. The Release
workflow then builds the installers and attaches them to that release:

| Runner | Target | Installers |
|---|---|---|
| `macos-latest` | `aarch64-apple-darwin` | `.dmg` |
| `macos-latest` | `x86_64-apple-darwin` (cross-compiled) | `.dmg` |
| `windows-latest` | `x86_64-pc-windows-msvc` | `-setup.exe`, `.msi` |
| `ubuntu-22.04` | `x86_64-unknown-linux-gnu` | `.deb`, `.rpm` |
| `ubuntu-22.04-arm` | `aarch64-unknown-linux-gnu` | `.deb`, `.rpm` |

Each job runs `pnpm fetch-ffmpeg --target <triple>` before building, so every installer carries
the FFmpeg for its platform.

## Building without releasing

Run the Release workflow from the Actions tab. It skips release-please, builds every platform,
and attaches the installers to the run as artifacts.

## Signing

Installers are not signed, so users see Gatekeeper and SmartScreen warnings. See
[Installing](../installing.md) for how users can open them anyway.
