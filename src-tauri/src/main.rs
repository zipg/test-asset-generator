#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod ffmpeg;
mod generator;

use config::AppConfig;
use generator::{get_cancel, reset_cancel, set_cancel, random_hex};
use rand::Rng;
use tauri::Emitter;

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
            download_ffmpeg,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn get_config(app: tauri::AppHandle) -> AppConfig {
    config::load_config(&app)
}
#[tauri::command]
async fn download_ffmpeg() -> Result<String, String> {
    let os = std::env::consts::OS;

    // Detect system architecture
    let arch = if os == "macos" {
        let output = std::process::Command::new("uname").arg("-m").output();
        match output {
            Ok(o) => {
                let arch_str = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if arch_str == "arm64" { "arm64" } else { "x86_64" }
            },
            Err(_) => "x86_64",
        }
    } else {
        "x86_64"
    };

    let (url, exe_name) = match os {
        "windows" => (
            "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip",
            "ffmpeg.exe",
        ),
        "macos" => (
            "https://evermeet.cx/ffmpeg/getrelease/ffmpeg/zip",
            "ffmpeg",
        ),
        _ => return Err(format!("Unsupported OS: {}", os)),
    };

    // Create download directory
    let download_dir = ffmpeg::get_ffmpeg_dir()
        .ok_or_else(|| "Failed to get app data directory".to_string())?;

    let ffmpeg_path = download_dir.join(exe_name);

    // Check if already downloaded and executable
    if ffmpeg_path.exists() {
        match std::process::Command::new(&ffmpeg_path).arg("--version").output() {
            Ok(output) if output.status.success() => {
                return Ok("already_exists".to_string());
            }
            _ => {
                // Invalid or incompatible binary, remove it
                let _ = std::fs::remove_file(&ffmpeg_path);
            }
        }
    }

    std::fs::create_dir_all(&download_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    // Download
    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("Failed to download FFmpeg: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }

    let bytes = response.bytes()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    // Extract based on OS
    if os == "windows" {
        let cursor = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor)
            .map_err(|e| format!("Failed to read zip: {}", e))?;

        let mut ffmpeg_file = std::fs::File::create(&ffmpeg_path)
            .map_err(|e| format!("Failed to create file: {}", e))?;

        let mut found = false;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)
                .map_err(|e| format!("Failed to read zip entry: {}", e))?;
            if file.name().ends_with("ffmpeg.exe") {
                std::io::copy(&mut file, &mut ffmpeg_file)
                    .map_err(|e| format!("Failed to write ffmpeg.exe: {}", e))?;
                found = true;
                break;
            }
        }
        if !found {
            return Err("ffmpeg.exe not found in zip".to_string());
        }
    } else {
        // macOS
        let mut ffmpeg_file = std::fs::File::create(&ffmpeg_path)
            .map_err(|e| format!("Failed to create file: {}", e))?;
        std::io::copy(&mut std::io::Cursor::new(bytes), &mut ffmpeg_file)
            .map_err(|e| format!("Failed to write ffmpeg: {}", e))?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&ffmpeg_path, std::fs::Permissions::from_mode(0o755))
                .map_err(|e| format!("Failed to set permissions: {}", e))?;
        }
    }

    // Verify the downloaded file is executable
    match std::process::Command::new(&ffmpeg_path).arg("--version").output() {
        Ok(output) if output.status.success() => {
            Ok("downloaded".to_string())
        }
        Ok(_) => {
            Err("FFmpeg downloaded but failed to run".to_string())
        }
        Err(e) => {
            Err(format!("FFmpeg is not executable: {}. Consider installing via: brew install ffmpeg", e))
        }
    }
}
#[tauri::command]
fn save_config(app: tauri::AppHandle, cfg: AppConfig) {
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
async fn select_save_path(app: tauri::AppHandle) -> Result<Option<String>, String> {
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
    match media_type.as_str() {
        "image" => {
            let w: u64 = cfg["width"].as_u64().unwrap_or(1080);
            let h: u64 = cfg["height"].as_u64().unwrap_or(1920);
            let count: u64 = cfg["count"].as_u64().unwrap_or(1);
            let format = cfg["format"].as_str().unwrap_or("PNG");
            let bytes_per_pixel = match format {
                "JPG" | "jpg" => 0.5,
                "WEBP" | "webp" => 0.3,
                _ => 3.0,
            };
            let size = (w * h) as f64 * bytes_per_pixel * (count as f64) / 1_048_576.0;
            format!("~{:.1} MB", size.max(0.01))
        }
        "audio" => {
            let duration: f64 = cfg["duration"].as_f64().unwrap_or(60.0);
            let count: u64 = cfg["count"].as_u64().unwrap_or(1);
            let format = cfg["format"].as_str().unwrap_or("MP3");
            let kbps = match format {
                "WAV" | "wav" => 1411.0,
                "AAC" | "aac" => 128.0,
                _ => 128.0,
            };
            let size = (duration * kbps * 1000.0 / 8.0) * (count as f64) / 1_048_576.0;
            format!("~{:.1} MB", size.max(0.01))
        }
        "video" => {
            let w: u64 = cfg["width"].as_u64().unwrap_or(1080);
            let h: u64 = cfg["height"].as_u64().unwrap_or(1920);
            let duration: f64 = cfg["duration"].as_f64().unwrap_or(60.0);
            let fps: u64 = cfg["fps"].as_u64().unwrap_or(30);
            let count: u64 = cfg["count"].as_u64().unwrap_or(1);
            let kbps = (w * h * fps) as f64 * 0.1 / 1000.0;
            let size = (duration * kbps * 1000.0 / 8.0) * (count as f64) / 1_048_576.0;
            format!("~{:.1} MB", size.max(0.01))
        }
        _ => "~0 MB".to_string(),
    }
}

#[tauri::command]
async fn generate_images(
    app: tauri::AppHandle,
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

        let random_str = random_hex(6);
        let ext = match config.format.as_str() {
            "JPG" | "jpg" => "jpg",
            "WEBP" | "webp" => "webp",
            _ => "png",
        };
        let filename = format!("{}_{}_{:03}.{}", config.prefix, random_str, i, ext);
        let output_path = output_dir.join(&filename);

        let filter = build_image_filter(&config.content_type, config.width, config.height);

        let mut args: Vec<String> = vec![
            "-f".to_string(), "lavfi".to_string(),
            "-i".to_string(), filter,
            "-vframes".to_string(), "1".to_string(),
            "-y".to_string(),
        ];

        match ext {
            "jpg" => args.extend_from_slice(&["-q:v".to_string(), "2".to_string()]),
            "webp" => args.extend_from_slice(&["-quality".to_string(), "90".to_string()]),
            _ => {}
        }

        args.push(output_path.to_str().unwrap().to_string());

        match ffmpeg::run_ffmpeg(&args) {
            Ok(_) => success += 1,
            Err(e) => {
                failed += 1;
                errors.push(serde_json::json!({ "file": filename, "error": e }));
            }
        }

        let elapsed = i as f64;
        let total_done = success + failed;
        let eta = if total_done > 0 {
            ((total - i) as f64 / total_done as f64 * elapsed).max(0.0) as u32
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
async fn generate_audio(
    app: tauri::AppHandle,
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
        "WAV" | "wav" => "wav",
        "AAC" | "aac" => "aac",
        _ => "mp3",
    };
    let duration_str = format_duration(config.duration);

    for i in 1..=total {
        if get_cancel() {
            break;
        }

        let random_str = random_hex(6);
        let filename = format!("{}_{}_{:03}.{}", config.prefix, random_str, i, ext);
        let output_path = output_dir.join(&filename);

        let amplitude: f32 = rand::thread_rng().gen_range(0.1..0.5);
        let seed: u32 = rand::thread_rng().gen();

        let anoisesa = format!(
            "anoisesa=d={}:a={}:r={}:c={}:s={}",
            duration_str, amplitude, config.sample_rate, channels, seed
        );

        let mut args: Vec<String> = vec![
            "-f".to_string(), "lavfi".to_string(),
            "-i".to_string(), anoisesa,
            "-y".to_string(),
        ];

        if ext != "wav" {
            let codec = if ext == "aac" { "aac" } else { "mp3" };
            args.extend_from_slice(&["-acodec".to_string(), codec.to_string()]);
        }

        args.push(output_path.to_str().unwrap().to_string());

        match ffmpeg::run_ffmpeg(&args) {
            Ok(_) => success += 1,
            Err(e) => {
                failed += 1;
                errors.push(serde_json::json!({ "file": filename, "error": e }));
            }
        }

        let elapsed = i as f64;
        let total_done = success + failed;
        let eta = if total_done > 0 {
            ((total - i) as f64 / total_done as f64 * elapsed).max(0.0) as u32
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
    app: tauri::AppHandle,
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
        "MOV" | "mov" => "mov",
        "WEBM" | "webm" => "webm",
        _ => "mp4",
    };
    let codec = if config.codec == "h264" { "libx264" } else { "libx265" };
    let duration_str = format_duration(config.duration);

    for i in 1..=total {
        if get_cancel() {
            break;
        }

        let random_str = random_hex(6);
        let filename = format!("{}_{}_{:03}.{}", config.prefix, random_str, i, ext);
        let output_path = output_dir.join(&filename);

        let seed: u32 = rand::thread_rng().gen();
        let hue: f32 = rand::thread_rng().gen_range(0.0..360.0);

        let filter = match config.content_type.as_str() {
            "solid" => format!(
                "color=c=0x{:06x}:s={}x{}:d={}",
                (hue / 360.0 * 16777215.0) as u32,
                config.width, config.height, duration_str
            ),
            "gradient" => format!(
                "gradients=s={}x{}:c0=random:c1=random:seed={}:d={}",
                config.width, config.height, seed, duration_str
            ),
            "pattern" => format!(
                "testsrc2=size={}x{}",
                config.width, config.height
            ),
            _ => format!(
                "cellauto=rule=18:seed={}:size={}x{}:pattern=random,scale={}:{}:flags=neighbor;framerate=fps={}",
                seed, config.width, config.height, config.width, config.height, config.fps
            ),
        };

        let fps_str = config.fps.to_string();
        let args: Vec<String> = vec![
            "-f".to_string(), "lavfi".to_string(),
            "-i".to_string(), filter,
            "-c:v".to_string(), codec.to_string(),
            "-r".to_string(), fps_str,
            "-t".to_string(), duration_str.clone(),
            "-pix_fmt".to_string(), "yuv420p".to_string(),
            "-y".to_string(),
            output_path.to_str().unwrap().to_string(),
        ];

        match ffmpeg::run_ffmpeg(&args) {
            Ok(_) => success += 1,
            Err(e) => {
                failed += 1;
                errors.push(serde_json::json!({ "file": filename, "error": e }));
            }
        }

        let elapsed = i as f64;
        let total_done = success + failed;
        let eta = if total_done > 0 {
            ((total - i) as f64 / total_done as f64 * elapsed).max(0.0) as u32
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

fn create_timestamp_dir(base: &str) -> Result<std::path::PathBuf, String> {
    let now = chrono_lite_timestamp();
    let dir = std::path::PathBuf::from(base).join(&now);
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create directory: {}", e))?;
    Ok(dir)
}

fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap();
    let secs = dur.as_secs();
    let days = secs / 86400;
    let remaining = secs % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;
    let year_days = days;
    let year = 1970 + year_days / 365;
    let yday = year_days % 365;
    let month = yday / 30 + 1;
    let day = yday % 30 + 1;
    format!("{:04}-{:02}-{:02}-{:02}-{:02}-{:02}", year, month, day, hours, minutes, seconds)
}

fn build_image_filter(content_type: &str, width: u32, height: u32) -> String {
    let seed: u32 = rand::thread_rng().gen();
    let hue: f32 = rand::thread_rng().gen_range(0.0..360.0);
    match content_type {
        "solid" => format!(
            "color=c=0x{:06x}:s={}x{}:d=1",
            (hue / 360.0 * 16777215.0) as u32,
            width, height
        ),
        "gradient" => format!(
            "gradients=s={}x{}:c0=random:c1=random:seed={}",
            width, height, seed
        ),
        "pattern" => format!("testsrc2=size={}x{}", width, height),
        _ => format!(
            "cellauto=rule=18:seed={}:size={}x{}:pattern=random,scale={}:{}:flags=neighbor",
            seed, width, height, width, height
        ),
    }
}

fn format_duration(secs: f64) -> String {
    if secs == secs.floor() {
        format!("{:.0}", secs)
    } else {
        format!("{:.2}", secs)
    }
}
