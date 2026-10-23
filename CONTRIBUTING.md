# Contributing

## Commit messages

Commits follow [Conventional Commits](https://www.conventionalcommits.org): `type(scope): summary`,
for example `fix(video): keep the audio track when changing the frame rate`. When a PR is squash
merged, its title becomes the commit message, so the title has to follow the same format.

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
| `chore` | Anything else | No |

Mark a breaking change with `!` after the type, for example `feat!: drop macOS 10.15 support`.
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
pnpm ui:build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```
