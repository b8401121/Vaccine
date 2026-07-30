#![windows_subsystem = "windows"]

use std::env;
use std::fs;
use std::process::Command;

fn main() {
    let temp_dir = env::temp_dir().join("VaccineApp_Standalone");
    if let Err(_) = fs::create_dir_all(&temp_dir) {
        return;
    }

    let app_exe_bytes = include_bytes!("../../vaccine-app/src-tauri/target/release/vaccine-app.exe");
    let dll_bytes = include_bytes!("../../vaccine-app/src-tauri/target/release/WebView2Loader.dll");

    let target_app_exe = temp_dir.join("vaccine-app.exe");
    let target_dll = temp_dir.join("WebView2Loader.dll");

    let _ = fs::write(&target_app_exe, app_exe_bytes);
    let _ = fs::write(&target_dll, dll_bytes);

    let _ = Command::new(&target_app_exe)
        .current_dir(&temp_dir)
        .status();
}
