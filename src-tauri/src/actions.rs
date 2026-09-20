use std::io::Write;
use tauri::AppHandle;
use tauri_plugin_dialog::DialogExt;

use crate::tools::{command, run, tool};

/// Encode text as an SVG QR code the frontend can drop straight into the DOM.
pub fn qr_svg(text: &str) -> Result<String, String> {
    use qrcode::render::svg;
    use qrcode::QrCode;

    let code = QrCode::new(text.as_bytes()).map_err(|e| e.to_string())?;
    Ok(code
        .render::<svg::Color>()
        .min_dimensions(180, 180)
        .dark_color(svg::Color("#0a0a1a"))
        .light_color(svg::Color("#ffffff"))
        .build())
}

#[tauri::command]
pub fn qr_for(serial: String) -> Result<String, String> {
    if serial.is_empty() {
        return Err("no device selected".into());
    }
    qr_svg(&format!("adb connect {serial}"))
}

#[tauri::command]
pub fn capture_screen(app: AppHandle, serial: String) -> Result<String, String> {
    let adb = tool(&app, "adb");
    let out = command(&adb)
        .args(["-s", &serial, "exec-out", "screencap", "-p"])
        .output()
        .map_err(|e| format!("screencap failed: {e}"))?;

    if !out.status.success() || out.stdout.is_empty() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }

    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let path = crate::tools::screenshot_dir(&app).join(format!("shot_{stamp}.png"));
    let mut file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
    file.write_all(&out.stdout).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn read_clipboard(app: AppHandle, serial: String) -> Result<String, String> {
    let adb = tool(&app, "adb");
    let (out, err, ok) = run(
        &adb,
        &["-s", &serial, "exec-out", "cmd", "clipboard", "get", "text"],
    )?;
    if !ok {
        return Err(if err.is_empty() {
            "could not read the device clipboard".into()
        } else {
            err
        });
    }
    if out.trim().is_empty() {
        return Err("the device clipboard is empty".into());
    }
    Ok(out)
}

/// Copy the PC clipboard onto the device using a file push, which avoids all
/// quoting problems with the usual `am broadcast` one-liner.
#[tauri::command]
pub fn send_clipboard(app: AppHandle, serial: String, text: String) -> Result<(), String> {
    let adb = tool(&app, "adb");
    let local = crate::tools::data_dir(&app).join("clipboard.txt");
    std::fs::write(&local, text.as_bytes()).map_err(|e| e.to_string())?;
    let remote = "/sdcard/mirror_rust_clipboard.txt";

    let (_, err, ok) = run(
        &adb,
        &["-s", &serial, "push", &local.to_string_lossy(), remote],
    )?;
    if !ok {
        return Err(err);
    }

    let quoted = format!("\"$(cat {remote})\"");
    let (_, err, ok) = run(
        &adb,
        &[
            "-s",
            &serial,
            "shell",
            "cmd",
            "clipboard",
            "set",
            "text",
            &quoted,
        ],
    )?;
    if !ok {
        return Err(err);
    }
    let _ = std::fs::remove_file(&local);
    Ok(())
}

#[tauri::command]
pub fn push_file(app: AppHandle, serial: String, path: String) -> Result<String, String> {
    let adb = tool(&app, "adb");
    let name = std::path::Path::new(&path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".into());
    let target = format!("/sdcard/Download/{name}");
    let (out, err, ok) = run(&adb, &["-s", &serial, "push", &path, &target])?;
    if ok {
        Ok(target)
    } else if err.is_empty() {
        Err(out)
    } else {
        Err(err)
    }
}

#[tauri::command]
pub fn pick_file(app: AppHandle) -> Option<String> {
    app.dialog()
        .file()
        .set_title("Push a file to the device")
        .blocking_pick_file()
        .map(|p| p.to_string())
}

#[tauri::command]
pub fn device_info(app: AppHandle, serial: String) -> Result<String, String> {
    let adb = tool(&app, "adb");
    let mut parts = Vec::new();
    for prop in [
        "ro.product.manufacturer",
        "ro.product.model",
        "ro.build.version.release",
        "ro.build.version.sdk",
    ] {
        if let Ok((out, _, true)) = run(&adb, &["-s", &serial, "shell", "getprop", prop]) {
            if !out.trim().is_empty() {
                parts.push(out.trim().to_string());
            }
        }
    }
    if parts.is_empty() {
        Err("no device info returned".into())
    } else {
        Ok(parts.join(" · "))
    }
}

#[tauri::command]
pub fn open_folder(app: AppHandle, path: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|e| e.to_string())
}
