use super::*;

// Library grouping, not execution approval. Keep components accessible for future
// pipelines; do not mistake a tensor container for a standalone generator.
pub(super) fn component_location(path: &Path) -> bool {
    let name = path.file_stem().unwrap_or_default().to_string_lossy().to_lowercase();
    let tiny_autoencoder = ["taesd", "taesdxl", "taesd3", "taef1", "taef2", "taesana"]
        .iter().any(|prefix| name == format!("{prefix}_encoder") || name == format!("{prefix}_decoder"));
    let known_component = tiny_autoencoder || matches!(name.as_str(), "ae" | "clip_l" | "clip_g" | "t5xxl")
        || name.ends_with("_vae") || name.ends_with("-vae") || name.starts_with("t5xxl_") || name.starts_with("umt5_");
    let parts: Vec<_> = path.components().filter_map(|part| match part {
        Component::Normal(value) => Some(value.to_string_lossy().to_lowercase()), _ => None,
    }).collect();
    // Restrict folder hints to established model layouts, not arbitrary ancestors.
    let in_component_folder = parts.windows(2).any(|pair| pair[0] == "models" && matches!(pair[1].as_str(),
        "vae" | "vae_approx" | "text_encoders" | "clip" | "loras" | "lora" | "embeddings" | "controlnet" | "clip_vision" | "ipadapter"));
    known_component || in_component_folder
}

// These are application/dependency/test locations, not a size-based model heuristic.
// A deliberately selected folder can still inspect its contents.
pub(super) fn excluded_location(path: &Path) -> Option<&'static str> {
    let parts: Vec<_> = path.components().filter_map(|part| match part {
        Component::Normal(value) => Some(value.to_string_lossy().to_lowercase()), _ => None,
    }).collect();
    if parts.iter().any(|p| matches!(p.as_str(), "$recycle.bin" | "system volume information")) { return Some("local_system_files"); }
    if parts.first().is_some_and(|p| matches!(p.as_str(), "windows" | "program files" | "program files (x86)")) { return Some("local_application_files"); }
    if parts.iter().any(|p| matches!(p.as_str(), "site-packages" | "dist-packages" | "node_modules" | ".git" | ".svn" | "__pycache__" | ".artifacts")) { return Some("local_dependency_files"); }
    None
}

fn plain_text_checkpoint(path: &Path) -> bool {
    let Ok(file) = safe_file(path) else { return false; };
    let mut prefix = Vec::new();
    if file.take(8192).read_to_end(&mut prefix).is_err() || prefix.is_empty() { return false; }
    let text = match std::str::from_utf8(&prefix) {
        Ok(text) => text,
        Err(error) if error.error_len().is_none() => std::str::from_utf8(&prefix[..error.valid_up_to()]).unwrap_or(""),
        Err(_) => return false,
    };
    !text.is_empty() && text.chars().all(|c| !c.is_control() || c.is_whitespace())
}

// Safetensors is also used for cached latents/embeddings, not just model weights.
// Unknown naming schemes remain reviewable candidates instead of disappearing.
fn has_weight_names(path: &Path) -> Option<bool> {
    let mut file = safe_file(path).ok()?;
    let mut length = [0; 8]; file.read_exact(&mut length).ok()?;
    let length = u64::from_le_bytes(length); if !(2..=MAX_HEADER).contains(&length) { return None; }
    let mut header = vec![0; length as usize]; file.read_exact(&mut header).ok()?;
    let header: Value = serde_json::from_slice(&header).ok()?;
    Some(header.as_object()?.keys().any(|name| {
        name != "__metadata__" && (matches!(name.as_str(), "weight" | "bias") || name.ends_with(".weight") || name.ends_with(".bias") || name.ends_with("_weight") || name.ends_with("_bias") || name.contains("lora_"))
    }))
}

pub(super) fn classify_discovery(entry: &mut LocalModel, mode: ScanMode) {
    if component_location(Path::new(&entry.path)) { entry.kind = "component".into(); }
    entry.scan_mode = Some(mode);
    if mode != ScanMode::Folder {
        if let Some(reason) = excluded_location(Path::new(&entry.path)) {
            entry.discovery = Discovery::Excluded; entry.discovery_reason = reason.into(); return;
        }
    }
    if entry.format == "pytorch" && plain_text_checkpoint(Path::new(&entry.path)) {
        entry.discovery = Discovery::Excluded; entry.discovery_reason = "local_plain_text".into(); return;
    }
    // Keep a previously identified model visible when it is damaged or unavailable.
    let tensor_data = entry.format == "safetensors" && entry.discovery != Discovery::Model
        && matches!(entry.status.as_str(), "checked" | "recognized") && has_weight_names(Path::new(&entry.path)) == Some(false);
    let identified = entry.discovery == Discovery::Model || (!tensor_data && matches!(entry.status.as_str(), "checked" | "recognized"));
    entry.discovery = if identified { Discovery::Model } else { Discovery::Candidate };
    entry.discovery_reason = if tensor_data { "local_tensor_data" } else if identified { "local_identified_format" } else { "local_unconfirmed_format" }.into();
}

pub(super) fn migrate_discovery(db: &mut Connection) -> Result<()> {
    let version: u32 = db.query_row("PRAGMA user_version", [], |row| row.get(0)).map_err(|_| "local_storage")?;
    if version >= 1 { return Ok(()); }
    let entries = {
        let mut query = db.prepare("SELECT json FROM local_models").map_err(|_| "local_storage")?;
        let rows = query.query_map([], |row| row.get::<_, String>(0)).map_err(|_| "local_storage")?;
        rows.map(|row| serde_json::from_str::<LocalModel>(&row.map_err(|_| "local_storage")?).map_err(|_| "local_storage".to_string())).collect::<Result<Vec<_>>>()?
    };
    let tx = db.transaction().map_err(|_| "local_storage")?;
    for mut entry in entries {
        let mode = entry.scan_mode.unwrap_or_else(|| if Path::new(&entry.source_root).parent().is_none() { ScanMode::Full } else { ScanMode::Folder });
        classify_discovery(&mut entry, mode);
        tx.execute("UPDATE local_models SET json=?1 WHERE id=?2", params![serde_json::to_string(&entry).map_err(|_| "local_storage")?, entry.id]).map_err(|_| "local_storage")?;
    }
    tx.pragma_update(None, "user_version", 1).map_err(|_| "local_storage")?;
    tx.commit().map_err(|_| "local_storage")?; Ok(())
}
