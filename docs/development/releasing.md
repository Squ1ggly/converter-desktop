# Releasing

Releases are automated with [release-please](https://github.com/googleapis/release-please) and
the Release workflow in `.github/workflows/release.yml`. A release is only published once the
installers for every platform have been built, installed on clean machines and launched.

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

GitHub opens the release PR as `github-actions`, so the CI runs its own events create wait for
approval. Instead, the Release workflow starts CI on the release PR's branch after every push to
`main`, and those checks show on the PR.

## 3. Merge it

Merging the release PR tags the commit as `vx.y.z` and creates a **draft** release. The Release
workflow then:

1. Builds the installers for every platform.
2. Installs each one on clean machines and checks it works (see [Installer tests](#installer-tests)).
3. Attaches the installers, the FFmpeg source tarball and each FFmpeg build configuration to
   the draft.
4. Publishes the release.

If any step fails, the release stays a draft and nobody can download it. Fix the cause, then
click **Re-run failed jobs** on the workflow run. Don't publish the draft by hand.

| Runner | Target | Installers |
|---|---|---|
| `macos-latest` | `aarch64-apple-darwin` | `.dmg` |
| `macos-latest` | `x86_64-apple-darwin` (cross-compiled) | `.dmg` |
| `windows-latest` | `x86_64-pc-windows-msvc` | `-setup.exe`, `.msi` |
| `windows-11-arm` | `aarch64-pc-windows-msvc` | `-setup.exe`, `.msi` |
| `ubuntu-24.04`, in an `ubuntu:22.04` container | `x86_64-unknown-linux-gnu` | `.deb`, `.rpm` |
| `ubuntu-24.04-arm`, in an `ubuntu:22.04` container | `aarch64-unknown-linux-gnu` | `.deb`, `.rpm` |

Linux packages are built on Ubuntu 22.04 so they need no newer glibc than 2.35, which is also
what the bundled FFmpeg needs. Building in a container keeps this working after GitHub retires
its Ubuntu 22.04 runners.

Each build runs `pnpm fetch-ffmpeg --target <triple>`, so every installer carries the FFmpeg for
its platform.

### The first release

`.release-please-manifest.json` starts at `0.0.0`, which release-please treats as "never
released", so it uses `initial-version` from `release-please-config.json` and proposes `0.1.0`.
Once `v0.1.0` exists, the manifest holds `0.1.0` and versions follow the commits from there.
`initial-version` is ignored after the first release, so nothing needs removing.

To force a particular version later, add a `Release-As: x.y.z` line to the body of a commit on
`main`. Don't use the `release-as` config option: it pins every future release to that version.

## Installer tests

The Installers workflow (`.github/workflows/installers.yml`) builds the real installers and
tests them. It runs on PRs that change packaging, from the Actions tab, and inside every release.

| Platform | Machines | What it checks |
|---|---|---|
| macOS | Apple Silicon and Intel runners | Installs the `.dmg` into `/Applications`, verifies the signature, checks the minimum macOS version of the app and of FFmpeg, launches the app and quits it |
| Windows | x64, ARM64, and the x64 build on ARM64 | Installs the NSIS and the MSI installer silently, checks the files, their processor type and, for signed builds, their signatures, launches the app and its WebView2, uninstalls |
| Linux | x64 and ARM64 containers of Ubuntu 22.04 and 24.04, Debian 12 and 13, Fedora 40 and latest, openSUSE Leap 15.6 | Installs the `.deb` or `.rpm` with the system's package manager, checks every library resolves, launches the app and its web view under Xvfb, removes the package |

On every machine, `scripts/smoke-tests/ffmpeg.sh` runs the installed `converter-ffmpeg` through
real conversions (H.264, HEVC, VP9, MPEG-4, AAC, Opus, Vorbis, MP3, WebP and AVIF) and decodes
each result.

## Building without releasing

Run the Release workflow from the Actions tab. It skips release-please, builds and tests every
platform, and attaches the installers to the run as artifacts.

## Signing

Builds are signed automatically once the secrets and variables below exist in the repository's
Actions settings. Only the Release workflow signs; PR builds never do. Without the secrets, macOS builds
are ad-hoc signed and Windows installers are unsigned, and users see the warnings described in
[Installing](../installing.md).

### macOS signing and notarisation

Needs a paid Apple Developer account and a **Developer ID Application** certificate.

| Secret | Value |
|---|---|
| `APPLE_CERTIFICATE` | The certificate and its private key exported as `.p12`, base64-encoded (`base64 -i certificate.p12`) |
| `APPLE_CERTIFICATE_PASSWORD` | The password used when exporting the `.p12` |
| `APPLE_SIGNING_IDENTITY` | The certificate's name, for example `Developer ID Application: Your Name (TEAMID)`. Required with `APPLE_CERTIFICATE`. |

For notarisation, add either an App Store Connect API key (recommended):

| Secret | Value |
|---|---|
| `APPLE_API_ISSUER` | The issuer ID shown above the keys list in App Store Connect |
| `APPLE_API_KEY` | The key ID |
| `APPLE_API_PRIVATE_KEY` | The contents of the downloaded `AuthKey_<key ID>.p8` file |

or an Apple ID:

| Secret | Value |
|---|---|
| `APPLE_ID` | The Apple ID email |
| `APPLE_PASSWORD` | An app-specific password for that Apple ID |
| `APPLE_TEAM_ID` | The team ID |

Notarisation only runs when `APPLE_CERTIFICATE` is set. Tauri imports the certificate into a
temporary keychain with a random password, so no keychain password secret is needed.

### Windows signing

Choose one. If more than one is set up, SignPath is used first, then the certificate, then Azure
Artifact Signing.

**SignPath Foundation** (free for open source projects, and what this repository uses). SignPath
Foundation owns the certificate, so Windows shows it as the publisher. To set it up:

1. Apply at [signpath.org](https://signpath.org/apply.html). SignPath Foundation needs
   [CODE_SIGNING.md](../../CODE_SIGNING.md), linked from the README and from every release. The
   Release workflow adds the link to each release's notes. Keep the page's team list current.
2. Turn on two-factor authentication for GitHub and SignPath. Everyone on the team needs it.
3. In SignPath, add the predefined **GitHub.com** trusted build system to the organisation and
   link it to the project.
4. In the project, create two artifact configurations with the slugs `app` and `installers`,
   from the files in `.signpath/artifact-configurations/`.
5. Create a CI user that can submit requests to the release signing policy, and an API token for
   it.
6. Add these to the repository:

   | Kind | Name | Value |
   |---|---|---|
   | Secret | `SIGNPATH_API_TOKEN` | The CI user's API token |
   | Variable | `SIGNPATH_ORGANIZATION_ID` | The SignPath organisation ID |
   | Variable | `SIGNPATH_PROJECT_SLUG` | The SignPath project slug |
   | Variable | `SIGNPATH_SIGNING_POLICY_SLUG` | The release signing policy's slug, usually `release-signing` |

7. Run the Release workflow from the Actions tab to try it. It doesn't release anything, but it
   does send signing requests.

Each Windows build sends two signing requests: one for `converter.exe`, then one for the
installers built around the signed app. A release builds x64 and ARM64, so it needs four
approvals in SignPath. Each build waits up to an hour for an approval, then fails; click
**Re-run failed jobs** to send the request again. The installers are bundled with
`--no-binary-patching`, so bundling doesn't change the signed app. The Windows install test
checks that a signed installer installs a signed app.

**Azure Artifact Signing** (paid, and the cheapest paid way to build SmartScreen reputation):

| Kind | Name | Value |
|---|---|---|
| Secret | `AZURE_CLIENT_ID` | The app registration's client ID |
| Secret | `AZURE_CLIENT_SECRET` | The app registration's client secret |
| Secret | `AZURE_TENANT_ID` | The Microsoft Entra tenant ID |
| Variable | `ARTIFACT_SIGNING_ENDPOINT` | The account's endpoint, for example `https://eus.codesigning.azure.net` |
| Variable | `ARTIFACT_SIGNING_ACCOUNT` | The Artifact Signing account name |
| Variable | `ARTIFACT_SIGNING_PROFILE` | The certificate profile name |

**A code signing certificate** exported as `.pfx`:

| Kind | Name | Value |
|---|---|---|
| Secret | `WINDOWS_CERTIFICATE` | The `.pfx`, base64-encoded |
| Secret | `WINDOWS_CERTIFICATE_PASSWORD` | The `.pfx` password |
| Variable | `WINDOWS_TIMESTAMP_URL` | Optional. Defaults to `http://timestamp.digicert.com`. |

Certificates issued since June 2023 keep their private key on a hardware token or a cloud HSM and
can't be exported as `.pfx`. For those, use Azure Artifact Signing, or replace the sign command in
`scripts/release/windows-signing-config.ps1` with the certificate authority's signing tool.

`.github/workflows/installers.yml`, `scripts/release/build-installers.sh` and
`scripts/release/windows-signing-config.ps1` hold the logic.

## Third-party notices

`THIRD-PARTY-NOTICES.md` lists FFmpeg, every npm package built into the UI and every Rust crate
compiled into the app, with their licences. Every installer includes it. CI fails when it is out
of date, so run this after changing dependencies, including in Dependabot PRs:

```bash
pnpm notices
```

## Updating FFmpeg

Update the version, URLs and SHA-256 checksums in `scripts/ffmpeg-builds.ts`, then run
`pnpm notices`. Every platform's build must come from the same FFmpeg release, because each
release attaches that one source tarball. The Windows ARM64 build is a `.7z` archive, so fetching
it needs 7-Zip (`7z`) on the `PATH`; GitHub's Windows runners have it.
If the FFmpeg release changes, also replace `licenses/FFmpeg/` with the files from the new source
tarball; the Installers workflow checks they match. The Installers workflow also checks each build
was configured with `--enable-gpl --enable-version3`, which is what the notices say.

## Updating Rust

`rust-toolchain.toml` pins the Rust version CI and releases use. Update it in its own PR, so new
Clippy lints are fixed in that PR rather than breaking unrelated ones.
