# Test Asset Generator Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a cross-platform Tauri + React desktop app that generates test image/video/audio files via bundled FFmpeg binaries, with GUI controls for format, resolution, duration, count, content type, and file naming.

**Architecture:** React TypeScript frontend communicates with Rust Tauri backend via IPC commands. Rust backend manages config persistence, invokes FFmpeg binaries bundled in the app, and emits progress events back to the frontend. FFmpeg binaries are stored under `ffmpeg/{os}/` and selected at runtime based on `std::env::consts::OS`.

**Tech Stack:** Tauri 2.x, React 18, TypeScript, Vite, Rust (tokio, serde, rand, serde_json), FFmpeg (static binaries)

---

## File Structure

```
test-asset-generator/                  # project root
├── src/                               # React frontend
│   ├── types/index.ts                 # shared TypeScript types
│   ├── components/
│   │   ├── Header.tsx                 # title + path display/selector
│   │   ├── TabBar.tsx                 # 图片/音频/视频 tab switcher
│   │   ├── ImageTab.tsx               # image config form
│   │   ├── AudioTab.tsx               # audio config form
│   │   ├── VideoTab.tsx               # video config form
│   │   ├── ProgressPanel.tsx          # progress bar, ETA, current file, cancel
│   │   └── ResultSummary.tsx           # success/fail count + error details
│   ├── hooks/
│   │   └── useGenerator.ts            # Tauri invoke wrapper + state
│   ├── App.tsx                        # main layout
│   └── main.tsx
├── src-tauri/                         # Rust backend
│   ├── src/
│   │   ├── main.rs                    # entry, app setup, event listener
│   │   ├── commands.rs                # Tauri command handlers
│   │   ├── ffmpeg.rs                  # FFmpeg binary resolution + invocation
│   │   ├── generator.rs               # per-type generation logic
│   │   └── config.rs                  # config load/save
│   ├── Cargo.toml
│   └── tauri.conf.json
├── ffmpeg/
│   ├── mac/ffmpeg
│   └── win/ffmpeg.exe
└── package.json
```

---

## Task 1: Scaffold Tauri + React Project

**Files:**
- Create: `package.json`, `vite.config.ts`, `tsconfig.json`, `index.html`
- Create: `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `src-tauri/src/main.rs`
- Create: `src/main.tsx`, `src/App.tsx`, `src/types/index.ts`

- [ ] **Step 1: Create project directory structure and package.json**

```bash
mkdir -p test-asset-generator/src/components test-asset-generator/src/hooks test-asset-generator/src/types
mkdir -p test-asset-generator/src-tauri/src
mkdir -p test-asset-generator/ffmpeg/mac test-asset-generator/ffmpeg/win
```

```json
// package.json
{
  "name": "test-asset-generator",
  "version": "1.0.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "tauri": "tauri"
  },
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tauri-apps/plugin-dialog": "^2.0.0",
    "react": "^18.3.1",
    "react-dom": "^18.3.1"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0",
    "@types/react": "^18.3.1",
    "@types/react-dom": "^18.3.1",
    "@vitejs/plugin-react": "^4.3.1",
    "typescript": "^5.5.0",
    "vite": "^5.4.0"
  }
}
```

```json
// vite.config.ts
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
});
```

```json
// tsconfig.json
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "isolatedModules": true,
    "moduleDetection": "force",
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true
  },
  "include": ["src"]
}
```

```html
<!-- index.html -->
<!doctype html>
<html lang="zh-CN">
  <head>
    <meta charset="UTF-8" />
    <title>Test Asset Generator</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
```

- [ ] **Step 2: Create Tauri Cargo.toml**

```toml
[package]
name = "test-asset-generator"
version = "1.0.0"
edition = "2021"

[build-dependencies]
tauri-build = { version = "2.0", features = [] }

[dependencies]
tauri = { version = "2.0", features = [] }
tauri-plugin-dialog = "2.0"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
rand = "0.8"
tokio = { version = "1.0", features = ["process", "rt-multi-thread"] }
dirs = "5.0"

[profile.release]
panic = "abort"
codegen-units = 1
lto = true
opt-level = "s"
strip = true
```

- [ ] **Step 3: Create Tauri config**

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "Test Asset Generator",
  "version": "1.0.0",
  "identifier": "com.test.asset-generator",
  "build": {
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:1420",
    "beforeBuildCommand": "npm run build",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "Test Asset Generator",
        "width": 560,
        "height": 700,
        "resizable": true,
        "center": true
      }
    ]
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ]
  },
  "plugins": {
    "dialog": {}
  }
}
```

```rust
// src-tauri/build.rs
fn main() {
    tauri_build::build();
}
```

- [ ] **Step 4: Create Tauri main.rs with minimal setup**

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

