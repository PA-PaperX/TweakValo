use serde::Serialize;
use std::fs;
use std::os::windows::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

const SKIP_FOLDERS: &[&str] = &["CrashReportClient", "WindowsClient"];

#[derive(Debug, Clone, Serialize)]
pub struct StretchResult {
    pub success: bool,
    pub needs_custom_res: bool,
    pub native_w: u32,
    pub native_h: u32,
    pub message: String,
}

pub struct Resolution {
    pub w: u32,
    pub h: u32,
    pub label: &'static str,
    pub desc: &'static str,
}

pub fn get_stretch_resolutions() -> Vec<(&'static str, Resolution)> {
    vec![
        ("1440x1080", Resolution { w: 1440, h: 1080, label: "1440 x 1080", desc: "4:3" }),
        ("1280x960", Resolution { w: 1280, h: 960, label: "1280 x 960", desc: "4:3" }),
        ("1024x768", Resolution { w: 1024, h: 768, label: "1024 x 768", desc: "4:3 classic" }),
    ]
}

fn get_config_base() -> Option<PathBuf> {
    std::env::var("LOCALAPPDATA")
        .ok()
        .map(|l| Path::new(&l).join("VALORANT").join("Saved").join("Config"))
}

fn get_account_folders() -> Vec<PathBuf> {
    let base = match get_config_base() {
        Some(b) => b,
        None => return vec![],
    };
    if !base.exists() {
        return vec![];
    }

    let mut folders = Vec::new();
    if let Ok(entries) = fs::read_dir(&base) {
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name().to_string_lossy().to_string();
            if SKIP_FOLDERS.contains(&name.as_str()) {
                continue;
            }
            let parts: Vec<&str> = name.split('-').collect();
            if parts.len() >= 2 && parts[1].len() > 20 {
                continue;
            }
            let full = entry.path();
            if full.is_dir() {
                folders.push(full);
            }
        }
    }
    folders
}

fn make_writable(path: &Path) {
    if path.exists() {
        if let Err(e) = Command::new("attrib")
            .args(["-r", &path.to_string_lossy()])
            .creation_flags(0x08000000)
            .output() 
        {
            log::error!("Failed to make writable {:?}: {}", path, e);
        }
    }
}

fn backup_file(path: &Path) {
    let backup_path = PathBuf::from(format!("{}.stretch_backup", path.display()));
    if !backup_path.exists() && path.exists() {
        if let Err(e) = fs::copy(path, &backup_path) {
            log::error!("Failed to backup {:?}: {}", path, e);
        }
    }
}

fn modify_game_settings(ini_path: &Path, width: u32, height: u32) -> bool {
    make_writable(ini_path);
    backup_file(ini_path);

    let content = match fs::read_to_string(ini_path) {
        Ok(c) => c,
        Err(_) => return false,
    };

    let mut new_lines = Vec::new();
    let mut found_fullscreen_mode = false;
    let mut found_hdr_idx: Option<usize> = None;

    for line in content.lines() {
        let stripped = line.trim();

        if stripped.starts_with("ResolutionSizeX=") {
            new_lines.push(format!("ResolutionSizeX={}", width));
            continue;
        }
        if stripped.starts_with("ResolutionSizeY=") {
            new_lines.push(format!("ResolutionSizeY={}", height));
            continue;
        }
        if stripped.starts_with("LastUserConfirmedResolutionSizeX=") {
            new_lines.push(format!("LastUserConfirmedResolutionSizeX={}", width));
            continue;
        }
        if stripped.starts_with("LastUserConfirmedResolutionSizeY=") {
            new_lines.push(format!("LastUserConfirmedResolutionSizeY={}", height));
            continue;
        }
        if stripped.starts_with("FullscreenMode=") {
            new_lines.push("FullscreenMode=2".to_string());
            found_fullscreen_mode = true;
            continue;
        }
        if stripped.starts_with("LastConfirmedFullscreenMode=") {
            new_lines.push("LastConfirmedFullscreenMode=0".to_string());
            continue;
        }
        if stripped.starts_with("PreferredFullscreenMode=") {
            new_lines.push("PreferredFullscreenMode=0".to_string());
            continue;
        }
        if stripped.starts_with("bShouldLetterbox=") {
            new_lines.push("bShouldLetterbox=False".to_string());
            continue;
        }
        if stripped.starts_with("bLastConfirmedShouldLetterbox=") {
            new_lines.push("bLastConfirmedShouldLetterbox=False".to_string());
            continue;
        }
        if stripped.starts_with("HDRDisplayOutputNits=") {
            found_hdr_idx = Some(new_lines.len());
        }
        new_lines.push(line.to_string());
    }

    if !found_fullscreen_mode {
        if let Some(idx) = found_hdr_idx {
            new_lines.insert(idx + 1, "FullscreenMode=2".to_string());
        }
    }

    let result = new_lines.join("\n") + "\n";
    fs::write(ini_path, result).is_ok()
}

