use crate::config::{AudioConfig, ImageConfig, VideoConfig, MusicConfig};
use crate::melody;
use crate::ffmpeg;
use rand::Rng;

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

pub fn random_hex(len: usize) -> String {
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyz0123456789".chars().collect();
    (0..len)
        .map(|_| chars[rng.gen_range(0..chars.len())])
        .collect()
}

pub fn format_duration(secs: f64) -> String {
    if secs == secs.floor() {
        format!("{:.0}", secs)
    } else {
        format!("{:.2}", secs)
    }
}

pub fn build_image_filter(content_type: &str, width: u32, height: u32) -> String {
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
        "pattern" => format!(
            "testsrc2=size={}x{},hue=h={}[v]",
            width,
            height,
            (seed % 360) as f32
        ),
        _ /* noise */ => format!(
            "cellauto=rule=18:seed={}:size={}x{}:pattern=random,scale={}:{}:flags=neighbor[v]",
            seed, width, height, width, height
        ),
    }
}

pub fn generate_image(config: &ImageConfig, output_dir: &std::path::Path) -> Result<(), String> {
    let ext = match config.format.as_str() {
        "JPG" | "jpg" => "jpg",
        "WEBP" | "webp" => "webp",
        _ => "png",
    };

    for i in 1..=config.count {
        if get_cancel() {
            return Err("Cancelled".to_string());
        }

        let random_str = random_hex(6);
        let filename = format!("{}_{:03}_{}.{}", config.prefix, i, random_str, ext);
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

        ffmpeg::run_ffmpeg_for_app(None, &args, 30).map_err(|e| format!("{}: {}", filename, e))?;
    }

    Ok(())
}

pub fn generate_audio(config: &AudioConfig, output_dir: &std::path::Path) -> Result<(), String> {
    let channels = if config.channels == "stereo" { "2" } else { "1" };
    let ext = match config.format.as_str() {
        "WAV" | "wav" => "wav",
        "AAC" | "aac" => "aac",
        _ => "mp3",
    };
    let duration_str = format_duration(config.duration);

    for i in 1..=config.count {
        if get_cancel() {
            return Err("Cancelled".to_string());
        }

        let random_str = random_hex(6);
        let filename = format!("{}_{:03}_{}.{}", config.prefix, i, random_str, ext);
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

        ffmpeg::run_ffmpeg_for_app(None, &args, 10).map_err(|e| format!("{}: {}", filename, e))?;
    }

    Ok(())
}