static CANCEL_FLAG: AtomicBool = AtomicBool::new(false);

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|_app| {
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_cancelled,
            reset_cancelled,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn get_cancelled() -> bool {
    CANCEL_FLAG.load(Ordering::SeqCst)
}

#[tauri::command]
fn reset_cancelled() {
    CANCEL_FLAG.store(false, Ordering::SeqCst);
}
```

- [ ] **Step 5: Create React entry files**

```typescript
// src/types/index.ts
export type ImageFormat = "PNG" | "JPG" | "WEBP";
export type AudioFormat = "MP3" | "WAV" | "AAC";
export type VideoFormat = "MP4" | "MOV" | "WEBM";
export type ContentType = "solid" | "gradient" | "pattern" | "noise";
export type Codec = "h264" | "hevc";
export type SampleRate = 44100 | 48000;
export type Channels = "mono" | "stereo";
export type MediaType = "image" | "audio" | "video";

export interface ImageConfig {
  format: ImageFormat;
  width: number;
  height: number;
  contentType: ContentType;
  count: number;
  prefix: string;
}

export interface AudioConfig {
  format: AudioFormat;
  duration: number;
  sampleRate: SampleRate;
  channels: Channels;
  count: number;
  prefix: string;
}

export interface VideoConfig {
  format: VideoFormat;
  codec: Codec;
  width: number;
  height: number;
  fps: number;
  duration: number;
  contentType: ContentType;
  count: number;
  prefix: string;
}

export interface AppConfig {
  savePath: string;
  imageConfig: ImageConfig;
  audioConfig: AudioConfig;
  videoConfig: VideoConfig;
}

export interface ProgressPayload {
  current: number;
  total: number;
  currentFile: string;
  estimatedRemainingSecs: number;
}

export interface TaskResult {
  success: number;
  failed: number;
  errors: Array<{ file: string; error: string }>;
}
```

```tsx
// src/main.tsx
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import "./styles.css";

ReactDOM.createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);
```

```css
/* src/styles.css */
* { box-sizing: border-box; margin: 0; padding: 0; }
body { font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #f5f5f7; color: #1d1d1f; font-size: 14px; }
#root { display: flex; align-items: center; justify-content: center; min-height: 100vh; }
```

```tsx
// src/App.tsx - shell that renders tab + config panels (filled in Task 8)
import { useState } from "react";
import TabBar from "./components/TabBar";
import ImageTab from "./components/ImageTab";
import AudioTab from "./components/AudioTab";
import VideoTab from "./components/VideoTab";
import ProgressPanel from "./components/ProgressPanel";
import ResultSummary from "./components/ResultSummary";
import Header from "./components/Header";
import { MediaType } from "./types";

export default function App() {
  const [activeTab, setActiveTab] = useState<MediaType>("image");
  const [generating, setGenerating] = useState(false);
  const [progress, setProgress] = useState<{
    current: number;
    total: number;
    currentFile: string;
    estimatedRemainingSecs: number;
  } | null>(null);
  const [result, setResult] = useState<{
    success: number;
    failed: number;
    errors: Array<{ file: string; error: string }>;
  } | null>(null);

  return (
    <div className="app-container">
      <Header />
      <TabBar active={activeTab} onChange={setActiveTab} />
      <div className="tab-content">
        {activeTab === "image" && <ImageTab onStart={() => setGenerating(true)} />}
        {activeTab === "audio" && <AudioTab onStart={() => setGenerating(true)} />}
        {activeTab === "video" && <VideoTab onStart={() => setGenerating(true)} />}
      </div>
      {generating && progress && <ProgressPanel {...progress} />}
      {result && <ResultSummary {...result} />}
    </div>
  );
}
```

- [ ] **Step 6: Commit**

```bash
cd test-asset-generator && git init && git add -A && git commit -m "chore: scaffold Tauri + React project

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>"
```

---

## Task 2: Download FFmpeg Binaries

**Files:**
- Create: `ffmpeg/mac/ffmpeg` (downloaded)
- Create: `ffmpeg/win/ffmpeg.exe` (downloaded)

- [ ] **Step 1: Check if ffmpeg is available locally for packaging**

```bash
which ffmpeg && ffmpeg -version | head -1
```

- [ ] **Step 2: Download macOS static FFmpeg (if not present)**

If `which ffmpeg` returns empty, download:
```bash
mkdir -p test-asset-generator/ffmpeg/mac
curl -L "https://evermeet.cx/ffmpeg/getrelease/ffmpeg/zip" -o /tmp/ffmpeg-mac.zip
unzip -o /tmp/ffmpeg-mac.zip -d test-asset-generator/ffmpeg/mac/
chmod +x test-asset-generator/ffmpeg/mac/ffmpeg
rm /tmp/ffmpeg-mac.zip
```

Verify: `test-asset-generator/ffmpeg/mac/ffmpeg -version | head -1`

- [ ] **Step 3: Download Windows FFmpeg**

Download from `https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip`, extract `ffmpeg.exe` to `test-asset-generator/ffmpeg/win/`.

If on macOS and can't download Windows binary, create a `.gitkeep` placeholder and document that the Windows binary needs to be downloaded on a Windows machine or via CI.

```bash
mkdir -p test-asset-generator/ffmpeg/win
# On a Windows machine or CI, download:
# curl -L "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip" -o /tmp/ffmpeg-win.zip
# unzip -o /tmp/ffmpeg-win.zip "ffmpeg-master-latest-win64-gpl/bin/ffmpeg.exe" -d test-asset-generator/ffmpeg/win/
touch test-asset-generator/ffmpeg/win/.gitkeep  # placeholder until CI downloads real binary
```

- [ ] **Step 4: Verify macOS binary works**

```bash
test-asset-generator/ffmpeg/mac/ffmpeg -version | head -1
# Expected: ffmpeg version X.X.X ...
```

- [ ] **Step 5: Commit**

```bash
git add ffmpeg/ && git commit -m "chore: add FFmpeg binaries

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: Rust Backend - Config Management

**Files:**
- Create: `src-tauri/src/config.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Create config.rs with default values and load/save**

```rust
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageConfig {
    pub format: String,
    pub width: u32,
    pub height: u32,
    pub content_type: String,
    pub count: u32,
    pub prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioConfig {
    pub format: String,
    pub duration: f64,
    pub sample_rate: u32,
    pub channels: String,
    pub count: u32,
    pub prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoConfig {
    pub format: String,
    pub codec: String,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub duration: f64,
    pub content_type: String,
    pub count: u32,
    pub prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub save_path: Option<String>,
    pub image_config: ImageConfig,
    pub audio_config: AudioConfig,
    pub video_config: VideoConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            save_path: None,
            image_config: ImageConfig {
                format: "PNG".to_string(),
                width: 1080,
                height: 1920,
                content_type: "noise".to_string(),
                count: 10,
                prefix: "测试图片".to_string(),
            },
            audio_config: AudioConfig {
                format: "MP3".to_string(),
                duration: 60.0,
                sample_rate: 44100,
                channels: "mono".to_string(),
                count: 10,
                prefix: "测试音频".to_string(),
            },
            video_config: VideoConfig {
                format: "MP4".to_string(),
                codec: "hevc".to_string(),
                width: 1080,
                height: 1920,
                fps: 30,
                duration: 60.0,
                content_type: "noise".to_string(),
                count: 10,
                prefix: "测试视频".to_string(),
            },
        }
    }
}

fn config_path(app_handle: &tauri::AppHandle) -> PathBuf {
    let dir = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir");
    fs::create_dir_all(&dir).expect("Failed to create app data dir");
    dir.join("config.json")
}

pub fn load_config(app_handle: &tauri::AppHandle) -> AppConfig {
    let path = config_path(app_handle);
    if path.exists() {
        let contents = fs::read_to_string(&path).expect("Failed to read config");
        serde_json::from_str(&contents).unwrap_or_default()
    } else {
        AppConfig::default()
    }
}

pub fn save_config(app_handle: &tauri::AppHandle, config: &AppConfig) {
    let path = config_path(app_handle);
    let contents = serde_json::to_string_pretty(config).expect("Failed to serialize config");
    fs::write(&path, contents).expect("Failed to write config");
}
```

- [ ] **Step 2: Update main.rs to export config commands**

Add to main.rs:
```rust
mod config;

#[tauri::command]
fn get_config(app: tauri::AppHandle) -> config::AppConfig {
    config::load_config(&app)
}

#[tauri::command]
fn save_config_cmd(app: tauri::AppHandle, cfg: config::AppConfig) {
    config::save_config(&app, &cfg);
}
```

Update `tauri::generate_handler!` to include `get_config, save_config_cmd`.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/config.rs src-tauri/src/main.rs && git commit -m "feat(backend): add config load/save with defaults

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>"
```

---

## Task 4: Rust Backend - FFmpeg Invocation

**Files:**
- Create: `src-tauri/src/ffmpeg.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Create ffmpeg.rs with binary resolution**

```rust
use std::path::PathBuf;
use std::process::Command;

pub fn get_ffmpeg_path() -> PathBuf {
    let os = std::env::consts::OS;
    let exe_name = if os == "windows" { "ffmpeg.exe" } else { "ffmpeg" };

    // Try bundled path first (relative to executable)
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()));

    if let Some(dir) = exe_dir {
        let bundled = dir.join("ffmpeg").join(os).join(exe_name);
        if bundled.exists() {
            return bundled;
        }
    }

    // Fallback: development path (project root/ffmpeg/{os}/ffmpeg)
    let dev_path = std::env::current_dir()
        .ok()
        .map(|p| p.join("ffmpeg").join(os).join(exe_name));

    if let Some(p) = dev_path {
        if p.exists() {
            return p;
        }
    }

    // Last resort: system ffmpeg
    if os == "windows" {
        PathBuf::from("ffmpeg.exe")
    } else {
        PathBuf::from("ffmpeg")
    }
}

pub fn run_ffmpeg(args: &[&str]) -> Result<String, String> {
    let ffmpeg_path = get_ffmpeg_path();
    let output = Command::new(&ffmpeg_path)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute ffmpeg: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stderr).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

pub fn run_ffmpeg_with_cancel<F>(
    args: &[&str],
    check_cancel: F,
) -> Result<String, String>
where
    F: Fn() -> bool,
{
    let ffmpeg_path = get_ffmpeg_path();
    let mut child = std::process::Command::new(&ffmpeg_path)
        .args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn ffmpeg: {}", e))?;

    // Wait for child, checking cancellation periodically
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let stderr = child
                    .stderr
                    .take()
                    .map(|mut s| {
                        use std::io::Read;
                        let mut buf = String::new();
                        s.read_to_string(&mut buf).ok();
                        buf
                    })
                    .unwrap_or_default();

                if status.success() {
                    return Ok(stderr);
                } else {
                    return Err(stderr);
                }
            }
            Ok(None) => {
                if check_cancel() {
                    child.kill().ok();
                    return Err("Cancelled by user".to_string());
                }
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
            Err(e) => return Err(format!("Error waiting for ffmpeg: {}", e)),
        }
    }
}
```

- [ ] **Step 2: Add module to main.rs**

```rust
mod ffmpeg;
```

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/ffmpeg.rs src-tauri/src/main.rs && git commit -m "feat(backend): FFmpeg binary resolution and invocation

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>"
```

---

## Task 5: Rust Backend - Generator Logic

**Files:**
- Create: `src-tauri/src/generator.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Create generator.rs with all three generation functions**

```rust
use crate::config::{AudioConfig, ImageConfig, VideoConfig};
use crate::ffmpeg;
use rand::Rng;
use std::path::Path;
use tauri::{AppHandle, Emitter};

static CANCEL_FLAG: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

pub fn reset_cancel() {
    CANCEL_FLAG.store(false, std::sync::atomic::Ordering::SeqCst);
}

pub fn get_cancel() -> bool {
    CANCEL_FLAG.load(std::sync::atomic::Ordering::SeqCst)
}

