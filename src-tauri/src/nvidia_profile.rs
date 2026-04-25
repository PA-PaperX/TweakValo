use std::path::PathBuf;
use std::process::Command;
use std::os::windows::process::CommandExt;

const SETTING_AA_TRANSPARENCY_SS: u32 = 0x10D48A85;
const SETTING_AUTO_LOD_BIAS: u32 = 0x00638E8F;
const SETTING_LOD_BIAS: u32 = 0x00738E8F;

const AA_MODE_REPLAY_ALL: u32 = 0x00000008;
const AUTO_LOD_BIAS_OFF: u32 = 0x00000000;

fn lod_preset_value(key: &str) -> Option<u32> {
    match key {
        "low" => Some(0x00000008),
        "medium" => Some(0x00000010),
        "high" => Some(0x00000018),
        "valocraft" => Some(0x00000036),
        _ => None,
    }
}

fn get_npi_exe(app_handle: &tauri::AppHandle) -> Option<PathBuf> {
    use tauri::Manager;
    let resource_dir = app_handle.path().resource_dir().ok()?;
    let path = resource_dir.join("resources").join("bin")
        .join("npi").join("nvidiaProfileInspector.exe");
    if path.exists() {
        return Some(path);
    }
    // Fallback for dev mode
    let exe = std::env::current_exe().ok()?;
    let base = exe.parent()?.parent()?.parent()?;
    let path = base.join("resources").join("bin")
        .join("npi").join("nvidiaProfileInspector.exe");
    if path.exists() { Some(path) } else { None }
}

#[tauri::command]
pub fn apply_nvidia_profile(app_handle: tauri::AppHandle, lod_key: String) -> (bool, String) {
    let lod_key = lod_key.to_lowercase();
    let lod_value = match lod_preset_value(&lod_key) {
        Some(v) => v,
        None => return (false, format!("Unknown preset: {}", lod_key)),
    };

    let npi_exe = match get_npi_exe(&app_handle) {
        Some(p) => p,
        None => return (false, "NPI executable not found".to_string()),
    };

    let cmd_result = Command::new(&npi_exe)
        .args([
            "-applyExe",
            "VALORANT-Win64-Shipping.exe",
            "Valorant",
            &format!("0x{:08X}=0x{:08X}", SETTING_AA_TRANSPARENCY_SS, AA_MODE_REPLAY_ALL),
            &format!("0x{:08X}=0x{:08X}", SETTING_AUTO_LOD_BIAS, AUTO_LOD_BIAS_OFF),
            &format!("0x{:08X}=0x{:08X}", SETTING_LOD_BIAS, lod_value),
        ])
        .creation_flags(0x08000000)
        .output();

    match cmd_result {
        Ok(output) if output.status.success() => {
            let info = String::from_utf8_lossy(&output.stdout).trim().to_string();
            (
                true,
                format!("Applied NVIDIA profile ({}) — {}", lod_key, if info.is_empty() { "success" } else { &info }),
            )
        }
        Ok(output) => {
            let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
            (false, format!("NPI failed: {}", if err.is_empty() { "unknown error" } else { &err }))
        }
        Err(e) => (false, format!("NPI error: {}", e)),
    }
}
