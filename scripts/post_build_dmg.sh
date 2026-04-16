#!/bin/bash
# Post-build DMG customization script
# Usage: ./post_build_dmg.sh <path_to_dmg>
#
# This script adds a README file to the DMG

set -e

DMG_PATH="$1"

if [ -z "$DMG_PATH" ] || [ ! -f "$DMG_PATH" ]; then
    echo "Usage: $0 <path_to_dmg>"
    exit 1
fi

if [ "$(uname)" != "Darwin" ]; then
    echo "This script only works on macOS"
    exit 1
fi

echo "Adding README to DMG: $DMG_PATH"

# Create temp directory
WORK_DIR=$(mktemp -d)
MOUNT_POINT="$WORK_DIR/volume"
RW_DMG="$WORK_DIR/rw.dmg"

# Create the README file
cat > "$WORK_DIR/首次运行前请先看我.txt" << 'EOF'
首次运行前请先看我
====================

如果 App 复制到 Applications 后无法启动（提示"无法打开"），
请在终端运行以下命令：

    xattr -cr /Applications/Muse_Generator.app

然后重新双击打开 App 即可正常运行。
EOF

# Convert DMG to read-write
echo "Converting to read-write..."
hdiutil convert "$DMG_PATH" -format UDRW -o "$RW_DMG" 2>/dev/null

# Mount the read-write DMG
echo "Mounting DMG..."
hdiutil attach "$RW_DMG" -mountpoint "$MOUNT_POINT" -nobrowse 2>/dev/null

# Copy the README to DMG volume
cp "$WORK_DIR/首次运行前请先看我.txt" "$MOUNT_POINT/"

# Unmount
echo "Unmounting DMG..."
hdiutil detach "$MOUNT_POINT" -force 2>/dev/null || true

# Convert back to compressed
echo "Creating final DMG..."
NEW_DMG="${DMG_PATH%.dmg}_new.dmg"
hdiutil convert "$RW_DMG" -format UDZO -o "$NEW_DMG" 2>/dev/null

# Replace original
mv "$NEW_DMG" "$DMG_PATH"

# Clean up
rm -rf "$WORK_DIR"

echo "Done! README has been added to the DMG."