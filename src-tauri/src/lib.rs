mod benchmarks;
use benchmarks::*;
mod desktop_features;
use desktop_features::{background_hide,desktop_accent};
use hardware::hardware_live;
mod resources;
use resources::resource_status;
mod ai;
use ai::*;
mod updater;
use updater::*;
pub use updater::helper as run_update_helper;
mod app_menu;
use app_menu::app_menu_update;
mod project_opens;
use project_opens::*;
mod projects;
mod maintenance;
use maintenance::*;
use projects::*;
mod gallery;
use gallery::*;
mod image_engine;
use image_engine::*;
mod model_library;
use model_library::*;
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
mod shortcuts;
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
    app: tauri::AppHandle,
    state: State<'_, Arc<Core>>,
) -> Result<Settings, String> {
    let core = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {let saved=core.save_settings(settings)?;resources::configure(saved.parallel_generation);desktop_features::refresh(&app,saved.language=="de")?;Ok(saved)})
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
async fn mark_clean_exit(image_action: Option<ImageExitAction>, images: State<'_, Arc<ImageEngine>>, library: State<'_, Arc<ModelLibrary>>, downloads: State<'_, Arc<Downloads>>, auth: State<'_, Arc<hf_auth::HfAuth>>, state: State<'_, Arc<Core>>) -> Result<(), String> {
    let images = images.inner().clone();
    let settings = state.current_settings()?;
    let gallery = state.storage_paths()?.gallery;
    tauri::async_runtime::spawn_blocking(move || images.finish_session(image_action, std::path::Path::new(&gallery), settings.restore_session)).await.map_err(|_| "image_storage")??;
    library.shutdown().await;
    let _ = auth.cancel();
    downloads.shutdown().await;
    let core = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || core.shutdown())
        .await
        .map_err(|e| e.to_string())?
}

