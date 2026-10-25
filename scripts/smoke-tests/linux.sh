#!/bin/sh
# Installs a Converter .deb or .rpm with the distribution's package manager, checks every library
# resolves, runs FFmpeg, launches the app under Xvfb, then removes the package.
# Runs as root in a clean container of the distribution under test.
#
#   scripts/smoke-tests/linux.sh <package>
set -eu

package_path=$(realpath "$1")
script_directory=$(cd "$(dirname "$0")" && pwd)
resources_directory=/usr/lib/Converter
launch_wait_seconds=15
log_path=$(mktemp)

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

install_package() {
  case "$package_path" in
    *.deb)
      apt-get update -q
      DEBIAN_FRONTEND=noninteractive apt-get install -y -q "$package_path" xvfb xauth
      ;;
    *.rpm)
      if command -v dnf >/dev/null; then
        dnf install -y -q "$package_path" /usr/bin/xvfb-run /usr/bin/xauth
      else
        zypper --non-interactive install --allow-unsigned-rpm "$package_path" xvfb-run
      fi
      ;;
  esac
}

remove_package() {
  case "$package_path" in
    *.deb)
      apt-get remove -y -q "$(dpkg-deb --field "$package_path" Package)"
      ;;
    *.rpm)
      rpm --erase "$(rpm --query --package --queryformat '%{NAME}' "$package_path")"
      ;;
  esac
}

# A WebKitWebProcess means the window's web view started, not only the main process.
web_process_is_running() {
  grep -qs 'WebKitWebProces[s]' /proc/[0-9]*/cmdline
}

. /etc/os-release
echo "Testing $(basename "$package_path") on $PRETTY_NAME ($(uname -m))"

install_package
echo "ok   installed"

missing_libraries=$(ldd /usr/bin/converter | grep "not found" || true)
if [ -n "$missing_libraries" ]; then
  fail "missing libraries: $missing_libraries"
fi
echo "ok   every library of /usr/bin/converter resolves"

for resource in LICENSE THIRD-PARTY-NOTICES.md licenses/FFmpeg/COPYING.GPLv3 licenses/FFmpeg/LICENSE.md; do
  if [ ! -s "$resources_directory/$resource" ]; then
    fail "$resources_directory/$resource is missing"
  fi
done
echo "ok   licences and notices are installed"

sh "$script_directory/ffmpeg.sh" /usr/bin/converter-ffmpeg

# `timeout` exits with 124 when it has to stop the app, so 124 means it was still running.
xvfb-run --auto-servernum timeout "$launch_wait_seconds" converter >"$log_path" 2>&1 &
launcher_pid=$!
sleep $((launch_wait_seconds - 5))
if ! web_process_is_running; then
  cat "$log_path"
  fail "the web view did not start"
fi
launch_status=0
wait "$launcher_pid" || launch_status=$?
cat "$log_path"
if [ "$launch_status" -ne 124 ]; then
  fail "Converter exited with $launch_status within $launch_wait_seconds seconds"
fi
echo "ok   Converter and its web view were still running after $launch_wait_seconds seconds"

remove_package
if [ -e /usr/bin/converter ]; then
  fail "/usr/bin/converter is still there after removing the package"
fi
echo "ok   removed"
echo "All Linux checks passed"
