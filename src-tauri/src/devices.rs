use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::tools::{command, run, tool};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub serial: String,
    pub ip: String,
    pub port: String,
    pub model: String,
    pub state: String,
    pub source: String,
}

pub fn model_from_info(info: &str) -> String {
    for field in info.split_whitespace() {
        if let Some(rest) = field.strip_prefix("model:") {
            return rest.replace('_', " ");
        }
    }
    String::new()
}

fn split_serial(serial: &str) -> (String, String) {
    match serial.rsplit_once(':') {
        Some((ip, port)) => (ip.to_string(), port.to_string()),
        None => (serial.to_string(), String::new()),
    }
}

/// Parse `adb devices -l` into device records.
///
/// The header and any `* daemon ...` notices are skipped rather than assuming
/// they occupy exactly the first line: adb prints the daemon notices when it
/// has to start the server, pushing the header down.
pub fn parse_devices(output: &str) -> Vec<Device> {
    let mut devices = Vec::new();
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('*') || line.contains("List of devices attached") {
            continue;
        }
        let mut parts = line.split_whitespace();
        let Some(serial) = parts.next() else { continue };
        let Some(state) = parts.next() else { continue };
        let info = parts.collect::<Vec<_>>().join(" ");
        let (ip, port) = split_serial(serial);
        devices.push(Device {
            serial: serial.to_string(),
            ip,
            port,
            model: model_from_info(&info),
            state: state.to_string(),
            source: "adb".into(),
        });
    }
    devices
}

pub fn adb_list(app: &AppHandle) -> Vec<Device> {
    let adb = tool(app, "adb");
    match run(&adb, &["devices", "-l"]) {
        Ok((out, _, true)) => parse_devices(&out),
        _ => Vec::new(),
    }
}

/// Ask the adb mDNS daemon for wireless devices advertising `_adb-tls-connect`.
pub fn mdns_list(app: &AppHandle) -> Vec<Device> {
    let adb = tool(app, "adb");
    let _ = run(&adb, &["mdns", "check"]);
    let Ok((out, _, true)) = run(&adb, &["mdns", "services"]) else {
        return Vec::new();
    };

    let mut devices = Vec::new();
    for line in out.lines() {
        if !line.contains("_adb-tls-connect") && !line.contains("_adb._tcp") {
            continue;
        }
        let Some(addr) = line.split_whitespace().next() else {
            continue;
        };
        let (ip, port) = split_serial(addr);
        if ip.is_empty() {
            continue;
        }
        devices.push(Device {
            serial: addr.to_string(),
            ip,
            port,
            model: String::new(),
            state: "device".into(),
            source: "mdns".into(),
        });
    }
    devices
}

/// USB entries first, then mDNS discoveries, de-duplicated by address.
pub fn full_discover(app: &AppHandle) -> Vec<Device> {
    let mut found: Vec<Device> = adb_list(app)
        .into_iter()
        .filter(|d| d.source == "adb")
        .collect();
    for candidate in mdns_list(app) {
        if !found.iter().any(|d| d.serial == candidate.serial) {
            found.push(candidate);
        }
    }
    found
}

#[tauri::command]
pub fn discover_devices(app: AppHandle) -> Vec<Device> {
    full_discover(&app)
}

#[tauri::command]
pub fn list_devices(app: AppHandle) -> Vec<Device> {
    adb_list(&app)
}

#[tauri::command]
pub fn connect_device(app: AppHandle, ip: String, port: String) -> Result<String, String> {
    let adb = tool(&app, "adb");
    let target = if port.is_empty() {
        ip.clone()
    } else {
        format!("{ip}:{port}")
    };
    let (out, err, ok) = run(&adb, &["connect", &target])?;
    let combined = format!("{out} {err}").to_lowercase();
    if ok && (combined.contains("connected") || combined.contains("already")) {
        Ok(out)
    } else {
        Err(if out.is_empty() { err } else { out })
    }
}

#[tauri::command]
pub fn pair_device(
    app: AppHandle,
    ip: String,
    port: String,
    code: String,
) -> Result<String, String> {
    let adb = tool(&app, "adb");
    let target = format!("{ip}:{port}");
    let out = command(&adb)
        .args(["pair", &target])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(stdin) = child.stdin.as_mut() {
                let _ = writeln!(stdin, "{code}");
            }
            child.wait_with_output()
        })
        .map_err(|e| e.to_string())?;

    let text = format!(
        "{} {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    if text.to_lowercase().contains("paired") || text.to_lowercase().contains("success") {
        Ok(text.trim().to_string())
    } else {
        Err(text.trim().to_string())
    }
}

#[tauri::command]
pub fn disconnect_device(app: AppHandle, ip: String) -> Result<String, String> {
    let adb = tool(&app, "adb");
    let (out, err, _) = run(&adb, &["disconnect", &ip])?;
    Ok(if out.is_empty() { err } else { out })
}

/// Ping sweep across the machine's subnet, looking for phones that mDNS missed.
#[tauri::command]
pub async fn scan_subnet(app: AppHandle) -> Result<Vec<String>, String> {
    let local = local_ip().ok_or("could not determine the local IP")?;
    let prefix = match local.rsplit_once('.') {
        Some((head, _)) => format!("{head}."),
        None => return Err("unexpected local address".into()),
    };

    let mut handles = Vec::new();
    for host in 1..255 {
        let address = format!("{prefix}{host}");
        handles.push(std::thread::spawn(move || {
            let mut cmd = std::process::Command::new("ping");
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                cmd.creation_flags(0x0800_0000);
                cmd.args(["-n", "1", "-w", "700", &address]);
            }
            #[cfg(not(windows))]
            cmd.args(["-c", "1", "-W", "1", &address]);

            let ok = cmd
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            (address, ok)
        }));
    }

    let mut alive = Vec::new();
    for handle in handles {
        if let Ok((address, true)) = handle.join() {
            alive.push(address);
        }
    }
    let _ = app.emit("scan-finished", alive.len());
    Ok(alive)
}

pub fn local_ip() -> Option<String> {
    use std::net::UdpSocket;
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}

#[tauri::command]
pub fn restart_adb(app: AppHandle) -> Result<(), String> {
    let adb = tool(&app, "adb");
    let _ = run(&adb, &["kill-server"]);
    let _ = run(&adb, &["start-server"]);
    Ok(())
}
