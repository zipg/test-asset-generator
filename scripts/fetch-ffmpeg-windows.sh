#!/usr/bin/env bash
# 下载 BtbN win64-gpl zip，解压出 bin/ffmpeg.exe 到 src-tauri/resources/ffmpeg.exe
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEST_DIR="$ROOT/src-tauri/resources"
DEST="$DEST_DIR/ffmpeg.exe"
ZIP_URL="${MUSE_FFMPEG_WINDOWS_ZIP_URL:-https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip}"

mkdir -p "$DEST_DIR"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "Downloading FFmpeg zip..."
curl -fsSL "$ZIP_URL" -o "$TMP/ffmpeg.zip"

echo "Extracting ffmpeg.exe..."
unzip -q "$TMP/ffmpeg.zip" -d "$TMP/out"
FFMPEG="$(find "$TMP/out" -type f -name ffmpeg.exe -path "*/bin/ffmpeg.exe" 2>/dev/null | head -1)"
if [[ -z "$FFMPEG" ]]; then
  FFMPEG="$(find "$TMP/out" -type f -name ffmpeg.exe 2>/dev/null | head -1)"
fi
if [[ -z "$FFMPEG" ]]; then
  echo "Could not find ffmpeg.exe in zip" >&2
  exit 1
fi

cp -f "$FFMPEG" "$DEST"
echo "Wrote $DEST ($(wc -c < "$DEST") bytes)"
