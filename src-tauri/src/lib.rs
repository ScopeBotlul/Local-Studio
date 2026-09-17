mod downloads;
use downloads::*;
mod hf_browser;
use hf_browser::*;
mod hf_auth;
mod hub;
mod hub_commands;
use hub_commands::*;
mod core;
mod database;
mod hardware;
mod settings;
mod types;
mod worker;

use core::Core;
use std::sync::Arc;
use tauri::{Manager, State};
use types::{AppSnapshot, HardwareInfo, Job, Settings};
pub use worker::run_worker;

#[tauri::command]
async fn bootstrap(state: State<'_, Arc<Core>>) -> Result<AppSnapshot, String> {
    let core = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || core.snapshot())
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn save_settings(
    settings: Settings,
    state: State<'_, Arc<Core>>,
) -> Result<Settings, String> {
    let core = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || core.save_settings(settings))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_hardware() -> Result<HardwareInfo, String> {
    tauri::async_runtime::spawn_blocking(hardware::discover)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn list_jobs(state: State<'_, Arc<Core>>) -> Result<Vec<Job>, String> {
    state.jobs()
}

#[tauri::command]
async fn enqueue_hash_job(path: String, state: State<'_, Arc<Core>>) -> Result<Job, String> {
    let core = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || core.enqueue(&path))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn cancel_job(id: String, state: State<'_, Arc<Core>>) -> Result<(), String> {
    let core = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || core.cancel(&id))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn dismiss_recovery(state: State<'_, Arc<Core>>) -> Result<(), String> {
    state.dismiss_recovery()
}

#[tauri::command]
fn get_logs(state: State<'_, Arc<Core>>) -> Result<String, String> {
    state.logs()
}

#[tauri::command]
async fn mark_clean_exit(downloads: State<'_, Arc<Downloads>>, auth: State<'_, Arc<hf_auth::HfAuth>>, state: State<'_, Arc<Core>>) -> Result<(), String> {
    let _ = auth.cancel();
    downloads.shutdown().await;
    let core = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || core.shutdown())
        .await
        .map_err(|e| e.to_string())?
}

pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let executable = std::env::current_exe()?;
            let executable_dir = executable.parent().ok_or("Application directory unavailable.")?;
            let portable = executable_dir.join("portable.marker").is_file();
            let mut config_dir = if portable { executable_dir.join("Local-Studio-Data/config") } else { app.path().app_config_dir()? };
            #[cfg(debug_assertions)]
            if let Some(isolated) = std::env::var_os("LOCAL_STUDIO_CONFIG_DIR") {
                let path = std::path::PathBuf::from(isolated);
                if !path.is_absolute() { return Err("LOCAL_STUDIO_CONFIG_DIR must be absolute.".into()); }
                config_dir = path;
            }
            let data_dir = if portable { executable_dir.join("Local-Studio-Data") } else { config_dir.join("Data") };
            let core = Core::new(config_dir.clone(), data_dir, portable)?;
            let auth = hf_auth::HfAuth::new(&config_dir);
            app.manage(Downloads::new(&config_dir, auth.clone())?);
            app.manage(auth);
            app.manage(hf_browser::HfBrowser::new(&config_dir));
            core.start_scheduler();
            app.manage(core);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![download_list, download_plan, download_start, download_action, hf_browser_mount, hf_browser_layout, hf_browser_hide, hf_browser_state, hf_browser_action, hf_status, hf_start_login, hf_cancel_login, hf_connect_token, hf_logout, hf_verify, hf_search, hf_model_detail, hf_model_size, hf_open_page, bootstrap, save_settings, get_hardware, list_jobs, enqueue_hash_job, cancel_job, dismiss_recovery, get_logs, mark_clean_exit])
        .build(tauri::generate_context!())
        .expect("Local Studio could not initialize. Check that the local configuration folder is writable and no other instance is using it.");
    app.run(|handle, event| {
        if matches!(
            event,
            tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit
        ) {
            if let Some(downloads) = handle.try_state::<Arc<Downloads>>() { downloads.stop(); }
            if let Some(auth) = handle.try_state::<Arc<hf_auth::HfAuth>>() { let _ = auth.cancel(); }
            if let Some(core) = handle.try_state::<Arc<Core>>() {
                let _ = core.shutdown();
            }
        }
    });
}
