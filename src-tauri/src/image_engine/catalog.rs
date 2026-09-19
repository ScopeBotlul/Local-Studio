use super::*;
use std::{collections::HashMap, sync::OnceLock};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageModel { id: String, name: String, path: String, total_bytes: u64, restricted: bool }

// This bounded cache is only a picker hint. Admission always validates the file again.
static COMPATIBILITY: OnceLock<Mutex<HashMap<String, bool>>> = OnceLock::new();
fn compatible(path: &Path) -> bool {
    let Ok(file) = read_locked(path) else { return false; };
    let Ok(binding) = crate::gallery::file_binding(&file) else { return false; };
    let key = format!("{}:{binding}", path.display());
    let cache = COMPATIBILITY.get_or_init(Default::default);
    if let Some(result) = cache.lock().ok().and_then(|c| c.get(&key).copied()) { return result; }
    let result = model_parts(path).is_ok_and(|(_, missing)| missing.is_empty());
    if let Ok(mut cache) = cache.lock() { if cache.len() >= 2048 { cache.clear(); } cache.insert(key, result); }
    result
}

#[tauri::command]
pub async fn image_model_catalog(state: tauri::State<'_, Arc<model_library::ModelLibrary>>) -> Result<Vec<ImageModel>> {
    let library = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut models = Vec::new();
        for entry in library.assistant_models()? {
            if entry.format != "safetensors" || !compatible(Path::new(&entry.path)) { continue; }
            models.push(ImageModel { restricted: crate::privacy::model(Path::new(&entry.path)), id: entry.id, name: entry.name, path: entry.path, total_bytes: entry.total_bytes });
        }
        models.sort_by_cached_key(|m| m.name.to_lowercase());
        Ok(models)
    }).await.map_err(|_| "image_storage")?
}