pub fn set_cancel(val: bool) {
    CANCEL_FLAG.store(val, std::sync::atomic::Ordering::SeqCst);
}

fn random_hex(len: usize) -> String {
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyz0123456789".chars().collect();
    (0..len)
        .map(|_| chars[rng.gen_range(0..chars.len())])
        .collect()
}

fn format_duration(secs: f64) -> String {
    if secs == secs.floor() {
        format!("{:.0}", secs)
    } else {
        format!("{:.2}", secs)
    }
}

fn build_image_filter(content_type: &str, width: u32, height: u32) -> String {
    let seed: u32 = rand::thread_rng().gen();
    match content_type {
        "solid" => {
            let hue: f32 = rand::thread_rng().gen_range(0.0..360.0);
            format!(
                "color=c=hsl({},100%,50%:s={}x{}:d=1[v]"
                    .replace("{}", &format!("{},1.0,0.5", hue))
                    .replace("color=c=hsl({},1.0,0.5", &format!("color=c=hsl({},1.0,0.5", hue))
                    .replace("{}/{}:{}/{}", &format!("{}/{}/{}/{}", hue, hue, hue, hue))
                    ,
                width, height
            )
        }
        "gradient" => format!(
            "gradients=s={}x{}:c0=random:c1=random:seed={}[v]",
            width, height, seed
        ),
        "pattern" => format!(
            "testsrc2=size={}x{}:rate=1[v]",
            width, height
        ),
        _ /* noise */ => format!(
            "cellauto=rule=18:seed={}:size={}x{}:pattern=random,scale={}:{}:flags=neighbor[v]",
            seed, width, height, width, height
        ),
    }
}

pub fn generate_image(config: &ImageConfig, output_dir: &Path, app: &AppHandle) -> Result<(), String> {
    let filter = build_image_filter(&config.content_type, config.width, config.height);
    let ext = match config.format.as_str() {
        "JPG" => "jpg",
        "WEBP" => "webp",
        _ => "png",
    };

    for i in 1..=config.count {
        if get_cancel() {
            return Err("Cancelled".to_string());
        }

        let random_str = random_hex(6);
        let filename = format!("{}_{}_{:03}.{}", config.prefix, random_str, i, ext);
        let output_path = output_dir.join(&filename);

        let mut args = vec![
            "-f", "lavfi",
            "-i", &filter,
            "-vframes", "1",
            "-y",
        ];

        match ext {
            "jpg" => args.extend_from_slice(&["-q:v", "2"]),
            "webp" => args.extend_from_slice(&["-quality", "90"]),
            _ => {}
        };

        args.push(output_path.to_str().unwrap());

        let stderr = ffmpeg::run_ffmpeg(&args).map_err(|e| {
            format!("{}: {}", filename, e)
        })?;

        // Emit progress
        let _ = app.emit("generation-progress", serde_json::json!({
            "current": i,
            "total": config.count,
            "currentFile": filename,
            "estimatedRemainingSecs": 0,
        }));
    }

    Ok(())
}

pub fn generate_audio(config: &AudioConfig, output_dir: &Path, app: &AppHandle) -> Result<(), String> {
    let channels = if config.channels == "stereo" { "2" } else { "1" };
    let ext = match config.format.as_str() {
        "WAV" => "wav",
        "AAC" => "aac",
        _ => "mp3",
    };
    let duration = format_duration(config.duration);

    for i in 1..=config.count {
        if get_cancel() {
            return Err("Cancelled".to_string());
        }

        let random_str = random_hex(6);
        let filename = format!("{}_{}_{:03}.{}", config.prefix, random_str, i, ext);
        let output_path = output_dir.join(&filename);

        let seed: u32 = rand::thread_rng().gen();
        let amplitude: f32 = rand::thread_rng().gen_range(0.1..0.5);

        let args = if ext == "wav" {
            vec![
                "-f", "lavfi",
                "-i", &format!("anoisesa=d={}:a={}:r={}:c={}", duration, amplitude, config.sample_rate, channels),
                "-y",
                output_path.to_str().unwrap(),
            ]
        } else {
            let codec = if ext == "aac" { "aac" } else { "libmp3lame" };
            vec![
                "-f", "lavfi",
                "-i", &format!("anoisesa=d={}:a={}:r={}:c={}", duration, amplitude, config.sample_rate, channels),
                "-acodec", codec,
                "-y",
                output_path.to_str().unwrap(),
            ]
        };

        let _ = ffmpeg::run_ffmpeg(&args).map_err(|e| {
            format!("{}: {}", filename, e)
        })?;

        let _ = app.emit("generation-progress", serde_json::json!({
            "current": i,
            "total": config.count,
            "currentFile": filename,
            "estimatedRemainingSecs": 0,
        }));
    }

    Ok(())
}

