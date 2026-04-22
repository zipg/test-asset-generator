/// SoundFont 下载和管理模块
/// 优先使用内置的 SoundFont，必要时复制到用户数据目录

use std::path::PathBuf;
use tauri::path::BaseDirectory;
use tauri::AppHandle;
use tauri::Manager;

/// 获取 SoundFont 存储目录
pub fn get_soundfont_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|p| p.join("Muse_Generator").join("soundfont"))
}

/// 获取内置 SoundFont 路径（通过 AppHandle 解析）
fn get_bundled_soundfont_path(app: &AppHandle) -> Option<PathBuf> {
    // Tauri v2 使用 app.path().resolve() 解析资源路径
    for rel in ["default.sf3", "resources/default.sf3"] {
        if let Ok(p) = app.path().resolve(rel, BaseDirectory::Resource) {
            if p.exists() && p.metadata().map(|m| m.len() > 1_000_000).unwrap_or(false) {
                return Some(p);
            }
        }
    }
    None
}

/// 检查 SoundFont 是否可用（内置或已下载）
pub fn is_soundfont_downloaded(app: &AppHandle) -> bool {
    // 首先检查内置资源
    if get_bundled_soundfont_path(app).is_some() {
        return true;
    }
    // 然后检查用户目录
    if let Some(dir) = get_soundfont_dir() {
        let sf3_path = dir.join("default.sf3");
        if sf3_path.exists() && sf3_path.metadata().map(|m| m.len() > 1_000_000).unwrap_or(false) {
            return true;
        }
        let sf2_path = dir.join("default.sf2");
        sf2_path.exists() && sf2_path.metadata().map(|m| m.len() > 1_000_000).unwrap_or(false)
    } else {
        false
    }
}

/// 获取 SoundFont 路径（如果存在）
pub fn get_soundfont_path(app: &AppHandle) -> Option<PathBuf> {
    // 首先检查内置资源
    if let Some(p) = get_bundled_soundfont_path(app) {
        return Some(p);
    }
    // 然后检查用户目录
    if let Some(dir) = get_soundfont_dir() {
        let sf3_path = dir.join("default.sf3");
        if sf3_path.exists() && sf3_path.metadata().map(|m| m.len() > 1_000_000).unwrap_or(false) {
            return Some(sf3_path);
        }
        let sf2_path = dir.join("default.sf2");
        if sf2_path.exists() && sf2_path.metadata().map(|m| m.len() > 1_000_000).unwrap_or(false) {
            return Some(sf2_path);
        }
    }
    None
}

/// 下载 SoundFont 文件（保留用于手动下载场景）
pub fn download_soundfont(app: &AppHandle) -> Result<String, String> {
    // 如果内置资源存在，不需要下载
    if get_bundled_soundfont_path(app).is_some() {
        return Ok("使用内置 SoundFont".to_string());
    }

    let Some(dir) = get_soundfont_dir() else {
        return Err("无法获取应用数据目录".to_string());
    };

    std::fs::create_dir_all(&dir).map_err(|e| format!("创建目录失败: {}", e))?;

    let sf_path = dir.join("default.sf3");

    // 如果已存在且大小正常，跳过下载
    if sf_path.exists() {
        if let Ok(metadata) = sf_path.metadata() {
            if metadata.len() > 1_000_000 {
                return Ok("SoundFont 已存在".to_string());
            }
        }
    }

    // 下载 SoundFont
    let url = "https://ftp.osuosl.org/pub/musescore/soundfont/MuseScore_General/MuseScore_General.sf3";

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {}", e))?;

    let mut response = client
        .get(url)
        .send()
        .map_err(|e| format!("下载失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("下载失败: HTTP {}", response.status()));
    }

    let mut file = std::fs::File::create(&sf_path)
        .map_err(|e| format!("创建文件失败: {}", e))?;

    std::io::copy(&mut response, &mut file)
        .map_err(|e| format!("写入文件失败: {}", e))?;

    Ok(format!("SoundFont 下载成功: {}", sf_path.display()))
}
