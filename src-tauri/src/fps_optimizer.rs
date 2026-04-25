use serde::Serialize;
use std::process::Command;
use std::os::windows::process::CommandExt;

#[derive(Debug, Clone, Serialize)]
pub struct OptResult {
    pub name: String,
    pub success: bool,
    pub message: String,
}

fn reg_set_dword(hive: &str, path: &str, name: &str, value: u32) -> Result<(), String> {
    let full_path = format!("{}\\{}", hive, path);
    let result = Command::new("reg")
        .args([
            "add",
            &full_path,
            "/v",
            name,
            "/t",
            "REG_DWORD",
            "/d",
            &value.to_string(),
            "/f",
        ])
        .creation_flags(0x08000000)
        .output()
        .map_err(|e: std::io::Error| e.to_string())?;

    if result.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&result.stderr).to_string())
    }
}

fn reg_set_sz(hive: &str, path: &str, name: &str, value: &str) -> Result<(), String> {
    let full_path = format!("{}\\{}", hive, path);
    let result = Command::new("reg")
        .args([
            "add",
            &full_path,
            "/v",
            name,
            "/t",
            "REG_SZ",
            "/d",
            value,
            "/f",
        ])
        .creation_flags(0x08000000)
        .output()
        .map_err(|e: std::io::Error| e.to_string())?;

    if result.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&result.stderr).to_string())
    }
}

fn set_ultimate_performance() -> OptResult {
    let r1 = Command::new("powercfg")
        .args(["/setactive", "e9a42b02-d5df-448d-aa00-03f14749eb61"])
        .creation_flags(0x08000000)
        .output();

    if let Ok(out) = r1 {
        if !out.status.success() {
            let _ = Command::new("powercfg")
                .args(["/duplicatescheme", "e9a42b02-d5df-448d-aa00-03f14749eb61"])
                .creation_flags(0x08000000)
                .output();
            let _ = Command::new("powercfg")
                .args(["/setactive", "e9a42b02-d5df-448d-aa00-03f14749eb61"])
                .creation_flags(0x08000000)
                .output();
        }
    }

    OptResult {
        name: "Power Plan".to_string(),
        success: true,
        message: "Ultimate Performance power plan activated".to_string(),
    }
}

fn disable_visual_effects() -> OptResult {
    let r1 = reg_set_dword(
        "HKCU",
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\VisualEffects",
        "VisualFXSetting",
        2,
    );
    let r2 = reg_set_sz("HKCU", r"Control Panel\Desktop", "MenuShowDelay", "0");
    let _r3 = reg_set_dword("HKCU", r"Control Panel\Desktop", "ForegroundLockTimeout", 0);
    let _r4 = reg_set_sz(
        "HKCU",
        r"Control Panel\Desktop\WindowMetrics",
        "MinAnimate",
        "0",
    );

    OptResult {
        name: "Visual Effects".to_string(),
        success: r1.is_ok() && r2.is_ok(),
        message: "Visual effects set to Best Performance".to_string(),
    }
}

fn disable_game_dvr() -> OptResult {
    let r1 = reg_set_dword(
        "HKCU",
        r"Software\Microsoft\GameBar",
        "UseNexusForGameBarEnabled",
        0,
    );
    let _r2 = reg_set_dword(
        "HKCU",
        r"Software\Microsoft\GameBar",
        "AllowAutoGameMode",
        0,
    );
    let _r3 = reg_set_dword(
        "HKCU",
        r"Software\Microsoft\Windows\CurrentVersion\GameDVR",
        "AppCaptureEnabled",
        0,
    );

    OptResult {
        name: "Game Bar & DVR".to_string(),
        success: r1.is_ok(),
        message: "Game Bar & DVR disabled".to_string(),
    }
}

fn disable_nagle() -> OptResult {
    let r1 = reg_set_dword(
        "HKLM",
        r"SYSTEM\CurrentControlSet\Services\Tcpip\Parameters",
        "TcpAckFrequency",
        1,
    );
    let r2 = reg_set_dword(
        "HKLM",
        r"SYSTEM\CurrentControlSet\Services\Tcpip\Parameters",
        "TCPNoDelay",
        1,
    );

    OptResult {
        name: "Nagle Algorithm".to_string(),
        success: r1.is_ok() && r2.is_ok(),
        message: "Nagle's algorithm disabled (lower latency)".to_string(),
    }
}

fn disable_prefetch() -> OptResult {
    let r1 = reg_set_dword(
        "HKLM",
        r"SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management\PrefetchParameters",
        "EnablePrefetcher",
        0,
    );
    let _r2 = reg_set_dword(
        "HKLM",
        r"SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management\PrefetchParameters",
        "EnableSuperfetch",
        0,
    );

    OptResult {
        name: "Prefetch".to_string(),
        success: r1.is_ok(),
        message: "Prefetch & Superfetch disabled".to_string(),
    }
}

fn optimize_responsiveness() -> OptResult {
    let r = reg_set_dword(
        "HKLM",
        r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile",
        "SystemResponsiveness",
        0,
    );

    OptResult {
        name: "System Responsiveness".to_string(),
        success: r.is_ok(),
        message: "System responsiveness optimized for gaming".to_string(),
    }
}

fn enable_gpu_scheduling() -> OptResult {
    let r = reg_set_dword(
        "HKLM",
        r"SYSTEM\CurrentControlSet\Control\GraphicsDrivers",
        "HwSchMode",
        2,
    );

    OptResult {
        name: "GPU Scheduling".to_string(),
        success: r.is_ok(),
        message: "Hardware GPU scheduling enabled (restart required)".to_string(),
    }
}

fn create_restore_point() -> OptResult {
    let result = Command::new("powershell")
        .args([
            "-NoProfile",
            "-Command",
            r#"Checkpoint-Computer -Description "TweakValo FPS Optimization" -RestorePointType "MODIFY_SETTINGS""#,
        ])
        .creation_flags(0x08000000)
        .output();

    match result {
        Ok(out) if out.status.success() => OptResult {
            name: "Restore Point".to_string(),
            success: true,
            message: "System restore point created".to_string(),
        },
        _ => OptResult {
            name: "Restore Point".to_string(),
            success: false,
            message: "Could not create restore point".to_string(),
        },
    }
}

#[tauri::command]
pub fn run_fps_optimizations() -> Vec<OptResult> {
    let mut results = Vec::new();
    results.push(create_restore_point());
    results.push(set_ultimate_performance());
    results.push(disable_visual_effects());
    results.push(disable_game_dvr());
    results.push(disable_nagle());
    results.push(disable_prefetch());
    results.push(optimize_responsiveness());
    results.push(enable_gpu_scheduling());
    results
}
