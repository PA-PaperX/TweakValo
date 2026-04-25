use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::os::windows::process::CommandExt;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::System;

const GAME_PROCESS: &str = "VALORANT-Win64-Shipping.exe";
const DEFAULT_PAKS_DIR: &str = r"C:\Riot Games\VALORANT\live\ShooterGame\Content\Paks";

const BLOOD_FILES: &[&str] = &[
    "MatureData-WindowsClient.pak",
    "MatureData-WindowsClient.sig",
    "MatureData-WindowsClient.ucas",
    "MatureData-WindowsClient.utoc",
];

const RIOT_CLIENT_PATHS: &[&str] = &[
    r"C:\Riot Games\Riot Client\RiotClientServices.exe",
    r"D:\Riot Games\Riot Client\RiotClientServices.exe",
    r"E:\Riot Games\Riot Client\RiotClientServices.exe",
    r"F:\Riot Games\Riot Client\RiotClientServices.exe",
    r"C:\Program Files\Riot Games\Riot Client\RiotClientServices.exe",
    r"D:\Program Files\Riot Games\Riot Client\RiotClientServices.exe",
    r"C:\Program Files (x86)\Riot Games\Riot Client\RiotClientServices.exe",
];

#[derive(Debug, Clone, Serialize)]
pub struct LaunchResult {
    pub success: bool,
    pub message: String,
}

fn get_backup_dir() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));
    exe_dir.join(".originals_backup")
}

fn get_blood_dir(app_handle: &tauri::AppHandle) -> Option<PathBuf> {
    app_handle
        .resolve_resource("resources/blood")
        .or_else(|| {
            // Fallback for dev mode
            let exe = std::env::current_exe().ok()?;
            let base = exe.parent()?.parent()?.parent()?;
            Some(base.join("resources").join("blood"))
        })
}

pub fn is_game_running() -> bool {
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    sys.processes().values().any(|p| {
        p.name()
            .to_str()
            .map(|n| n == GAME_PROCESS)
            .unwrap_or(false)
    })
}

fn is_riot_client_running() -> bool {
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    sys.processes().values().any(|p| {
        p.name()
            .to_str()
            .map(|n| n == "RiotClientServices.exe")
            .unwrap_or(false)
    })
}

