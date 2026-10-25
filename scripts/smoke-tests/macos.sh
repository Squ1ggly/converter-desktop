#!/bin/sh
# Installs Converter from a .dmg into /Applications the way a user would, checks the signature,
# the minimum macOS version and the bundled files, runs FFmpeg, launches the app, then removes it.
#
#   scripts/smoke-tests/macos.sh <Converter.dmg>
set -eu

dmg_path=$1
script_directory=$(cd "$(dirname "$0")" && pwd)
app_path=/Applications/Converter.app
mount_point=$(mktemp -d)

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

# "12.0" -> 12000, so versions compare as numbers.
version_number() {
  echo "$1" | awk -F. '{ printf "%d", $1 * 1000 + $2 }'
}

check_minimum_os() {
  binary_path=$1
  declared_minimum=$2
  # A universal binary lists one minimum per architecture; keep the highest.
  binary_minimum=$(vtool -show-build "$binary_path" | awk '
    $1 == "minos" { split($2, parts, "."); number = parts[1] * 1000 + parts[2] }
    $1 == "minos" && number > highest { highest = number; version = $2 }
    END { print version }')
  if [ -z "$binary_minimum" ]; then
    fail "vtool found no minimum macOS version in $binary_path"
  fi
  if [ "$(version_number "$binary_minimum")" -gt "$(version_number "$declared_minimum")" ]; then
    fail "$binary_path needs macOS $binary_minimum, but the app declares $declared_minimum"
  fi
  echo "ok   $(basename "$binary_path") needs macOS $binary_minimum"
}

hdiutil attach -nobrowse -readonly -mountpoint "$mount_point" "$dmg_path" >/dev/null
rm -rf "$app_path"
ditto "$mount_point/Converter.app" "$app_path"
hdiutil detach "$mount_point" >/dev/null
echo "ok   installed $app_path"

codesign --verify --deep --strict --verbose=2 "$app_path"
codesign --display --verbose=2 "$app_path" 2>&1 | grep -E "^(Signature|Authority|TeamIdentifier)="
echo "ok   signature is valid"

declared_minimum=$(/usr/libexec/PlistBuddy -c "Print :LSMinimumSystemVersion" "$app_path/Contents/Info.plist")
check_minimum_os "$app_path/Contents/MacOS/converter" "$declared_minimum"
check_minimum_os "$app_path/Contents/MacOS/converter-ffmpeg" "$declared_minimum"

for resource in LICENSE THIRD-PARTY-NOTICES.md licenses/FFmpeg/COPYING.GPLv3 licenses/FFmpeg/LICENSE.md; do
  if [ ! -s "$app_path/Contents/Resources/$resource" ]; then
    fail "$resource is missing from the app"
  fi
done
echo "ok   licences and notices are bundled"

sh "$script_directory/ffmpeg.sh" "$app_path/Contents/MacOS/converter-ffmpeg"

open -n "$app_path"
sleep 15
if ! pgrep -x converter >/dev/null; then
  fail "Converter is not running 15 seconds after launch"
fi
echo "ok   Converter is still running after 15 seconds"

osascript -e 'quit app "Converter"'
sleep 5
if pgrep -x converter >/dev/null; then
  fail "Converter did not quit"
fi
echo "ok   Converter quit"

rm -rf "$app_path"
echo "All macOS checks passed for $(basename "$dmg_path")"
