**界面预览**
<img width="2560" height="1594" alt="image" src="https://github.com/user-attachments/assets/c83cd418-dcee-4d2d-8cc8-ce56be691396" />


**技术架构**
  跨平台桌面框架: Tauri 2.0
  前端: React 18 + TypeScript + Vite
  后端: Rust
  音视频处理: FFmpeg (libavfilter lavfi)
  构建目标: macOS (.dmg) + Windows (exe) 

  Rust 后端 (src-tauri/src/)                                                                                                                         
- main.rs — Tauri 应用入口，命令注册（生成图片/音频/视频、配置管理、FFmpeg 下载检测等）
- generator.rs — 核心生成逻辑，基于 FFmpeg filter 构建图片（solid/gradient/pattern/noise）、音频（anoisesa）、视频（color/testsrc2/cellauto）      
- ffmpeg.rs — FFmpeg 路径解析（四级回退：绑定路径 → Homebrew → 应用数据目录 → PATH）和进程执行（含超时控制）                                 
- config.rs — 配置持久化（JSON 存储于 AppData/Muse_Generator/config.json），含 schema 版本迁移
  React 前端 (src/)
- App.tsx — 根组件，管理 Tab 切换、生成状态、进度显示
- components/VideoTab.tsx 等 — 各媒体类型的配置面板（格式、分辨率、帧率、时长、数量等）
- hooks/useGenerator.ts — 封装所有 Tauri IPC 调用（invoke）和状态管理
- types/index.ts — TypeScript 类型定义
 
 **数据流**
1. 用户在前端选择媒体类型、配置参数和保存路径
2. 前端调用 invoke("generate_images/audio/videos") 将配置传给 Rust
3. Rust 调用 ffmpeg::run_ffmpeg() 执行 FFmpeg 命令行
4. 生成过程中通过 tauri::Emitter 实时推送进度事件到前端         
5. 完成后返回 TaskResult（成功/失败数量及错误列表）
  **依赖要点**
- rand / md5 / hex — 随机文件名和内容种子                                                                                                          
- dirs — 跨平台获取 AppData/LocalAppData 路径     
**功能特性**                                                                                                                                           
  - 图片生成：支持 PNG/JPG/WEBP，可配置分辨率、内容类型（纯色/渐变/测试图案/噪声）、生成数量及文件名前缀
  - 音频生成：支持 MP3/WAV/AAC，可配置时长、采样率（44100/48000）、声道（单声道/立体声），可生成随机音乐/音符/噪音
  - 视频生成：支持 MP4/MOV/WEBM，可配置编码器（h264/hevc）、分辨率、帧率、时长、内容类型
  - FFmpeg 自动检测与下载：Windows端已内置FFmpeg；macOS 优先使用 Homebrew 安装的ffmpeg
  - 生成进度跟踪：实时显示当前文件、预计剩余时间，支持中途取消
