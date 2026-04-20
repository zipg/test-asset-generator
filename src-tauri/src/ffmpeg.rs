use std::env;
use std::io::{self, Cursor};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::thread;
use std::time::{Duration, Instant};
use tauri::path::BaseDirectory;
use tauri::AppHandle;
use tauri::Manager;

use crate::process_ext::command;

fn where_ffmpeg_exe_lines() -> Option<String> {
    let output = command("where.exe").arg("ffmpeg.exe").output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// First path reported by `where` that exists on disk (no exec check). Used to align
/// `download_ffmpeg` with `check_ffmpeg` when `--version` cannot be run in-process.
pub fn windows_ffmpeg_path_from_where_exists() -> Option<PathBuf> {
    let text = where_ffmpeg_exe_lines()?;
    for line in text.lines() {
        let p = line.trim();
        if p.is_empty() {
            continue;
        }
        let pb = PathBuf::from(p);
        if pb.exists() {
            return Some(pb);
        }
    }
    None
}

/// Prefer a `where.exe` candidate that successfully runs `ffmpeg.exe -version`.
pub fn first_working_windows_ffmpeg_from_where() -> Option<PathBuf> {
    let text = where_ffmpeg_exe_lines()?;
    for line in text.lines() {
        let p = line.trim();
        if p.is_empty() {
            continue;
        }
        let pb = PathBuf::from(p);
        if !pb.exists() {
            continue;
        }
        if let Ok(o) = command(&pb).arg("--version").output() {
            if o.status.success() {
                return Some(pb);
            }
        }
    }
    None
}

/// True if the Windows installer / dev resources ship `ffmpeg.exe` (see `bundle.resources`).
pub fn bundled_resource_ffmpeg_exists(app: &tauri::AppHandle) -> bool {
    if std::env::consts::OS != "windows" {
        return false;
    }
    for rel in ["ffmpeg.exe", "resources/ffmpeg.exe"] {
        if let Ok(p) = app.path().resolve(rel, BaseDirectory::Resource) {
            if p.exists() {
                return true;
            }
        }
    }
    false
}

/// True if the macOS app bundle ships `ffmpeg` under Resources (see `tauri.macos.bundle.json`).
pub fn bundled_resource_ffmpeg_exists_mac(app: &tauri::AppHandle) -> bool {
    if std::env::consts::OS != "macos" {
        return false;
    }
    for rel in ["ffmpeg", "resources/ffmpeg"] {
        if let Ok(p) = app.path().resolve(rel, BaseDirectory::Resource) {
            if p.exists() {
                return true;
            }
        }
    }
    false
}

/// Copy shipped `resources/ffmpeg.exe` from the app bundle to LocalAppData (first run / repair).
pub fn ensure_windows_bundled_ffmpeg_copied(app: &tauri::AppHandle) -> Result<(), String> {
    if std::env::consts::OS != "windows" {
        return Ok(());
    }
    for rel in ["ffmpeg.exe", "resources/ffmpeg.exe"] {
        let src = match app.path().resolve(rel, BaseDirectory::Resource) {
            Ok(p) => p,
            Err(_) => continue,
        };
        if !src.exists() {
            continue;
        }
        let Some(dest_dir) = get_ffmpeg_dir() else {
            return Ok(());
        };
        let dest = dest_dir.join("ffmpeg.exe");
        let need_copy = !dest.exists()
            || command(&dest)
                .arg("--version")
                .output()
                .map(|o| !o.status.success())
                .unwrap_or(true);
        if !need_copy {
            return Ok(());
        }
        std::fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
        std::fs::copy(&src, &dest).map_err(|e| {
            format!(
                "Failed to copy bundled FFmpeg to {}: {}",
                dest.display(),
                e
            )
        })?;
        return Ok(());
    }
    Ok(())
}

/// Copy shipped `resources/ffmpeg` from the app bundle to Application Support (first run / repair).
pub fn ensure_macos_bundled_ffmpeg_copied(app: &tauri::AppHandle) -> Result<(), String> {
    if std::env::consts::OS != "macos" {
        return Ok(());
    }
    for rel in ["ffmpeg", "resources/ffmpeg"] {
        let src = match app.path().resolve(rel, BaseDirectory::Resource) {
            Ok(p) => p,
            Err(_) => continue,
        };
        if !src.exists() {
            continue;
        }
        let Some(dest_dir) = get_ffmpeg_dir() else {
            return Ok(());
        };
        let dest = dest_dir.join("ffmpeg");
        let need_copy = !dest.exists()
            || command(&dest)
                .arg("--version")
                .output()
                .map(|o| !o.status.success())
                .unwrap_or(true);
        if !need_copy {
            return Ok(());
        }
        std::fs::create_dir_all(&dest_dir).map_err(|e| e.to_string())?;
        std::fs::copy(&src, &dest).map_err(|e| {
            format!(
                "Failed to copy bundled FFmpeg to {}: {}",
                dest.display(),
                e
            )
        })?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&dest, std::fs::Permissions::from_mode(0o755));
        }
        return Ok(());
    }
    Ok(())
}

/// Optional portable layout: `ffmpeg.exe` or `ffmpeg/ffmpeg.exe` next to the app `.exe`.
pub fn bundled_ffmpeg_beside_executable_windows() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let dir = exe.parent()?;
    for rel in ["ffmpeg/ffmpeg.exe", "ffmpeg.exe"] {
        let p = dir.join(Path::new(rel));
        if p.exists() {
            return Some(p);
        }
    }
    None
}

