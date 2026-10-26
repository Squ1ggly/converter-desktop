# Code signing policy

Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by
[SignPath Foundation](https://signpath.org).

## What is signed

The Windows downloads on the [releases page](https://github.com/Squ1ggly/converter-desktop/releases):

- the NSIS installer (`-setup.exe`) and the MSI installer
- `converter.exe`, the app they install

Only files built from this repository are signed. They are built from the release's tagged
commit by the [Release workflow](.github/workflows/release.yml) on GitHub-hosted runners, and each
signing request is approved by hand.

These files are not signed with this certificate:

- `converter-ffmpeg.exe`, the bundled FFmpeg. It is built by a third party; see
  [Third-party notices](THIRD-PARTY-NOTICES.md).
- `uninstall.exe`, which the NSIS installer writes when it installs the app.

## Team roles

| Role | Members |
|---|---|
| Committers and reviewers | [Squ1ggly](https://github.com/Squ1ggly) |
| Approvers | [Squ1ggly](https://github.com/Squ1ggly) |

Committers can change the source without further review. Pull requests from anyone else are
reviewed by a committer before they are merged. Approvers decide whether each release is signed.

## Privacy policy

This program will not transfer any information to other networked systems unless specifically
requested by the user or the person installing or operating it.

Converter converts files on your own computer and never uploads them. Links in the Help page
open in your web browser when you click them. On Windows, if Microsoft Edge WebView2 is missing,
the installer downloads it from Microsoft.

## Reporting a problem

To report a signed file that behaves badly, follow the [security policy](SECURITY.md).