pub fn generate_video(config: &VideoConfig, output_dir: &Path, app: &AppHandle) -> Result<(), String> {
    let ext = match config.format.as_str() {
        "MOV" => "mov",
        "WEBM" => "webm",
        _ => "mp4",
    };
    let duration = format_duration(config.duration);
    let codec = if config.codec == "h264" { "libx264" } else { "libx265" };

    let seed: u32 = rand::thread_rng().gen();
    let filter = match config.content_type.as_str() {
        "solid" => {
            let hue: f32 = rand::thread_rng().gen_range(0.0..360.0);
            format!("color=c=hsl\\({:.1},1.0,0.5\\):s={}x{}:d={}[v]", hue, config.width, config.height, duration)
        }
        "gradient" => format!("gradients=s={}x{}:c0=random:c1=random:seed={}:d={}[v]", config.width, config.height, seed, duration),
        "pattern" => format!("testsrc2=size={}x{}:rate={}[v]", config.width, config.height, config.fps),
        _ /* noise */ => format!(
            "cellauto=rule=18:seed={}:size={}x{}:pattern=random,scale={}:{}:flags=neighbor[v]",
            seed, config.width, config.height, config.width, config.height
        ),
    };

    for i in 1..=config.count {
        if get_cancel() {
            return Err("Cancelled".to_string());
        }

        let random_str = random_hex(6);
        let filename = format!("{}_{}_{:03}.{}", config.prefix, random_str, i, ext);
        let output_path = output_dir.join(&filename);

        let mut args = vec![
            "-f", "lavfi",
            "-i", &filter,
            "-c:v", codec,
            "-r", &config.fps.to_string(),
            "-t", &duration,
            "-pix_fmt", "yuv420p",
            "-y",
            output_path.to_str().unwrap(),
        ];

        let _ = ffmpeg::run_ffmpeg(&args).map_err(|e| {
            format!("{}: {}", filename, e)
        })?;

        let _ = app.emit("generation-progress", serde_json::json!({
            "current": i,
            "total": config.count,
            "currentFile": filename,
            "estimatedRemainingSecs": 0,
        }));
    }

    Ok(())
}
```

- [ ] **Step 2: Create the main Tauri command handlers in main.rs**

Replace main.rs with:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod ffmpeg;
mod generator;

use config::AppConfig;
use generator::{get_cancel, reset_cancel, set_cancel};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|_app| Ok(()))
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            get_cancelled,
            reset_cancelled,
            set_cancelled,
            generate_images,
            generate_audio,
            generate_videos,
            select_save_path,
            estimate_size,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn get_config(app: AppHandle) -> AppConfig {
    config::load_config(&app)
}

#[tauri::command]
fn save_config(app: AppHandle, cfg: AppConfig) {
    config::save_config(&app, &cfg);
}

#[tauri::command]
fn get_cancelled() -> bool {
    get_cancel()
}

#[tauri::command]
fn reset_cancelled() {
    reset_cancel();
}

#[tauri::command]
fn set_cancelled(val: bool) {
    set_cancel(val);
}

#[tauri::command]
async fn select_save_path(app: AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let result = app.dialog()
        .file()
        .blocking_pick_folder();

    match result {
        Some(path) => Ok(Some(path.to_string())),
        None => Ok(None),
    }
}

#[tauri::command]
fn estimate_size(media_type: String, cfg: serde_json::Value) -> String {
    // Rough estimation in MB
    match media_type.as_str() {
        "image" => {
            let w: u64 = cfg["width"].as_u64().unwrap_or(1080);
            let h: u64 = cfg["height"].as_u64().unwrap_or(1920);
            let count: u64 = cfg["count"].as_u64().unwrap_or(1);
            let format = cfg["format"].as_str().unwrap_or("PNG");
            let bytes_per_pixel = match format {
                "JPG" => 0.5,
                "WEBP" => 0.3,
                _ => 3.0,
            };
            let size = (w * h) as f64 * bytes_per_pixel * count / 1_048_576.0;
            format!("~{:.1f} MB", size.max(0.01))
        }
        "audio" => {
            let duration: f64 = cfg["duration"].as_f64().unwrap_or(60.0);
            let count: u64 = cfg["count"].as_u64().unwrap_or(1);
            let format = cfg["format"].as_str().unwrap_or("MP3");
            let kbps = match format {
                "WAV" => 1411.0,
                "AAC" => 128.0,
                _ => 128.0,
            };
            let size = (duration * kbps * 1000.0 / 8.0) * (count as f64) / 1_048_576.0;
            format!("~{:.1f} MB", size.max(0.01))
        }
        "video" => {
            let w: u64 = cfg["width"].as_u64().unwrap_or(1080);
            let h: u64 = cfg["height"].as_u64().unwrap_or(1920);
            let duration: f64 = cfg["duration"].as_f64().unwrap_or(60.0);
            let fps: u64 = cfg["fps"].as_u64().unwrap_or(30);
            let count: u64 = cfg["count"].as_u64().unwrap_or(1);
            let kbps = (w * h * fps) as f64 * 0.1 / 1000.0;
            let size = (duration * kbps * 1000.0 / 8.0) * (count as f64) / 1_048_576.0;
            format!("~{:.1f} MB", size.max(0.01))
        }
        _ => "~0 MB".to_string(),
    }
}

#[tauri::command]
async fn generate_images(
    app: AppHandle,
    config: config::ImageConfig,
    save_path: String,
) -> Result<serde_json::Value, String> {
    reset_cancel();
    let output_dir = create_timestamp_dir(&save_path)?;

    let total = config.count;
    let mut success = 0u32;
    let mut failed = 0u32;
    let mut errors: Vec<serde_json::Value> = Vec::new();

    for i in 1..=total {
        if get_cancel() {
            break;
        }

        let random_str = generator::random_hex(6);
        let filename = format!("{}_{}_{:03}.png", config.prefix, random_str, i);
        let output_path = output_dir.join(&filename);

        let filter = build_image_filter(&config.content_type, config.width, config.height);
        let ext = match config.format.as_str() {
            "JPG" => "jpg",
            "WEBP" => "webp",
            _ => "png",
        };
        let out_filename = format!("{}_{}_{:03}.{}", config.prefix, random_str, i, ext);
        let out_path = output_dir.join(&out_filename);

        let mut args = vec!["-f", "lavfi", "-i", &filter, "-vframes", "1", "-y"];
        match ext {
            "jpg" => args.extend_from_slice(&["-q:v", "2"]),
            "webp" => args.extend_from_slice(&["-quality", "90"]),
            _ => {}
        };
        args.push(out_path.to_str().unwrap());

        match ffmpeg::run_ffmpeg(&args) {
            Ok(_) => success += 1,
            Err(e) => {
                failed += 1;
                errors.push(serde_json::json!({ "file": out_filename, "error": e }));
            }
        }

        let elapsed = i as f64;
        let eta = if success + failed > 0 {
            ((total - i) as f64 / (success + failed) as f64 * elapsed).max(0.0) as u32
        } else {
            0u32
        };

        let _ = app.emit("generation-progress", serde_json::json!({
            "current": i,
            "total": total,
            "currentFile": out_filename,
            "estimatedRemainingSecs": eta,
        }));
    }

    Ok(serde_json::json!({
        "success": success,
        "failed": failed,
        "errors": errors,
    }))
}

#[tauri::command]
async fn generate_audio(
    app: AppHandle,
    config: config::AudioConfig,
    save_path: String,
) -> Result<serde_json::Value, String> {
    reset_cancel();
    let output_dir = create_timestamp_dir(&save_path)?;

    let total = config.count;
    let mut success = 0u32;
    let mut failed = 0u32;
    let mut errors: Vec<serde_json::Value> = Vec::new();

    let channels = if config.channels == "stereo" { "2" } else { "1" };
    let ext = match config.format.as_str() {
        "WAV" => "wav",
        "AAC" => "aac",
        _ => "mp3",
    };
    let duration_str = if config.duration == config.duration.floor() {
        format!("{:.0}", config.duration)
    } else {
        format!("{:.2}", config.duration)
    };

    for i in 1..=total {
        if get_cancel() {
            break;
        }

        let random_str = generator::random_hex(6);
        let filename = format!("{}_{}_{:03}.{}", config.prefix, random_str, i, ext);
        let output_path = output_dir.join(&filename);

        let amplitude: f32 = rand::thread_rng().gen_range(0.1..0.5);
        let seed: u32 = rand::thread_rng().gen();

        let mut args: Vec<&str> = vec![
            "-f", "lavfi",
            "-i",
            &format!(
                "anoisesa=d={}:a={}:r={}:c={}:s={}",
                duration_str, amplitude, config.sample_rate, channels, seed
            ),
            "-y",
        ];

        if ext != "wav" {
            let codec = if ext == "aac" { "aac" } else { "libmp3lame" };
            args.extend_from_slice(&["-acodec", codec]);
        }

        args.push(output_path.to_str().unwrap());

        match ffmpeg::run_ffmpeg(&args) {
            Ok(_) => success += 1,
            Err(e) => {
                failed += 1;
                errors.push(serde_json::json!({ "file": filename, "error": e }));
            }
        }

        let elapsed = i as f64;
        let eta = if success + failed > 0 {
            ((total - i) as f64 / (success + failed) as f64 * elapsed).max(0.0) as u32
        } else {
            0u32
        };

        let _ = app.emit("generation-progress", serde_json::json!({
            "current": i,
            "total": total,
            "currentFile": filename,
            "estimatedRemainingSecs": eta,
        }));
    }

    Ok(serde_json::json!({
        "success": success,
        "failed": failed,
        "errors": errors,
    }))
}

#[tauri::command]
async fn generate_videos(
    app: AppHandle,
    config: config::VideoConfig,
    save_path: String,
) -> Result<serde_json::Value, String> {
    reset_cancel();
    let output_dir = create_timestamp_dir(&save_path)?;

    let total = config.count;
    let mut success = 0u32;
    let mut failed = 0u32;
    let mut errors: Vec<serde_json::Value> = Vec::new();

    let ext = match config.format.as_str() {
        "MOV" => "mov",
        "WEBM" => "webm",
        _ => "mp4",
    };
    let codec = if config.codec == "h264" { "libx264" } else { "libx265" };
    let duration_str = if config.duration == config.duration.floor() {
        format!("{:.0}", config.duration)
    } else {
        format!("{:.2}", config.duration)
    };

    for i in 1..=total {
        if get_cancel() {
            break;
        }

        let random_str = generator::random_hex(6);
        let filename = format!("{}_{}_{:03}.{}", config.prefix, random_str, i, ext);
        let output_path = output_dir.join(&filename);

        let seed: u32 = rand::thread_rng().gen();
        let hue: f32 = rand::thread_rng().gen_range(0.0..360.0);

        let filter = match config.content_type.as_str() {
            "solid" => format!(
                "color=c=hsl\\({:.1},1.0,0.5\\):s={}x{}:d={}[v]",
                hue, config.width, config.height, duration_str
            ),
            "gradient" => format!(
                "gradients=s={}x{}:c0=random:c1=random:seed={}:d={}[v]",
                config.width, config.height, seed, duration_str
            ),
            "pattern" => format!(
                "testsrc2=size={}x{}:rate={}[v]",
                config.width, config.height, config.fps
            ),
            _ => format!(
                "cellauto=rule=18:seed={}:size={}x{}:pattern=random,scale={}:{}:flags=neighbor[v]",
                seed, config.width, config.height, config.width, config.height
            ),
        };

        let args: Vec<&str> = vec![
            "-f", "lavfi",
            "-i", &filter,
            "-c:v", codec,
            "-r", &config.fps.to_string(),
            "-t", &duration_str,
            "-pix_fmt", "yuv420p",
            "-y",
            output_path.to_str().unwrap(),
        ];

        match ffmpeg::run_ffmpeg(&args) {
            Ok(_) => success += 1,
            Err(e) => {
                failed += 1;
                errors.push(serde_json::json!({ "file": filename, "error": e }));
            }
        }

        let elapsed = i as f64;
        let eta = if success + failed > 0 {
            ((total - i) as f64 / (success + failed) as f64 * elapsed).max(0.0) as u32
        } else {
            0u32
        };

        let _ = app.emit("generation-progress", serde_json::json!({
            "current": i,
            "total": total,
            "currentFile": filename,
            "estimatedRemainingSecs": eta,
        }));
    }

    Ok(serde_json::json!({
        "success": success,
        "failed": failed,
        "errors": errors,
    }))
}

fn create_timestamp_dir(base: &str) -> Result<PathBuf, String> {
    let now = chrono_lite_timestamp();
    let dir = PathBuf::from(base).join(&now);
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create directory: {}", e))?;
    Ok(dir)
}

fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap();
    let secs = dur.as_secs();
    // Format as YYYY-MM-DD-HH-mm-ss manually
    let days = secs / 86400;
    let remaining = secs % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;
    // Use an approximate year/month/day from days since epoch
    let year_days = days;
    let year = 1970 + year_days / 365;
    let yday = year_days % 365;
    let month = yday / 30 + 1;
    let day = yday % 30 + 1;
    format!("{:04}-{:02}-{:02}-{:02}-{:02}-{:02}", year, month, day, hours, minutes, seconds)
}

fn build_image_filter(content_type: &str, width: u32, height: u32) -> String {
    use rand::Rng;
    let seed: u32 = rand::thread_rng().gen();
    let hue: f32 = rand::thread_rng().gen_range(0.0..360.0);
    match content_type {
        "solid" => format!(
            "color=c=0x{:06x}:s={}x{}:d=1[v]",
            (hue / 360.0 * 16777215.0) as u32,
            width, height
        ),
        "gradient" => format!(
            "gradients=s={}x{}:c0=random:c1=random:seed={}[v]",
            width, height, seed
        ),
        "pattern" => format!("testsrc2=size={}x{}:rate=1[v]", width, height),
        _ => format!(
            "cellauto=rule=18:seed={}:size={}x{}:pattern=random,scale={}:{}:flags=neighbor[v]",
            seed, width, height, width, height
        ),
    }
}
```

