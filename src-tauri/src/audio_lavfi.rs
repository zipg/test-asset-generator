//! Lavfi 音频源：随机噪音、简单节奏、随机音符（半音阶步进）。

fn fmt_duration(secs: f64) -> String {
    if secs == secs.floor() {
        format!("{:.0}", secs)
    } else {
        format!("{:.2}", secs)
    }
}

/// 返回可供 `ffmpeg -f lavfi -i ...` 使用的滤镜字符串（多为单声道；立体声可在后续用 `-ac 2` 上混）。
pub fn build_lavfi_audio(
    content: &str,
    duration_secs: f64,
    sample_rate: u32,
    channels: &str,
    seed: u32,
) -> String {
    let dur = fmt_duration(duration_secs);
    let sr = sample_rate;
    let ch = if channels == "stereo" { 2u32 } else { 1u32 };

    match content {
        "noise" => {
            let amplitude = 0.1 + ((seed % 40) as f64 / 100.0);
            format!(
                "anoisesrc=d={}:a={:.3}:r={}:c={}:s={}",
                dur, amplitude, sr, ch, seed
            )
        }
        "rhythm" => {
            // 载波频率与包络频率随 seed 变化，保证每次输出不同（MD5 可区分）
            let f0 = 118.0 + (seed % 72) as f64;
            let env_hz = 1.85 + (seed % 47) as f64 * 0.02;
            format!(
                "aevalsrc=0.18*sin(2*PI*({f0})*t)*(0.52+0.48*sin(2*PI*({env_hz})*t)):d={}:s={}:c=mono",
                dur, sr
            )
        }
        "notes" => {
            // lavfi 中表达式里的逗号会拆成多段滤镜，必须写成 \,
            let m1 = (seed % 12) as f64;
            let m2 = ((seed / 12) % 12) as f64;
            let rate = 2.2 + (seed % 180) as f64 * 0.01;
            format!(
                "aevalsrc=0.14*sin(2*PI*220*pow(1.059463094359\\,mod(floor(t*({rate}))+{m1}+{m2}\\,12))*t):d={}:s={}:c=mono",
                dur, sr
            )
        }
        _ => {
            let amplitude = 0.15 + ((seed % 35) as f64 / 100.0);
            format!(
                "anoisesrc=d={}:a={:.3}:r={}:c={}:s={}",
                dur, amplitude, sr, ch, seed
            )
        }
    }
}

/// 嵌入视频：48kHz；噪音可为双声道，节奏/音符为单声道（需配合 `needs_stereo_upmix_video` 上混）。
pub fn build_lavfi_audio_for_video(content: &str, duration_secs: f64, seed: u32) -> String {
    let ch = if content == "noise" { "stereo" } else { "mono" };
    build_lavfi_audio(content, duration_secs, 48_000, ch, seed)
}

pub fn needs_stereo_upmix_video(content: &str) -> bool {
    matches!(content, "rhythm" | "notes")
}

/// 若 lavfi 为单声道而需要立体声，返回 true（需在编码前加 `-ac 2`）。
pub fn needs_stereo_upmix(content: &str, channels: &str) -> bool {
    channels == "stereo" && (content == "rhythm" || content == "notes")
}
