use std::path::PathBuf;
use std::process::Command;
use tauri::{AppHandle, Manager};

/// Directory holding the app's own files: settings, logs, screenshots.
pub fn data_dir(app: &AppHandle) -> PathBuf {
    if let Ok(dir) = app.path().app_config_dir() {
        let _ = std::fs::create_dir_all(&dir);
        return dir;
    }
    PathBuf::from(".")
}

pub fn screenshot_dir(app: &AppHandle) -> PathBuf {
    let dir = data_dir(app).join("screenshots");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Resolve a bundled adb or scrcpy.
///
/// In a packaged build the binaries sit in the resource directory. During
/// development they are read from the neighbouring Python project, which holds
/// the scrcpy distribution this project reuses.
pub fn tool(app: &AppHandle, name: &str) -> PathBuf {
    let exe_name = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };

    if let Ok(res) = app.path().resource_dir() {
        for candidate in [
            res.join(&exe_name),
            res.join("resources").join(&exe_name),
            res.join(name),
        ] {
            if candidate.exists() {
                return candidate;
            }
        }
    }

    let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("mirrorpy")
        .join(&exe_name);
    if dev.exists() {
        return dev;
    }

    PathBuf::from(exe_name)
}

/// Build a Command with CREATE_NO_WINDOW so no console flashes on Windows.
pub fn command(program: &PathBuf) -> Command {
    #[allow(unused_mut)]
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

pub fn run(program: &PathBuf, args: &[&str]) -> Result<(String, String, bool), String> {
    let out = command(program)
        .args(args)
        .output()
        .map_err(|e| format!("{}: {e}", program.display()))?;
    Ok((
        String::from_utf8_lossy(&out.stdout).trim().to_string(),
        String::from_utf8_lossy(&out.stderr).trim().to_string(),
        out.status.success(),
    ))
}
