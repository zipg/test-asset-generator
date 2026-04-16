Windows 离线安装包需要把 ffmpeg.exe 打进安装程序，以免用户直连 GitHub 下载失败。

1. 在发布 Windows 安装包前，在仓库根目录执行：
   bash scripts/fetch-ffmpeg-windows.sh
   会在本目录生成 ffmpeg.exe（约 100MB，已 gitignore）。

2. 使用合并配置进行打包（见根目录 package.json 脚本 tauri:build:win）：
   npm run tauri:build:win

也可设置环境变量 MUSE_FFMPEG_WINDOWS_ZIP_URL 指定 zip 下载地址（需与 BtbN win64-gpl zip 结构一致）。
