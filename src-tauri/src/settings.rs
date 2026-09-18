use crate::types::{Settings, StoragePaths};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path};

pub fn defaults(root: &Path) -> Settings {
    Settings {
        live_hardware: false, system_accent: false, minimize_to_tray: false, parallel_generation: false,
        auto_update_check: true,
        auto_model_updates: true,
        language: if sys_locale::get_locale()
            .unwrap_or_default()
            .to_lowercase()
            .starts_with("de")
        {
            "de"
        } else {
            "en"
        }
        .into(),
        theme: "system".into(),
        accent_color: "#a78bfa".into(),
        ui_scale: 1.0,
        data_root: root.to_string_lossy().into_owned(),
        restore_session: false,
        setup_complete: false,
        max_undo: 100,
        temp_retention_days: 7,
        auto_cleanup: true,
        storage_overrides: Default::default(),
        shortcuts: crate::shortcuts::defaults(),
    }
}

pub fn validate(settings: &Settings) -> Result<(), String> {
    if !["de", "en"].contains(&settings.language.as_str()) {
        return Err("Unsupported language.".into());
    }
    if !["system", "dark", "light"].contains(&settings.theme.as_str()) {
        return Err("Unsupported theme.".into());
    }
    let color = settings.accent_color.as_bytes();
    if color.len() != 7 || color[0] != b'#' || !color[1..].iter().all(u8::is_ascii_hexdigit) {
        return Err("Accent color must be #RRGGBB.".into());
    }
    if !settings.ui_scale.is_finite() || !(0.75..=1.5).contains(&settings.ui_scale) {
        return Err("UI scale must be between 75% and 150%.".into());
    }
    if !(1..=1000).contains(&settings.max_undo) {
        return Err("Undo limit must be between 1 and 1000.".into());
    }
    if !(1..=365).contains(&settings.temp_retention_days) {
        return Err("Retention must be between 1 and 365 days.".into());
    }
    let root = Path::new(&settings.data_root);
    if settings.data_root.trim().is_empty()
        || !root.is_absolute()
        || root.parent().is_none()
        || root.components().any(|c| c == Component::ParentDir)
    {
        return Err(
            "Choose an absolute data folder, not a drive root or a path containing '..'.".into(),
        );
    }
    crate::shortcuts::validate(&settings.shortcuts)?;
    validate_overrides(settings)?;
    Ok(())
}

pub fn paths(root: &str) -> StoragePaths {
    let join = |name: &str| Path::new(root).join(name).to_string_lossy().into_owned();
    StoragePaths {
        models: join("models"),
        assistant_models: join("models/assistant"),
        vision_models: join("models/vision"),
        downloads: join("downloads"),
        gallery: join("gallery"),
        projects: join("projects"),
        temporary: join("temporary"),
        recovery: join("recovery"),
        cache: join("cache"),
        proxies: join("proxies"),
    }
}

pub const PATH_KEYS: &[&str] = &["models", "assistantModels", "visionModels", "downloads", "gallery", "projects", "temporary", "recovery", "cache", "proxies"];

pub fn effective_paths(settings: &Settings) -> StoragePaths {
    let mut paths = paths(&settings.data_root);
    for (key, destination) in &settings.storage_overrides {
        let slot = match key.as_str() {
            "models" => &mut paths.models, "assistantModels" => &mut paths.assistant_models,
            "visionModels" => &mut paths.vision_models, "downloads" => &mut paths.downloads,
            "gallery" => &mut paths.gallery, "projects" => &mut paths.projects,
            "temporary" => &mut paths.temporary, "recovery" => &mut paths.recovery,
            "cache" => &mut paths.cache, "proxies" => &mut paths.proxies,
            _ => continue, // validate() rejects unknown names before persisting.
        };
        *slot = destination.clone();
    }
    paths
}

