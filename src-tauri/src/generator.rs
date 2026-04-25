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
    let w = config.width;
    let h = config.height;
    let fps = config.fps;
    // 画面动态 → 速度系数 0.2~2.0
    let dyn_speed = config.dynamics as f64 / 5.0;

    for i in 1..=config.count {
        if get_cancel() {
            return Err("Cancelled".to_string());
        }

        let random_str = random_hex(6);
        let filename = format!("{}_{:03}_{}.{}", config.prefix, i, random_str, ext);
        let output_path = output_dir.join(&filename);
        let seed: u32 = rand::thread_rng().gen();

        // —— 音频可视化：生成临时音频 + showwaves ——
        if config.content_type == "audioviz" {
            let temp_audio = output_dir.join(format!("viza_{}.wav", random_str));
            // 用 FFmpeg 多正弦波混合快速生成音频
            let audio_filter = format!(
                "sine=f=261.63:r=44100:d={dur},sine=f=329.63:r=44100:d={dur},sine=f=392.00:r=44100:d={dur},amix=inputs=3:duration=first",
                dur = duration_str
            );
            let audio_args: Vec<String> = vec![
                "-f".to_string(), "lavfi".to_string(),
                "-i".to_string(), audio_filter,
                "-ac".to_string(), "1".to_string(),
                "-y".to_string(),
                temp_audio.to_str().unwrap().to_string(),
            ];
            ffmpeg::run_ffmpeg_for_app(None, &audio_args, 15)?;

            let viz_args: Vec<String> = vec![
                "-i".to_string(), temp_audio.to_str().unwrap().to_string(),
                "-filter_complex".to_string(), format!(
                    "[0:a]showwaves=s={w}x{h}:mode=cline:rate={fps}:scale=sqrt:colors=random,format=yuv420p[v]",
                ),
                "-map".to_string(), "[v]".to_string(),
                "-c:v".to_string(), codec.to_string(),
                "-pix_fmt".to_string(), "yuv420p".to_string(),
                "-t".to_string(), duration_str.clone(),
                "-y".to_string(),
                output_path.to_str().unwrap().to_string(),
            ];
            ffmpeg::run_ffmpeg_for_app(None, &viz_args, 30)
                .map_err(|e| format!("{}: {}", filename, e))?;
            let _ = std::fs::remove_file(&temp_audio);
            continue;
        }

        // —— 其他类型：lavfi 滤镜链 ——
        let filter = match config.content_type.as_str() {
            "gradient" => format!(
                "gradients=s={w}x{h}:c0=random:c1=random:seed={seed}:d={dur},setpts={sp}*PTS[v]",
                dur = duration_str, sp = 1.0 / dyn_speed
            ),
            "pattern" => format!(
                "testsrc2=size={w}x{h},setpts={sp}*PTS[v]",
                sp = 1.0 / dyn_speed
            ),
            "plasma" => format!(
                "geq=r='128+127*sin(X/60+T*{d}*2)*cos(Y/50+T*{d}*1.7)':g='128+127*sin(Y/70+T*{d}*2.2)*cos(X/55+T*{d}*1.5)':b='128+127*cos(X/65+T*{d}*1.9)*sin(Y/45+T*{d}*2.3)',format=yuv420p,scale={w}:{h}[v]",
                d = dyn_speed
            ),
            "waves" => format!(
                "geq=r='128+120*sin(Y/20+T*{d}*2)':g='128+120*sin(X/25+T*{d}*2.5)':b='200+55*sin((X+Y)/30+T*{d}*3)',format=yuv420p,scale={w}:{h}[v]",
                d = dyn_speed
            ),
            "kaleidoscope" => format!(
                "geq=r='128+127*sin(X/35+T*{d})*cos(Y/35+T*{d})':g='128+127*sin(Y/35+T*{d}+1.571)*cos(X/35+T*{d}+0.785)':b='128+127*cos((X+Y)/45+T*{d}+3.142)*sin(abs(X-Y)/45+T*{d}+1.047)',format=yuv420p[w];[w]split=2[a][b];[a]hflip[c];[b][c]hstack[top];[top]split=2[p][q];[q]vflip[r];[p][r]vstack[v]",
                d = dyn_speed
            ),
            "fractal" => {
                let src = if seed % 2 == 0 { "mandelbrot" } else { "julia" };
                format!(
                    "{src}=rate={fps}:size={w}x{h},zoompan=z='zoom+0.002*{d}':d=125:x='iw/2':y='ih/2',fps={fps},setpts=PTS[vs];[vs]scale={w}:{h}[v]",
                    d = dyn_speed
                )
            },
            "life" => format!(
                "life=size={w}x{h}:rate={fps}:mold=10:ratio=0.5:death_color=MidnightBlue:life_color=white:seed={seed},fps={fps},setpts={sp}*PTS[v]",
                sp = 1.0 / dyn_speed
            ),
            // noise (cellauto) / fallback
            _ => format!(
                "cellauto=rule=18:seed={seed}:size={w}x{h}:pattern=random,scale={w}:{h}:flags=neighbor,setpts={sp}*PTS,fps={fps}[v]",
                sp = 1.0 / dyn_speed
            ),
        };

        let fps_str = fps.to_string();
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

/// 生成单个音乐文件
pub fn generate_single_music(
    app: &tauri::AppHandle,
    config: &MusicConfig,
    output_dir: &std::path::Path,
    file_index: u32,
) -> Result<(), String> {
    let ext = match config.format.as_str() {
        "WAV" | "wav" => "wav",
        "AAC" | "aac" => "aac",
        _ => "mp3",
    };

    if get_cancel() {
        return Err("Cancelled".to_string());
    }

    let random_str = random_hex(6);
    let filename = format!("{}_{:03}_{}.{}", config.prefix, file_index, random_str, ext);
    let output_path = output_dir.join(&filename);

    // 获取旋律：从音乐库选择真实完整乐谱
    let melody = if config.melody == "random" || config.melody == "library" {
        let all_music = crate::music_library::get_all_music();
        if !all_music.is_empty() {
            let idx = rand::thread_rng().gen_range(0..all_music.len());
            (all_music[idx].notes)()
        } else {
            melody::get_melody_by_template("random")
        }
    } else if let Some(theme) = crate::music_library::get_music_by_id(&config.melody) {
        theme
    } else {
        melody::get_melody_by_template(&config.melody)
    };

    // 随机移调 -6 到 +6 半音
    let transpose_semitones = rand::thread_rng().gen_range(-6..=6);
    let transposed = melody::transpose(&melody, transpose_semitones);

    // 随机调整 BPM（±20%）
    let bpm_variation = rand::thread_rng().gen_range(0.8..1.2);
    let actual_bpm = (config.bpm as f64 * bpm_variation) as u32;

    // 如果使用 FluidSynth 引擎且 SoundFont 可用，使用 FluidSynth 渲染
    if config.sound_engine == "fluidsynth" {
        if let Some(soundfont_path) = crate::fluidsynth_render::check_soundfont_exists(app) {
            let temp_wav = output_dir.join(format!("temp_{}.wav", random_str));

            // 乐器选择
            let instrument: u8 = if config.instrument == "random" {
                crate::fluidsynth_render::random_instrument().0
            } else {
                config.instrument.parse().unwrap_or(0)
            };

            match crate::fluidsynth_render::render_with_fluidsynth(
                &transposed,
                actual_bpm,
                config.duration,
                &soundfont_path,
                &temp_wav,
                44100,
                instrument,
                config.enable_drums,
                config.enable_harmony,
                config.gain_db,
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
                    return Ok(()); // 成功
                }
                Err(e) => {
                    // FluidSynth 失败，回退到 FFmpeg
                    eprintln!("FluidSynth failed: {}, falling back to FFmpeg", e);
                }
            }
        }
    }

    // 使用 FFmpeg sine 滤镜渲染（默认或回退）
    let beat_duration = 60.0 / actual_bpm as f64;
    let total_beats: f32 = transposed.iter().map(|(_, dur)| dur).sum();
    let scale_factor = config.duration / (total_beats as f64 * beat_duration);

    let mut filter_parts = Vec::new();

    for (i, (freq, duration)) in transposed.iter().enumerate() {
        let note_duration = (*duration as f64 * beat_duration * scale_factor).max(0.05);
        let fade_duration = (note_duration * 0.1).min(0.05);
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

    let duration_str = format_duration(config.duration);
    let mut args: Vec<String> = vec![
        "-f".to_string(), "lavfi".to_string(),
        "-i".to_string(), filter,
        "-t".to_string(), duration_str,
        "-y".to_string(),
    ];

    if ext != "wav" {
        let codec = if ext == "aac" { "aac" } else { "libmp3lame" };
        args.extend_from_slice(&["-acodec".to_string(), codec.to_string()]);
    }

    args.push(output_path.to_str().unwrap().to_string());

    crate::ffmpeg::run_ffmpeg_for_app(None, &args, 30).map_err(|e| format!("{}: {}", filename, e))?;

    Ok(())
}

/// 生成多个音乐文件（保留用于批量生成）
pub fn generate_music(app: &tauri::AppHandle, config: &MusicConfig, output_dir: &std::path::Path) -> Result<(), String> {
    for i in 1..=config.count {
        generate_single_music(app, config, output_dir, i)?;
    }
    Ok(())
}
