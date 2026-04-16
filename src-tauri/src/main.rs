#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod ffmpeg;
mod generator;

use config::AppConfig;
use generator::{get_cancel, reset_cancel, set_cancel, random_hex};
use rand::Rng;
use tauri::Emitter;
/// Compute MD5 hash of a file
fn file_md5(path: &std::path::Path) -> Result<String, String> {
    let data = std::fs::read(path).map_err(|e| format!("Failed to read file: {}", e))?;
    let hash = md5::compute(&data);
    Ok(hex::encode(*hash))
}

/// Generate a truly unique seed using nanosecond precision timestamp + loop counter
/// This ensures no two calls within the same process will ever produce the same seed
fn unique_seed() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(0);

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
    ((nanos as u32).wrapping_add(counter)) | 1 // |1 ensures odd number
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if std::env::consts::OS == "windows" {
                let _ = ffmpeg::ensure_windows_bundled_ffmpeg_copied(&app.handle());
            }
            Ok(())
        })
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
            check_ffmpeg,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn check_ffmpeg(app: tauri::AppHandle) -> String {
    let os = std::env::consts::OS;

    if os == "windows" {
        let _ = ffmpeg::ensure_windows_bundled_ffmpeg_copied(&app);
        if ffmpeg::bundled_resource_ffmpeg_exists(&app) {
            return "found".to_string();
        }
    }

    // On macOS, try to use the system "which" command to find FFmpeg
    // This uses the same PATH as the terminal
    if os == "macos" {
        if let Ok(output) = std::process::Command::new("/usr/bin/which")
            .arg("ffmpeg")
            .output()
        {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() && path != "ffmpeg not found" {
                // Found it, verify it works
                if let Ok(verify) = std::process::Command::new(&path).arg("--version").output() {
                    if verify.status.success() {
                        return "found".to_string();
                    }
                }
            }
        }
    }

    // Windows: PATH uses `;` — discovery lives in ffmpeg.rs (`where.exe` + split_paths).
    if os == "windows" {
        if let Some(p) = ffmpeg::bundled_ffmpeg_beside_executable_windows() {
            if let Ok(verify) = std::process::Command::new(&p).arg("--version").output() {
                if verify.status.success() {
                    return "found".to_string();
                }
            }
        }
        if ffmpeg::first_working_windows_ffmpeg_from_where().is_some() {
            return "found".to_string();
        }
        if ffmpeg::windows_ffmpeg_path_from_where_exists().is_some() {
            return "found".to_string();
        }
    }

    // Try common homebrew paths (Unix; not applicable on Windows)
    let homebrew_paths = [
        "/opt/homebrew/bin/ffmpeg",
        "/usr/local/bin/ffmpeg",
        "/opt/homebrew/opt/ffmpeg/bin/ffmpeg",
        "/usr/local/opt/ffmpeg/bin/ffmpeg",
    ];
    if os != "windows" {
        for path in &homebrew_paths {
            let p = std::path::Path::new(path);
            if p.exists() {
                if let Ok(output) = std::process::Command::new(p).arg("--version").output() {
                    if output.status.success() {
                        return "found".to_string();
                    }
                }
            }
        }
    }

    // Check app data directory
    let exe_name = if os == "windows" { "ffmpeg.exe" } else { "ffmpeg" };
    if let Some(app_data) = dirs::data_local_dir() {
        let downloaded = app_data.join("Muse_Generator").join("ffmpeg").join(exe_name);
        if downloaded.exists() {
            if let Ok(output) = std::process::Command::new(&downloaded).arg("--version").output() {
                if output.status.success() {
                    return "found".to_string();
                }
            }
        }
    }

    // On macOS, always return "found" if homebrew ffmpeg exists in common locations
    // The sandboxed app may not be able to execute it, but it exists on the system
    if os == "macos" {
        if std::path::Path::new("/opt/homebrew/bin/ffmpeg").exists()
            || std::path::Path::new("/usr/local/bin/ffmpeg").exists()
        {
            return "found".to_string();
        }
    }

    // Windows: typical users have no FFmpeg preinstalled; `download_ffmpeg` installs to AppData
    // on first generate. If we returned `not_found` here, `ffmpegReady` stays false and the UI
    // disables all actions — users could never click Generate to trigger the download (deadlock).
    if os == "windows" && dirs::data_local_dir().is_some() {
        return "found".to_string();
    }

    "not_found".to_string()
}

