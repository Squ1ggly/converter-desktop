# Security policy

## Supported versions

Only the latest release is supported. Fixes are made on `main` and shipped in the next release,
so update to the latest version before reporting.

## Reporting a vulnerability

Report vulnerabilities privately. Don't open a public issue.

1. Open the repository's **Security** tab.
2. Click **Report a vulnerability**, or go straight to
   [the private report form](https://github.com/Squ1ggly/converter-desktop/security/advisories/new).

## What to include

- The Converter version and your operating system and version
- What the problem is and what an attacker could do with it
- Steps to reproduce it, and a sample file if one triggers it
- The app's logs, if they show anything relevant. Their location is listed in
  [Settings and data](docs/settings-and-data.md#where-files-are-kept).

## What to expect

- An acknowledgement within a week.
- Updates as the report is investigated, and an agreed date before anything is made public.
- Credit in the published advisory, unless you prefer not to be named.

## Scope

Converter works on local files only. Reports about how it handles input files, output paths and
the bundled FFmpeg are in scope. Vulnerabilities in FFmpeg itself belong with the
[FFmpeg project](https://ffmpeg.org/security.html). Once FFmpeg publishes a fix, the bundled build
is updated in the next release.
