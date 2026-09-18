use crate::{core::Core, gallery, image_engine::ImageEngine, projects::{Projects,VideoEngine}};
use serde::Serialize;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
type Result<T> = std::result::Result<T, String>;
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub category: String,
    pub owner: String,
    pub path: String,
    pub bytes: u64,
    #[serde(skip)]
    pub binding: String,
}
#[derive(Default)]
pub(crate) struct Inventory {
    pub files: Vec<Candidate>,
    pub protected_bytes: u64,
    pub unavailable: usize,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preview {
    token: String,
    files: Vec<Candidate>,
    bytes: u64,
    protected_bytes: u64,
    unavailable: usize,
    retention_days: u32,
}
#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub deleted: usize,
    pub bytes: u64,
    pub skipped: usize,
    pub errors: Vec<String>,
}
struct Plan {
    at: Instant,
    days: u32,
    files: Vec<Candidate>,
}
pub struct Maintenance {
    plans: Mutex<HashMap<String, Plan>>,
    gate: Mutex<()>,
    last_auto: Mutex<Option<Instant>>,
}
impl Maintenance {
    pub fn new() -> Self {
        Self {
            plans: Mutex::new(HashMap::new()),
            gate: Mutex::new(()),
            last_auto: Mutex::new(None),
        }
    }
    fn preview(&self, days: u32, projects: &Projects, images: &ImageEngine, video: Option<&VideoEngine>) -> Result<Preview> {
        let cutoff = cutoff(days);
        let mut inventory = projects.cleanup_inventory(cutoff)?;
        let image = images.cleanup_inventory(cutoff)?;
        inventory.files.extend(image.files);
        inventory.protected_bytes += image.protected_bytes;
        inventory.unavailable += image.unavailable;
        if let Some(video)=video {let v=video.cleanup_inventory(cutoff,projects)?;inventory.files.extend(v.files);inventory.protected_bytes+=v.protected_bytes;inventory.unavailable+=v.unavailable;}
        // Bound each review/execution batch; a subsequent preview covers the remainder.
        inventory.files.truncate(1000);
        let token = uuid::Uuid::new_v4().to_string();
        let bytes = inventory.files.iter().map(|c| c.bytes).sum();
        let mut plans = self.plans.lock().map_err(|_| "cleanup_storage")?;
        plans.retain(|_, p| p.at.elapsed() < Duration::from_secs(300));
        if plans.len() >= 8 {
            plans.clear();
        }
        plans.insert(
            token.clone(),
            Plan {
                at: Instant::now(),
                days,
                files: inventory.files.clone(),
            },
        );
        Ok(Preview {
            token,
            files: inventory.files,
            bytes,
            protected_bytes: inventory.protected_bytes,
            unavailable: inventory.unavailable,
            retention_days: days,
        })
    }
    fn apply(
        &self,
        token: &str,
        confirmed: bool,
        days: u32,
        projects: &Projects,
        images: &ImageEngine,
        video: Option<&VideoEngine>,
    ) -> Result<Report> {
        if !confirmed {
            return Err("cleanup_confirmation".into());
        }
        let _gate = self.gate.lock().map_err(|_| "cleanup_storage")?;
        let plan = self
            .plans
            .lock()
            .map_err(|_| "cleanup_storage")?
            .remove(token)
            .ok_or("cleanup_expired")?;
        if plan.at.elapsed() > Duration::from_secs(300) || plan.days != days {
            return Err("cleanup_expired".into());
        }
        let mut report = Report::default();
        let cutoff = cutoff(days);
        for file in plan.files {
            let result = if file.category == "project" {
                projects.cleanup_file(&file, cutoff)
            } else if file.category=="video" {
                video.ok_or("cleanup_storage")?.cleanup_file(&file,cutoff,projects)
            } else {
                images.cleanup_file(&file, cutoff)
            };
            match result {
                Ok(true) => {
                    report.deleted += 1;
                    report.bytes += file.bytes;
                }
                Ok(false) => report.skipped += 1,
                Err(error) => {
                    report.skipped += 1;
                    report.errors.push(format!("{}: {}", file.path, error));
                }
            }
        }
        Ok(report)
    }
}
fn cutoff(days: u32) -> i64 {
    chrono::Utc::now().timestamp() - i64::from(days) * 86400
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cleanup_requires_confirmation_and_current_single_use_plan() {
        let t = tempfile::tempdir().unwrap();
        let projects = Projects::new(t.path()).unwrap();
        let images = ImageEngine::new(t.path(), t.path().join("runtime")).unwrap();
        let m = Maintenance::new();
        let p = m.preview(7, &projects, &images, None).unwrap();
        assert!(m.apply(&p.token, false, 7, &projects, &images, None).is_err());
        assert!(m.apply(&p.token, true, 8, &projects, &images, None).is_err());
        let p = m.preview(7, &projects, &images, None).unwrap();
        assert_eq!(
            m.apply(&p.token, true, 7, &projects, &images, None)
                .unwrap()
                .deleted,
            0
        );
        assert!(m.apply(&p.token, true, 7, &projects, &images, None).is_err());
        let p = m.preview(7, &projects, &images, None).unwrap();
        m.plans.lock().unwrap().get_mut(&p.token).unwrap().at =
            Instant::now() - Duration::from_secs(301);
        assert!(m.apply(&p.token, true, 7, &projects, &images, None).is_err());
    }
}
pub(crate) fn inspect(path: &Path, category: &str, owner: &str) -> Result<Candidate> {
    let _pins = gallery::directory_guards(path.parent().ok_or("cleanup_path")?)?;
    let file = gallery::lock_file(path)?;
    Ok(Candidate {
        category: category.into(),
        owner: owner.into(),
        path: path.to_string_lossy().into(),
        bytes: file.metadata().map_err(|_| "cleanup_storage")?.len(),
        binding: gallery::file_binding(&file)?,
    })
}
pub(crate) fn checked_file(candidate: &Candidate) -> Result<(std::fs::File, Vec<std::fs::File>)> {
    let path = PathBuf::from(&candidate.path);
    let guards = gallery::directory_guards(path.parent().ok_or("cleanup_path")?)?;
    let file = gallery::destructive_file(&path)?;
    if gallery::file_binding(&file)? != candidate.binding
        || file.metadata().map_err(|_| "cleanup_storage")?.len() != candidate.bytes
    {
        return Err("cleanup_changed".into());
    }
    Ok((file, guards))
}
#[tauri::command]
pub async fn storage_cleanup_preview(
    state: tauri::State<'_, Arc<Maintenance>>,
    core: tauri::State<'_, Arc<Core>>,
    projects: tauri::State<'_, Arc<Projects>>,
    images: tauri::State<'_, Arc<ImageEngine>>,
    video: tauri::State<'_,Arc<VideoEngine>>,
) -> Result<Preview> {
    let m = state.inner().clone();
    let core = core.inner().clone();
    let p = projects.inner().clone();
    let i = images.inner().clone();
    let v = video.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        m.preview(core.current_settings()?.temp_retention_days, &p, &i, Some(&v))
    })
    .await
    .map_err(|_| "cleanup_storage")?
}
#[tauri::command]
pub async fn storage_cleanup_apply(
    token: String,
    confirmed: bool,
    state: tauri::State<'_, Arc<Maintenance>>,
    core: tauri::State<'_, Arc<Core>>,
    projects: tauri::State<'_, Arc<Projects>>,
    images: tauri::State<'_, Arc<ImageEngine>>,
    video: tauri::State<'_,Arc<VideoEngine>>,
) -> Result<Report> {
    let m = state.inner().clone();
    let core = core.inner().clone();
    let p = projects.inner().clone();
    let i = images.inner().clone();
    let v = video.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        m.apply(
            &token,
            confirmed,
            core.current_settings()?.temp_retention_days,
            &p,
            &i,
            Some(&v),
        )
    })
    .await
    .map_err(|_| "cleanup_storage")?
}
#[tauri::command]
pub async fn storage_cleanup_auto(
    state: tauri::State<'_, Arc<Maintenance>>,
    core: tauri::State<'_, Arc<Core>>,
    projects: tauri::State<'_, Arc<Projects>>,
    images: tauri::State<'_, Arc<ImageEngine>>,
    video: tauri::State<'_,Arc<VideoEngine>>,
) -> Result<Report> {
    let m = state.inner().clone();
    let core = core.inner().clone();
    let p = projects.inner().clone();
    let i = images.inner().clone();
    let v = video.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let settings = core.current_settings()?;
        if !settings.auto_cleanup {
            return Ok(Report::default());
        }
        let mut last = m.last_auto.lock().map_err(|_| "cleanup_storage")?;
        if last.is_some_and(|at| at.elapsed() < Duration::from_secs(3600)) {
            return Ok(Report::default());
        }
        *last = Some(Instant::now());
        drop(last);
        let preview = m.preview(settings.temp_retention_days, &p, &i, Some(&v))?;
        let current = core.current_settings()?;
        if !current.auto_cleanup {
            return Ok(Report::default());
        }
        m.apply(&preview.token, true, current.temp_retention_days, &p, &i, Some(&v))
    })
    .await
    .map_err(|_| "cleanup_storage")?
}
