use std::path::PathBuf;
use std::process::Command;

pub fn get_ffmpeg_path() -> PathBuf {
    let os = std::env::consts::OS;
    let exe_name = if os == "windows" { "ffmpeg.exe" } else { "ffmpeg" };

    // 1. Try app data directory first (where we download FFmpeg to)
    if let Some(app_data) = dirs::data_local_dir() {
        let downloaded = app_data.join("Muse_Generator").join("ffmpeg").join(exe_name);
        if downloaded.exists() {
            // Verify it's actually executable
            if Command::new(&downloaded).arg("--version").output().map(|o| o.status.success()).unwrap_or(false) {
                return downloaded;
            }
        }
    }

    // 2. Try homebrew paths for macOS (important for Apple Silicon)
    if os == "macos" {
        let homebrew_paths = [
            "/opt/homebrew/bin/ffmpeg",      // Apple Silicon homebrew
            "/usr/local/bin/ffmpeg",          // Intel homebrew
            "/opt/homebrew/opt/ffmpeg/bin/ffmpeg",
            "/usr/local/opt/ffmpeg/bin/ffmpeg",
        ];
        for path in &homebrew_paths {
            let p = std::path::Path::new(path);
            if p.exists() {
                return p.to_path_buf();
            }
        }
    }

    // 3. Try system PATH
    if let Ok(path_var) = std::env::var("PATH") {
        for path_dir in path_var.split(std::path::MAIN_SEPARATOR) {
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

pub fn run_ffmpeg(args: &[String]) -> Result<String, String> {
    let ffmpeg_path = get_ffmpeg_path();
    let output = Command::new(&ffmpeg_path)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute ffmpeg ({}): {}", ffmpeg_path.display(), e))?;

    // Combine stdout and stderr for better error messages
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined = if stdout.is_empty() { stderr.clone() } else { stdout };

    if output.status.success() {
        Ok(combined)
    } else {
        let exit_code = output.status.code().map(|c| c.to_string()).unwrap_or_else(|| "unknown".to_string());
        Err(format!("FFmpeg failed: {} | exit: {} | path: {}", combined.trim(), exit_code, ffmpeg_path.display()))
    }
}