fn modify_windowsclient_settings(ini_path: &Path, width: u32, height: u32) -> bool {
    make_writable(ini_path);
    backup_file(ini_path);

    let content = match fs::read_to_string(ini_path) {
        Ok(c) => c,
        Err(_) => return false,
    };

    let mut new_lines = Vec::new();
    for line in content.lines() {
        let stripped = line.trim();
        if stripped.starts_with("ResolutionSizeX=") {
            new_lines.push(format!("ResolutionSizeX={}", width));
        } else if stripped.starts_with("ResolutionSizeY=") {
            new_lines.push(format!("ResolutionSizeY={}", height));
        } else if stripped.starts_with("LastUserConfirmedResolutionSizeX=") {
            new_lines.push(format!("LastUserConfirmedResolutionSizeX={}", width));
        } else if stripped.starts_with("LastUserConfirmedResolutionSizeY=") {
            new_lines.push(format!("LastUserConfirmedResolutionSizeY={}", height));
        } else if stripped.starts_with("LastConfirmedFullscreenMode=") {
            new_lines.push("LastConfirmedFullscreenMode=0".to_string());
        } else if stripped.starts_with("PreferredFullscreenMode=") {
            new_lines.push("PreferredFullscreenMode=0".to_string());
        } else if stripped.starts_with("bShouldLetterbox=") {
            new_lines.push("bShouldLetterbox=False".to_string());
        } else if stripped.starts_with("bLastConfirmedShouldLetterbox=") {
            new_lines.push("bLastConfirmedShouldLetterbox=False".to_string());
        } else {
            new_lines.push(line.to_string());
        }
    }

    let result = new_lines.join("\n") + "\n";
    fs::write(ini_path, result).is_ok()
}

fn is_resolution_supported(width: u32, height: u32) -> bool {
    // Use EnumDisplaySettingsW via powershell
    let script = format!(
        r#"Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public class DisplayHelper {{
    [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Ansi)]
    public struct DEVMODE {{
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst=32)] public string dmDeviceName;
        public short dmSpecVersion; public short dmDriverVersion; public short dmSize; public short dmDriverExtra;
        public int dmFields; public int dmPositionX; public int dmPositionY; public int dmDisplayOrientation;
        public int dmDisplayFixedOutput; public short dmColor; public short dmDuplex; public short dmYResolution;
        public short dmTTOption; public short dmCollate;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst=32)] public string dmFormName;
        public short dmLogPixels; public int dmBitsPerPel; public int dmPelsWidth; public int dmPelsHeight;
        public int dmDisplayFlags; public int dmDisplayFrequency;
        public int dmICMMethod; public int dmICMIntent; public int dmMediaType; public int dmDitherType;
        public int dmReserved1; public int dmReserved2; public int dmPanningWidth; public int dmPanningHeight;
    }}
    [DllImport("user32.dll")] public static extern bool EnumDisplaySettings(string lpszDeviceName, int iModeNum, ref DEVMODE lpDevMode);
}}
"@
$dm = New-Object DisplayHelper+DEVMODE; $dm.dmSize = [System.Runtime.InteropServices.Marshal]::SizeOf($dm)
$i = 0; while([DisplayHelper]::EnumDisplaySettings($null, $i, [ref]$dm)) {{ if($dm.dmPelsWidth -eq {0} -and $dm.dmPelsHeight -eq {1}) {{ Write-Output 'FOUND'; exit }} $i++ }}
Write-Output 'NOT_FOUND'"#,
        width, height
    );

    if let Ok(output) = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .creation_flags(0x08000000)
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return stdout.contains("FOUND");
    }
    false
}

fn get_native_resolution() -> (u32, u32) {
    let script = r#"
Add-Type -TypeDefinition @"
using System; using System.Runtime.InteropServices;
public class NativeRes {
    [DllImport("user32.dll")] public static extern int GetSystemMetrics(int nIndex);
}
"@
Write-Output "$([NativeRes]::GetSystemMetrics(0))x$([NativeRes]::GetSystemMetrics(1))"
"#;
    if let Ok(output) = Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .creation_flags(0x08000000)
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let parts: Vec<&str> = stdout.split('x').collect();
        if parts.len() == 2 {
            if let (Ok(w), Ok(h)) = (parts[0].parse(), parts[1].parse()) {
                return (w, h);
            }
        }
    }
    (1920, 1080)
}

