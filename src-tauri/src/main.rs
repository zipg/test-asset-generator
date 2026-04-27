#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod audio_lavfi;
mod audio_music;
mod config;
mod ffmpeg;
mod generator;
mod melody;
mod music_library;
mod fluidsynth_render;
mod soundfont_manager;
mod process_ext;

use crate::process_ext::command;
use config::AppConfig;
use generator::{get_cancel, reset_cancel, set_cancel, random_hex};
use rand::Rng;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;
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
            if std::env::consts::OS == "macos" {
                let _ = ffmpeg::ensure_macos_bundled_ffmpeg_copied(&app.handle());
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
            generate_music,
            select_save_path,
            open_folder,
            estimate_size,
            download_ffmpeg,
            check_ffmpeg,
            host_os,
            check_soundfont,
            download_soundfont,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Returns `std::env::consts::OS` (e.g. `macos`, `windows`, `linux`) for UI copy.
#[tauri::command]
fn host_os() -> String {
    std::env::consts::OS.to_string()
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

    // macOS: same idea as Windows — if the .app ships ffmpeg, treat as ready without exec probe
    // (GUI/spawn checks can false-negative while the bundled binary is valid).
    if os == "macos" {
        let _ = ffmpeg::ensure_macos_bundled_ffmpeg_copied(&app);
        if ffmpeg::bundled_resource_ffmpeg_exists_mac(&app) {
            return "found".to_string();
        }
    }

    // On macOS, try to use the system "which" command to find FFmpeg
    // This uses the same PATH as the terminal
    if os == "macos" {
        if let Ok(output) = command("/usr/bin/which")
            .arg("ffmpeg")
            .output()
        {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() && path != "ffmpeg not found" {
                // Found it, verify it works
                if let Ok(verify) = command(&path).arg("--version").output() {
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
            if let Ok(verify) = command(&p).arg("--version").output() {
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
                if let Ok(output) = command(p).arg("--version").output() {
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
            if let Ok(output) = command(&downloaded).arg("--version").output() {
                if output.status.success() {
                    return "found".to_string();
                }
            }
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

async fn fetch_network_image_bytes(width: u32, height: u32, crop: bool) -> Result<Vec<u8>, String> {
    let (fetch_w, fetch_h) = if crop {
        ((width as f64 * 1.3).ceil() as u32, (height as f64 * 1.3).ceil() as u32)
    } else {
        (width, height)
    };

    let urls = [
        format!("https://picsum.photos/{}/{}", fetch_w, fetch_h),
        format!("https://loremflickr.com/{}/{}", fetch_w, fetch_h),
        format!("https://random.imagecdn.app/{}/{}", fetch_w, fetch_h),
    ];

    let mut errors: Vec<String> = Vec::new();
    for url in &urls {
        match fetch_url_bytes(url).await {
            Ok(data) => return Ok(data),
            Err(e) => errors.push(format!("{}: {}", url, e)),
        }
    }
    Err(format!("所有网络图源均不可用:\n{}", errors.join("\n")))
}

// Boudoir API 速率限制: 仅 ortlinde 有明确限制 (300s/100次/封30分钟)
static BOUDOIR_TIMESTAMPS: Mutex<Option<Vec<Instant>>> = Mutex::new(None);

async fn fetch_boudoir_image() -> Result<Vec<u8>, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(300))
        .connect_timeout(Duration::from_secs(45))
        .build()
        .map_err(|e| format!("HTTP client: {}", e))?;

    let mut errors: Vec<String> = Vec::new();

    // 主 API: ortlinde (有明确频率限制: 300s/100次/封30分钟)
    let ortlinde_ok = {
        let mut guard = BOUDOIR_TIMESTAMPS.lock().unwrap();
        let timestamps = guard.get_or_insert_with(Vec::new);
        let cutoff = Instant::now() - Duration::from_secs(300);
        timestamps.retain(|t| *t > cutoff);
        if timestamps.len() >= 100 {
            errors.push("ortlinde: 300秒内已达100次上限".to_string());
            false
        } else {
            timestamps.push(Instant::now());
            true
        }
    };

    if ortlinde_ok {
        match client
            .get("https://boudoir.ortlinde.com/random")
            .send()
            .await
        {
            Ok(resp) => match resp.status().as_u16() {
                200 => {
                    return resp.bytes().await
                        .map(|b| b.to_vec())
                        .map_err(|e| format!("读取响应失败: {}", e));
                }
                403 => errors.push("ortlinde: IP已被临时封禁 (403)".to_string()),
                429 => errors.push("ortlinde: 请求过于频繁 (429)".to_string()),
                other => errors.push(format!("ortlinde: HTTP {}", other)),
            },
            Err(e) => errors.push(format!("ortlinde: {}", e)),
        }
    }

    // 备用 API 1: img.api.sld.tw (无已知频率限制)
    match client
        .get("https://img.api.sld.tw/pic?json=h")
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            let body = resp.bytes().await
                .map_err(|e| format!("读取备用API响应失败: {}", e))?;
            let json: serde_json::Value = serde_json::from_slice(&body)
                .map_err(|e| format!("解析备用API响应失败: {}", e))?;
            if let Some(img_url) = json["url"].as_str() {
                match client.get(img_url).send().await {
                    Ok(r) if r.status().is_success() => {
                        return r.bytes().await
                            .map(|b| b.to_vec())
                            .map_err(|e| format!("读取备用图片失败: {}", e));
                    }
                    Ok(r) => errors.push(format!("sld.tw 图片: HTTP {}", r.status())),
                    Err(e) => errors.push(format!("sld.tw 图片: {}", e)),
                }
            } else {
                errors.push("sld.tw: 未返回图片URL".to_string());
            }
        }
        Ok(resp) => errors.push(format!("sld.tw: HTTP {}", resp.status())),
        Err(e) => errors.push(format!("sld.tw: {}", e)),
    }

    Err(format!("所有18+接口均不可用:\n{}", errors.join("\n")))
}

async fn fetch_anime_image() -> Result<Vec<u8>, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(300))
        .connect_timeout(Duration::from_secs(45))
        .build()
        .map_err(|e| format!("HTTP client: {}", e))?;

    let mut errors: Vec<String> = Vec::new();

    // 主: api.neix.in
    match client
        .get("https://api.neix.in/random/mobile")
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            return resp.bytes().await
                .map(|b| b.to_vec())
                .map_err(|e| format!("读取二次元图片失败: {}", e));
        }
        Ok(resp) => errors.push(format!("neix.in: HTTP {}", resp.status())),
        Err(e) => errors.push(format!("neix.in: {}", e)),
    }

    // 备用: app.zichen.zone (302 跳转到图片地址, reqwest 自动跟随)
    match client
        .get("https://app.zichen.zone/api/acg/api.php")
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            return resp.bytes().await
                .map(|b| b.to_vec())
                .map_err(|e| format!("读取备用二次元图片失败: {}", e));
        }
        Ok(resp) => errors.push(format!("zichen.zone: HTTP {}", resp.status())),
        Err(e) => errors.push(format!("zichen.zone: {}", e)),
    }

    Err(format!("所有二次元接口均不可用:\n{}", errors.join("\n")))
}