/// Resolve FFmpeg binary. Pass `Some(app)` from Tauri commands so macOS can use the bundled
/// copy inside `Resources/` when the Application Support mirror is missing.
pub fn resolve_ffmpeg_executable(app: Option<&AppHandle>) -> PathBuf {
    let os = std::env::consts::OS;
    let exe_name = if os == "windows" { "ffmpeg.exe" } else { "ffmpeg" };

    if os == "windows" {
        if let Some(p) = bundled_ffmpeg_beside_executable_windows() {
            return p;
        }
        // Prefer LocalAppData (bundled copy or prior download) over whatever `where` finds on PATH.
        if let Some(app_data) = dirs::data_local_dir() {
            let local = app_data.join("Muse_Generator").join("ffmpeg").join(&exe_name);
            if local.exists() {
                return local;
            }
        }
        if let Some(p) = first_working_windows_ffmpeg_from_where() {
            return p;
        }
    }

    let homebrew_paths = [
        "/opt/homebrew/bin/ffmpeg",
        "/usr/local/bin/ffmpeg",
        "/opt/homebrew/opt/ffmpeg/bin/ffmpeg",
        "/usr/local/opt/ffmpeg/bin/ffmpeg",
    ];

    // macOS: Application Support mirror, then bundle Resources. Release DMGs ship ffmpeg here,
    // so we return before `which`/Homebrew — system FFmpeg is only a fallback for dev builds
    // without `resources/ffmpeg`.
    if os == "macos" {
        if let Some(handle) = app {
            let _ = ensure_macos_bundled_ffmpeg_copied(handle);
        }
        if let Some(app_data) = dirs::data_local_dir() {
            let local = app_data.join("Muse_Generator").join("ffmpeg").join(&exe_name);
            if local.exists() {
                return local;
            }
        }
        if let Some(handle) = app {
            for rel in ["ffmpeg", "resources/ffmpeg"] {
                if let Ok(p) = handle.path().resolve(rel, BaseDirectory::Resource) {
                    if p.exists() {
                        return p;
                    }
                }
            }
        }
        if let Ok(output) = command("/usr/bin/which").arg("ffmpeg").output() {
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

    // 1. Bundled / downloaded copy in app data (Windows zip extract, or non-mac unix fallback)
    if let Some(app_data) = dirs::data_local_dir() {
        let downloaded = app_data.join("Muse_Generator").join("ffmpeg").join(&exe_name);
        if downloaded.exists() {
            return downloaded;
        }
    }

    // 2. Homebrew-style paths (Linux/Homebrew on Linux; skip Windows — meaningless there)
    if os != "macos" && os != "windows" {
        for path in &homebrew_paths {
            let p = std::path::Path::new(path);
            if p.exists() {
                return p.to_path_buf();
            }
        }
    }

    // 3. PATH (Windows uses `;`, Unix uses `:` — use std::env::split_paths)
    if let Ok(path_var) = env::var("PATH") {
        for path_dir in env::split_paths(&path_var) {
            let from_path = path_dir.join(&exe_name);
            if from_path.exists() {
                return from_path;
            }
        }
    }

    // 4. Last resort: just return executable name
    PathBuf::from(exe_name)
}

/// Evermeet ships a zip; eugeneware static builds are a raw Mach-O. Detect zip by local header.
pub fn install_mac_ffmpeg_from_download_bytes(bytes: &[u8], dest: &Path) -> Result<(), String> {
    let is_zip = bytes.len() >= 4
        && bytes[0] == b'P'
        && bytes[1] == b'K'
        && matches!(bytes[2], 3 | 5 | 7)
        && matches!(bytes[3], 4 | 6 | 8);

    if is_zip {
        let cursor = Cursor::new(bytes.to_vec());
        let mut archive =
            zip::ZipArchive::new(cursor).map_err(|e| format!("Invalid FFmpeg zip: {}", e))?;
        let mut idx: Option<usize> = None;
        for i in 0..archive.len() {
            let file = archive.by_index(i).map_err(|e| e.to_string())?;
            let name = file.name().replace('\\', "/");
            if name.contains("__MACOSX") {
                continue;
            }
            if name == "ffmpeg" || name.ends_with("/ffmpeg") {
                idx = Some(i);
                break;
            }
        }
        let i = idx.ok_or_else(|| "ffmpeg binary not found inside zip".to_string())?;
        let mut file = archive.by_index(i).map_err(|e| e.to_string())?;
        let mut out = std::fs::File::create(dest).map_err(|e| e.to_string())?;
        io::copy(&mut file, &mut out).map_err(|e| e.to_string())?;
    } else {
        let mut out = std::fs::File::create(dest).map_err(|e| e.to_string())?;
        io::copy(&mut Cursor::new(bytes), &mut out).map_err(|e| e.to_string())?;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(dest, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// True if the copied bundle binary or Resources binary runs `ffmpeg --version`.
pub fn mac_bundled_ffmpeg_runnable(app: &AppHandle) -> bool {
    if std::env::consts::OS != "macos" {
        return false;
    }
    let _ = ensure_macos_bundled_ffmpeg_copied(app);
    if !bundled_resource_ffmpeg_exists_mac(app) {
        return false;
    }
    let p = resolve_ffmpeg_executable(Some(app));
    command(&p)
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn get_ffmpeg_dir() -> Option<PathBuf> {
    dirs::data_local_dir().map(|p| p.join("Muse_Generator").join("ffmpeg"))
}

pub fn run_ffmpeg_for_app(
    app: Option<&AppHandle>,
    args: &[String],
    timeout_secs: u64,
) -> Result<String, String> {
    let ffmpeg_path = resolve_ffmpeg_executable(app);
    let timeout = Duration::from_secs(timeout_secs);

    let mut child = command(&ffmpeg_path)
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