fn set_nvidia_scaling_fullscreen() -> bool {
    let ps_script = r#"
$configPath = 'HKLM:\SYSTEM\CurrentControlSet\Control\GraphicsDrivers\Configuration'
$found = 0
Get-ChildItem $configPath -Recurse -ErrorAction SilentlyContinue | ForEach-Object {
    $props = $_ | Get-ItemProperty -ErrorAction SilentlyContinue
    if ($props.PSObject.Properties.Name -contains 'Scaling') {
        if ($props.PSObject.Properties.Name -notcontains 'ScalingBackup') {
            Set-ItemProperty -Path $_.PSPath -Name 'ScalingBackup' -Value $props.Scaling -ErrorAction SilentlyContinue
        }
        Set-ItemProperty -Path $_.PSPath -Name 'Scaling' -Value 3 -ErrorAction SilentlyContinue
        $found++
    }
}
Write-Output $found
"#;
    if let Ok(output) = Command::new("powershell")
        .args(["-NoProfile", "-Command", ps_script])
        .creation_flags(0x08000000)
        .output()
    {
        let count: i32 = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse()
            .unwrap_or(0);
        return count > 0;
    }
    false
}

fn set_desktop_resolution(width: u32, height: u32) -> bool {
    let script = format!(
        r#"Add-Type -TypeDefinition @"
using System; using System.Runtime.InteropServices;
public class DispChange {{
    [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Ansi)]
    public struct DEVMODE {{
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst=32)] public string dmDeviceName;
        public short dmSpecVersion; public short dmDriverVersion; public short dmSize; public short dmDriverExtra;
        public int dmFields; public int dmPositionX; public int dmPositionY; public int dmDisplayOrientation;
        public int dmDisplayFixedOutput; public short dmColor; public short dmDuplex; public short dmYResolution;
        public short dmTTOption; public short dmCollate;
        [MarshalAs(UnmanagedType.ByValTStr, SizeConst=32)] public string dmFormName;
        public short dmLogPixels; public int dmBitsPerPel; public int dmPelsWidth; public int dmPelsHeight;
        public int dmDisplayFlags; public int dmDisplayFrequency;
        public int dmICMMethod; public int dmICMIntent; public int dmMediaType; public int dmDitherType;
        public int dmReserved1; public int dmReserved2; public int dmPanningWidth; public int dmPanningHeight;
    }}
    [DllImport("user32.dll")] public static extern bool EnumDisplaySettings(string n, int i, ref DEVMODE d);
    [DllImport("user32.dll")] public static extern int ChangeDisplaySettingsEx(string n, ref DEVMODE d, IntPtr h, int f, IntPtr l);
}}
"@
$dm = New-Object DispChange+DEVMODE; $dm.dmSize = [System.Runtime.InteropServices.Marshal]::SizeOf($dm)
[DispChange]::EnumDisplaySettings($null, -1, [ref]$dm)
$dm.dmPelsWidth = {0}; $dm.dmPelsHeight = {1}
$dm.dmFields = 0x80000 -bor 0x100000
$r = [DispChange]::ChangeDisplaySettingsEx($null, [ref]$dm, [IntPtr]::Zero, 1, [IntPtr]::Zero)
Write-Output $r"#,
        width, height
    );

    if let Ok(output) = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .creation_flags(0x08000000)
        .output()
    {
        let result: i32 = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse()
            .unwrap_or(-1);
        return result == 0;
    }
    false
}

fn restore_file(path: &Path) -> bool {
    let backup_path = PathBuf::from(format!("{}.stretch_backup", path.display()));
    if backup_path.exists() {
        make_writable(path);
        if let Ok(content) = fs::read_to_string(&backup_path) {
            if let Err(e) = fs::write(path, content) {
                log::error!("Failed to restore {:?}: {}", path, e);
            }
            if let Err(e) = fs::remove_file(&backup_path) {
                log::error!("Failed to remove backup {:?}: {}", backup_path, e);
            }
            return true;
        }
    }
    make_writable(path);
    false
}

// ── Tauri Commands ──

#[tauri::command]
pub fn get_resolutions() -> Vec<(String, String, String)> {
    get_stretch_resolutions()
        .into_iter()
        .map(|(key, r)| (key.to_string(), r.label.to_string(), r.desc.to_string()))
        .collect()
}