- [ ] **Step 3: Add chrono-lite or use std::time for timestamp. Fix main.rs to use inline timestamp formatting (no external chrono crate). The above `chrono_lite_timestamp` function uses std::time only.**

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/generator.rs src-tauri/src/main.rs && git commit -m "feat(backend): generation logic for images, audio, and video

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>"
```

---

## Task 6: React Frontend - Core Hook

**Files:**
- Create: `src/hooks/useGenerator.ts`
- Modify: `src/types/index.ts`

- [ ] **Step 1: Create the useGenerator hook**

```typescript
import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import type { AppConfig, ProgressPayload, TaskResult } from "../types";

export function useGenerator() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    invoke<AppConfig>("get_config")
      .then(setConfig)
      .catch(console.error);
  }, []);

  const updateConfig = useCallback(
    (updated: AppConfig) => {
      setConfig(updated);
      invoke("save_config", { cfg: updated }).catch(console.error);
    },
    []
  );

  const estimateSize = useCallback(
    async (
      mediaType: "image" | "audio" | "video",
      cfg: Record<string, unknown>
    ): Promise<string> => {
      return invoke<string>("estimate_size", { mediaType, cfg });
    },
    []
  );

  const selectPath = useCallback(async (): Promise<string | null> => {
    return invoke<string | null>("select_save_path");
  }, []);

  const generateImages = useCallback(
    async (
      imageConfig: Record<string, unknown>,
      savePath: string
    ): Promise<TaskResult> => {
      setLoading(true);
      try {
        const result = await invoke<TaskResult>("generate_images", {
          config: imageConfig,
          savePath,
        });
        return result;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  const generateAudio = useCallback(
    async (
      audioConfig: Record<string, unknown>,
      savePath: string
    ): Promise<TaskResult> => {
      setLoading(true);
      try {
        const result = await invoke<TaskResult>("generate_audio", {
          config: audioConfig,
          savePath,
        });
        return result;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  const generateVideos = useCallback(
    async (
      videoConfig: Record<string, unknown>,
      savePath: string
    ): Promise<TaskResult> => {
      setLoading(true);
      try {
        const result = await invoke<TaskResult>("generate_videos", {
          config: videoConfig,
          savePath,
        });
        return result;
      } finally {
        setLoading(false);
      }
    },
    []
  );

  const cancelGeneration = useCallback(async () => {
    await invoke("set_cancelled", { val: true });
  }, []);

  return {
    config,
    updateConfig,
    loading,
    estimateSize,
    selectPath,
    generateImages,
    generateAudio,
    generateVideos,
    cancelGeneration,
  };
}
```

- [ ] **Step 2: Commit**

```bash
git add src/hooks/useGenerator.ts && git commit -m "feat(frontend): add useGenerator hook

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>"
```

---

## Task 7: React Frontend - UI Components

**Files:**
- Create: `src/components/Header.tsx`, `src/components/TabBar.tsx`, `src/components/ImageTab.tsx`, `src/components/AudioTab.tsx`, `src/components/VideoTab.tsx`, `src/components/ProgressPanel.tsx`, `src/components/ResultSummary.tsx`
- Modify: `src/App.tsx`

- [ ] **Step 1: Create Header.tsx**

```tsx
import { useCallback } from "react";
import { open } from "@tauri-apps/plugin-dialog";

interface Props {
  savePath: string | undefined;
  onPathChange: (path: string) => void;
}

export default function Header({ savePath, onPathChange }: Props) {
  const handleSelect = useCallback(async () => {
    const selected = await open({ directory: true, multiple: false });
    if (selected) {
      onPathChange(selected as string);
    }
  }, [onPathChange]);

  return (
    <header className="header">
      <h1>Test Asset Generator</h1>
      <div className="path-row">
        <span className="path-label">保存路径:</span>
        <span className="path-value" title={savePath}>
          {savePath || "未设置"}
        </span>
        <button className="btn-small" onClick={handleSelect}>
          选择
        </button>
      </div>
    </header>
  );
}
```

- [ ] **Step 2: Create TabBar.tsx**

```tsx
import type { MediaType } from "../types";

interface Props {
  active: MediaType;
  onChange: (tab: MediaType) => void;
}

export default function TabBar({ active, onChange }: Props) {
  const tabs: { key: MediaType; label: string }[] = [
    { key: "image", label: "图片" },
    { key: "audio", label: "音频" },
    { key: "video", label: "视频" },
  ];

  return (
    <div className="tab-bar">
      {tabs.map((tab) => (
        <button
          key={tab.key}
          className={`tab-btn${active === tab.key ? " active" : ""}`}
          onClick={() => onChange(tab.key)}
        >
          {tab.label}
        </button>
      ))}
    </div>
  );
}
```

- [ ] **Step 3: Create ImageTab.tsx**

```tsx
import { useState, useEffect, useCallback } from "react";
import type { ImageConfig, ImageFormat, ContentType } from "../types";

interface Props {
  config: ImageConfig;
  savePath: string | undefined;
  onConfigChange: (cfg: Partial<ImageConfig>) => void;
  onGenerate: () => void;
  onEstimate: (cfg: Record<string, unknown>) => Promise<string>;
  generating: boolean;
}

const FORMAT_OPTIONS: ImageFormat[] = ["PNG", "JPG", "WEBP"];
const CONTENT_OPTIONS: { value: ContentType; label: string }[] = [
  { value: "noise", label: "随机噪声" },
  { value: "solid", label: "纯色" },
  { value: "gradient", label: "渐变" },
  { value: "pattern", label: "图案(彩条)" },
];

export default function ImageTab({
  config,
  savePath,
  onConfigChange,
  onGenerate,
  onEstimate,
  generating,
}: Props) {
  const [estimate, setEstimate] = useState("");

  useEffect(() => {
    onEstimate({
      format: config.format,
      width: config.width,
      height: config.height,
      count: config.count,
    }).then(setEstimate);
  }, [config, onEstimate]);

  const handleStart = useCallback(() => {
    if (!savePath) {
      alert("请先选择保存路径");
      return;
    }
    onGenerate();
  }, [savePath, onGenerate]);

  return (
    <div className="tab-panel">
      <div className="form-row">
        <label>格式</label>
        <select
          value={config.format}
          onChange={(e) => onConfigChange({ format: e.target.value as ImageFormat })}
        >
          {FORMAT_OPTIONS.map((f) => (
            <option key={f} value={f}>{f}</option>
          ))}
        </select>
      </div>
      <div className="form-row">
        <label>分辨率</label>
        <div className="resolution-row">
          <input
            type="number"
            value={config.width}
            min={1}
            onChange={(e) => onConfigChange({ width: parseInt(e.target.value) || 1 })}
          />
          <span>x</span>
          <input
            type="number"
            value={config.height}
            min={1}
            onChange={(e) => onConfigChange({ height: parseInt(e.target.value) || 1 })}
          />
        </div>
      </div>
      <div className="form-row">
        <label>内容类型</label>
        <select
          value={config.contentType}
          onChange={(e) => onConfigChange({ contentType: e.target.value as ContentType })}
        >
          {CONTENT_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>{opt.label}</option>
          ))}
        </select>
      </div>
      <div className="form-row">
        <label>文件数量</label>
        <input
          type="number"
          value={config.count}
          min={1}
          onChange={(e) => onConfigChange({ count: parseInt(e.target.value) || 1 })}
        />
      </div>
      <div className="form-row">
        <label>前缀</label>
        <input
          type="text"
          value={config.prefix}
          onChange={(e) => onConfigChange({ prefix: e.target.value })}
        />
      </div>
      <div className="estimate-row">
        <span>预计体积: {estimate}</span>
        <span>{config.count} 个文件</span>
      </div>
      <button
        className="btn-primary"
        onClick={handleStart}
        disabled={generating}
      >
        {generating ? "生成中..." : "开始生成"}
      </button>
    </div>
  );
}
```

- [ ] **Step 4: Create AudioTab.tsx**

```tsx
import { useState, useEffect, useCallback } from "react";
import type { AudioConfig, AudioFormat, SampleRate, Channels } from "../types";

interface Props {
  config: AudioConfig;
  savePath: string | undefined;
  onConfigChange: (cfg: Partial<AudioConfig>) => void;
  onGenerate: () => void;
  onEstimate: (cfg: Record<string, unknown>) => Promise<string>;
  generating: boolean;
}

const FORMAT_OPTIONS: AudioFormat[] = ["MP3", "WAV", "AAC"];
const RATE_OPTIONS: SampleRate[] = [44100, 48000];
const CHANNEL_OPTIONS: { value: Channels; label: string }[] = [
  { value: "mono", label: "单声道" },
  { value: "stereo", label: "立体声" },
];

export default function AudioTab({
  config,
  savePath,
  onConfigChange,
  onGenerate,
  onEstimate,
  generating,
}: Props) {
  const [estimate, setEstimate] = useState("");

  useEffect(() => {
    onEstimate({
      format: config.format,
      duration: config.duration,
      count: config.count,
    }).then(setEstimate);
  }, [config, onEstimate]);

  const handleStart = useCallback(() => {
    if (!savePath) {
      alert("请先选择保存路径");
      return;
    }
    onGenerate();
  }, [savePath, onGenerate]);

  return (
    <div className="tab-panel">
      <div className="form-row">
        <label>格式</label>
        <select
          value={config.format}
          onChange={(e) => onConfigChange({ format: e.target.value as AudioFormat })}
        >
          {FORMAT_OPTIONS.map((f) => (
            <option key={f} value={f}>{f}</option>
          ))}
        </select>
      </div>
      <div className="form-row">
        <label>时长 (秒)</label>
        <input
          type="number"
          value={config.duration}
          min={1}
          onChange={(e) => onConfigChange({ duration: parseFloat(e.target.value) || 1 })}
        />
      </div>
      <div className="form-row">
        <label>采样率</label>
        <select
          value={config.sampleRate}
          onChange={(e) => onConfigChange({ sampleRate: parseInt(e.target.value) as SampleRate })}
        >
          {RATE_OPTIONS.map((r) => (
            <option key={r} value={r}>{r} Hz</option>
          ))}
        </select>
      </div>
      <div className="form-row">
        <label>声道</label>
        <select
          value={config.channels}
          onChange={(e) => onConfigChange({ channels: e.target.value as Channels })}
        >
          {CHANNEL_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>{opt.label}</option>
          ))}
        </select>
      </div>
      <div className="form-row">
        <label>文件数量</label>
        <input
          type="number"
          value={config.count}
          min={1}
          onChange={(e) => onConfigChange({ count: parseInt(e.target.value) || 1 })}
        />
      </div>
      <div className="form-row">
        <label>前缀</label>
        <input
          type="text"
          value={config.prefix}
          onChange={(e) => onConfigChange({ prefix: e.target.value })}
        />
      </div>
      <div className="estimate-row">
        <span>预计体积: {estimate}</span>
        <span>{config.count} 个文件</span>
      </div>
      <button
        className="btn-primary"
        onClick={handleStart}
        disabled={generating}
      >
        {generating ? "生成中..." : "开始生成"}
      </button>
    </div>
  );
}
```

- [ ] **Step 5: Create VideoTab.tsx**

```tsx
import { useState, useEffect, useCallback } from "react";
import type { VideoConfig, VideoFormat, Codec, ContentType } from "../types";

interface Props {
  config: VideoConfig;
  savePath: string | undefined;
  onConfigChange: (cfg: Partial<VideoConfig>) => void;
  onGenerate: () => void;
  onEstimate: (cfg: Record<string, unknown>) => Promise<string>;
  generating: boolean;
}

const FORMAT_OPTIONS: VideoFormat[] = ["MP4", "MOV", "WEBM"];
const CODEC_OPTIONS: { value: Codec; label: string }[] = [
  { value: "hevc", label: "H.265" },
  { value: "h264", label: "H.264" },
];
const FPS_OPTIONS = [30, 60];
const CONTENT_OPTIONS: { value: ContentType; label: string }[] = [
  { value: "noise", label: "随机噪声" },
  { value: "solid", label: "纯色" },
  { value: "gradient", label: "渐变" },
  { value: "pattern", label: "图案(彩条)" },
];

export default function VideoTab({
  config,
  savePath,
  onConfigChange,
  onGenerate,
  onEstimate,
  generating,
}: Props) {
  const [estimate, setEstimate] = useState("");

  useEffect(() => {
    onEstimate({
      format: config.format,
      codec: config.codec,
      width: config.width,
      height: config.height,
      fps: config.fps,
      duration: config.duration,
      count: config.count,
    }).then(setEstimate);
  }, [config, onEstimate]);

  const handleStart = useCallback(() => {
    if (!savePath) {
      alert("请先选择保存路径");
      return;
    }
    onGenerate();
  }, [savePath, onGenerate]);

  return (
    <div className="tab-panel">
      <div className="form-row">
        <label>格式</label>
        <select
          value={config.format}
          onChange={(e) => onConfigChange({ format: e.target.value as VideoFormat })}
        >
          {FORMAT_OPTIONS.map((f) => (
            <option key={f} value={f}>{f}</option>
          ))}
        </select>
      </div>
      <div className="form-row">
        <label>编码</label>
        <select
          value={config.codec}
          onChange={(e) => onConfigChange({ codec: e.target.value as Codec })}
        >
          {CODEC_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>{opt.label}</option>
          ))}
        </select>
      </div>
      <div className="form-row">
        <label>分辨率</label>
        <div className="resolution-row">
          <input
            type="number"
            value={config.width}
            min={1}
            onChange={(e) => onConfigChange({ width: parseInt(e.target.value) || 1 })}
          />
          <span>x</span>
          <input
            type="number"
            value={config.height}
            min={1}
            onChange={(e) => onConfigChange({ height: parseInt(e.target.value) || 1 })}
          />
        </div>
      </div>
      <div className="form-row">
        <label>帧率</label>
        <select
          value={config.fps}
          onChange={(e) => onConfigChange({ fps: parseInt(e.target.value) })}
        >
          {FPS_OPTIONS.map((f) => (
            <option key={f} value={f}>{f} fps</option>
          ))}
        </select>
      </div>
      <div className="form-row">
        <label>时长 (秒)</label>
        <input
          type="number"
          value={config.duration}
          min={1}
          onChange={(e) => onConfigChange({ duration: parseFloat(e.target.value) || 1 })}
        />
      </div>
      <div className="form-row">
        <label>内容类型</label>
        <select
          value={config.contentType}
          onChange={(e) => onConfigChange({ contentType: e.target.value as ContentType })}
        >
          {CONTENT_OPTIONS.map((opt) => (
            <option key={opt.value} value={opt.value}>{opt.label}</option>
          ))}
        </select>
      </div>
      <div className="form-row">
        <label>文件数量</label>
        <input
          type="number"
          value={config.count}
          min={1}
          onChange={(e) => onConfigChange({ count: parseInt(e.target.value) || 1 })}
        />
      </div>
      <div className="form-row">
        <label>前缀</label>
        <input
          type="text"
          value={config.prefix}
          onChange={(e) => onConfigChange({ prefix: e.target.value })}
        />
      </div>
      <div className="estimate-row">
        <span>预计体积: {estimate}</span>
        <span>{config.count} 个文件</span>
      </div>
      <button
        className="btn-primary"
        onClick={handleStart}
        disabled={generating}
      >
        {generating ? "生成中..." : "开始生成"}
      </button>
    </div>
  );
}
```

- [ ] **Step 6: Create ProgressPanel.tsx**

```tsx
interface Props {
  current: number;
  total: number;
  currentFile: string;
  estimatedRemainingSecs: number;
  onCancel: () => void;
}

