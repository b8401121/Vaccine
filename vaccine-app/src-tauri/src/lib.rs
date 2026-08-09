#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use vaccine_core::*;

#[tauri::command]
fn get_eligible_vaccines(
    year: i32,
    month: u32,
    day: u32,
    is_roc: bool,
    gender: String,
    location: String,
) -> Result<VaccineResponse, String> {
    get_eligible_vaccines_core(year, month, day, is_roc, gender, location)
}

#[tauri::command]
fn get_all_vaccines() -> Vec<VaccineDetailDoc> {
    get_all_vaccines_core()
}

#[tauri::command]
fn calculate_catch_up(
    vaccine_id: String,
    last_dose_num: i32,
    year: i32,
    month: u32,
    day: u32,
    is_roc: bool,
) -> Result<CatchUpResponse, String> {
    calculate_catch_up_core(vaccine_id, last_dose_num, year, month, day, is_roc)
}

#[tauri::command]
fn get_travel_advisory(
    destination: String,
    purpose: String,
) -> Result<TravelAdvisoryResponse, String> {
    get_travel_advisory_core(destination, purpose)
}

#[tauri::command]
fn calculate_growth_percentile(
    gender: String,
    age_months: i32,
    height: f64,
    weight: f64,
    head: Option<f64>,
) -> Result<GrowthResponse, String> {
    calculate_growth_percentile_core(gender, age_months, height, weight, head)
}

#[tauri::command]
fn launch_external_calendar_url(app: tauri::AppHandle, url: String) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;
    if let Err(e) = app.opener().open_url(url, None::<&str>) {
        return Err(format!("Failed to open URL: {}", e));
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn ensure_webview2_loader() {
    let dll_path = std::env::current_exe().unwrap().parent().unwrap().join("WebView2Loader.dll");
    let loader_bytes = include_bytes!("../WebView2Loader.dll");
    if !dll_path.exists() {
        let _ = std::fs::write(&dll_path, loader_bytes);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "windows")]
    ensure_webview2_loader();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_eligible_vaccines,
            get_all_vaccines,
            calculate_catch_up,
            get_travel_advisory,
            calculate_growth_percentile,
            launch_external_calendar_url
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