#[tauri::command]
fn get_config(app: tauri::AppHandle) -> AppConfig {
    config::load_config(&app)
}
fn windows_ffmpeg_zip_url_candidates() -> Vec<String> {
    if let Ok(u) = std::env::var("MUSE_FFMPEG_WINDOWS_ZIP_URL") {
        let u = u.trim();
        if !u.is_empty() {
            return vec![u.to_string()];
        }
    }
    vec![
        "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip".to_string(),
        // Fallback for regions where GitHub is slow (may change over time; override with env above).
        "https://mirror.ghproxy.com/https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip".to_string(),
    ]
}

async fn fetch_url_bytes(url: &str) -> Result<Vec<u8>, String> {
    let response = reqwest::get(url)
        .await
        .map_err(|e| format!("{}", e))?;
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }
    response
        .bytes()
        .await
        .map(|b| b.to_vec())
        .map_err(|e| format!("Failed to read response: {}", e))
}

#[tauri::command]
async fn download_ffmpeg(app: tauri::AppHandle) -> Result<String, String> {
    let os = std::env::consts::OS;

    if os == "windows" {
        let _ = ffmpeg::ensure_windows_bundled_ffmpeg_copied(&app);
    }

    // First check if a valid FFmpeg already exists
    let existing_path = ffmpeg::get_ffmpeg_path();
    if let Ok(output) = std::process::Command::new(&existing_path).arg("--version").output() {
        if output.status.success() {
            return Ok("already_exists".to_string());
        }
    }

    // Must stay consistent with check_ffmpeg(): that command returns "found" on macOS when
    // Homebrew binaries exist on disk even if this process cannot exec-verify them (e.g. some
    // GUI environments). Without this branch we always hit the network and reqwest may fail
    // with "error decoding response body" while the user already has brew ffmpeg.
    if os == "macos" {
        if std::path::Path::new("/opt/homebrew/bin/ffmpeg").exists()
            || std::path::Path::new("/usr/local/bin/ffmpeg").exists()
        {
            return Ok("already_exists".to_string());
        }
    }

    if os == "windows" {
        if ffmpeg::bundled_ffmpeg_beside_executable_windows().is_some() {
            return Ok("already_exists".to_string());
        }
        if ffmpeg::windows_ffmpeg_path_from_where_exists().is_some() {
            return Ok("already_exists".to_string());
        }
    }

    // No valid FFmpeg found, need to download (or no bundled resource in installer)
    let exe_name = if os == "windows" {
        "ffmpeg.exe"
    } else if os == "macos" {
        "ffmpeg"
    } else {
        return Err(format!("Unsupported OS: {}", os));
    };

    // Create download directory
    let download_dir = ffmpeg::get_ffmpeg_dir()
        .ok_or_else(|| "Failed to get app data directory".to_string())?;

    let ffmpeg_path = download_dir.join(exe_name);

    // Clean up any invalid existing download
    if ffmpeg_path.exists() {
        let _ = std::fs::remove_file(&ffmpeg_path);
    }

    std::fs::create_dir_all(&download_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    let bytes: Vec<u8> = if os == "windows" {
        let urls = windows_ffmpeg_zip_url_candidates();
        let mut last_err = String::new();
        let mut got: Option<Vec<u8>> = None;
        for url in &urls {
            match fetch_url_bytes(url).await {
                Ok(b) => {
                    got = Some(b);
                    break;
                }
                Err(e) => {
                    last_err = format!("{} — {}", url, e);
                }
            }
        }
        got.ok_or_else(|| {
            format!(
                "Failed to download FFmpeg (tried {} URL(s)). Last error: {}",
                urls.len(),
                last_err
            )
        })?
    } else {
        let url = "https://evermeet.cx/ffmpeg/getrelease/ffmpeg/zip";
        fetch_url_bytes(url)
            .await
            .map_err(|e| format!("Failed to download FFmpeg: {}", e))?
    };

    // Extract based on OS
    if os == "windows" {
        let cursor = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor)
            .map_err(|e| format!("Failed to read zip: {}", e))?;

        // BtbN zips may contain several ffmpeg.exe entries; prefer .../bin/ffmpeg.exe.
        let mut best: Option<(usize, u8)> = None;
        for i in 0..archive.len() {
            let file = archive
                .by_index(i)
                .map_err(|e| format!("Failed to read zip entry: {}", e))?;
            let name = file.name().replace('\\', "/");
            if name.ends_with("ffmpeg.exe") {
                let rank = if name.contains("/bin/") { 0u8 } else { 1u8 };
                let better = match best {
                    None => true,
                    Some((_, r)) => rank < r,
                };
                if better {
                    best = Some((i, rank));
                }
            }
        }
        let (idx, _) = best.ok_or_else(|| "ffmpeg.exe not found in zip".to_string())?;

        let mut file = archive
            .by_index(idx)
            .map_err(|e| format!("Failed to read zip entry: {}", e))?;
        let mut out = std::fs::File::create(&ffmpeg_path)
            .map_err(|e| format!("Failed to create file: {}", e))?;
        std::io::copy(&mut file, &mut out)
            .map_err(|e| format!("Failed to write ffmpeg.exe: {}", e))?;
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
            let hint = if os == "windows" {
                "Install FFmpeg (e.g. winget install ffmpeg) or place ffmpeg.exe on PATH."
            } else {
                "Consider installing via: brew install ffmpeg"
            };
            Err(format!("FFmpeg is not executable: {}. {}", e, hint))
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
    let output_dir = create_timestamp_dir(&save_path, &config.prefix)?;

    let total = config.count;
    let mut success = 0u32;
    let mut failed = 0u32;
    let mut errors: Vec<serde_json::Value> = Vec::new();

    let mut seen_md5s: std::collections::HashSet<String> = std::collections::HashSet::new();

    for i in 1..=total {
        if get_cancel() {
            break;
        }

        // Try generating until we get a unique MD5 or max retries
        let mut retries = 0;
        let max_retries = 10;
        let mut generated = false;

        while retries < max_retries {
            if get_cancel() {
                break;
            }

            let random_str = random_hex(6);
            let ext = match config.format.as_str() {
                "JPG" | "jpg" => "jpg",
                "WEBP" | "webp" => "webp",
                _ => "png",
            };
            let filename = format!("{}_{:03}_{}.{}", config.prefix, i, random_str, ext);
            let output_path = output_dir.join(&filename);
            let seed: u32 = unique_seed();

            let filter = build_image_filter(&config.content_type, config.width, config.height, seed);

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

            match ffmpeg::run_ffmpeg(&args, 30) {
                Ok(_) => {
                    // Check MD5 for uniqueness
                    match file_md5(&output_path) {
                        Ok(md5_hash) => {
                            if seen_md5s.contains(&md5_hash) {
                                // Duplicate MD5, retry with new seed
                                retries += 1;
                                let _ = std::fs::remove_file(&output_path);
                                continue;
                            }
                            seen_md5s.insert(md5_hash);
                            generated = true;
                            break;
                        }
                        Err(e) => {
                            failed += 1;
                            errors.push(serde_json::json!({ "file": filename, "error": format!("MD5 check failed: {}", e) }));
                            generated = true;
                            break;
                        }
                    }
                }
                Err(e) => {
                    failed += 1;
                    errors.push(serde_json::json!({ "file": filename, "error": e }));
                    generated = true;
                    break;
                }
            }
        }

        if !generated && retries >= max_retries {
            failed += 1;
            errors.push(serde_json::json!({ "file": format!("{}_{:03}", config.prefix, i), "error": "Failed to generate unique file after 10 retries" }));
        }

        if generated {
            success += 1;
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
            "currentFile": format!("{}_{:03}", config.prefix, i),
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
    let output_dir = create_timestamp_dir(&save_path, &config.prefix)?;

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

    let mut seen_md5s: std::collections::HashSet<String> = std::collections::HashSet::new();

    for i in 1..=total {
        if get_cancel() {
            break;
        }

        // Try generating until we get a unique MD5 or max retries
        let mut retries = 0;
        let max_retries = 10;
        let mut generated = false;

        while retries < max_retries {
            if get_cancel() {
                break;
            }

            let random_str = random_hex(6);
            let filename = format!("{}_{:03}_{}.{}", config.prefix, i, random_str, ext);
            let output_path = output_dir.join(&filename);

            let amplitude: f32 = rand::thread_rng().gen_range(0.1..0.5);
            let seed: u32 = unique_seed();

            let anoisesrc = format!(
                "anoisesrc=d={}:a={}:r={}:c={}:s={}",
                duration_str, amplitude, config.sample_rate, channels, seed
            );

            let mut args: Vec<String> = vec![
                "-f".to_string(), "lavfi".to_string(),
                "-i".to_string(), anoisesrc,
                "-y".to_string(),
            ];

            if ext != "wav" {
                let codec = if ext == "aac" { "aac" } else { "mp3" };
                args.extend_from_slice(&["-acodec".to_string(), codec.to_string()]);
            }

            args.push(output_path.to_str().unwrap().to_string());

            match ffmpeg::run_ffmpeg(&args, 30) {
                Ok(_) => {
                    // Check MD5 for uniqueness
                    match file_md5(&output_path) {
                        Ok(md5_hash) => {
                            if seen_md5s.contains(&md5_hash) {
                                // Duplicate MD5, retry with new seed
                                retries += 1;
                                let _ = std::fs::remove_file(&output_path);
                                continue;
                            }
                            seen_md5s.insert(md5_hash);
                            generated = true;
                            break;
                        }
                        Err(e) => {
                            failed += 1;
                            errors.push(serde_json::json!({ "file": filename, "error": format!("MD5 check failed: {}", e) }));
                            generated = true;
                            break;
                        }
                    }
                }
                Err(e) => {
                    failed += 1;
                    errors.push(serde_json::json!({ "file": filename, "error": e }));
                    generated = true;
                    break;
                }
            }
        }

        if !generated && retries >= max_retries {
            failed += 1;
            errors.push(serde_json::json!({ "file": format!("{}_{:03}", config.prefix, i), "error": "Failed to generate unique file after 10 retries" }));
        }

        if generated {
            success += 1;
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
            "currentFile": format!("{}_{:03}", config.prefix, i),
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
    let output_dir = create_timestamp_dir(&save_path, &config.prefix)?;

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

    let mut seen_md5s: std::collections::HashSet<String> = std::collections::HashSet::new();

    for i in 1..=total {
        if get_cancel() {
            break;
        }

        // Try generating until we get a unique MD5 or max retries
        let mut retries = 0;
        let max_retries = 10;
        let mut generated = false;

        while retries < max_retries {
            if get_cancel() {
                break;
            }

            let random_str = random_hex(6);
            let filename = format!("{}_{:03}_{}.{}", config.prefix, i, random_str, ext);
            let output_path = output_dir.join(&filename);

            let seed: u32 = unique_seed();

            let filter = match config.content_type.as_str() {
                "solid" => {
                    // Use seed to ensure unique colors
                    let color_hue = (seed % 360) as f32;
                    format!(
                        "color=c=0x{:06x}:s={}x{}:d={}",
                        (color_hue / 360.0 * 16777215.0) as u32,
                        config.width, config.height, duration_str
                    )
                }
                "gradient" => format!(
                    "gradients=s={}x{}:c0=random:c1=random:seed={}:d={}",
                    config.width, config.height, seed, duration_str
                ),
                "pattern" => format!(
                    "testsrc2=size={}x{}",
                    config.width, config.height
                ),
                _ => {
                    // Use random_fill_ratio to ensure unique output - pattern=random ignores seed
                    let rules = [18u32, 22, 26, 30, 34, 38, 42, 46, 50, 54, 58, 62, 66, 70, 74, 78, 82, 86, 90, 94, 98, 102, 106, 110, 114, 118, 122, 126, 130, 134, 138, 142, 146, 150];
                    let rule = rules[(seed % rules.len() as u32) as usize];
                    let fill_ratio = 0.3 + (seed % 50) as f64 / 100.0;
                    format!(
                        "cellauto=rule={}:size={}x{}:random_seed={}:random_fill_ratio={},scale={}:{}:flags=neighbor",
                        rule, config.width, config.height, seed, fill_ratio, config.width, config.height
                    )
                },
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

            match ffmpeg::run_ffmpeg(&args, 30) {
                Ok(_) => {
                    // Check MD5 for uniqueness
                    match file_md5(&output_path) {
                        Ok(md5_hash) => {
                            if seen_md5s.contains(&md5_hash) {
                                // Duplicate MD5, retry with new seed
                                retries += 1;
                                let _ = std::fs::remove_file(&output_path);
                                continue;
                            }
                            seen_md5s.insert(md5_hash);
                            generated = true;
                            break;
                        }
                        Err(e) => {
                            failed += 1;
                            errors.push(serde_json::json!({ "file": filename, "error": format!("MD5 check failed: {}", e) }));
                            generated = true;
                            break;
                        }
                    }
                }
                Err(e) => {
                    failed += 1;
                    errors.push(serde_json::json!({ "file": filename, "error": e }));
                    generated = true;
                    break;
                }
            }
        }

        if !generated && retries >= max_retries {
            failed += 1;
            errors.push(serde_json::json!({ "file": format!("{}_{:03}", config.prefix, i), "error": "Failed to generate unique file after 10 retries" }));
        }

        if generated {
            success += 1;
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
            "currentFile": format!("{}_{:03}", config.prefix, i),
            "estimatedRemainingSecs": eta,
        }));
    }

    Ok(serde_json::json!({
        "success": success,
        "failed": failed,
        "errors": errors,
    }))
}

fn create_timestamp_dir(base: &str, prefix: &str) -> Result<std::path::PathBuf, String> {
    let now = chrono_lite_timestamp();
    let dir_name = format!("{}_{}", prefix, now);
    let dir = std::path::PathBuf::from(base).join(&dir_name);
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create directory: {}", e))?;
    Ok(dir)
}

fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap();
    let secs = dur.as_secs();
    let remaining = secs % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;
    let days = secs / 86400;
    let year_days = days;
    let year = 1970 + year_days / 365;
    let yday = year_days % 365;
    let month = yday / 30 + 1;
    let day = yday % 30 + 1;
    format!("{:02}{:02}_{:02}{:02}{:02}", month, day, hours, minutes, seconds)
}

fn build_image_filter(content_type: &str, width: u32, height: u32, seed: u32) -> String {
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
        _ => {
            // Use random_fill_ratio to ensure unique output - pattern=random ignores seed
            // but random_fill_ratio + random_seed together produce truly unique outputs
            let rules = [18u32, 22, 26, 30, 34, 38, 42, 46, 50, 54, 58, 62, 66, 70, 74, 78, 82, 86, 90, 94, 98, 102, 106, 110, 114, 118, 122, 126, 130, 134, 138, 142, 146, 150];
            let rule = rules[(seed % rules.len() as u32) as usize];
            let fill_ratio = 0.3 + (seed % 50) as f64 / 100.0; // range 0.30 to 0.79
            format!(
                "cellauto=rule={}:size={}x{}:random_seed={}:random_fill_ratio={},scale={}:{}:flags=neighbor",
                rule, width, height, seed, fill_ratio, width, height
            )
        }
    }
}

fn format_duration(secs: f64) -> String {
    if secs == secs.floor() {
        format!("{:.0}", secs)
    } else {
        format!("{:.2}", secs)
    }
}
