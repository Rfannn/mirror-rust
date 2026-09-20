use serde::Serialize;

use crate::tools::tool;
use tauri::AppHandle;

#[derive(Debug, Clone, Serialize)]
pub struct Preset {
    pub name: &'static str,
    pub resolution: &'static str,
    pub fps: &'static str,
    pub bitrate: &'static str,
    pub description: &'static str,
}

pub const PRESETS: [Preset; 4] = [
    Preset {
        name: "Low",
        resolution: "720",
        fps: "15",
        bitrate: "2M",
        description: "720p · 15fps · 2 Mbps",
    },
    Preset {
        name: "Medium",
        resolution: "720",
        fps: "30",
        bitrate: "4M",
        description: "720p · 30fps · 4 Mbps",
    },
    Preset {
        name: "High",
        resolution: "1080",
        fps: "60",
        bitrate: "8M",
        description: "1080p · 60fps · 8 Mbps",
    },
    Preset {
        name: "Ultra",
        resolution: "",
        fps: "",
        bitrate: "50M",
        description: "Native resolution · unlimited fps · 50 Mbps",
    },
];

fn flags(preset: &str) -> Vec<String> {
    let chosen = PRESETS
        .iter()
        .find(|p| p.name.eq_ignore_ascii_case(preset))
        .unwrap_or(&PRESETS[1]);

    let mut args = Vec::new();
    if !chosen.resolution.is_empty() {
        args.push("--max-size".into());
        args.push(chosen.resolution.into());
    }
    if !chosen.fps.is_empty() {
        args.push("--max-fps".into());
        args.push(chosen.fps.into());
    }
    args.push("--video-bit-rate".into());
    args.push(chosen.bitrate.into());
    args
}

/// Start scrcpy detached. Returns immediately so the UI stays responsive.
#[tauri::command]
pub fn start_mirror(app: AppHandle, serial: String, preset: String) -> Result<String, String> {
    let scrcpy = tool(&app, "scrcpy");
    let mut args: Vec<String> = Vec::new();
    if !serial.is_empty() {
        args.push("-s".into());
        args.push(serial.clone());
    }
    args.extend(flags(&preset));

    let child = crate::tools::command(&scrcpy)
        .args(&args)
        .spawn()
        .map_err(|e| format!("could not start scrcpy: {e}"))?;

    Ok(format!(
        "mirroring {serial} with {preset} preset (pid {})",
        child.id()
    ))
}

/// One-click: discover, connect if wireless, then mirror.
#[tauri::command]
pub async fn quick_mirror(app: AppHandle, preset: String) -> Result<String, String> {
    let devices = crate::devices::full_discover(&app);
    if devices.is_empty() {
        return Err("no devices found".into());
    }
    let device = devices
        .iter()
        .find(|d| d.state == "device" && d.source == "adb")
        .or_else(|| devices.iter().find(|d| d.state == "device"))
        .unwrap_or(&devices[0])
        .clone();

    if device.source == "mdns" || !device.port.is_empty() {
        let _ = crate::devices::connect_device(app.clone(), device.ip.clone(), device.port.clone());
    }

    start_mirror(app, device.serial.clone(), preset)
}

pub fn scrcpy_available(app: &AppHandle) -> bool {
    tool(app, "scrcpy").exists()
}
