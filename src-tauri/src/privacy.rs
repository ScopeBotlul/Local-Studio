//! Compatibility boundary after removal of the former local 18+ content lock.
//!
//! Existing projects and databases may still contain historic classification fields.
//! They are intentionally ignored. The no-op surface keeps old project formats readable
//! while all content remains accessible.
use std::{path::Path, sync::Arc};

type Result<T> = std::result::Result<T, String>;

pub struct Privacy;

impl Privacy {
    pub fn new(_: &Path) -> Result<Arc<Self>> {
        Ok(Arc::new(Self))
    }
}

pub fn install(_: Arc<Privacy>) -> Result<()> {
    Ok(())
}

pub fn locked() -> bool {
    false
}

pub fn epoch() -> u64 {
    0
}

pub fn finish<T>(_: u64, result: Result<T>) -> Result<T> {
    result
}

pub fn model(_: &Path) -> bool {
    false
}

pub fn media(_: &Path) -> bool {
    false
}

pub fn check(_: &Path) -> Result<()> {
    Ok(())
}

pub fn mark(_: &Path) -> Result<()> {
    Ok(())
}

pub fn register_model(_: &Path) -> Result<()> {
    Ok(())
}

pub fn protect_path(_: &Path) -> Result<()> {
    Ok(())
}

pub fn inherit(_: &Path, _: &Path) -> Result<()> {
    Ok(())
}

pub fn request(_: &crate::image_engine::ImageRequest) -> bool {
    false
}

pub fn check_request(_: &crate::image_engine::ImageRequest) -> Result<()> {
    Ok(())
}

pub fn redact(_: &mut crate::image_engine::ImageRequest) {}

pub fn has_protected() -> bool {
    false
}

/// Preserve the local-main-window IPC boundary that used to share this guard.
pub fn guard(invoke: &tauri::ipc::Invoke) -> Result<()> {
    if invoke.message.webview_ref().label() == "main" {
        Ok(())
    } else {
        Err("desktop_window".into())
    }
}

pub fn dispatch(f: impl Fn(tauri::ipc::Invoke) -> bool, invoke: tauri::ipc::Invoke) -> bool {
    f(invoke)
}
