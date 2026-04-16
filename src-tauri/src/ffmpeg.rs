use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

pub fn get_ffmpeg_path() -> PathBuf {
    let os = std::env::consts::OS;
    let exe_name = if os == "windows" { "ffmpeg.exe" } else { "ffmpeg" };

    let homebrew_paths = [
        "/opt/homebrew/bin/ffmpeg",
        "/usr/local/bin/ffmpeg",
        "/opt/homebrew/opt/ffmpeg/bin/ffmpeg",
        "/usr/local/opt/ffmpeg/bin/ffmpeg",
    ];

    // macOS: match check_ffmpeg resolution — prefer `which` and Homebrew before any
    // previously downloaded copy (avoids a corrupt partial download shadowing brew).
    if os == "macos" {
        if let Ok(output) = Command::new("/usr/bin/which").arg("ffmpeg").output() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() && path != "ffmpeg not found" {
                let p = std::path::Path::new(&path);
                if p.exists() {
                    return p.to_path_buf();
                }
            }
        }
        for path in &homebrew_paths {
            let p = std::path::Path::new(path);
            if p.exists() {
                return p.to_path_buf();
            }
        }
    }

    // 1. Bundled / downloaded copy in app data (Windows zip extract, or macOS fallback)
    if let Some(app_data) = dirs::data_local_dir() {
        let downloaded = app_data.join("Muse_Generator").join("ffmpeg").join(exe_name);
        if downloaded.exists() {
            return downloaded;
        }
    }

    // 2. Homebrew paths when not handled above (e.g. non-macOS never runs the macOS block)
    if os != "macos" {
        for path in &homebrew_paths {
            let p = std::path::Path::new(path);
            if p.exists() {
                return p.to_path_buf();
            }
        }
    }

    // 3. Try system PATH
    if let Ok(path_var) = std::env::var("PATH") {
        for path_dir in path_var.split(':') {
            let from_path = std::path::Path::new(path_dir).join(exe_name);
            if from_path.exists() {
                return from_path;
            }
        }
    }

    // 4. Last resort: just return executable name
    PathBuf::from(exe_name)
}

pub fn get_ffmpeg_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|p| p.join("Muse_Generator").join("ffmpeg"))
}

pub fn run_ffmpeg(args: &[String], timeout_secs: u64) -> Result<String, String> {
    let ffmpeg_path = get_ffmpeg_path();
    let timeout = Duration::from_secs(timeout_secs);

    let mut child = Command::new(&ffmpeg_path)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to execute ffmpeg ({}): {}", ffmpeg_path.display(), e))?;

    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let stderr = child.stderr.take()
                    .map(|mut s| {
                        use std::io::Read;
                        let mut buf = String::new();
                        s.read_to_string(&mut buf).ok();
                        buf
                    })
                    .unwrap_or_default();

                if status.success() {
                    return Ok(stderr);
                } else {
                    let exit_code = status.code().map(|c| c.to_string()).unwrap_or_else(|| "unknown".to_string());
                    return Err(format!("FFmpeg failed (exit {}): {}", exit_code, stderr.trim()));
                }
            }
            Ok(None) => {
                if start.elapsed() >= timeout {
                    let _ = child.kill();
                    return Err(format!("FFmpeg timed out after {} seconds", timeout_secs));
                }
                thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                return Err(format!("Error waiting for FFmpeg: {}", e));
            }
        }
    }
}
