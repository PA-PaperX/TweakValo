use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub game_path: String,
    #[serde(default)]
    pub riot_client_path: String,
    #[serde(default = "default_true")]
    pub enable_blood: bool,
    #[serde(default = "default_true")]
    pub minimize_to_tray: bool,
    #[serde(default = "default_lang")]
    pub language: String,
    #[serde(default)]
    pub custom_width: String,
    #[serde(default)]
    pub custom_height: String,
    #[serde(default)]
    pub last_resolution: String,
    #[serde(default)]
    pub gpu_override: String,
    #[serde(default)]
    pub welcome_shown: bool,
    #[serde(default)]
    pub auto_timer_resolution: bool,
    #[serde(default)]
    pub auto_standby_cleaner: bool,
    #[serde(default)]
    pub auto_priority: bool,
    #[serde(default)]
    pub auto_affinity: bool,
}

fn default_true() -> bool {
    true
}

fn default_lang() -> String {
    "th".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            game_path: String::new(),
            riot_client_path: String::new(),
            enable_blood: true,
            minimize_to_tray: true,
            language: "th".to_string(),
            custom_width: String::new(),
            custom_height: String::new(),
            last_resolution: String::new(),
            gpu_override: "auto".to_string(),
            welcome_shown: false,
            auto_timer_resolution: false,
            auto_standby_cleaner: false,
            auto_priority: false,
            auto_affinity: false,
        }
    }
}

pub struct ConfigState(pub Mutex<AppConfig>);

fn config_path() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    exe_dir.join("config.json")
}

pub fn load_config() -> AppConfig {
    let path = config_path();
    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => AppConfig::default(),
        }
    } else {
        let config = AppConfig::default();
        save_config(&config);
        config
    }
}

pub fn save_config(config: &AppConfig) {
    let path = config_path();
    match serde_json::to_string_pretty(config) {
        Ok(json) => {
            if let Err(e) = fs::write(&path, json) {
                log::error!("Failed to save config to {:?}: {}", path, e);
            }
        }
        Err(e) => log::error!("Failed to serialize config: {}", e),
    }
}

#[tauri::command]
pub fn get_config(state: tauri::State<'_, ConfigState>) -> AppConfig {
    state.0.lock().unwrap().clone()
}

#[tauri::command]
pub fn update_config(state: tauri::State<'_, ConfigState>, config: AppConfig) {
    save_config(&config);
    *state.0.lock().unwrap() = config;
}

#[tauri::command]
pub fn update_config_field(state: tauri::State<'_, ConfigState>, key: String, value: String) {
    let mut config = state.0.lock().unwrap();
    match key.as_str() {
        "game_path" => config.game_path = value,
        "riot_client_path" => config.riot_client_path = value,
        "enable_blood" => config.enable_blood = value == "true",
        "minimize_to_tray" => config.minimize_to_tray = value == "true",
        "language" => config.language = value,
        "custom_width" => config.custom_width = value,
        "custom_height" => config.custom_height = value,
        "last_resolution" => config.last_resolution = value,
        "gpu_override" => config.gpu_override = value,
        "welcome_shown" => config.welcome_shown = value == "true",
        "auto_timer_resolution" => config.auto_timer_resolution = value == "true",
        "auto_standby_cleaner" => config.auto_standby_cleaner = value == "true",
        "auto_priority" => config.auto_priority = value == "true",
        "auto_affinity" => config.auto_affinity = value == "true",
        _ => {}
    }
    save_config(&config);
}
