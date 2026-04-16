#!/bin/bash
# Injects 首次运行说明 into the built DMG (hidden filename: less clutter in scripts/)
# Usage: ./scripts/.inject-dmg-readme.sh <path_to.dmg>

set -e

DMG_PATH="$1"

if [ -z "$DMG_PATH" ] || [ ! -f "$DMG_PATH" ]; then
    echo "Usage: $0 <path_to.dmg>"
    exit 1
fi

if [ "$(uname)" != "Darwin" ]; then
    echo "This script only works on macOS"
    exit 1
fi

echo "Adding 首次运行前请先看我.txt to DMG: $DMG_PATH"

WORK_DIR=$(mktemp -d)
MOUNT_POINT="$WORK_DIR/volume"
RW_DMG="$WORK_DIR/rw.dmg"

cat > "$WORK_DIR/首次运行前请先看我.txt" << 'EOF'
首次运行前请先看我
==================

1. 将 Muse_Generator.app 拖拽复制到「应用程序」文件夹 (Applications)。

2. 打开「终端」，运行：
   xattr -cr /Applications/Muse_Generator.app

3. 启动本工具（可在启动台或应用程序中找到）。
EOF

echo "Converting to read-write..."
hdiutil convert "$DMG_PATH" -format UDRW -o "$RW_DMG" 2>/dev/null

echo "Mounting DMG..."
hdiutil attach "$RW_DMG" -mountpoint "$MOUNT_POINT" -nobrowse 2>/dev/null

cp "$WORK_DIR/首次运行前请先看我.txt" "$MOUNT_POINT/"

echo "Unmounting DMG..."
hdiutil detach "$MOUNT_POINT" -force 2>/dev/null || true

echo "Creating final DMG..."
NEW_DMG="${DMG_PATH%.dmg}_new.dmg"
hdiutil convert "$RW_DMG" -format UDZO -o "$NEW_DMG" 2>/dev/null

mv "$NEW_DMG" "$DMG_PATH"

rm -rf "$WORK_DIR"

echo "Done."
