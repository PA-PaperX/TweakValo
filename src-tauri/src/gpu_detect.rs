use serde::Serialize;
use std::os::windows::process::CommandExt;
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct GpuInfo {
    pub names: Vec<String>,
    pub nvidia: bool,
    pub intel: bool,
    pub amd: bool,
    pub category: String,
}

pub fn detect_gpus() -> GpuInfo {
    let mut names = Vec::new();

    // Use PowerShell to query Win32_VideoController
    if let Ok(output) = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            "Get-CimInstance Win32_VideoController | Select-Object -ExpandProperty Name",
        ])
        .creation_flags(0x08000000) // CREATE_NO_WINDOW
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            names = stdout
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect();
        }
    }

    let nvidia = names.iter().any(|n| n.to_lowercase().contains("nvidia"));
    let intel = names.iter().any(|n| n.to_lowercase().contains("intel"));
    let amd = names
        .iter()
        .any(|n| n.to_lowercase().contains("amd") || n.to_lowercase().contains("radeon"));

    let category = if names.is_empty() {
        "unknown".to_string()
    } else if nvidia && (intel || amd) {
        "hybrid".to_string()
    } else if nvidia {
        "nvidia_only".to_string()
    } else if intel || amd {
        "non_nvidia".to_string()
    } else {
        "unknown".to_string()
    };

    GpuInfo {
        names,
        nvidia,
        intel,
        amd,
        category,
    }
}

#[tauri::command]
pub async fn get_gpu_info() -> GpuInfo {
    detect_gpus()
}
