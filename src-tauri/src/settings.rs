use serde::{Deserialize, Serialize};
use tauri::AppHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub ip: String,
    pub connect_port: String,
    pub pair_port: String,
    pub quality: String,
    pub theme: String,
    pub accent: String,
    pub auto_connect: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ip: String::new(),
            connect_port: "5555".into(),
            pair_port: "5555".into(),
            quality: "Medium".into(),
            theme: "dark".into(),
            accent: "pink".into(),
            auto_connect: false,
        }
    }
}

fn settings_path(app: &AppHandle) -> std::path::PathBuf {
    crate::tools::data_dir(app).join("settings.json")
}

pub fn load(app: &AppHandle) -> Settings {
    std::fs::read_to_string(settings_path(app))
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Settings {
    load(&app)
}

#[tauri::command]
pub fn save_settings(app: AppHandle, settings: Settings) -> Result<(), String> {
    let path = settings_path(&app);
    let body = serde_json::to_string_pretty(&settings).map_err(|e| e.to_string())?;
    std::fs::write(path, body).map_err(|e| e.to_string())
}