function formatTime(secs: number): string {
  if (secs < 60) return `00:${String(Math.floor(secs)).padStart(2, "0")}`;
  const m = Math.floor(secs / 60);
  const s = Math.floor(secs % 60);
  return `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}

export default function ProgressPanel({
  current,
  total,
  currentFile,
  estimatedRemainingSecs,
  onCancel,
}: Props) {
  const pct = total > 0 ? (current / total) * 100 : 0;

  return (
    <div className="progress-panel">
      <div className="progress-header">
        <span>进度: {current} / {total}</span>
        <span>剩余 {formatTime(estimatedRemainingSecs)}</span>
      </div>
      <div className="progress-bar">
        <div className="progress-fill" style={{ width: `${pct}%` }} />
      </div>
      <div className="progress-file">{currentFile}</div>
      <button className="btn-cancel" onClick={onCancel}>取消</button>
    </div>
  );
}
```

- [ ] **Step 7: Create ResultSummary.tsx**

```tsx
interface Props {
  success: number;
  failed: number;
  errors: Array<{ file: string; error: string }>;
}

export default function ResultSummary({ success, failed, errors }: Props) {
  return (
    <div className="result-summary">
      <div className="result-counts">
        <span className="success">成功 {success} 个</span>
        {failed > 0 && <span className="failed">失败 {failed} 个</span>}
      </div>
      {errors.length > 0 && (
        <div className="error-list">
          {errors.map((e, i) => (
            <div key={i} className="error-item">
              <strong>{e.file}:</strong> {e.error.slice(0, 200)}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
```

- [ ] **Step 8: Update App.tsx to wire everything together**

```tsx
import { useState, useCallback, useEffect } from "react";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import Header from "./components/Header";
import TabBar from "./components/TabBar";
import ImageTab from "./components/ImageTab";
import AudioTab from "./components/AudioTab";
import VideoTab from "./components/VideoTab";
import ProgressPanel from "./components/ProgressPanel";
import ResultSummary from "./components/ResultSummary";
import { useGenerator } from "./hooks/useGenerator";
import type { MediaType, ProgressPayload, TaskResult } from "./types";

export default function App() {
  const [activeTab, setActiveTab] = useState<MediaType>("image");
  const [generating, setGenerating] = useState(false);
  const [progress, setProgress] = useState<ProgressPayload | null>(null);
  const [result, setResult] = useState<TaskResult | null>(null);

  const {
    config,
    updateConfig,
    estimateSize,
    generateImages,
    generateAudio,
    generateVideos,
    cancelGeneration,
  } = useGenerator();

  useEffect(() => {
    let unlisten: UnlistenFn;
    listen<ProgressPayload>("generation-progress", (event) => {
      setProgress(event.payload);
    }).then((fn) => { unlisten = fn; });
    return () => { unlisten?.(); };
  }, []);

  const handlePathChange = useCallback(
    (path: string) => {
      if (config) {
        updateConfig({ ...config, savePath: path });
      }
    },
    [config, updateConfig]
  );

  const handleImageConfig = useCallback(
    (partial: Record<string, unknown>) => {
      if (!config) return;
      updateConfig({ ...config, imageConfig: { ...config.imageConfig, ...partial } });
    },
    [config, updateConfig]
  );

  const handleAudioConfig = useCallback(
    (partial: Record<string, unknown>) => {
      if (!config) return;
      updateConfig({ ...config, audioConfig: { ...config.audioConfig, ...partial } });
    },
    [config, updateConfig]
  );

  const handleVideoConfig = useCallback(
    (partial: Record<string, unknown>) => {
      if (!config) return;
      updateConfig({ ...config, videoConfig: { ...config.videoConfig, ...partial } });
    },
    [config, updateConfig]
  );

  const handleGenerateImages = useCallback(async () => {
    if (!config?.savePath) return;
    setGenerating(true);
    setResult(null);
    setProgress(null);
    try {
      const res = await generateImages(config.imageConfig as unknown as Record<string, unknown>, config.savePath);
      setResult(res as TaskResult);
    } catch (e) {
      setResult({ success: 0, failed: 1, errors: [{ file: "unknown", error: String(e) }] });
    } finally {
      setGenerating(false);
    }
  }, [config, generateImages]);

  const handleGenerateAudio = useCallback(async () => {
    if (!config?.savePath) return;
    setGenerating(true);
    setResult(null);
    setProgress(null);
    try {
      const res = await generateAudio(config.audioConfig as unknown as Record<string, unknown>, config.savePath);
      setResult(res as TaskResult);
    } catch (e) {
      setResult({ success: 0, failed: 1, errors: [{ file: "unknown", error: String(e) }] });
    } finally {
      setGenerating(false);
    }
  }, [config, generateAudio]);

  const handleGenerateVideos = useCallback(async () => {
    if (!config?.savePath) return;
    setGenerating(true);
    setResult(null);
    setProgress(null);
    try {
      const res = await generateVideos(config.videoConfig as unknown as Record<string, unknown>, config.savePath);
      setResult(res as TaskResult);
    } catch (e) {
      setResult({ success: 0, failed: 1, errors: [{ file: "unknown", error: String(e) }] });
    } finally {
      setGenerating(false);
    }
  }, [config, generateVideos]);

  if (!config) {
    return <div className="app-container">加载中...</div>;
  }

  return (
    <div className="app-container">
      <Header savePath={config.savePath ?? undefined} onPathChange={handlePathChange} />
      <TabBar active={activeTab} onChange={setActiveTab} />
      <div className="tab-content">
        {activeTab === "image" && (
          <ImageTab
            config={config.imageConfig}
            savePath={config.savePath ?? undefined}
            onConfigChange={handleImageConfig}
            onGenerate={handleGenerateImages}
            onEstimate={(c) => estimateSize("image", c)}
            generating={generating}
          />
        )}
        {activeTab === "audio" && (
          <AudioTab
            config={config.audioConfig}
            savePath={config.savePath ?? undefined}
            onConfigChange={handleAudioConfig}
            onGenerate={handleGenerateAudio}
            onEstimate={(c) => estimateSize("audio", c)}
            generating={generating}
          />
        )}
        {activeTab === "video" && (
          <VideoTab
            config={config.videoConfig}
            savePath={config.savePath ?? undefined}
            onConfigChange={handleVideoConfig}
            onGenerate={handleGenerateVideos}
            onEstimate={(c) => estimateSize("video", c)}
            generating={generating}
          />
        )}
      </div>
      {generating && progress && (
        <ProgressPanel
          current={progress.current}
          total={progress.total}
          currentFile={progress.currentFile}
          estimatedRemainingSecs={progress.estimatedRemainingSecs}
          onCancel={cancelGeneration}
        />
      )}
      {result && <ResultSummary {...result} />}
    </div>
  );
}
```

- [ ] **Step 9: Add comprehensive CSS**

Replace `src/styles.css`:

```css
* { box-sizing: border-box; margin: 0; padding: 0; }
body {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  background: #f5f5f7;
  color: #1d1d1f;
  font-size: 14px;
}
#root { display: flex; align-items: center; justify-content: center; min-height: 100vh; padding: 16px; }

.app-container {
  background: #fff;
  border-radius: 12px;
  box-shadow: 0 4px 24px rgba(0,0,0,0.12);
  width: 100%;
  max-width: 520px;
  overflow: hidden;
}

/* Header */
.header { padding: 20px 24px 16px; border-bottom: 1px solid #eee; }
.header h1 { font-size: 18px; font-weight: 600; margin-bottom: 12px; }
.path-row { display: flex; align-items: center; gap: 8px; font-size: 13px; }
.path-label { color: #666; }
.path-value { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: #333; }

/* Tabs */
.tab-bar { display: flex; border-bottom: 1px solid #eee; padding: 0 24px; }
.tab-btn {
  padding: 12px 16px;
  border: none;
  background: none;
  cursor: pointer;
  font-size: 14px;
  color: #666;
  border-bottom: 2px solid transparent;
  transition: all 0.2s;
}
.tab-btn:hover { color: #333; }
.tab-btn.active { color: #007aff; border-bottom-color: #007aff; font-weight: 500; }

/* Tab Content */
.tab-content { padding: 24px; }
.tab-panel { display: flex; flex-direction: column; gap: 14px; }

/* Form */
.form-row { display: flex; align-items: center; gap: 12px; }
.form-row label { width: 80px; flex-shrink: 0; color: #555; font-size: 13px; }
.form-row input,
.form-row select {
  flex: 1;
  height: 34px;
  padding: 0 10px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 13px;
  outline: none;
  transition: border-color 0.2s;
}
.form-row input:focus,
.form-row select:focus { border-color: #007aff; }
.form-row input[type="number"] { width: 80px; flex: none; }
.resolution-row { display: flex; align-items: center; gap: 6px; flex: 1; }
.resolution-row input { flex: 1; }

/* Estimate */
.estimate-row { display: flex; justify-content: space-between; font-size: 13px; color: #888; padding: 4px 0; }

/* Buttons */
.btn-primary {
  height: 40px;
  background: #007aff;
  color: #fff;
  border: none;
  border-radius: 8px;
  font-size: 15px;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s;
}
.btn-primary:hover { background: #005bb5; }
.btn-primary:disabled { background: #ccc; cursor: not-allowed; }
.btn-small {
  padding: 4px 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  background: #fff;
  font-size: 12px;
  cursor: pointer;
}
.btn-small:hover { background: #f0f0f0; }
.btn-cancel {
  padding: 6px 16px;
  border: 1px solid #e00;
  border-radius: 6px;
  background: #fff;
  color: #e00;
  font-size: 13px;
  cursor: pointer;
}
.btn-cancel:hover { background: #fee; }

/* Progress */
.progress-panel {
  margin: 0 24px;
  padding: 16px;
  background: #f9f9fb;
  border-radius: 8px;
  border: 1px solid #eee;
}
.progress-header { display: flex; justify-content: space-between; font-size: 13px; margin-bottom: 8px; }
.progress-bar { height: 8px; background: #e5e5ea; border-radius: 4px; overflow: hidden; margin-bottom: 8px; }
.progress-fill { height: 100%; background: #007aff; border-radius: 4px; transition: width 0.3s; }
.progress-file { font-size: 12px; color: #888; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; margin-bottom: 8px; }

/* Result */
.result-summary { margin: 16px 24px 24px; padding: 16px; background: #f9f9fb; border-radius: 8px; border: 1px solid #eee; }
.result-counts { font-size: 14px; display: flex; gap: 16px; }
.result-counts .success { color: #34c759; font-weight: 500; }
.result-counts .failed { color: #ff3b30; font-weight: 500; }
.error-list { margin-top: 10px; }
.error-item { font-size: 12px; color: #ff3b30; padding: 4px 0; border-bottom: 1px solid #fdd; }
```

- [ ] **Step 10: Commit**

```bash
git add src/components/ src/App.tsx src/styles.css && git commit -m "feat(frontend): complete UI components and styling

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>"
```

---

## Task 8: Build and Test

**Files:**
- Modify: `src-tauri/tauri.conf.json` (devtools), `Cargo.toml` (chrono dep if needed)

- [ ] **Step 1: Install npm dependencies**

```bash
cd test-asset-generator && npm install
```

- [ ] **Step 2: Verify TypeScript compilation**

```bash
cd test-asset-generator && npx tsc --noEmit
```

Expected: No errors.

- [ ] **Step 3: Verify Rust compilation**

```bash
cd test-asset-generator/src-tauri && cargo check
```

Expected: No errors. Fix any compilation errors before proceeding.

- [ ] **Step 4: Run dev server to test**

```bash
cd test-asset-generator && npm run tauri dev
```

Expected: A window opens with the GUI. Test path selection, tab switching, and form inputs.

- [ ] **Step 5: Test FFmpeg binary detection**

Verify the app can find and run the bundled FFmpeg. Check Tauri logs for any "Failed to execute ffmpeg" errors.

- [ ] **Step 6: Test generation**

1. Select a save path
2. Switch to Image tab, set count=3
3. Click "开始生成"
4. Verify files appear in the timestamp folder
5. Verify MD5 of each file is different

- [ ] **Step 7: Test cancellation**

Start a video generation with count=10, click cancel, verify some files exist and the task stops.

- [ ] **Step 8: Commit**

```bash
git add -A && git commit -m "test: build verification and manual smoke test

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>"
```

---

## Task 9: Production Build

**Files:**
- Modify: `src-tauri/tauri.conf.json` (app identifier, icon), build scripts

- [ ] **Step 1: Configure tauri.conf.json for production**

Ensure `bundle.identifier` is set, `build.devtools` is `true` for debugging, and FFmpeg files are included in bundle resources.

Add to `tauri.conf.json` under `bundle`:

```json
"resources": {
  "ffmpeg/*": "ffmpeg/"
}
```

- [ ] **Step 2: Build macOS app**

```bash
cd test-asset-generator && npm run tauri build -- --target aarch64-apple-darwin
```

- [ ] **Step 3: Commit build artifacts (optional) or document CI approach**

Create `.github/workflows/build.yml` for cross-platform CI:

```yaml
name: Build
on: [push, pull_request]
jobs:
  build:
    strategy:
      matrix:
        include:
          - platform: "macos-latest"
            args: "--target aarch64-apple-darwin"
            artifact: "Test Asset Generator-macOS"
          - platform: "windows-latest"
            args: "--target x86_64-pc-windows-msvc"
            artifact: "Test Asset Generator-Windows"
    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install dependencies (macOS)
        if: matrix.platform == 'macos-latest'
        run: |
          curl -L "https://evermeet.cx/ffmpeg/getrelease/ffmpeg/zip" -o /tmp/ffmpeg.zip
          unzip -o /tmp/ffmpeg.zip -d src-tauri/ffmpeg/mac/
          chmod +x src-tauri/ffmpeg/mac/ffmpeg
      - name: Build
        run: npm install && npm run tauri build ${{ matrix.args }}
      - uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.artifact }}
          path: src-tauri/target/*/release/bundle/*
```

- [ ] **Step 4: Commit CI workflow**

```bash
git add .github/ && git commit -m "ci: add GitHub Actions build workflow for macOS and Windows

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>"
```

---

## Self-Review Checklist

- [ ] All spec requirements mapped to tasks: GUI (Task 1, 7), image formats (Task 5), audio formats (Task 5), video formats (Task 5), MD5 uniqueness (Task 5), estimated size (Task 3), progress/ETA (Task 5, 7), cancel (Task 5, 7), result summary (Task 7), file retention on error (Task 5), path selection (Task 3, 7), timestamp folders (Task 5), config persistence (Task 3), bundled FFmpeg (Task 2).
- [ ] No placeholders: all code is complete with actual implementation
- [ ] Type consistency: Rust `ImageConfig`/`AudioConfig`/`VideoConfig` fields match TypeScript `types/index.ts`
- [ ] FFmpeg filter syntax verified: `color`, `gradients`, `testsrc2`, `cellauto` filters are standard FFmpeg libavfilter filters
- [ ] The `anoisesa` filter uses seed for MD5 uniqueness alongside random amplitude
- [ ] Timestamp format `YYYY-MM-DD-HH-mm-ss` implemented inline without chrono crate
- [ ] Cancel flag uses `std::sync::atomic::AtomicBool` accessible from async Tauri commands