#[tauri::command]
pub fn apply_stretch(resolution_key: String, custom_w: u32, custom_h: u32) -> StretchResult {
    let (target_w, target_h) = if resolution_key == "custom" {
        if custom_w < 640 || custom_h < 480 {
            return StretchResult {
                success: false,
                needs_custom_res: false,
                native_w: 0,
                native_h: 0,
                message: "Resolution too small (min 640x480)".to_string(),
            };
        }
        (custom_w, custom_h)
    } else {
        let resolutions = get_stretch_resolutions();
        match resolutions.iter().find(|(k, _)| *k == resolution_key) {
            Some((_, r)) => (r.w, r.h),
            None => {
                return StretchResult {
                    success: false,
                    needs_custom_res: false,
                    native_w: 0,
                    native_h: 0,
                    message: format!("Unknown resolution: {}", resolution_key),
                }
            }
        }
    };

    let (native_w, native_h) = get_native_resolution();

    if !is_resolution_supported(target_w, target_h) {
        return StretchResult {
            success: false,
            needs_custom_res: true,
            native_w,
            native_h,
            message: format!("Resolution {}x{} not in display mode list", target_w, target_h),
        };
    }

    // Set NVIDIA scaling
    set_nvidia_scaling_fullscreen();

    // Change desktop resolution
    let res_ok = set_desktop_resolution(target_w, target_h);

    // Modify all account configs
    let account_folders = get_account_folders();
    let mut applied = 0;
    for folder in &account_folders {
        let game_ini = folder.join("WindowsClient").join("GameUserSettings.ini");
        if game_ini.exists() && modify_game_settings(&game_ini, target_w, target_h) {
            applied += 1;
        }
    }

    // Global WindowsClient config
    if let Some(base) = get_config_base() {
        let wc_ini = base.join("WindowsClient").join("GameUserSettings.ini");
        if wc_ini.exists() {
            modify_windowsclient_settings(&wc_ini, target_w, target_h);
        }
    }

    StretchResult {
        success: res_ok || applied > 0,
        needs_custom_res: false,
        native_w,
        native_h,
        message: format!(
            "Applied {}x{} to {} account(s)",
            target_w, target_h, applied
        ),
    }
}

fn restore_nvidia_scaling() {
    let ps_script = r#"
$configPath = 'HKLM:\SYSTEM\CurrentControlSet\Control\GraphicsDrivers\Configuration'
Get-ChildItem $configPath -Recurse -ErrorAction SilentlyContinue | ForEach-Object {
    $props = $_ | Get-ItemProperty -ErrorAction SilentlyContinue
    if ($props.PSObject.Properties.Name -contains 'ScalingBackup') {
        Set-ItemProperty -Path $_.PSPath -Name 'Scaling' -Value $props.ScalingBackup -ErrorAction SilentlyContinue
    }
}
"#;
    if let Err(e) = Command::new("powershell")
        .args(["-NoProfile", "-Command", ps_script])
        .creation_flags(0x08000000)
        .output()
    {
        log::error!("Failed to restore NVIDIA scaling: {}", e);
    }
}

#[tauri::command]
pub fn revert_stretch(native_w: u32, native_h: u32) -> StretchResult {
    // Restore desktop resolution
    if native_w > 0 && native_h > 0 {
        set_desktop_resolution(native_w, native_h);
    }
    
    // Restore NVIDIA scaling from backup manually via registry script
    restore_nvidia_scaling();

    // Restore all account configs
    let mut reverted = 0;
    for folder in get_account_folders() {
        let game_ini = folder.join("WindowsClient").join("GameUserSettings.ini");
        if game_ini.exists() && restore_file(&game_ini) {
            reverted += 1;
        }
        let riot_ini = folder.join("Windows").join("RiotUserSettings.ini");
        if riot_ini.exists() {
            restore_file(&riot_ini);
        }
    }

    // Restore global
    if let Some(base) = get_config_base() {
        let wc_ini = base.join("WindowsClient").join("GameUserSettings.ini");
        if wc_ini.exists() && restore_file(&wc_ini) {
            reverted += 1;
        }
    }

    StretchResult {
        success: true,
        needs_custom_res: false,
        native_w,
        native_h,
        message: format!("Reverted {} config(s)", reverted),
    }
}

#[tauri::command]
pub fn open_nvidia_cp() -> (bool, String) {
    let nvcp_paths = [
        r"C:\Program Files\NVIDIA Corporation\Control Panel Client\nvcplui.exe",
        r"C:\Windows\System32\nvcplui.exe",
    ];
    for exe in &nvcp_paths {
        if Path::new(exe).exists() {
            if Command::new(exe)
                .creation_flags(0x08000000)
                .spawn()
                .is_ok()
            {
                return (true, "NVIDIA Control Panel opened".to_string());
            }
        }
    }

    // Try MS Store version
    if Command::new("explorer.exe")
        .arg(r"shell:AppsFolder\NVIDIACorp.NVIDIAControlPanel_56jybvy8sckqj!NVIDIACorp.NVIDIAControlPanel")
        .creation_flags(0x08000000)
        .spawn()
        .is_ok()
    {
        return (true, "NVIDIA Control Panel opened".to_string());
    }

    (false, "NVIDIA Control Panel not found".to_string())
}
