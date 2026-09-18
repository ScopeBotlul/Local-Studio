//! A mask uses the same renderer as the layer editor. No second brush implementation.
use super::*;

#[tauri::command]
pub async fn canvas_export_mask(
    id: String,
    document: Canvas,
    layer_id: String,
    state: tauri::State<'_, Arc<Projects>>,
    core: tauri::State<'_, Arc<Core>>,
    catalog: tauri::State<'_, Arc<gallery::GalleryCatalog>>,
) -> Result<String> {let privacy_epoch=crate::privacy::epoch();let privacy_result=(async {
    let permit = gallery::editor_permit().await?;
    let projects = state.inner().clone();
    let catalog = catalog.inner().clone();
    let root = gallery::root(&core)?;
    tauri::async_runtime::spawn_blocking(move || {
        let _permit = permit;
        let project = projects.creative_snapshot(&id)?;
        validate(&Creative { revision: 0, image: Some(document.clone()), video: None }, &project.assets)?;
        let layer = document.layers.iter().find(|l| l.id == layer_id).ok_or("creative_source")?;
        let mask = layer.mask.as_ref().ok_or("creative_parameters")?;
        let (_, _, file, _pins) = Projects::creative_source(&project, &layer.asset_id)?;
        let (source, _) = gallery::render_rgba(file, &layer.operations)?;
        let pixels = canvas::paint(mask, source.width(), source.height())?;
        let bytes = gallery::encode_rgba(image::DynamicImage::ImageLuma8(pixels).to_rgba8(), "png", 100)?;
        catalog.export_project_edit(&root, "Mask", &bytes, serde_json::json!({"restricted":project_restricted(&project),"project":id,"layerId":layer_id,"assetId":layer.asset_id,"operations":layer.operations,"mask":mask}), "png")
    }).await.map_err(err)?
}).await;crate::privacy::finish(privacy_epoch,privacy_result)}
