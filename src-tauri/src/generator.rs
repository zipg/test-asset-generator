use crate::config::{AudioConfig, ImageConfig, VideoConfig};
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
        "pattern" => format!("testsrc2=size={}x{}[v]", width, height),
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

        ffmpeg::run_ffmpeg(&args).map_err(|e| format!("{}: {}", filename, e))?;
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

        ffmpeg::run_ffmpeg(&args).map_err(|e| format!("{}: {}", filename, e))?;
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
        let filename = format!("{}_{}_{:03}.{}", config.prefix, random_str, i, ext);
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

        ffmpeg::run_ffmpeg(&args).map_err(|e| format!("{}: {}", filename, e))?;
    }

    Ok(())
}