pub fn generate_video(config: &VideoConfig, output_dir: &std::path::Path) -> Result<(), String> {
    let ext = match config.format.as_str() {
        "MOV" | "mov" => "mov",
        "WEBM" | "webm" => "webm",
        _ => "mp4",
    };
    let codec = if config.codec == "h264" { "libx264" } else { "libx265" };
    let duration_str = format_duration(config.duration);

    for i in 1..=config.count {
        if get_cancel() {
            return Err("Cancelled".to_string());
        }

        let random_str = random_hex(6);
        let filename = format!("{}_{:03}_{}.{}", config.prefix, i, random_str, ext);
        let output_path = output_dir.join(&filename);

        let seed: u32 = rand::thread_rng().gen();
        let hue: f32 = rand::thread_rng().gen_range(0.0..360.0);

        let filter = match config.content_type.as_str() {
            "solid" => format!(
                "color=c=0x{:06x}:s={}x{}:d={}[v]",
                (hue / 360.0 * 16777215.0) as u32,
                config.width, config.height, duration_str
            ),
            "gradient" => format!(
                "gradients=s={}x{}:c0=random:c1=random:seed={}:d={}[v]",
                config.width, config.height, seed, duration_str
            ),
            "pattern" => format!(
                "testsrc2=size={}x{}[v]",
                config.width, config.height
            ),
            _ => format!(
                "cellauto=rule=18:seed={}:size={}x{}:pattern=random,scale={}:{}:flags=neighbor[v];[v]framerate=fps={}[v]",
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

        ffmpeg::run_ffmpeg_for_app(None, &args, 30).map_err(|e| format!("{}: {}", filename, e))?;
    }

    Ok(())
}

pub fn generate_music(config: &MusicConfig, output_dir: &std::path::Path) -> Result<(), String> {
    let ext = match config.format.as_str() {
        "WAV" | "wav" => "wav",
        "AAC" | "aac" => "aac",
        _ => "mp3",
    };
    let duration_str = format_duration(config.duration);

    for i in 1..=config.count {
        if get_cancel() {
            return Err("Cancelled".to_string());
        }

        let random_str = random_hex(6);
        let filename = format!("{}_{:03}_{}.{}", config.prefix, i, random_str, ext);
        let output_path = output_dir.join(&filename);

        // 获取旋律模板（优先从音乐库获取）
        let melody = if let Some(notes) = crate::music_library::get_music_by_id(&config.melody) {
            notes
        } else {
            melody::get_melody_by_template(&config.melody)
        };

        // 随机移调 -6 到 +6 半音
        let transpose_semitones = rand::thread_rng().gen_range(-6..=6);
        let transposed = melody::transpose(&melody, transpose_semitones);

        // 如果启用 FluidSynth 且 SoundFont 可用，使用 FluidSynth 渲染
        if config.use_fluidsynth {
            if let Some(soundfont_path) = crate::fluidsynth_render::check_soundfont_exists() {
                let temp_wav = output_dir.join(format!("temp_{}.wav", random_str));

                match crate::fluidsynth_render::render_with_fluidsynth(
                    &transposed,
                    config.bpm,
                    config.duration,
                    &soundfont_path,
                    &temp_wav,
                    44100,
                ) {
                    Ok(_) => {
                        // 如果需要转换格式，使用 FFmpeg
                        if ext != "wav" {
                            let codec = if ext == "aac" { "aac" } else { "libmp3lame" };
                            let args = vec![
                                "-i".to_string(), temp_wav.to_str().unwrap().to_string(),
                                "-acodec".to_string(), codec.to_string(),
                                "-y".to_string(),
                                output_path.to_str().unwrap().to_string(),
                            ];
                            crate::ffmpeg::run_ffmpeg_for_app(None, &args, 30)?;
                            let _ = std::fs::remove_file(&temp_wav);
                        } else {
                            std::fs::rename(&temp_wav, &output_path).map_err(|e| e.to_string())?;
                        }
                        continue; // 成功，跳到下一个文件
                    }
                    Err(e) => {
                        // FluidSynth 失败，回退到 FFmpeg
                        eprintln!("FluidSynth failed: {}, falling back to FFmpeg", e);
                    }
                }
            }
        }

        // 使用 FFmpeg sine 滤镜渲染（默认或回退）

        // 计算每个音符的时长（秒）
        let beat_duration = 60.0 / config.bpm as f64; // 每拍的秒数
        
        // 计算总时长并调整以匹配目标时长
        let total_beats: f32 = transposed.iter().map(|(_, dur)| dur).sum();
        let scale_factor = config.duration / (total_beats as f64 * beat_duration);

        // 构建 FFmpeg 滤镜链，添加谐波、包络和混响增强音色
        let mut filter_parts = Vec::new();

        for (i, (freq, duration)) in transposed.iter().enumerate() {
            let note_duration = (*duration as f64 * beat_duration * scale_factor).max(0.05);

            // 为每个音符创建多个谐波（基频 + 2倍频 + 3倍频），模拟更丰富的音色
            // 添加淡入淡出包络，让音符更自然
            let fade_duration = (note_duration * 0.1).min(0.05); // 淡入淡出时长
            let fade_out_start = note_duration - fade_duration;
            let harmonics = format!(
                "sine=f={}:d={}[h{}0];sine=f={}:d={}[h{}1];sine=f={}:d={}[h{}2];\
                [h{}0][h{}1][h{}2]amix=inputs=3:weights=1.0 0.3 0.15[m{}];\
                [m{}]afade=t=in:st=0:d={}:curve=esin,afade=t=out:st={}:d={}:curve=esin[a{}]",
                freq, note_duration, i,
                freq * 2.0, note_duration, i,
                freq * 3.0, note_duration, i,
                i, i, i, i,
                i, fade_duration,
                fade_out_start, fade_duration, i
            );
            filter_parts.push(harmonics);
        }

        // 使用 concat 滤镜连接所有音符，然后添加混响效果
        let filter = if filter_parts.len() > 1 {
            let concat_inputs: Vec<String> = (0..filter_parts.len())
                .map(|i| format!("[a{}]", i))
                .collect();
            format!("{};{}concat=n={}:v=0:a=1[concat];[concat]aecho=0.8:0.88:60:0.4[out]",
                filter_parts.join(";"),
                concat_inputs.join(""),
                filter_parts.len())
        } else {
            format!("{}[single];[single]aecho=0.8:0.88:60:0.4[out]", filter_parts[0])
        };

        let mut args: Vec<String> = vec![
            "-f".to_string(), "lavfi".to_string(),
            "-i".to_string(), filter,
            "-t".to_string(), duration_str.clone(),
            "-y".to_string(),
        ];

        if ext != "wav" {
            let codec = if ext == "aac" { "aac" } else { "libmp3lame" };
            args.extend_from_slice(&["-acodec".to_string(), codec.to_string()]);
        }

        args.push(output_path.to_str().unwrap().to_string());

        crate::ffmpeg::run_ffmpeg_for_app(None, &args, 30).map_err(|e| format!("{}: {}", filename, e))?;
    }

    Ok(())
}
