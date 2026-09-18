use super::*;
use crate::image_engine::ImageEngine;

#[tauri::command]
pub async fn project_add_gallery(
    id: String,
    root_id: String,
    targets: Vec<gallery::FileTarget>,
    state: tauri::State<'_, Arc<Projects>>,
    core: tauri::State<'_, Arc<Core>>,
) -> Result<Project> {
    let p = state.inner().clone();
    let core = core.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let sources = gallery::project_sources(&core, &root_id, &targets)?;
        p.add_to(
            &id,
            sources
                .iter()
                .map(|(path, _, _)| path.to_string_lossy().into())
                .collect(),
        )
    })
    .await
    .map_err(err)?
}
#[tauri::command]
pub async fn project_add_image(
    id: String,
    job_id: String,
    state: tauri::State<'_, Arc<Projects>>,
    images: tauri::State<'_, Arc<ImageEngine>>,
) -> Result<Project> {
    let p = state.inner().clone();
    let images = images.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let source = images.project_source(&job_id)?;
        p.add_to(&id, vec![source.0.to_string_lossy().into()])
    })
    .await
    .map_err(err)?
}
