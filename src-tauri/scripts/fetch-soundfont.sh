#!/bin/bash
# 获取 SoundFont 文件用于打包
# 在发布前执行此脚本，会在 resources 目录生成 default.sf3

RESOURCES_DIR="$(dirname "$0")/../resources"
SF_PATH="$RESOURCES_DIR/default.sf3"

# 如果已存在且大于 1MB，跳过
if [ -f "$SF_PATH" ] && [ $(stat -f%z "$SF_PATH" 2>/dev/null || stat -c%s "$SF_PATH" 2>/dev/null) -gt 1000000 ]; then
    echo "SoundFont already exists: $SF_PATH"
    exit 0
fi

echo "Downloading SoundFont (MuseScore_General.sf3, ~30MB)..."

# MuseScore General SoundFont
URL="https://ftp.osuosl.org/pub/musescore/soundfont/MuseScore_General/MuseScore_General.sf3"

if command -v curl &> /dev/null; then
    curl -L -o "$SF_PATH" "$URL"
elif command -v wget &> /dev/null; then
    wget -O "$SF_PATH" "$URL"
else
    echo "Error: curl or wget is required"
    exit 1
fi

if [ $? -eq 0 ] && [ -f "$SF_PATH" ]; then
    SIZE=$(stat -f%z "$SF_PATH" 2>/dev/null || stat -c%s "$SF_PATH" 2>/dev/null)
    echo "Downloaded SoundFont: $SF_PATH ($SIZE bytes)"
else
    echo "Failed to download SoundFont"
    exit 1
fi