pub fn find_riot_client(custom_path: &str) -> Option<String> {
    if !custom_path.is_empty() && Path::new(custom_path).exists() {
        return Some(custom_path.to_string());
    }

    // Check registry
    if let Ok(hklm) = winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE)
        .open_subkey(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Riot Game valorant.live")
    {
        if let Ok(loc) = hklm.get_value::<String, _>("InstallLocation") {
            let riot = Path::new(&loc)
                .parent()
                .and_then(|p| p.parent())
                .map(|p| p.join("Riot Client").join("RiotClientServices.exe"));
            if let Some(path) = riot {
                if path.exists() {
                    return Some(path.to_string_lossy().to_string());
                }
            }
        }
    }

    // Check RiotClientInstalls.json
    if let Ok(programdata) = std::env::var("PROGRAMDATA") {
        let installs = Path::new(&programdata)
            .join("Riot Games")
            .join("RiotClientInstalls.json");
        if installs.exists() {
            if let Ok(content) = fs::read_to_string(&installs) {
                if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(obj) = data.as_object() {
                        for v in obj.values() {
                            if let Some(s) = v.as_str() {
                                if s.ends_with(".exe") && Path::new(s).exists() {
                                    return Some(s.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Check running process
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    for p in sys.processes().values() {
        if p.name().to_str() == Some("RiotClientServices.exe") {
            if let Some(exe) = p.exe() {
                return Some(exe.to_string_lossy().to_string());
            }
        }
    }

    // Check known paths
    for path in RIOT_CLIENT_PATHS {
        if Path::new(path).exists() {
            return Some(path.to_string());
        }
    }

    // Also check LOCALAPPDATA
    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let path = Path::new(&local)
            .join("Riot Games")
            .join("Riot Client")
            .join("RiotClientServices.exe");
        if path.exists() {
            return Some(path.to_string_lossy().to_string());
        }
    }

    None
}

fn resolve_paks_dir(base: &str) -> Option<PathBuf> {
    for sub in &["live/ShooterGame/Content/Paks", "ShooterGame/Content/Paks"] {
        let p = Path::new(base).join(sub);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn get_paks_dir(game_path: &str) -> Option<PathBuf> {
    if !game_path.is_empty() {
        if let Some(p) = resolve_paks_dir(game_path) {
            return Some(p);
        }
    }
    let default = Path::new(DEFAULT_PAKS_DIR);
    if default.exists() {
        Some(default.to_path_buf())
    } else {
        None
    }
}

fn read_lockfile() -> Option<(u16, String)> {
    let local = std::env::var("LOCALAPPDATA").ok()?;
    let path = Path::new(&local)
        .join("Riot Games")
        .join("Riot Client")
        .join("Config")
        .join("lockfile");
    let content = fs::read_to_string(path).ok()?;
    let parts: Vec<&str> = content.trim().split(':').collect();
    if parts.len() >= 5 {
        let port = parts[2].parse().ok()?;
        let password = parts[3].to_string();
        Some((port, password))
    } else {
        None
    }
}

fn riot_api_launch(port: u16, password: &str) -> bool {
    let url = format!(
        "https://127.0.0.1:{}/product-launcher/v1/products/valorant/patchlines/live",
        port
    );
    match reqwest::blocking::Client::builder().danger_accept_invalid_certs(true).build() {
        Ok(client) => match client.post(&url).basic_auth("riot", Some(password)).send() {
            Ok(res) => {
                let status = res.status();
                if !status.is_success() {
                    log::warn!("riot_api_launch got HTTP {}", status);
                }
                status.is_success()
            }
            Err(e) => {
                log::warn!("riot_api_launch request failed: {}", e);
                false
            }
        },
        Err(e) => {
            log::warn!("riot_api_launch client build failed: {}", e);
            false
        }
    }
}

fn riot_api_ping(port: u16, password: &str) -> bool {
    let url = format!("https://127.0.0.1:{}/riotclient/region-locale", port);
    match reqwest::blocking::Client::builder().danger_accept_invalid_certs(true).build() {
        Ok(client) => match client.get(&url).basic_auth("riot", Some(password)).send() {
            Ok(res) => res.status().is_success(),
            Err(_) => false,
        },
        Err(_) => false,
    }
}



fn cleanup_watcher(
    paks_dir: PathBuf,
    backup_dir: PathBuf,
    injected_blood: Vec<String>,
    running: Arc<AtomicBool>,
) {
    // Wait for game to exit
    while is_game_running() && running.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_secs(2));
    }

    // Restore
    for fname in &injected_blood {
        let game_path = paks_dir.join(fname);
        let backup_path = backup_dir.join(fname);
        if backup_path.exists() {
            if let Err(e) = fs::copy(&backup_path, &game_path) {
                log::error!("Failed to restore backup {}: {}", fname, e);
            }
            let _ = fs::remove_file(&backup_path);
        } else if game_path.exists() {
            if let Err(e) = fs::remove_file(&game_path) {
                log::error!("Failed to remove injected file {}: {}", fname, e);
            }
        }
    }

    // Clean empty backup dir
    if backup_dir.exists() {
        if let Ok(entries) = fs::read_dir(&backup_dir) {
            if entries.count() == 0 {
                let _ = fs::remove_dir(&backup_dir);
            }
        }
    }
}

#[allow(dead_code)]
pub fn emergency_cleanup(paks_dir_opt: Option<&str>) {
    let backup_dir = get_backup_dir();
    if !backup_dir.exists() {
        return;
    }

    let paks_dir = match paks_dir_opt {
        Some(p) if Path::new(p).exists() => PathBuf::from(p),
        _ => {
            let default = Path::new(DEFAULT_PAKS_DIR);
            if default.exists() {
                default.to_path_buf()
            } else {
                return;
            }
        }
    };

    if let Ok(entries) = fs::read_dir(&backup_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let fname = entry.file_name();
            let backup_path = entry.path();
            let game_path = paks_dir.join(&fname);
            if let Err(e) = fs::copy(&backup_path, &game_path) {
                log::error!("Emergency cleanup failed to restore {:?}: {}", fname, e);
            }
            let _ = fs::remove_file(&backup_path);
        }
    }

    // Remove injected blood files without backups
    for fname in BLOOD_FILES {
        let game_path = paks_dir.join(fname);
        if game_path.exists() && !backup_dir.join(fname).exists() {
            if let Err(e) = fs::remove_file(&game_path) {
                log::error!("Emergency cleanup failed to remove injected {:?}: {}", fname, e);
            }
        }
    }

    if let Ok(entries) = fs::read_dir(&backup_dir) {
        if entries.count() == 0 {
            let _ = fs::remove_dir(&backup_dir);
        }
    }
}

// ── Tauri Commands ──

#[tauri::command]
pub fn check_riot_client(custom_path: String) -> LaunchResult {
    match find_riot_client(&custom_path) {
        Some(path) => LaunchResult {
            success: true,
            message: path,
        },
        None => LaunchResult {
            success: false,
            message: "Riot Client not found".to_string(),
        },
    }
}

#[tauri::command]
pub fn check_game_path(game_path: String) -> LaunchResult {
    if !game_path.is_empty() {
        if let Some(paks) = resolve_paks_dir(&game_path) {
            return LaunchResult {
                success: true,
                message: paks.to_string_lossy().to_string(),
            };
        }
    }
    if Path::new(DEFAULT_PAKS_DIR).exists() {
        return LaunchResult {
            success: true,
            message: DEFAULT_PAKS_DIR.to_string(),
        };
    }
    LaunchResult {
        success: false,
        message: "Path not found".to_string(),
    }
}

#[tauri::command]
pub async fn launch_game(
    app_handle: tauri::AppHandle,
    game_path: String,
    riot_client_path: String,
    enable_blood: bool,
) -> Result<Vec<String>, String> {
    let mut logs: Vec<String> = Vec::new();

    // Step 1: Find Riot Client
    let riot_exe = find_riot_client(&riot_client_path)
        .ok_or("Riot Client not found")?;
    logs.push(format!("Riot Client: {}", riot_exe));

    // Step 2: Check if Riot Client is already running
    let already_running = is_riot_client_running();
    if already_running {
        logs.push("Riot Client is already running.".to_string());
    }

    // Step 3: Only start a NEW Riot Client if none is running
    if !already_running {
        logs.push("Starting Riot Client...".to_string());
        Command::new("cmd")
            .args([
                "/c", "start", "", &riot_exe,
                "--launch-product=valorant",
                "--launch-patchline=live",
            ])
            .creation_flags(0x08000000)
            .spawn()
            .map_err(|e| format!("Failed to start Riot Client: {}", e))?;
    }

    // Step 4: Wait for lockfile to appear
    logs.push("Waiting for Riot Client lockfile...".to_string());
    let lockfile_path = std::env::var("LOCALAPPDATA")
        .map(|l| {
            Path::new(&l)
                .join("Riot Games")
                .join("Riot Client")
                .join("Config")
                .join("lockfile")
        })
        .unwrap_or_default();

    let start = Instant::now();
    let wait_secs = if already_running { 5 } else { 30 };
    while start.elapsed() < Duration::from_secs(wait_secs) {
        if lockfile_path.exists() && is_riot_client_running() {
            break;
        }
        thread::sleep(Duration::from_millis(500));
    }

    // Step 5: Wait for API to be ready — re-read lockfile each attempt in case Riot Client restarts
    // Give the API a moment to initialize after lockfile appears
    thread::sleep(Duration::from_secs(2));

    logs.push("Waiting for Riot Client API to be ready...".to_string());
    let mut api_port: u16 = 0;
    let mut api_password = String::new();
    let mut api_ready = false;

    for attempt in 1..=25 {
        if let Some((port, password)) = read_lockfile() {
            if port != api_port {
                if api_port > 0 {
                    logs.push(format!("Lockfile port changed: {} → {}", api_port, port));
                } else {
                    logs.push(format!("Lockfile found — API on port {}", port));
                }
                api_port = port;
                api_password = password.clone();
            }
            if riot_api_ping(port, &password) {
                logs.push(format!("Riot Client API ready (attempt {})", attempt));
                api_ready = true;
                break;
            }
        }
        thread::sleep(Duration::from_secs(1));
    }

    if api_ready && api_port > 0 {
        // Step 6: Trigger Play via API
        logs.push("Triggering Play via API...".to_string());
        let mut api_success = false;
        for attempt in 1..=5 {
            // Re-read lockfile in case port changed during retries
            let (port, password) = read_lockfile().unwrap_or((api_port, api_password.clone()));
            if riot_api_launch(port, &password) {
                api_success = true;
                logs.push(format!("Play triggered on attempt {}", attempt));
                break;
            }
            logs.push(format!("Launch attempt {} failed, retrying...", attempt));
            thread::sleep(Duration::from_secs(2));
        }
        if api_success {
            logs.push("Play triggered — waiting for game...".to_string());
        } else {
            logs.push("API launch failed after 5 attempts — please click Play manually".to_string());
        }
    } else {
        logs.push("API did not become ready in time — please click Play manually".to_string());
    }

    if !enable_blood {
        logs.push("Launched VALORANT without content injection".to_string());
        return Ok(logs);
    }

    // Wait for game process
    let timeout = Duration::from_secs(300);
    let start = Instant::now();
    let mut game_found = false;
    while start.elapsed() < timeout {
        if is_game_running() {
            logs.push("VALORANT DETECTED — injecting mods!".to_string());
            game_found = true;
            break;
        }
        thread::sleep(Duration::from_millis(50));
    }

    if !game_found {
        return Err("Timeout: VALORANT did not start within 5 minutes.".to_string());
    }

    // Inject mods
    let paks_dir = get_paks_dir(&game_path);
    let paks_dir = match paks_dir {
        Some(p) => p,
        None => return Err("Game Paks folder not found".to_string()),
    };

    let backup_dir = get_backup_dir();
    fs::create_dir_all(&backup_dir)
        .map_err(|e| format!("Failed to create backup directory: {}", e))?;

    let t0 = Instant::now();
    let mut injected_blood = Vec::new();
    let mut errors = Vec::new();

    if enable_blood {
        // Try to find blood dir from resources
        let blood_dir = get_blood_dir(&app_handle);
        if let Some(blood_dir) = blood_dir {
            for fname in BLOOD_FILES {
                let src = blood_dir.join(fname);
                let dst = paks_dir.join(fname);
                if src.exists() {
                    if dst.exists() {
                        if let Err(e) = fs::copy(&dst, backup_dir.join(fname)) {
                            log::error!("Failed to backup existing {}: {}", fname, e);
                        }
                    }
                    match fs::copy(&src, &dst) {
                        Ok(_) => injected_blood.push(fname.to_string()),
                        Err(e) => errors.push(format!("Copy {}: {}", fname, e)),
                    }
                }
            }
        } else {
            errors.push("Blood mod folder not found in resources".to_string());
        }
    }

    let elapsed = t0.elapsed().as_millis();
    if errors.is_empty() {
        logs.push(format!("Mods applied in {}ms — SUCCESS!", elapsed));
    } else {
        logs.push(format!("Mods applied in {}ms with errors:", elapsed));
        for err in &errors {
            logs.push(format!("  - {}", err));
        }
    }

    // Start cleanup watcher
    logs.push("Cleanup watcher active — files will be restored on game close".to_string());
    let running = Arc::new(AtomicBool::new(true));
    let paks_clone = paks_dir.clone();
    let backup_clone = backup_dir.clone();
    thread::spawn(move || {
        cleanup_watcher(paks_clone, backup_clone, injected_blood, running);
    });

    Ok(logs)
}

#[tauri::command]
pub fn check_game_running() -> bool {
    is_game_running()
}

// Trait helper for path_resolver (Tauri v2 compat)
trait PathResolverExt {
    fn resolve_resource(&self, path: &str) -> Option<PathBuf>;
}

impl PathResolverExt for tauri::AppHandle {
    fn resolve_resource(&self, path: &str) -> Option<PathBuf> {
        use tauri::Manager;
        let resource_dir = self.path().resource_dir().ok()?;
        let full = resource_dir.join(path);
        if full.exists() { Some(full) } else { None }
    }
}
