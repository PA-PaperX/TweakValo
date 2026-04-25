use std::fs;
use std::path::{Path, PathBuf};

fn find_all_game_settings() -> Vec<PathBuf> {
    let local = match std::env::var("LOCALAPPDATA") {
        Ok(l) => l,
        Err(_) => return vec![],
    };
    let base = Path::new(&local)
        .join("VALORANT")
        .join("Saved")
        .join("Config");
    if !base.exists() {
        return vec![];
    }

    let mut paths = Vec::new();
    if let Ok(entries) = fs::read_dir(&base) {
        for entry in entries.filter_map(|e| e.ok()) {
            let ini = entry
                .path()
                .join("Windows")
                .join("GameUserSettings.ini");
            if ini.exists() {
                paths.push(ini);
            }
        }
    }
    paths
}

fn backup_file(path: &Path) {
    let backup = PathBuf::from(format!("{}.hd_backup", path.display()));
    if !backup.exists() {
        if let Err(e) = fs::copy(path, &backup) {
            log::error!("Failed to backup {:?}: {}", path, e);
        }
    }
}

#[tauri::command]
pub fn apply_graphics_preset(preset_file: String) -> (bool, String) {
    let paths = find_all_game_settings();
    if paths.is_empty() {
        return (
            false,
            "Could not find any GameUserSettings.ini. Launch VALORANT at least once first."
                .to_string(),
        );
    }

    let preset = Path::new(&preset_file);
    if !preset.exists() {
        return (false, format!("Preset file not found: {}", preset_file));
    }

    let mut applied = 0;
    for settings_path in &paths {
        backup_file(settings_path);
        if fs::copy(preset, settings_path).is_ok() {
            applied += 1;
        }
    }

    (
        true,
        format!("Low graphics preset applied to {} account(s)", applied),
    )
}

#[tauri::command]
pub fn restore_graphics() -> (bool, String) {
    let paths = find_all_game_settings();
    if paths.is_empty() {
        return (false, "Could not find any GameUserSettings.ini".to_string());
    }

    let mut restored = 0;
    for settings_path in &paths {
        let backup = PathBuf::from(format!("{}.hd_backup", settings_path.display()));
        if backup.exists() {
            if fs::copy(&backup, settings_path).is_ok() {
                restored += 1;
            }
        }
    }

    if restored == 0 {
        (false, "No backups found. Cannot restore.".to_string())
    } else {
        (
            true,
            format!("Restored {} account(s)", restored),
        )
    }
}
