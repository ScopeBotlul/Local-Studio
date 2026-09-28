use super::*;
use std::{collections::HashMap, sync::OnceLock};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageModel {
    id: String,
    name: String,
    path: String,
    total_bytes: u64,
    restricted: bool,
}

// This bounded cache is only a picker hint. Admission always validates the file again.
static COMPATIBILITY: OnceLock<Mutex<HashMap<String, bool>>> = OnceLock::new();
fn compatible(path: &Path) -> bool {
    let Ok(file) = read_locked(path) else {
        return false;
    };
    let Ok(binding) = crate::gallery::file_binding(&file) else {
        return false;
    };
    let key = format!("{}:{binding}", path.display());
    let cache = COMPATIBILITY.get_or_init(Default::default);
    if let Some(result) = cache.lock().ok().and_then(|c| c.get(&key).copied()) {
        return result;
    }
    let result = qwen_image21_file(path) || model_parts(path).is_ok_and(|(_, missing)| missing.is_empty());
    if let Ok(mut cache) = cache.lock() {
        if cache.len() >= 2048 {
            cache.clear();
        }
        cache.insert(key, result);
    }
    result
}
fn qwen_image21_file(path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()).is_some_and(|ext| ext.eq_ignore_ascii_case("gguf"))
        && path.file_name().and_then(|name| name.to_str()).is_some_and(|name| name.to_ascii_lowercase().starts_with("qwen-image-2.1"))
}

#[tauri::command]
pub async fn image_model_catalog(
    state: tauri::State<'_, Arc<model_library::ModelLibrary>>,
) -> Result<Vec<ImageModel>> {
    let library = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut models = Vec::new();
        for entry in library.assistant_models()? {
            if !matches!(entry.format.as_str(), "safetensors" | "gguf") || !compatible(Path::new(&entry.path)) {
                continue;
            }
            models.push(ImageModel {
                restricted: crate::privacy::model(Path::new(&entry.path)),
                id: entry.id,
                name: entry.name,
                path: entry.path,
                total_bytes: entry.total_bytes,
            });
        }
        models.sort_by_cached_key(|m| m.name.to_lowercase());
        Ok(models)
    })
    .await
    .map_err(|_| "image_storage")?
}
