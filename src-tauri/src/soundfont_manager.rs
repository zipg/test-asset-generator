/// SoundFont 下载和管理模块

use std::path::PathBuf;

/// 获取 SoundFont 存储目录
pub fn get_soundfont_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|p| p.join("Muse_Generator").join("soundfont"))
}

/// 检查 SoundFont 是否已下载
pub fn is_soundfont_downloaded() -> bool {
    if let Some(dir) = get_soundfont_dir() {
        // 检查 SF3 格式（MuseScore_General.sf3）
        let sf3_path = dir.join("default.sf3");
        if sf3_path.exists() && sf3_path.metadata().map(|m| m.len() > 1_000_000).unwrap_or(false) {
            return true;
        }
        // 兼容 SF2 格式
        let sf2_path = dir.join("default.sf2");
        sf2_path.exists() && sf2_path.metadata().map(|m| m.len() > 1_000_000).unwrap_or(false)
    } else {
        false
    }
}

/// 下载 SoundFont 文件
/// 使用 FluidR3_GM.sf2 (约 140MB)
pub fn download_soundfont() -> Result<String, String> {
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
    // 使用 MuseScore 的 GeneralUser GS (约 30MB，比 FluidR3 小)
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

/// 获取 SoundFont 路径（如果存在）
pub fn get_soundfont_path() -> Option<PathBuf> {
    if let Some(dir) = get_soundfont_dir() {
        // 优先使用 SF3 格式
        let sf3_path = dir.join("default.sf3");
        if sf3_path.exists() && sf3_path.metadata().map(|m| m.len() > 1_000_000).unwrap_or(false) {
            return Some(sf3_path);
        }
        // 兼容 SF2 格式
        let sf2_path = dir.join("default.sf2");
        if sf2_path.exists() && sf2_path.metadata().map(|m| m.len() > 1_000_000).unwrap_or(false) {
            return Some(sf2_path);
        }
    }
    None
}
