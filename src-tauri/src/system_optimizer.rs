#![allow(non_snake_case)]
use std::process::Command;
use std::os::windows::process::CommandExt;

const CREATE_NO_WINDOW: u32 = 0x08000000;

#[link(name = "ntdll")]
extern "system" {
    fn NtSetTimerResolution(
        DesiredResolution: u32,
        SetResolution: u8,
        CurrentResolution: *mut u32,
    ) -> i32;
}

/// Applies a 0.5ms Timer Resolution natively via NtSetTimerResolution
#[tauri::command]
pub fn set_timer_resolution(enable: bool) -> Result<u32, String> {
    let mut current_res: u32 = 0;
    let status;

    unsafe {
        // Enforce 0.5000 ms resolution (5000 100-ns intervals)
        let requested_res = if enable { 5000 } else { 0 };
        let set_flag = if enable { 1 } else { 0 };
        
        status = NtSetTimerResolution(requested_res, set_flag, &mut current_res);
    }

    if status >= 0 {
        Ok(current_res)
    } else {
        Err(format!("NtSetTimerResolution failed with status: {:#X}", status))
    }
}

/// Runs a memory purge to clear the Windows Standby List (similar to ISLC)
#[tauri::command]
pub fn purge_standby_list() -> Result<(), String> {
    let ps_script = r#"
        Add-Type -TypeDefinition @"
        using System;
        using System.Runtime.InteropServices;
        public class MemClean {
            [DllImport("ntdll.dll")]
            public static extern int NtSetSystemInformation(int SystemInformationClass, IntPtr SystemInformation, int SystemInformationLength);

            [DllImport("advapi32.dll", SetLastError = true)]
            public static extern bool OpenProcessToken(IntPtr ProcessHandle, uint DesiredAccess, out IntPtr TokenHandle);

            [DllImport("advapi32.dll", SetLastError = true)]
            public static extern bool LookupPrivilegeValue(string lpSystemName, string lpName, out LUID lpLuid);

            [DllImport("advapi32.dll", SetLastError = true)]
            public static extern bool AdjustTokenPrivileges(IntPtr TokenHandle, bool DisableAllPrivileges, ref TOKEN_PRIVILEGES NewState, uint BufferLength, IntPtr PreviousState, IntPtr ReturnLength);

            public struct LUID { public uint LowPart; public int HighPart; }
            public struct TOKEN_PRIVILEGES { public uint PrivilegeCount; public LUID Luid; public uint Attributes; }

            public static void Run() {
                IntPtr token;
                OpenProcessToken(System.Diagnostics.Process.GetCurrentProcess().Handle, 0x0020 | 0x0008, out token);
                
                TOKEN_PRIVILEGES tp = new TOKEN_PRIVILEGES();
                tp.PrivilegeCount = 1;
                tp.Attributes = 2; // SE_PRIVILEGE_ENABLED
                LookupPrivilegeValue(null, "SeProfileSingleProcessPrivilege", out tp.Luid);
                AdjustTokenPrivileges(token, false, ref tp, 0, IntPtr.Zero, IntPtr.Zero);

                int cmd = 4; // MemoryPurgeStandbyList
                IntPtr ptr = Marshal.AllocHGlobal(4);
                Marshal.WriteInt32(ptr, cmd);
                NtSetSystemInformation(80, ptr, 4);
                Marshal.FreeHGlobal(ptr);
            }
        }
"@
        [MemClean]::Run()
    "#;

    let child = Command::new("powershell")
        .args(&["-NoProfile", "-NonInteractive", "-Command", ps_script])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    match child {
        Ok(out) => {
            if out.status.success() { Ok(()) }
            else { Err(format!("Failed to purge standby list: {:?}", String::from_utf8_lossy(&out.stderr))) }
        }
        Err(e) => Err(format!("Failed to run empty standby command: {}", e)),
    }
}
