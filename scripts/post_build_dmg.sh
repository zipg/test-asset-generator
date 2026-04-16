#!/bin/bash
# Post-build DMG customization script
# Usage: ./post_build_dmg.sh <path_to_dmg>
#
# This script adds fix_permissions.command to the DMG

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

echo "Adding fix_permissions.command to DMG: $DMG_PATH"

# Create temp directory
WORK_DIR=$(mktemp -d)
MOUNT_POINT="$WORK_DIR/volume"

# Create the fix_permissions.command script
cat > "$WORK_DIR/fix_permissions.command" << 'EOF'
#!/bin/bash
set -e

APP_NAME="Muse_Generator.app"
VOLUME_PATH="$(dirname "$(dirname "$0")")"
APP_PATH="$VOLUME_PATH/$APP_NAME"

if [ ! -d "$APP_PATH" ]; then
    echo "Error: $APP_NAME not found in the DMG"
    exit 1
fi

echo "Fixing permissions for $APP_NAME..."
xattr -cr "$APP_PATH"
echo "Done! You can now run the app."
echo ""
read -p "Press Enter to close this window..."
EOF

chmod +x "$WORK_DIR/fix_permissions.command"

# Mount DMG
echo "Mounting DMG..."
hdiutil attach "$DMG_PATH" -mountpoint "$MOUNT_POINT" -nobrowse 2>/dev/null

# Copy the script to DMG volume
cp "$WORK_DIR/fix_permissions.command" "$MOUNT_POINT/"

# Unmount
echo "Unmounting DMG..."
hdiutil detach "$MOUNT_POINT" -force 2>/dev/null || true

# Clean up
rm -rf "$WORK_DIR"

echo "Done! fix_permissions.command has been added to the DMG."
echo ""
echo "Note: Users can double-click fix_permissions.command after copying the app"
echo "to fix the 'cannot be opened' error on macOS."
