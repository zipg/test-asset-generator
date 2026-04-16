#!/bin/bash
# Post-build script to customize DMG for macOS
# Usage: ./post_build_dmg.sh <path_to_dmg>

set -e

DMG_PATH="$1"
APP_NAME="Muse_Generator"
SCRIPT_NAME="点我自动安装"

if [ -z "$DMG_PATH" ] || [ ! -f "$DMG_PATH" ]; then
    echo "Usage: $0 <path_to_dmg>"
    exit 1
fi

echo "Customizing DMG: $DMG_PATH"

# Create temporary mount point
MOUNT_POINT="/tmp/dmg_mount_$$"
mkdir -p "$MOUNT_POINT"

# Backup original DMG
cp "$DMG_PATH" "${DMG_PATH}.bak"

# Mount DMG read-write
hdiutil attach "$DMG_PATH" -mountpoint "$MOUNT_POINT" -shadow -nobrowse 2>/dev/null || {
    echo "Failed to mount DMG"
    exit 1
}

# Create the auto-install script
cat > "${MOUNT_POINT}/${SCRIPT_NAME}.sh" << 'SCRIPT_EOF'
#!/bin/bash

# Copy app to Applications
cp -R "/Volumes/Muse_Generator/Muse_Generator.app" /Applications/

# Remove quarantine attribute
xattr -cr /Applications/Muse_Generator.app

# Ask user if they want to launch
osascript -e 'display dialog "安装完成！是否立即启动 Muse_Generator？" buttons {"启动", "不了"} default button 1' \
    && open /Applications/Muse_Generator.app \
    || true
SCRIPT_EOF

chmod +x "${MOUNT_POINT}/${SCRIPT_NAME}.sh"

# Create a symbolic link to Applications folder
ln -sf /Applications "${MOUNT_POINT}/Applications"

# Unmount DMG
hdiutil detach "$MOUNT_POINT" -force 2>/dev/null || true

# Re-size DMG to fit contents
hdiutil resize "$DMG_PATH" 2>/dev/null || true

echo "DMG customization complete!"
echo "Auto-install script created: ${SCRIPT_NAME}.sh"
