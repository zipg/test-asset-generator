use std::path::PathBuf;
use std::process::Command;

pub fn get_ffmpeg_path() -> PathBuf {
    let os = std::env::consts::OS;
    let exe_name = if os == "windows" { "ffmpeg.exe" } else { "ffmpeg" };

    // 1. Try system PATH first (most reliable cross-platform)
    if let Ok(path_var) = std::env::var("PATH") {
        for path_dir in path_var.split(std::path::MAIN_SEPARATOR) {
            let from_path = std::path::Path::new(path_dir).join(exe_name);
            if from_path.exists() {
                return from_path;
            }
        }
    }

    // 2. Try development path (project root/ffmpeg/{os}/ffmpeg)
    if let Some(dev_path) = std::env::current_dir()
        .ok()
        .map(|p| p.join("ffmpeg").join(os).join(exe_name))
    {
        if dev_path.exists() {
            return dev_path;
        }
    }

    // 3. Try bundled path relative to executable
    if let Some(exe_dir) = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf())) {
        // macOS: Contents/MacOS/ -> Contents/Resources/
        let app_root = if os == "macos" {
            exe_dir.parent().and_then(|p| p.parent()).map(|p| p.to_path_buf())
        } else {
            Some(exe_dir.clone())
        };

        if let Some(root) = app_root {
            let bundled = root.join("Resources").join("ffmpeg").join(os).join(exe_name);
            if bundled.exists() {
                return bundled;
            }
        }

        // Fallback: next to executable
        let bundled = exe_dir.join("ffmpeg").join(os).join(exe_name);
        if bundled.exists() {
            return bundled;
        }
    }

    // 4. Last resort: just return executable name (hoping it's in PATH)
    PathBuf::from(exe_name)
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
