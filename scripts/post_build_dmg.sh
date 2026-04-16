#!/bin/bash
# Simple DMG post-processing script for macOS
# This script adds an auto-install application to the DMG

set -e

DMG_PATH="$1"
SCRIPT_APP_NAME="点我自动安装.app"

if [ -z "$DMG_PATH" ] || [ ! -f "$DMG_PATH" ]; then
    echo "Usage: $0 <path_to_dmg>"
    exit 1
fi

echo "Processing DMG: $DMG_PATH"

# Create temporary mount point
MOUNT_POINT="/tmp/dmg_$$"
VOLUME_NAME="Muse_Generator"

mkdir -p "$MOUNT_POINT"

# Mount DMG
hdiutil attach "$DMG_PATH" -mountpoint "$MOUNT_POINT" -nobrowse -shadow 2>/dev/null

sleep 1

# Create the auto-install application
SCRIPT_APP_PATH="${MOUNT_POINT}/${SCRIPT_APP_NAME}"
mkdir -p "${SCRIPT_APP_PATH}/Contents/MacOS"

# Create the install script
cat > "${SCRIPT_APP_PATH}/Contents/MacOS/run.sh" << 'SCRIPT_EOF'
#!/bin/bash

# Get the volume path
VOLUME_PATH="$(dirname "$(dirname "$(dirname "$0")")")"
APP_NAME="Muse_Generator"

# Copy app to Applications
if [ -d "/Applications/${APP_NAME}.app" ]; then
    rm -rf "/Applications/${APP_NAME}.app"
fi
cp -R "${VOLUME_PATH}/${APP_NAME}.app" /Applications/

# Remove quarantine attribute
xattr -cr /Applications/${APP_NAME}.app

# Ask user if they want to launch
osascript -e 'tell app "System Events" to display dialog "安装完成！是否立即启动 Muse_Generator？" buttons {"启动", "不了"} with icon note default button 1'
if [ $? -eq 0 ]; then
    open /Applications/${APP_NAME}.app
fi

# Unmount the volume
umount /Volumes/${APP_NAME} 2>/dev/null || true
SCRIPT_EOF

chmod +x "${SCRIPT_APP_PATH}/Contents/MacOS/run.sh"

# Create Info.plist
cat > "${SCRIPT_APP_PATH}/Contents/Info.plist" << 'PLIST_EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>run.sh</string>
    <key>CFBundleIdentifier</key>
    <string>com.muse.generator.autoinstall</string>
    <key>CFBundleName</key>
    <string>点我自动安装</string>
    <key>CFBundleVersion</key>
    <string>1.0</string>
    <key>LSBackgroundOnly</key>
    <false/>
    <key>LSUIElement</key>
    <true/>
</dict>
</plist>
PLIST_EOF

# Create a README file
cat > "${MOUNT_POINT}/README.txt" << 'README_EOF'
使用方法：
1. 将 "Muse_Generator.app" 拖拽到 Applications 文件夹
2. 或双击 "点我自动安装.app" 自动完成安装和启动

如需移除 quarantine 权限（解决"无法打开"问题），请运行：
xattr -cr /Applications/Muse_Generator.app
README_EOF

# Unmount
hdiutil detach "$MOUNT_POINT" -force 2>/dev/null || true
rmdir "$MOUNT_POINT" 2>/dev/null || true

echo "Done! DMG processed."