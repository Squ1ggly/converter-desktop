#!/bin/sh
# Proves an installed converter-ffmpeg has the codecs the app uses, by converting a generated clip
# the way the app does and decoding every result.
#
#   scripts/smoke-tests/ffmpeg.sh <path to converter-ffmpeg>
set -eu

ffmpeg_path=$1
work_directory=$(mktemp -d)
trap 'rm -rf "$work_directory"' EXIT

fail() {
  echo "FAIL: $*" >&2
  exit 1
}

require_codec() {
  codec_list=$1
  codec_name=$2
  if ! printf '%s\n' "$codec_list" | grep -Eq "^ [A-Z.]{6} $codec_name "; then
    fail "converter-ffmpeg has no $codec_name codec"
  fi
}

convert() {
  "$ffmpeg_path" -hide_banner -loglevel error -y "$@"
}

# Decodes the whole file and checks FFmpeg reports the expected stream.
expect_stream() {
  file_name=$1
  stream_pattern=$2
  convert -i "$work_directory/$file_name" -f null - || fail "$file_name does not decode"
  stream_info=$("$ffmpeg_path" -hide_banner -i "$work_directory/$file_name" 2>&1 || true)
  if ! printf '%s\n' "$stream_info" | grep -Eq "$stream_pattern"; then
    printf '%s\n' "$stream_info" >&2
    fail "$file_name has no stream matching '$stream_pattern'"
  fi
  echo "ok   $file_name: $stream_pattern"
}

"$ffmpeg_path" -hide_banner -version | head -n 1

encoders=$("$ffmpeg_path" -hide_banner -encoders)
for encoder in libx264 libx265 libvpx-vp9 libaom-av1 libwebp libopus libvorbis libmp3lame aac mpeg4 png; do
  require_codec "$encoders" "$encoder"
done

decoders=$("$ffmpeg_path" -hide_banner -decoders)
for decoder in h264 hevc vp9 mpeg4 aac opus mp3 png mjpeg; do
  require_codec "$decoders" "$decoder"
done
if ! printf '%s\n' "$decoders" | grep -Eq "^ [A-Z.]{6} (libdav1d|libaom-av1|av1) "; then
  fail "converter-ffmpeg has no AV1 decoder"
fi
echo "ok   encoders and decoders"

# HEVC video with Vorbis audio, so the conversions below also exercise those decoders.
convert -f lavfi -i "testsrc2=size=320x240:rate=25:duration=2" -f lavfi -i "sine=frequency=440:duration=2" \
  -c:v libx265 -pix_fmt yuv420p -c:a libvorbis -shortest "$work_directory/source.mkv"
expect_stream source.mkv "Video: hevc"

convert -i "$work_directory/source.mkv" -c:v libx264 -preset medium -crf 23 -pix_fmt yuv420p \
  -c:a aac -b:a 160k -movflags +faststart "$work_directory/video.mp4"
expect_stream video.mp4 "Video: h264"
expect_stream video.mp4 "Audio: aac"

convert -i "$work_directory/source.mkv" -c:v libvpx-vp9 -crf 33 -b:v 0 -row-mt 1 -pix_fmt yuv420p \
  -c:a libopus -b:a 128k "$work_directory/video.webm"
expect_stream video.webm "Video: vp9"
expect_stream video.webm "Audio: opus"

convert -i "$work_directory/source.mkv" -c:v mpeg4 -q:v 4 -c:a libmp3lame -b:a 160k "$work_directory/video.avi"
expect_stream video.avi "Video: mpeg4"
expect_stream video.avi "Audio: mp3"

convert -i "$work_directory/source.mkv" -frames:v 1 "$work_directory/frame.png"
expect_stream frame.png "Video: png"

convert -i "$work_directory/frame.png" -c:v libwebp -quality 80 -frames:v 1 "$work_directory/image.webp"
expect_stream image.webp "Video: webp"

convert -i "$work_directory/frame.png" -pix_fmt yuv420p -c:v libaom-av1 -still-picture 1 -crf 30 \
  -cpu-used 6 -frames:v 1 "$work_directory/image.avif"
expect_stream image.avif "Video: av1"

echo "All FFmpeg checks passed"
