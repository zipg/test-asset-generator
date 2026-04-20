#!/usr/bin/env bash
# 下载与当前 Rust target 匹配的静态 ffmpeg 到 src-tauri/resources/ffmpeg（供 Tauri bundle.resources）
# 来源: https://github.com/eugeneware/ffmpeg-static（MIT，见该仓库 LICENSE）
set -euo pipefail

TARGET="${1:?Usage: $0 <aarch64-apple-darwin|x86_64-apple-darwin>}"

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEST_DIR="$ROOT/src-tauri/resources"
DEST="$DEST_DIR/ffmpeg"

# 固定版本便于复现构建；升级时改此 tag 并做一次生成回归
REL="${MUSE_FFMPEG_MACOS_STATIC_TAG:-b6.1.1}"
BASE="https://github.com/eugeneware/ffmpeg-static/releases/download/${REL}"

case "$TARGET" in
  aarch64-apple-darwin) ASSET="ffmpeg-darwin-arm64" ;;
  x86_64-apple-darwin) ASSET="ffmpeg-darwin-x64" ;;
  *)
    echo "Unknown macOS Rust target: $TARGET (expected aarch64-apple-darwin or x86_64-apple-darwin)" >&2
    exit 1
    ;;
esac

mkdir -p "$DEST_DIR"
echo "Downloading ${BASE}/${ASSET} ..."
curl -fsSL "${BASE}/${ASSET}" -o "$DEST"
chmod +x "$DEST"
echo "Wrote $DEST ($(wc -c < "$DEST") bytes)"