async fn fetch_url_bytes(url: &str) -> Result<Vec<u8>, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(300))
        .connect_timeout(Duration::from_secs(45))
        .build()
        .map_err(|e| format!("HTTP client: {}", e))?;
    let response = client
        .get(url)
        .send()
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
    if os == "macos" {
        let _ = ffmpeg::ensure_macos_bundled_ffmpeg_copied(&app);
        if ffmpeg::bundled_resource_ffmpeg_exists_mac(&app) {
            return Ok("already_exists".to_string());
        }
    }

    // First check if a valid FFmpeg already exists (must resolve bundle path on macOS).
    let existing_path = ffmpeg::resolve_ffmpeg_executable(Some(&app));
    if let Ok(output) = command(&existing_path).arg("--version").output() {
        if output.status.success() {
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
        ffmpeg::install_mac_ffmpeg_from_download_bytes(&bytes, &ffmpeg_path)?;
    }

    // Verify the downloaded file is executable
    match command(&ffmpeg_path).arg("--version").output() {
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

/// 在系统文件管理器中打开目录（macOS Finder / Windows 资源管理器）。
#[tauri::command]
fn open_folder(path: String) -> Result<(), String> {
    let p = std::path::Path::new(&path);
    if !p.exists() {
        return Err("路径不存在".to_string());
    }
    if !p.is_dir() {
        return Err("不是目录".to_string());
    }
    open::that(&path).map_err(|e| format!("无法打开目录: {}", e))
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
            let image_source = cfg["imageSource"].as_str().unwrap_or("generated");
            if image_source == "network" || image_source == "anime" || image_source == "boudoir" {
                let count = cfg["count"].as_u64().unwrap_or(1);
                let size = count as f64 * 1.0; // ~1 MB per image from network
                return format!("~{:.1} MB (远程)", size.max(0.01));
            }
            let w: u64 = cfg["width"].as_u64().unwrap_or(1080);
            let h: u64 = cfg["height"].as_u64().unwrap_or(1920);
            let count: u64 = cfg["count"].as_u64().unwrap_or(1);
            let format = cfg["format"].as_str().unwrap_or("PNG");
            let bytes_per_pixel = match format {
                "JPG" | "jpg" | "JPEG" | "jpeg" => 0.5,
                "WEBP" | "webp" => 0.3,
                "GIF" | "gif" => 0.4,
                "BMP" | "bmp" => 3.0,
                "TIFF" | "tiff" | "TIF" | "tif" => 4.0,
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
            let add_audio = cfg["addAudioTrack"].as_bool() == Some(true);
            let kbps = (w * h * fps) as f64 * 0.1 / 1000.0;
            let mut size = (duration * kbps * 1000.0 / 8.0) * (count as f64) / 1_048_576.0;
            if add_audio {
                size += (duration * 128_000.0 / 8.0) * (count as f64) / 1_048_576.0;
            }
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
    let output_dir = create_timestamp_dir(&save_path, &config.prefix, None)?;
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
                "JPG" | "jpg" | "JPEG" | "jpeg" => "jpg",
                "WEBP" | "webp" => "webp",
                "GIF" | "gif" => "gif",
                "BMP" | "bmp" => "bmp",
                "TIFF" | "tiff" | "TIF" | "tif" => "tiff",
                _ => "png",
            };
            let filename = format!("{}_{:03}_{}.{}", config.prefix, i, random_str, ext);
            let output_path = output_dir.join(&filename);
            let seed: u32 = unique_seed();

            let gen_result = match config.image_source.as_str() {
                "network" | "anime" | "boudoir" => {
                    let raw_bytes: Vec<u8> = if config.image_source == "boudoir" {
                        fetch_boudoir_image().await?
                    } else if config.image_source == "anime" {
                        fetch_anime_image().await?
                    } else {
                        fetch_network_image_bytes(config.width, config.height, config.crop).await?
                    };

                    // 网络获取和其它默认不指定分辨率, 直接保存原始图片
                    if (config.image_source == "boudoir" || config.image_source == "network" || config.image_source == "anime") && !config.crop {
                        std::fs::write(&output_path, &raw_bytes)
                            .map_err(|e| format!("写入文件失败: {}", e))?;
                        Ok("saved".to_string())
                    } else {
                        let tmp_path = output_dir.join(format!("_tmp_{:06}_{}.jpg", i, seed));
                        std::fs::write(&tmp_path, &raw_bytes)
                            .map_err(|e| format!("写入临时文件失败: {}", e))?;

                        let vf = if config.crop {
                            format!(
                                "scale={}:{}:force_original_aspect_ratio=increase,crop={}:{}",
                                config.width, config.height, config.width, config.height
                            )
                        } else {
                            format!(
                                "scale={}:{}:force_original_aspect_ratio=decrease",
                                config.width, config.height
                            )
                        };

                        let mut args: Vec<String> = vec![
                            "-i".to_string(), tmp_path.to_str().unwrap().to_string(),
                            "-vf".to_string(), vf,
                            "-vframes".to_string(), "1".to_string(),
                            "-y".to_string(),
                        ];
                        match ext {
                            "jpg" => args.extend_from_slice(&["-q:v".to_string(), "2".to_string()]),
                            "webp" => args.extend_from_slice(&["-quality".to_string(), "90".to_string()]),
                            "tiff" => args.extend_from_slice(&["-compression_algo".to_string(), "deflate".to_string()]),
                            _ => {}
                        }
                        args.push(output_path.to_str().unwrap().to_string());

                        let result = ffmpeg::run_ffmpeg_for_app(Some(&app), &args, 60);
                        let _ = std::fs::remove_file(&tmp_path);
                        result
                    }
                }
                _ => {
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
                        "tiff" => args.extend_from_slice(&["-compression_algo".to_string(), "deflate".to_string()]),
                        _ => {}
                    }
                    args.push(output_path.to_str().unwrap().to_string());

                    ffmpeg::run_ffmpeg_for_app(Some(&app), &args, 30)
                }
            };

            match gen_result {
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
                            break;
                        }
                    }
                }
                Err(e) => {
                    failed += 1;
                    errors.push(serde_json::json!({ "file": filename, "error": e }));
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
    let output_dir = create_timestamp_dir(&save_path, &config.prefix, None)?;

    let total = config.count;
    let mut success = 0u32;
    let mut failed = 0u32;
    let mut errors: Vec<serde_json::Value> = Vec::new();

    let ext = match config.format.as_str() {
        "WAV" | "wav" => "wav",
        "AAC" | "aac" => "aac",
        _ => "mp3",
    };

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

            let mut args: Vec<String> = vec!["-y".to_string()];

            if config.audio_content == "random_music" {
                let sines = audio_music::sine_inputs(seed, config.sample_rate);
                for s in sines.iter() {
                    args.push("-f".to_string());
                    args.push("lavfi".to_string());
                    args.push("-i".to_string());
                    args.push(s.clone());
                }
                args.push("-filter_complex".to_string());
                args.push(audio_music::filter_concat_loop_atrim(
                    config.duration,
                    seed,
                    config.sample_rate,
                ));
                args.push("-map".to_string());
                args.push("[aout]".to_string());
            } else {
                let lavfi = audio_lavfi::build_lavfi_audio(
                    &config.audio_content,
                    config.duration,
                    config.sample_rate,
                    &config.channels,
                    seed,
                );
                args.push("-f".to_string());
                args.push("lavfi".to_string());
                args.push("-i".to_string());
                args.push(lavfi);
            }

            if audio_lavfi::needs_stereo_upmix(&config.audio_content, &config.channels) {
                args.extend_from_slice(&["-ac".to_string(), "2".to_string()]);
            }

            if ext != "wav" {
                let codec = if ext == "aac" { "aac" } else { "mp3" };
                args.extend_from_slice(&["-acodec".to_string(), codec.to_string()]);
            }

            if config.audio_content == "random_music" {
                args.extend_from_slice(&["-t".to_string(), format_duration(config.duration)]);
            }

            args.push(output_path.to_str().unwrap().to_string());

            let timeout_audio = if config.audio_content == "random_music" {
                150_u64
            } else {
                30_u64
            };

            match ffmpeg::run_ffmpeg_for_app(Some(&app), &args, timeout_audio) {
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
                            break;
                        }
                    }
                }
                Err(e) => {
                    failed += 1;
                    errors.push(serde_json::json!({ "file": filename, "error": e }));
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
    let output_dir = create_timestamp_dir(&save_path, &config.prefix, Some(content_type_label(&config.content_type)))?;

    let total = config.count;
    let mut success = 0u32;
    let mut failed = 0u32;
    let mut errors: Vec<serde_json::Value> = Vec::new();

    let fmt_upper = config.format.to_ascii_uppercase();
    let ext = match fmt_upper.as_str() {
        "MOV" => "mov",
        "WEBM" => "webm",
        "AVI" => "avi",
        "FLV" => "flv",
        "MKV" => "mkv",
        "3GP" => "3gp",
        _ => "mp4",
    };
    let duration_str = format_duration(config.duration);
    let ct_label = content_type_label(&config.content_type);

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
            let filename = format!("{}_{}_{:03}_{}.{}", config.prefix, ct_label, i, random_str, ext);
            let output_path = output_dir.join(&filename);

            let seed: u32 = unique_seed();

            let speed = config.dynamics as f32 / 5.0;
            let w = config.width.max(2);
            let h = config.height.max(2);
            let odd_dims = w % 2 != 0 || h % 2 != 0;
            let pix_fmt = if odd_dims { "yuv444p" } else { "yuv420p" };
            let f = config.fps;
            let hw = (w / 2).max(2);
            let hh = (h / 2).max(2);
            let seed_phase = (seed % 1000) as f32 * 0.01;
            let filter = match config.content_type.as_str() {
                "solid" => {
                    let color_hue = (seed % 360) as f32;
                    format!(
                        "color=c=0x{:06x}:s={}x{}:d={}",
                        (color_hue / 360.0 * 16777215.0) as u32,
                        w, h, duration_str
                    )
                }
                "gradient" => format!(
                    "gradients=s={}x{}:c0=random:c1=random:seed={}:d={}",
                    w, h, seed, duration_str
                ),
                "pattern" => format!(
                    "testsrc2=size={}x{}:rate={}:duration={},hue=H=t*{}",
                    w, h, f, duration_str,
                    (seed % 180 + 60) as f32 * speed
                ),
                "noise" => format!(
                    "nullsrc=size={}x{}:rate={}:duration={},geq=r='random(X+N+{sd})*255':g='random(Y+N*2+{sd})*255':b='random(X*Y+N*3+{sd})*255',scale={}x{}:flags=bilinear",
                    hw, hh, f, duration_str, w, h,
                    sd = seed,
                ),
                "plasma" => format!(
                    "nullsrc=size={}x{}:rate={}:duration={},geq=r='128+127*sin(X/W*6.283+T*{s}+{sp})*cos(Y/H*6.283+T*{s}*0.7+{sp})':g='128+127*sin((X+Y)/(W+H)*9.425+T*{s}*1.3+{sp})*cos((X-Y)/(W+H)*9.425+T*{s}*0.9+{sp})':b='128+127*cos(X/W*7.854+T*{s}*0.8+{sp})*sin(Y/H*7.854+T*{s}*1.1+{sp})',scale={}x{}:flags=bilinear",
                    hw, hh, f, duration_str, w, h,
                    s = speed * 2.0,
                    sp = seed_phase,
                ),
                "waves" => format!(
                    "nullsrc=size={}x{}:rate={}:duration={},geq=r='128+100*sin(Y/12+T*{s}+{sp})*cos(X/20+{sp})':g='128+100*cos(X/15+T*{s}*1.2+{sp})*sin(Y/18+{sp})':b='128+100*sin((X+Y)/18+T*{s}*1.4+{sp})',scale={}x{}:flags=bilinear",
                    hw, hh, f, duration_str, w, h,
                    s = speed * 2.0,
                    sp = seed_phase,
                ),
                "kaleidoscope" => format!(
                    "nullsrc=size={}x{}:rate={}:duration={},geq=r='128+127*cos(atan2(Y-H/2,X-W/2)*6+T*{s}+{sp})*sin(hypot(X-W/2,Y-H/2)/18+{sp})':g='128+127*cos(atan2(Y-H/2,X-W/2)*6+PI/3*2+T*{s}*0.8+{sp})*sin(hypot(X-W/2,Y-H/2)/18+{sp})':b='128+127*cos(atan2(Y-H/2,X-W/2)*6+PI/3*4+T*{s}*1.1+{sp})*sin(hypot(X-W/2,Y-H/2)/18+{sp})',scale={}x{}:flags=bilinear",
                    hw, hh, f, duration_str, w, h,
                    s = speed * 2.0,
                    sp = seed_phase,
                ),
                "audioviz" => format!(
                    "nullsrc=size={}x{}:rate={}:duration={},geq=r='if(lt(abs(30*X/W-floor(30*X/W)-0.5),0.2*abs(sin(0.5*floor(30*X/W)+T*{s}+{sp}))+0.03),255,0)':g='if(lt(abs(30*X/W-floor(30*X/W)-0.5),0.2*abs(cos(0.6*floor(30*X/W)+T*{s}*1.2+{sp}))+0.03),100,0)':b='if(lt(abs(30*X/W-floor(30*X/W)-0.5),0.2*abs(sin(0.7*floor(30*X/W)+T*{s}*1.5+{sp}))+0.03),40,0)',scale={}x{}:flags=bilinear",
                    hw, hh, f, duration_str, w, h,
                    s = speed * 2.0,
                    sp = seed_phase,
                ),
                _ => {
                    let rules = [18u32, 22, 26, 30, 34, 38, 42, 46, 50, 54, 58, 62, 66, 70, 74, 78, 82, 86, 90, 94, 98, 102, 106, 110, 114, 118, 122, 126, 130, 134, 138, 142, 146, 150];
                    let rule = rules[(seed % rules.len() as u32) as usize];
                    let fill_ratio = 0.3 + (seed % 50) as f64 / 100.0;
                    format!(
                        "cellauto=rule={}:size={}x{}:random_seed={}:random_fill_ratio={},scale={}:{}:flags=neighbor",
                        rule, w, h, seed, fill_ratio, w, h
                    )
                },
            };

            let fps_str = config.fps.to_string();
            let vcodec = match fmt_upper.as_str() {
                "WEBM" => "libvpx-vp9",
                "FLV" | "3GP" => "libx264",
                _ => {
                    if config.codec == "hevc" {
                        "libx265"
                    } else {
                        "libx264"
                    }
                }
            };

            let mut args: Vec<String> = vec!["-y".to_string()];
            let mut temp_wav: Option<std::path::PathBuf> = None;

            let add_audio = config.add_audio_track && config.audio_engine != "none";
            if add_audio {
                if config.audio_engine == "fluidsynth" {
                    // 真实乐器: 生成临时 WAV，再与视频合并
                    temp_wav = Some(output_dir.join(format!("vida_{}.wav", random_hex(4))));
                    let all_music = crate::music_library::get_all_music();
                    let piece = &all_music[(seed as usize) % all_music.len()];
                    let melody_notes = (piece.notes)();
                    let sf_path = crate::fluidsynth_render::check_soundfont_exists(&app)
                        .ok_or_else(|| "SoundFont 不可用".to_string())?;
                    let inst = if config.audio_content == "random" {
                        crate::fluidsynth_render::random_instrument().0
                    } else {
                        config.audio_content.parse::<u8>().unwrap_or(0)
                    };
                    crate::fluidsynth_render::render_with_fluidsynth(
                        &melody_notes,
                        120,
                        config.duration,
                        &sf_path,
                        &temp_wav.as_ref().unwrap(),
                        22050,
                        inst,
                        true,
                        true,
                        0.0,
                    )?;
                    args.extend_from_slice(&[
                        "-f".to_string(), "lavfi".to_string(),
                        "-i".to_string(), filter.clone(),
                        "-i".to_string(), temp_wav.as_ref().unwrap().to_str().unwrap().to_string(),
                        "-map".to_string(), "0:v".to_string(),
                        "-map".to_string(), "1:a".to_string(),
                        "-shortest".to_string(),
                        "-ac".to_string(), "2".to_string(),
                    ]);
                } else {
                    // 简易合成: 使用正弦波混合
                    let sines = audio_music::sine_inputs(seed, 48_000);
                    args.extend_from_slice(&[
                        "-f".to_string(),
                        "lavfi".to_string(),
                        "-i".to_string(),
                        filter.clone(),
                    ]);
                    for s in sines.iter() {
                        args.push("-f".to_string());
                        args.push("lavfi".to_string());
                        args.push("-i".to_string());
                        args.push(s.clone());
                    }
                    args.push("-filter_complex".to_string());
                    args.push(audio_music::filter_video_music_track(
                        config.duration,
                        seed,
                        48_000,
                    ));
                    args.extend_from_slice(&[
                        "-map".to_string(),
                        "0:v".to_string(),
                        "-map".to_string(),
                        "[mus]".to_string(),
                        "-ac".to_string(), "2".to_string(),
                    ]);
                }
                match vcodec {
                    "libvpx-vp9" => {
                        args.extend_from_slice(&[
                            "-c:v".to_string(),
                            "libvpx-vp9".to_string(),
                            "-b:v".to_string(),
                            "0".to_string(),
                            "-crf".to_string(),
                            "35".to_string(),
                            "-row-mt".to_string(),
                            "1".to_string(),
                        ]);
                    }
                    "libx265" => {
                        args.extend_from_slice(&[
                            "-c:v".to_string(),
                            "libx265".to_string(),
                            "-preset".to_string(),
                            "fast".to_string(),
                        ]);
                    }
                    _ => {
                        args.extend_from_slice(&[
                            "-c:v".to_string(),
                            "libx264".to_string(),
                            "-preset".to_string(),
                            "fast".to_string(),
                        ]);
                    }
                }
                args.push("-r".to_string());
                args.push(fps_str.clone());
                let aenc = match fmt_upper.as_str() {
                    "WEBM" => "libopus",
                    _ => "aac",
                };
                args.extend_from_slice(&[
                    "-c:a".to_string(),
                    aenc.to_string(),
                    "-b:a".to_string(),
                    "128k".to_string(),
                    "-pix_fmt".to_string(),
                    pix_fmt.to_string(),
                ]);
                if odd_dims && vcodec == "libvpx-vp9" {
                    args.extend_from_slice(&["-profile:v".to_string(), "1".to_string()]);
                }
                if matches!(fmt_upper.as_str(), "MP4" | "MOV" | "3GP") {
                    args.extend_from_slice(&["-movflags".to_string(), "+faststart".to_string()]);
                }
                if config.audio_engine == "simple" {
                    args.extend_from_slice(&["-t".to_string(), duration_str.clone()]);
                }
            } else {
                args.extend_from_slice(&["-f".to_string(), "lavfi".to_string(), "-i".to_string(), filter]);
                match vcodec {
                    "libvpx-vp9" => {
                        args.extend_from_slice(&[
                            "-c:v".to_string(),
                            "libvpx-vp9".to_string(),
                            "-b:v".to_string(),
                            "0".to_string(),
                            "-crf".to_string(),
                            "35".to_string(),
                            "-row-mt".to_string(),
                            "1".to_string(),
                        ]);
                    }
                    "libx265" => {
                        args.extend_from_slice(&[
                            "-c:v".to_string(),
                            "libx265".to_string(),
                            "-preset".to_string(),
                            "fast".to_string(),
                        ]);
                    }
                    _ => {
                        args.extend_from_slice(&[
                            "-c:v".to_string(),
                            "libx264".to_string(),
                            "-preset".to_string(),
                            "fast".to_string(),
                        ]);
                    }
                }
                args.extend_from_slice(&[
                    "-r".to_string(),
                    fps_str.clone(),
                    "-t".to_string(),
                    duration_str.clone(),
                    "-pix_fmt".to_string(),
                    pix_fmt.to_string(),
                ]);
                if odd_dims && vcodec == "libvpx-vp9" {
                    args.extend_from_slice(&["-profile:v".to_string(), "1".to_string()]);
                }
                if matches!(fmt_upper.as_str(), "MP4" | "MOV" | "3GP") {
                    args.extend_from_slice(&["-movflags".to_string(), "+faststart".to_string()]);
                }
            }

            args.push(output_path.to_str().unwrap().to_string());

            let timeout_secs = {
                let base = if vcodec.contains("vpx") {
                    90.0
                } else {
                    45.0
                };
                let extra = if config.add_audio_track && config.audio_engine != "none" {
                    90.0
                } else {
                    0.0
                };
                let t = base + config.duration * 3.0 + extra;
                t.min(900.0).max(25.0) as u64
            };

            match ffmpeg::run_ffmpeg_for_app(Some(&app), &args, timeout_secs) {
                Ok(_) => {
                    // Check MD5 for uniqueness
                    match file_md5(&output_path) {
                        Ok(md5_hash) => {
                            if seen_md5s.contains(&md5_hash) {
                                // Duplicate MD5, retry with new seed
                                retries += 1;
                                let _ = std::fs::remove_file(&output_path);
                                if let Some(ref p) = temp_wav { let _ = std::fs::remove_file(p); }
                                continue;
                            }
                            seen_md5s.insert(md5_hash);
                            generated = true;
                            if let Some(ref p) = temp_wav { let _ = std::fs::remove_file(p); }
                            break;
                        }
                        Err(e) => {
                            failed += 1;
                            errors.push(serde_json::json!({ "file": filename, "error": format!("MD5 check failed: {}", e) }));
                            if let Some(ref p) = temp_wav { let _ = std::fs::remove_file(p); }
                            break;
                        }
                    }
                }
                Err(e) => {
                    failed += 1;
                    errors.push(serde_json::json!({ "file": filename, "error": e }));
                    if let Some(ref p) = temp_wav { let _ = std::fs::remove_file(p); }
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

fn content_type_label(ct: &str) -> &str {
    match ct {
        "solid" => "纯色",
        "gradient" => "渐变",
        "pattern" => "彩条图案",
        "noise" => "元胞噪声",
        "plasma" => "等离子动态",
        "waves" => "波纹律动",
        "kaleidoscope" => "万花筒",
        "audioviz" => "音频可视化",
        _ => ct,
    }
}

fn create_timestamp_dir(base: &str, prefix: &str, content_type: Option<&str>) -> Result<std::path::PathBuf, String> {
    // 使用系统本地时区（国内 Mac 一般为北京时间）；格式为 MMDD_HHmmss
    let stamp = chrono::Local::now().format("%m%d_%H%M%S").to_string();
    let dir_name = if let Some(ct) = content_type {
        format!("{}_{}_{}", prefix, ct, stamp)
    } else {
        format!("{}_{}", prefix, stamp)
    };
    let dir = std::path::PathBuf::from(base).join(&dir_name);
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create directory: {}", e))?;
    Ok(dir)
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
        // testsrc2 alone is identical every run; hue shifts bars so each seed yields a distinct image (unique MD5).
        "pattern" => format!(
            "testsrc2=size={}x{},hue=h={}",
            width,
            height,
            (seed % 360) as f32
        ),
        "noise" => format!(
            "nullsrc=size={}x{}:rate=1,geq=r='random(X+{s})*255':g='random(Y+{s}*2)*255':b='random(X*Y+{s}*3)*255'",
            width, height,
            s = seed,
        ),
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

#[tauri::command]
async fn generate_music(
    app: tauri::AppHandle,
    config: config::MusicConfig,
    save_path: String,
) -> Result<serde_json::Value, String> {
    reset_cancel();
    let output_dir = create_timestamp_dir(&save_path, &config.prefix, None)?;

    let total = config.count;
    let mut success = 0u32;
    let mut failed = 0u32;
    let mut errors: Vec<serde_json::Value> = Vec::new();

    let start_time = std::time::Instant::now();

    for i in 1..=total {
        if get_cancel() {
            break;
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        let avg_time_per_file = if success + failed > 0 {
            elapsed / (success + failed) as f64
        } else {
            0.0
        };
        let remaining = total - (success + failed);
        let estimated_remaining = avg_time_per_file * remaining as f64;

        let _ = app.emit(
            "generation-progress",
            serde_json::json!({
                "current": success + failed,
                "total": total,
                "currentFile": format!("{}_{:03}", config.prefix, i),
                "estimatedRemainingSecs": estimated_remaining,
            }),
        );

        match generator::generate_single_music(&app, &config, &output_dir, i) {
            Ok(_) => success += 1,
            Err(e) => {
                failed += 1;
                errors.push(serde_json::json!({
                    "file": format!("{}_{:03}", config.prefix, i),
                    "error": e,
                }));
            }
        }
    }

    Ok(serde_json::json!({
        "success": success,
        "failed": failed,
        "errors": errors,
    }))
}

#[tauri::command]
fn check_soundfont(app: tauri::AppHandle) -> String {
    if soundfont_manager::is_soundfont_downloaded(&app) {
        "found".to_string()
    } else {
        "not_found".to_string()
    }
}

#[tauri::command]
fn download_soundfont(app: tauri::AppHandle) -> Result<String, String> {
    soundfont_manager::download_soundfont(&app)
}
