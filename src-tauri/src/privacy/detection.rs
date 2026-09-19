//! Conservative, passive local classification. Missing tags never mean "safe".
use serde_json::Value;
use std::{collections::HashMap, io::Read, path::Path, sync::{Mutex, OnceLock}};

static CACHE: OnceLock<Mutex<HashMap<String, bool>>> = OnceLock::new();
fn adult_tag(value: &str) -> bool {
    matches!(value.trim().to_ascii_lowercase().as_str(), "nsfw" | "18+" | "adult" | "not-for-all-audiences" | "explicit")
}
fn positive(value: &Value) -> bool {
    value.as_bool() == Some(true) || value.as_str().is_some_and(|s| matches!(s.trim().to_ascii_lowercase().as_str(), "true" | "1" | "yes"))
}
fn tags(value: &Value) -> bool {
    if let Some(items) = value.as_array() { return items.iter().any(|v| v.as_str().is_some_and(adult_tag)); }
    value.as_str().is_some_and(|s| {
        s.split([',', ';', '|']).any(adult_tag)
            || serde_json::from_str::<Vec<String>>(s).is_ok_and(|tags| tags.iter().any(|s| adult_tag(s)))
    })
}
fn declared(value: &Value) -> bool {
    let Some(object) = value.as_object() else { return false; };
    object.iter().any(|(key, value)| match key.to_ascii_lowercase().as_str() {
        "nsfw" | "adult" | "is_nsfw" => positive(value),
        "tags" | "modelspec.tags" => tags(value),
        "rating" | "content_rating" | "modelspec.content_rating" => value.as_str().is_some_and(adult_tag),
        // Only known metadata containers; training captions/sample prompts are not declarations.
        "model" | "metadata" | "__metadata__" => declared(value),
        _ => false,
    })
}
pub(super) fn detect(path: &Path) -> bool {
    if !path.extension().and_then(|s| s.to_str()).is_some_and(|s| s.eq_ignore_ascii_case("safetensors")) { return false; }
    let Ok(mut file) = crate::model_library::safe_file(path) else { return false; };
    let Ok(binding) = crate::gallery::file_binding(&file) else { return false; };
    let mut key = format!("{}:{binding}", path.display());
    let mut sidecars = Vec::new();
    for candidate in [path.with_extension("civitai.info"), path.with_extension("json"), path.with_extension("cm-info.json"), path.with_extension("safetensors.json")] {
        if let Ok(file) = crate::model_library::safe_file(&candidate) {
            if file.metadata().is_ok_and(|m| m.len() <= 1024 * 1024) {
                if let Ok(binding) = crate::gallery::file_binding(&file) { key.push_str(&format!("|{}:{binding}", candidate.display())); sidecars.push(file); }
            }
        }
    }
    let cache = CACHE.get_or_init(Default::default);
    if let Some(found) = cache.lock().ok().and_then(|c| c.get(&key).copied()) { return found; }
    let header = (|| {
        let mut size = [0; 8]; file.read_exact(&mut size).ok()?;
        let size = u64::from_le_bytes(size);
        if !(2..=16 * 1024 * 1024).contains(&size) || size + 8 > file.metadata().ok()?.len() { return None; }
        let mut bytes = vec![0; size as usize]; file.read_exact(&mut bytes).ok()?;
        serde_json::from_slice::<Value>(&bytes).ok()
    })();
    let mut found = header.as_ref().and_then(|v| v.get("__metadata__")).is_some_and(declared);
    for file in sidecars {
        let mut bytes = Vec::new();
        if file.take(1024 * 1024 + 1).read_to_end(&mut bytes).is_ok() && bytes.len() <= 1024 * 1024 {
            found |= serde_json::from_slice::<Value>(&bytes).is_ok_and(|v| declared(&v));
        }
    }
    if let Ok(mut cache) = cache.lock() { if cache.len() >= 2048 { cache.clear(); } cache.insert(key, found); }
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_declarations_only_and_cache_tracks_sidecars() {
        assert!(declared(&serde_json::json!({"model":{"nsfw":true}})));
        assert!(declared(&serde_json::json!({"modelspec.tags":"[\"nsfw\"]"})));
        assert!(!declared(&serde_json::json!({"nsfw":false,"description":"nsfw","trainedWords":["nsfw"]})));
        let dir=tempfile::tempdir().unwrap();let path=dir.path().join("weights.safetensors");
        let header=br#"{"__metadata__":{"tags":"landscape"}}"#;let mut bytes=(header.len()as u64).to_le_bytes().to_vec();bytes.extend(header);std::fs::write(&path,bytes).unwrap();
        assert!(!detect(&path));
        std::fs::write(path.with_extension("civitai.info"),br#"{"model":{"nsfw":true}}"#).unwrap();assert!(detect(&path));
        std::fs::remove_file(path.with_extension("civitai.info")).unwrap();assert!(!detect(&path));
    }
}
