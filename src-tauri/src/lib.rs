mod actions;
pub mod devices;
mod scrcpy;
mod settings;
mod tools;

#[tauri::command]
fn environment(app: tauri::AppHandle) -> serde_json::Value {
    serde_json::json!({
        "adb": tools::tool(&app, "adb").exists(),
        "scrcpy": scrcpy::scrcpy_available(&app),
        "data_dir": tools::data_dir(&app).to_string_lossy(),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            environment,
            devices::discover_devices,
            devices::list_devices,
            devices::connect_device,
            devices::pair_device,
            devices::disconnect_device,
            devices::scan_subnet,
            devices::restart_adb,
            scrcpy::start_mirror,
            scrcpy::quick_mirror,
            actions::qr_for,
            actions::capture_screen,
            actions::read_clipboard,
            actions::send_clipboard,
            actions::push_file,
            actions::pick_file,
            actions::device_info,
            actions::open_folder,
            settings::get_settings,
            settings::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running MirrorPy");
}