fn validate_overrides(settings: &Settings) -> Result<(), String> {
    for (key, value) in &settings.storage_overrides {
        if !PATH_KEYS.contains(&key.as_str()) { return Err(format!("Unknown storage folder: {key}")); }
        let path = Path::new(value);
        if value.len() > 32700 || value.trim() != value || !path.is_absolute() || path.parent().is_none()
            || path.components().any(|c| c == Component::ParentDir)
            || value.chars().any(|c| c.is_control()) {
            return Err(format!("Choose an absolute storage folder without '..': {key}"));
        }
        for component in path.components() {
            if let Component::Normal(name) = component {
                let name = name.to_string_lossy();
                if name.ends_with(['.', ' ']) || name.contains([':', '*', '?', '"', '<', '>', '|']) {
                    return Err(format!("Invalid storage folder: {key}"));
                }
            }
        }
    }
    // Storage categories must not alias each other. Nested assistant/vision folders under
    // models are intentional; media and disposable cache/temp folders stay separate.
    let p = effective_paths(settings);
    let entries = [&p.models, &p.assistant_models, &p.vision_models, &p.downloads, &p.gallery, &p.projects, &p.temporary, &p.recovery, &p.cache, &p.proxies];
    let normalized: Vec<_> = entries.iter().map(|p| {
        let p = std::fs::canonicalize(p).unwrap_or_else(|_| Path::new(p).to_path_buf());
        p.to_string_lossy().trim_start_matches(r"\\?\").replace('/', "\\").trim_end_matches('\\').to_lowercase()
    }).collect();
    for a in 0..entries.len() { for b in (a + 1)..entries.len() {
        let same = normalized[a] == normalized[b];
        let nested = normalized[a].starts_with(&(normalized[b].clone() + "\\")) || normalized[b].starts_with(&(normalized[a].clone() + "\\"));
        if same || (nested && !(a == 0 && b <= 2)) {
            return Err(format!("Storage folders must be separate: {} / {}", PATH_KEYS[a], PATH_KEYS[b]));
        }
    } }
    Ok(())
}

pub fn prepare_storage(settings: &Settings) -> Result<StoragePaths, String> {
    validate(settings)?;
    let paths = effective_paths(settings);
    for path in [
        &paths.models,
        &paths.assistant_models,
        &paths.vision_models,
        &paths.downloads,
        &paths.gallery,
        &paths.projects,
        &paths.temporary,
        &paths.recovery,
        &paths.cache,
        &paths.proxies,
    ] {
        // Check existing ancestors too, including junctions, before creating children.
        for ancestor in Path::new(path).ancestors().filter(|p| p.exists()) {
            crate::model_library::no_links(ancestor).map_err(|_| format!("Linked storage folders are not supported: {path}"))?;
        }
        fs::create_dir_all(path).map_err(|e| format!("Cannot create data folder {path}: {e}"))?;
        crate::model_library::no_links(Path::new(path)).map_err(|_| format!("Linked storage folders are not supported: {path}"))?;
        let probe =
            Path::new(path).join(format!(".local-studio-write-test-{}", uuid::Uuid::new_v4()));
        let result = (|| -> std::io::Result<()> {
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&probe)?;
            file.write_all(b"Local Studio storage check")?;
            file.sync_all()
        })();
        let _ = fs::remove_file(&probe);
        result.map_err(|e| format!("Data folder is not writable ({path}): {e}"))?;
    }
    // Canonical paths are now available even for newly created folders (including
    // aliases using '.' or Windows short names); recheck before committing settings.
    validate_overrides(settings)?;
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_settings_and_writable_tree() {
        let dir = tempfile::tempdir().unwrap();
        let mut settings = defaults(&dir.path().join("data"));
        assert!(prepare_storage(&settings).is_ok());
        assert!(Path::new(&paths(&settings.data_root).assistant_models).is_dir());
        settings.ui_scale = f64::NAN;
        assert!(validate(&settings).is_err());
        settings.ui_scale = 1.0;
        settings.accent_color = "red;evil".into();
        assert!(validate(&settings).is_err());
        settings.accent_color = "#12abCD".into();
        settings.data_root = "relative/path".into();
        assert!(validate(&settings).is_err());
    }
    #[test]
    fn custom_storage_is_persistent_separate_and_does_not_move_files() {
        let dir = tempfile::tempdir().unwrap(); let mut s = defaults(&dir.path().join("data"));
        let old = prepare_storage(&s).unwrap();
        fs::write(Path::new(&old.gallery).join("keep.png"), b"original").unwrap();
        let destination = dir.path().join("new-gallery");
        s.storage_overrides.insert("gallery".into(), destination.to_string_lossy().into());
        let new = prepare_storage(&s).unwrap(); assert_eq!(Path::new(&new.gallery), destination);
        assert_eq!(fs::read(Path::new(&old.gallery).join("keep.png")).unwrap(), b"original");
        assert!(!destination.join("keep.png").exists());
        let restored: Settings = serde_json::from_str(&serde_json::to_string(&s).unwrap()).unwrap(); assert_eq!(restored, s);
        s.storage_overrides.insert("cache".into(), destination.join("cache").to_string_lossy().into()); assert!(validate(&s).is_err());
        s.storage_overrides.remove("cache"); s.storage_overrides.insert("gallery".into(), "relative".into()); assert!(validate(&s).is_err());
        let mut aliases = defaults(&dir.path().join("alias-data"));
        aliases.storage_overrides.insert("gallery".into(), dir.path().join("new-folder/media").to_string_lossy().into());
        aliases.storage_overrides.insert("cache".into(), dir.path().join("new-folder/./media").to_string_lossy().into());
        assert!(prepare_storage(&aliases).is_err());
        let mut legacy = serde_json::to_value(defaults(&dir.path().join("data"))).unwrap();
        legacy.as_object_mut().unwrap().remove("storageOverrides"); legacy.as_object_mut().unwrap().remove("shortcuts");
        legacy.as_object_mut().unwrap().remove("autoCleanup");
        let migrated: Settings = serde_json::from_value(legacy).unwrap(); assert!(validate(&migrated).is_ok()); assert!(migrated.storage_overrides.is_empty());
        assert!(migrated.auto_cleanup);
        let mut previous = serde_json::to_value(defaults(&dir.path().join("data"))).unwrap();
        let keys = previous["shortcuts"].as_object_mut().unwrap(); keys.remove("projectSave"); keys.insert("rename".into(), serde_json::json!("Ctrl+Shift+R"));
        let migrated: Settings = serde_json::from_value(previous).unwrap(); assert!(validate(&migrated).is_ok());
        assert_eq!(migrated.shortcuts["projectSave"], "Ctrl+S"); assert_eq!(migrated.shortcuts["rename"], "Ctrl+Shift+R");
    }
}

pub fn auto_cleanup_default() -> bool { true }
