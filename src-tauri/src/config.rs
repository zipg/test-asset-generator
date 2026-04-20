use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::Manager;

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
    /// noise | rhythm | notes
    #[serde(default = "default_audio_content")]
    pub audio_content: String,
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
    #[serde(default)]
    pub add_audio_track: bool,
    /// 与音频 tab 一致：noise | rhythm | notes
    #[serde(default = "default_audio_content")]
    pub audio_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    /// Bumps when defaults change so we can migrate persisted JSON once.
    #[serde(default)]
    pub schema_version: u32,
    pub save_path: Option<String>,
    pub image_config: ImageConfig,
    pub audio_config: AudioConfig,
    pub video_config: VideoConfig,
}

fn default_audio_content() -> String {
    "notes".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            schema_version: 1,
            save_path: None,
            image_config: ImageConfig {
                format: "PNG".to_string(),
                width: 720,
                height: 1280,
                content_type: "gradient".to_string(),
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
                audio_content: "notes".to_string(),
            },
            video_config: VideoConfig {
                format: "MP4".to_string(),
                codec: "h264".to_string(),
                width: 720,
                height: 1280,
                fps: 30,
                duration: 30.0,
                content_type: "gradient".to_string(),
                count: 10,
                prefix: "测试视频".to_string(),
                add_audio_track: false,
                audio_content: "notes".to_string(),
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

/// Old installs defaulted image/video content to `noise`; product default is now `gradient`.
/// Old `config.json` had no `schemaVersion` (deserializes as 0): bump to 1 and persist once.
fn migrate_schema_version(cfg: &mut AppConfig) -> bool {
    if cfg.schema_version >= 1 {
        return false;
    }
    if cfg.image_config.content_type == "noise" {
        cfg.image_config.content_type = "gradient".to_string();
    }
    if cfg.video_config.content_type == "noise" {
        cfg.video_config.content_type = "gradient".to_string();
    }
    cfg.schema_version = 1;
    true
}

pub fn load_config(app_handle: &tauri::AppHandle) -> AppConfig {
    let path = config_path(app_handle);
    if path.exists() {
        let contents = fs::read_to_string(&path).expect("Failed to read config");
        let mut cfg: AppConfig = serde_json::from_str(&contents).unwrap_or_default();
        if migrate_schema_version(&mut cfg) {
            save_config(app_handle, &cfg);
        }
        cfg
    } else {
        AppConfig::default()
    }
}

pub fn save_config(app_handle: &tauri::AppHandle, config: &AppConfig) {
    let path = config_path(app_handle);
    let mut out = config.clone();
    // Frontend may omit `schemaVersion` in JSON; avoid downgrading a migrated file back to 0.
    if out.schema_version == 0 && path.exists() {
        if let Ok(contents) = fs::read_to_string(&path) {
            if let Ok(prev) = serde_json::from_str::<AppConfig>(&contents) {
                if prev.schema_version >= 1 {
                    out.schema_version = prev.schema_version;
                }
            }
        }
    }
    let contents = serde_json::to_string_pretty(&out).expect("Failed to serialize config");
    fs::write(&path, contents).expect("Failed to write config");
}
