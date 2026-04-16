#!/bin/bash
# DMG Post-Processing Script (macOS only)
# Run this AFTER building, BEFORE distributing the DMG
#
# Usage: ./post_build_dmg.sh <path_to_dmg>
#
# This script adds an "Auto Install" app to the DMG contents

set -e

DMG_PATH="$1"
APP_NAME="Muse_Generator"
SCRIPT_APP_NAME="点我自动安装"

if [ -z "$DMG_PATH" ] || [ ! -f "$DMG_PATH" ]; then
    echo "Usage: $0 <path_to_dmg>"
    exit 1
fi

if [ "$(uname)" != "Darwin" ]; then
    echo "This script only works on macOS"
    exit 1
fi

echo "Processing DMG: $DMG_PATH"

# Create a temporary directory
MOUNT_POINT="/tmp/dmg_mount_$$"
mkdir -p "$MOUNT_POINT"

# Mount DMG read-only
echo "Mounting DMG..."
hdiutil attach "$DMG_PATH" -mountpoint "$MOUNT_POINT" -readOnly -nobrowse 2>/dev/null

# Create a read-write DMG for modifications
RW_DMG="/tmp/dmg_rw_$$.dmg"
echo "Creating writable DMG..."
hdiutil convert "$DMG_PATH" -format UDRW -o "$RW_DMG" 2>/dev/null

# Detach and remount as read-write
hdiutil detach "$MOUNT_POINT" -force 2>/dev/null || true
echo "Remounting as read-write..."
hdiutil attach "$RW_DMG" -mountpoint "$MOUNT_POINT" -nobrowse 2>/dev/null

# Create the auto-install app
echo "Creating auto-install app..."
SCRIPT_APP="$MOUNT_POINT/$SCRIPT_APP_NAME.app"
mkdir -p "$SCRIPT_APP/Contents/MacOS"
mkdir -p "$SCRIPT_APP/Contents/Resources"

cat > "$SCRIPT_APP/Contents/MacOS/run" << 'SCRIPT_EOF'
#!/bin/bash
VOLUME_PATH="$(df "$0" | tail -1 | awk '{print $NF}')"
APP_NAME="Muse_Generator"

# Copy to Applications
if [ -d "/Applications/${APP_NAME}.app" ]; then
    rm -rf "/Applications/${APP_NAME}.app"
fi
cp -R "${VOLUME_PATH}/${APP_NAME}.app" /Applications/

# Remove quarantine
xattr -cr /Applications/${APP_NAME}.app 2>/dev/null || true

# Ask to launch
osascript << 'EOF'
display dialog "安装完成！是否立即启动？" buttons {"启动", "不了"} default button 1 with icon note
if button returned of result is "启动" then
    tell application "Muse_Generator" to activate
end if
end tell
EOF
SCRIPT_EOF
chmod +x "$SCRIPT_APP/Contents/MacOS/run"

cat > "$SCRIPT_APP/Contents/Info.plist" << 'PLIST_EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key><string>run</string>
    <key>CFBundleIdentifier</key><string>com.muse.generator.autoinstall</string>
    <key>CFBundleName</key><string>点我自动安装</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>CFBundleShortVersionString</key><string>1.0</string>
    <key>CFBundleVersion</key><string>1</string>
</dict>
</plist>
PLIST_EOF

# Create README
cat > "$MOUNT_POINT/README.txt" << 'README_EOF'
双击 "点我自动安装.app" 即可自动安装
或手动拖拽到 Applications 文件夹
README_EOF

# Unmount
echo "Unmounting..."
hdiutil detach "$MOUNT_POINT" -force 2>/dev/null || true
rm -rf "$MOUNT_POINT"

# Convert back to compressed
echo "Creating final DMG..."
NEW_DMG="${DMG_PATH%.dmg}_new.dmg"
hdiutil convert "$RW_DMG" -format UDZO -o "$NEW_DMG" 2>/dev/null
rm -f "$RW_DMG"

# Replace original
mv "$NEW_DMG" "$DMG_PATH"

echo "Done! DMG is ready: $DMG_PATH"