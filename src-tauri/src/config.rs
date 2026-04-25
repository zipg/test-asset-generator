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
    /// "generated" | "network" | "boudoir"
    #[serde(default = "default_image_source")]
    pub image_source: String,
    #[serde(default = "default_crop")]
    pub crop: bool,
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
    /// noise | rhythm | notes | random_music
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
    #[serde(default = "default_add_audio_track")]
    pub add_audio_track: bool,
    /// 画面动态强度 1-10
    #[serde(default = "default_video_dynamics")]
    pub dynamics: u32,
    /// 音频引擎: "none" | "simple" | "fluidsynth"
    #[serde(default = "default_audio_engine")]
    pub audio_engine: String,
    /// @deprecated 保留旧字段以兼容
    #[serde(default = "default_audio_content")]
    pub audio_content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MusicConfig {
    pub format: String,
    pub duration: f64,
    pub bpm: u32,
    pub melody: String,
    pub count: u32,
    pub prefix: String,
    pub use_fluidsynth: bool,
    /// "fluidsynth" | "simple"
    #[serde(default = "default_sound_engine")]
    pub sound_engine: String,
    /// "random" 或 GM 乐器 program_id 字符串
    #[serde(default = "default_instrument")]
    pub instrument: String,
    #[serde(default = "default_enable_drums")]
    pub enable_drums: bool,
    #[serde(default = "default_enable_harmony")]
    pub enable_harmony: bool,
    #[serde(default)]
    pub gain_db: f64, // 音量增益 0–10 dB
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
    #[serde(default = "default_music_config")]
    pub music_config: MusicConfig,
}

fn default_audio_engine() -> String {
    "fluidsynth".to_string()
}

fn default_audio_content() -> String {
    "random_music".to_string()
}

fn default_add_audio_track() -> bool {
    true
}

fn default_sound_engine() -> String {
    "fluidsynth".to_string()
}

fn default_instrument() -> String {
    "random".to_string()
}

fn default_enable_drums() -> bool {
    true
}

fn default_enable_harmony() -> bool {
    true
}

fn default_video_dynamics() -> u32 {
    5
}

fn default_image_source() -> String {
    "generated".to_string()
}

fn default_crop() -> bool {
    true
}

fn default_music_config() -> MusicConfig {
    MusicConfig {
        format: "MP3".to_string(),
        duration: 30.0,
        bpm: 120,
        melody: "random".to_string(),
        count: 10,
        prefix: "音乐".to_string(),
        use_fluidsynth: true,
        sound_engine: "fluidsynth".to_string(),
        instrument: "random".to_string(),
        enable_drums: true,
        enable_harmony: true,
        gain_db: 0.0,
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            schema_version: 3,
            save_path: None,
            image_config: ImageConfig {
                format: "PNG".to_string(),
                width: 720,
                height: 1280,
                content_type: "gradient".to_string(),
                count: 10,
                prefix: "测试图片".to_string(),
                image_source: "generated".to_string(),
                crop: true,
            },
            audio_config: AudioConfig {
                format: "MP3".to_string(),
                duration: 60.0,
                sample_rate: 44100,
                channels: "mono".to_string(),
                count: 10,
                prefix: "测试音频".to_string(),
                audio_content: "random_music".to_string(),
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
                add_audio_track: true,
                audio_content: "random_music".to_string(),
                audio_engine: "fluidsynth".to_string(),
                dynamics: 5,
            },
            music_config: default_music_config(),
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
