use std::path::PathBuf;
use std::process::Command;

pub fn get_ffmpeg_path() -> PathBuf {
    let os = std::env::consts::OS;
    let exe_name = if os == "windows" { "ffmpeg.exe" } else { "ffmpeg" };

    // Try bundled path relative to executable (production)
    if let Some(exe_dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf())) {
        let bundled = exe_dir.join("ffmpeg").join(os).join(exe_name);
        if bundled.exists() {
            return bundled;
        }
    }

    // Fallback: development path (project root/ffmpeg/{os}/ffmpeg)
    if let Some(dev_path) = std::env::current_dir()
        .ok()
        .map(|p| p.join("ffmpeg").join(os).join(exe_name))
    {
        if dev_path.exists() {
            return dev_path;
        }
    }

    // Last resort: system ffmpeg
    if os == "windows" {
        PathBuf::from("ffmpeg.exe")
    } else {
        PathBuf::from("ffmpeg")
    }
}

pub fn run_ffmpeg(args: &[String]) -> Result<String, String> {
    let ffmpeg_path = get_ffmpeg_path();
    let output = Command::new(&ffmpeg_path)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute ffmpeg: {}", e))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stderr).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
