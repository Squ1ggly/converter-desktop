#!/usr/bin/env bash
# Builds the installers for $TARGET and copies them into installers/.
#
# CI passes every signing secret as an environment variable, empty when the secret doesn't exist.
# Empty ones are removed first, so without secrets Tauri ad-hoc signs on macOS and leaves Windows
# installers unsigned.
set -euo pipefail

signing_variables=(
  APPLE_CERTIFICATE APPLE_CERTIFICATE_PASSWORD APPLE_SIGNING_IDENTITY
  APPLE_API_ISSUER APPLE_API_KEY APPLE_API_KEY_PATH APPLE_ID APPLE_PASSWORD APPLE_TEAM_ID
  AZURE_CLIENT_ID AZURE_CLIENT_SECRET AZURE_TENANT_ID
)
notarisation_variables=(APPLE_API_ISSUER APPLE_API_KEY APPLE_API_KEY_PATH APPLE_ID APPLE_PASSWORD APPLE_TEAM_ID)

for variable_name in "${signing_variables[@]}"; do
  if [ -z "${!variable_name:-}" ]; then
    unset "$variable_name"
  fi
done

# tauri.conf.json asks for ad-hoc signing ("-"), which only APPLE_SIGNING_IDENTITY overrides.
if [ -n "${APPLE_CERTIFICATE:-}" ] && [ -z "${APPLE_SIGNING_IDENTITY:-}" ]; then
  echo "APPLE_CERTIFICATE is set, so APPLE_SIGNING_IDENTITY must be set too." >&2
  exit 1
fi

# Apple only notarises apps signed with a Developer ID certificate.
if [ -z "${APPLE_CERTIFICATE:-}" ]; then
  unset "${notarisation_variables[@]}"
fi

build_arguments=(--target "$TARGET")
if [ -n "${WINDOWS_SIGNING_CONFIG:-}" ]; then
  build_arguments+=(--config "$WINDOWS_SIGNING_CONFIG")
fi

pnpm tauri build "${build_arguments[@]}"

mkdir -p installers
find "src-tauri/target/$TARGET/release/bundle" -type f \
  \( -name '*.dmg' -o -name '*-setup.exe' -o -name '*.msi' -o -name '*.deb' -o -name '*.rpm' \) \
  -exec cp {} installers/ \;
ls -l installers
