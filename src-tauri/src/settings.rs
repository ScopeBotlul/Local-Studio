use crate::types::{Settings, StoragePaths};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Component, Path};

pub fn defaults(root: &Path) -> Settings {
    Settings {
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

pub fn prepare_storage(settings: &Settings) -> Result<StoragePaths, String> {
    validate(settings)?;
    let paths = paths(&settings.data_root);
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
        fs::create_dir_all(path).map_err(|e| format!("Cannot create data folder {path}: {e}"))?;
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
}
