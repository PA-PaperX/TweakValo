use std::thread;
use std::time::Duration;
use sysinfo::System;
use windows::Win32::System::Threading::{
    GetProcessAffinityMask, OpenProcess, SetPriorityClass, SetProcessAffinityMask,
    HIGH_PRIORITY_CLASS, PROCESS_QUERY_INFORMATION, PROCESS_SET_INFORMATION,
};

pub fn spawn_director_thread() {
    thread::spawn(move || {
        let mut sys = System::new();
        
        loop {
            // Read config dynamically to catch live toggles
            let exe_dir = std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| std::path::PathBuf::from("."));
            let config_path = exe_dir.join("config.json");
            
            let mut auto_priority = false;
            let mut auto_affinity = false;
            
            if let Ok(content) = std::fs::read_to_string(&config_path) {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) {
                    auto_priority = v.get("auto_priority").and_then(|v| v.as_bool()).unwrap_or(false);
                    auto_affinity = v.get("auto_affinity").and_then(|v| v.as_bool()).unwrap_or(false);
                }
            }

            if auto_priority || auto_affinity {
                sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
                
                let valo_processes: Vec<u32> = sys.processes().values()
                    .filter(|p| {
                        let name = p.name().to_str().unwrap_or("").to_lowercase();
                        name == "valorant-win64-shipping.exe"
                    })
                    .map(|p| p.pid().as_u32())
                    .collect();

                for pid in valo_processes {
                    apply_optimizations(pid, auto_priority, auto_affinity);
                }
            }

            thread::sleep(Duration::from_secs(4));
        }
    });
}

fn apply_optimizations(pid: u32, use_priority: bool, use_affinity: bool) {
    unsafe {
        if let Ok(handle) = OpenProcess(PROCESS_SET_INFORMATION | PROCESS_QUERY_INFORMATION, false, pid) {
            if use_priority {
                let _ = SetPriorityClass(handle, HIGH_PRIORITY_CLASS);
            }
            if use_affinity {
                let mut proc_mask = 0;
                let mut sys_mask = 0;
                if GetProcessAffinityMask(handle, &mut proc_mask, &mut sys_mask).is_ok() {
                    // Disable Core 0 (Bit 0) by doing a bitwise AND NOT 1
                    let new_mask = proc_mask & !1;
                    if new_mask > 0 {
                        let _ = SetProcessAffinityMask(handle, new_mask);
                    }
                }
            }
            let _ = windows::Win32::Foundation::CloseHandle(handle);
        }
    }
}