pub fn run() {
    let pending=ProjectOpens::new();pending.receive(std::env::args().collect(),&std::env::current_dir().unwrap_or_default());
    let mut context=tauri::generate_context!();
    // Portable installations and isolated native tests use independent instance namespaces.
    if let Ok(exe)=std::env::current_exe(){if exe.parent().is_some_and(|p|p.join("portable.marker").is_file()) {
        use sha2::{Digest,Sha256};context.config_mut().identifier=format!("de.localstudio.portable.{:x}",Sha256::digest(exe.parent().unwrap().to_string_lossy().to_lowercase().as_bytes()));
    }}
    let app = tauri::Builder::default()
        .manage(pending)
        .on_menu_event(app_menu::event)
        .plugin(tauri_plugin_single_instance::init(project_opens::second))
        .plugin(tauri_plugin_dialog::init())
        .register_asynchronous_uri_scheme_protocol("gallery", gallery::protocol)
        .register_asynchronous_uri_scheme_protocol("project", projects::preview::protocol)
        .register_asynchronous_uri_scheme_protocol("video", projects::creative::video::protocol)
        .setup(|app| {
            let executable = std::env::current_exe()?;
            let executable_dir = executable.parent().ok_or("Application directory unavailable.")?;
            let portable = executable_dir.join("portable.marker").is_file();
            #[allow(unused_mut)]
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
            let runtime_dir = executable_dir.join("image-runtime");
            #[cfg(debug_assertions)]
            let runtime_dir = if runtime_dir.is_dir() { runtime_dir } else { std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.tools/image-runtime") };
            app.manage(Updater::new(&config_dir,executable_dir,portable));
            app.manage(Projects::new(&config_dir)?);
            let video_runtime=executable_dir.join("video-runtime");
            #[cfg(debug_assertions)] let video_runtime=if video_runtime.is_dir(){video_runtime}else{std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../.tools/video-runtime")};
            app.manage(VideoEngine::new(&config_dir,video_runtime)?);
            app.manage(AiEngine::new(&config_dir,executable_dir.to_path_buf())?);
            app.manage(Arc::new(Maintenance::new()));
            app.manage(GalleryCatalog::new(&config_dir)?);
            app.manage(Arc::new(GalleryWatch::new()));
            app.manage(Benchmarks::new(&config_dir)?);
            app.manage(ImageEngine::new(&config_dir, runtime_dir)?);
            app.manage(ModelLibrary::new(&config_dir)?);
            app.manage(Downloads::new(&config_dir, auth.clone())?);
            app.manage(auth);
            app.manage(hf_browser::HfBrowser::new(&config_dir));
            resources::configure(core.current_settings()?.parallel_generation);
            core.start_scheduler();
            app.manage(core);
            desktop_features::install(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![background_hide,desktop_accent,hardware_live,resource_status,ai_catalog,ai_models,ai_import,ai_download_plan,ai_adopt_download,assistant_status,assistant_load,assistant_unload,assistant_send,assistant_cancel,assistant_clear,transcription_start,transcription_jobs,transcription_cancel,video_frame,media_prepare,media_status,media_cancel,media_info,caption_read,caption_write,project_creative_save,canvas_preview,canvas_export,canvas_export_mask,video_probe,video_start,video_jobs,video_cancel,project_editor_preview,project_editor_save,project_editor_export,project_add_edit,update_status,update_check,update_download,update_cancel,update_arm,update_disarm,update_open_download, app_menu_update, editor_preview, editor_export, project_ack_open, gallery_watch, gallery_video_thumbnail_store, gallery_lineage, gallery_create_variant, gallery_set_primary, project_recent, project_forget_recent, project_take_open, project_history, project_checkpoint, project_restore_point, project_rename, project_restore_media, project_add_gallery, project_add_image, storage_cleanup_preview, storage_cleanup_apply, storage_cleanup_auto, project_snapshot, project_new, project_open, project_save, project_update, project_add, project_remove, project_close, project_recover, project_relink, project_export_gallery, gallery_compare, gallery_file_action, gallery_trash_list, gallery_trash_action, gallery_trash_detail, gallery_annotate_batch, gallery_thumbnail, gallery_thumbnail_clear, gallery_annotate, gallery_list, gallery_detail, gallery_import, gallery_create_folder, gallery_open_folder, image_workspace, image_workspace_save, image_recover, image_discard, image_resume, image_probe, image_jobs, image_generate, image_cancel, image_output, image_save, image_generate_batch, image_reference, preferences_list, preference_save, preference_forget, benchmark_list, benchmark_clear, model_updates_status, model_updates_check, model_updates_prepare, model_move_plan, model_move_start, model_move_status, model_move_cancel, model_library_list, model_scan_start, model_scan_full, model_scan_quick, model_scan_cancel, model_library_recheck, model_library_forget, download_list, download_plan, download_start, download_action, hf_browser_mount, hf_browser_layout, hf_browser_hide, hf_browser_state, hf_browser_action, hf_status, hf_start_login, hf_cancel_login, hf_connect_token, hf_logout, hf_verify, hf_search, hf_model_detail, hf_model_size, hf_open_page, bootstrap, save_settings, get_hardware, list_jobs, enqueue_hash_job, cancel_job, dismiss_recovery, get_logs, mark_clean_exit])
        .build(context)
        .expect("Local Studio could not initialize. Check that the local configuration folder is writable and no other instance is using it.");
    app.run(|handle, event| {
        if matches!(
            event,
            tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit
        ) {
            if let Some(ai)=handle.try_state::<Arc<AiEngine>>(){ai.stop();}
            if let Some(video)=handle.try_state::<Arc<VideoEngine>>(){video.stop();}
            if let Some(images) = handle.try_state::<Arc<ImageEngine>>() { images.stop(); }
            if let Some(library) = handle.try_state::<Arc<ModelLibrary>>() { library.stop(); }
            if let Some(downloads) = handle.try_state::<Arc<Downloads>>() { downloads.stop(); }
            if let Some(auth) = handle.try_state::<Arc<hf_auth::HfAuth>>() { let _ = auth.cancel(); }
            if let Some(core) = handle.try_state::<Arc<Core>>() {
                let _ = core.shutdown();
            }
        }
    });
}
