use super::*;
use rusqlite::OptionalExtension;
use std::collections::BTreeMap;

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImageWorkspace {
    pub request: Option<ImageRequest>,
    pub models: BTreeMap<String, ImageRequest>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSnapshot { pub workspace: ImageWorkspace, pub recovery_available: bool, pub unsaved: usize }
#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageExitAction { Save, Discard, Keep }

pub(super) fn load(db: &Connection, jobs: &[ImageJob]) -> Result<(ImageWorkspace, bool)> {
    db.execute_batch("CREATE TABLE IF NOT EXISTS image_session(key TEXT PRIMARY KEY, json TEXT NOT NULL);").map_err(|_| "image_storage")?;
    let stored: Option<String> = db.query_row("SELECT json FROM image_session WHERE key='workspace'", [], |r| r.get(0)).optional().map_err(|_| "image_storage")?;
    let workspace = match stored { Some(json) => serde_json::from_str(&json).map_err(|_| "image_storage")?, None => ImageWorkspace::default() };
    let clean: Option<String> = db.query_row("SELECT json FROM image_session WHERE key='clean'", [], |r| r.get(0)).optional().map_err(|_| "image_storage")?;
    let recovery = clean.as_deref() != Some("true") && (workspace.request.is_some() || jobs.iter().any(|j| ["running", "queued"].contains(&j.status.as_str()) || unsaved(j)));
    db.execute("INSERT OR REPLACE INTO image_session VALUES('clean','false')", []).map_err(|_| "image_storage")?;
    Ok((workspace, recovery))
}
fn unsaved(job: &ImageJob) -> bool { job.status == "completed" && job.saved_path.is_none() && (!job.discarded || job.output.is_some()) }
fn persist_workspace(state: &State, workspace: &ImageWorkspace) -> Result<()> {
    state.db.execute("INSERT OR REPLACE INTO image_session VALUES('workspace',?1)", [serde_json::to_string(workspace).map_err(|_| "image_storage")?]).map_err(|_| "image_storage")?;
    Ok(())
}
impl ImageEngine {
    pub fn workspace(&self) -> Result<WorkspaceSnapshot> {
        let state = self.state.lock().map_err(|_| "image_storage")?;
        Ok(WorkspaceSnapshot { workspace: state.workspace.clone(), recovery_available: state.recovery_available, unsaved: state.jobs.iter().filter(|j| unsaved(j)).count() })
    }
    pub fn save_workspace(&self, workspace: ImageWorkspace) -> Result<()> {
        if workspace.models.len() > 256 { return Err("image_parameters".into()); }
        for request in workspace.request.iter().chain(workspace.models.values()) {
            if request.model_path.len() > 32768 { return Err("image_path".into()); }
            if request.prompt.len() > 4000 || request.prompt.contains('\0') { return Err("image_prompt".into()); }
            // An unfinished form may have an empty prompt; executable requests still use validate().
            let mut checked = request.clone(); if checked.prompt.trim().is_empty() { checked.prompt = "draft".into(); }
            validate(&checked)?;
        }
        if workspace.models.keys().any(|p| p.len() > 32768) { return Err("image_path".into()); }
        let mut state = self.state.lock().map_err(|_| "image_storage")?;
        if self.stopped.load(Ordering::Relaxed) { return Err("image_closing".into()); }
        if state.recovery_available { return Err("image_recovery_pending".into()); }
        persist_workspace(&state, &workspace)?; state.workspace = workspace; Ok(())
    }
    pub fn recover(&self) -> Result<WorkspaceSnapshot> {
        self.state.lock().map_err(|_| "image_storage")?.recovery_available = false;
        self.workspace()
    }
    pub fn discard(&self, id: &str) -> Result<()> {
        // Only our UUID directory and exact generated filenames are eligible; never recursive deletion.
        if uuid::Uuid::parse_str(id).map(|v| v.to_string()).ok().as_deref() != Some(id) { return Err("image_path".into()); }
        let mut state = self.state.lock().map_err(|_| "image_storage")?;
        let mut job = state.jobs.iter().find(|j| j.id == id && j.status == "completed" && j.saved_path.is_none()).cloned().ok_or("image_missing")?;
        let directory = job_directory(&job, &self.config)?;
        if directory.exists() { model_library::no_links(&directory).map_err(|_| "image_path")?; }
        if job.output.as_ref().is_some_and(|p| Path::new(p) != directory.join("image.png")) { return Err("image_path".into()); }
        let paths = ["image.png", "prompt.txt", "negative.txt", "metadata.json"].map(|name| directory.join(name));
        for path in &paths { if path.exists() { model_library::no_links(path).map_err(|_| "image_path")?; if !path.is_file() { return Err("image_path".into()); } } }
        // Persist intent before deleting: a crash cannot resurrect a partially discarded result.
        job.discarded = true; persist(&state, &job)?;
        *state.jobs.iter_mut().find(|j| j.id == id).unwrap() = job.clone();
        for path in paths { match fs::remove_file(path) { Ok(()) => {}, Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}, Err(_) => return Err("image_storage".into()) } }
        let _ = fs::remove_dir(directory); // Unknown user files keep the directory alive.
        job.output = None; persist(&state, &job)?;
        *state.jobs.iter_mut().find(|j| j.id == id).unwrap() = job; Ok(())
    }
    pub fn finish_session(&self, action: Option<ImageExitAction>, gallery: &Path, restore: bool) -> Result<()> {
        if matches!(action, Some(ImageExitAction::Keep)) && !restore { return Err("image_parameters".into()); }
        self.shutdown(); // Prevent new starts and join before enumerating results, including completion races.
        let result = (|| {
            let jobs = self.list()?;
            let drafts: Vec<_> = jobs.iter().filter(|j| unsaved(j)).collect();
            if !drafts.is_empty() && action.is_none() { return Err("image_unsaved".into()); }
            for job in drafts {
                if job.discarded { self.discard(&job.id)?; }
                else { match action { Some(ImageExitAction::Save) => { self.save(&job.id, gallery)?; }, Some(ImageExitAction::Discard) => self.discard(&job.id)?, Some(ImageExitAction::Keep) => {}, None => unreachable!() } }
            }
            let mut state = self.state.lock().map_err(|_| "image_storage")?;
            let mut workspace = state.workspace.clone(); if !restore { workspace.request = None; }
            persist_workspace(&state, &workspace)?;
            state.db.execute("INSERT OR REPLACE INTO image_session VALUES('clean','true')", []).map_err(|_| "image_storage")?;
            state.workspace = workspace; state.recovery_available = false; Ok(())
        })();
        if result.is_err() { self.stopped.store(false, Ordering::Relaxed); }
        result
    }
}
#[tauri::command]
pub fn image_workspace(state: tauri::State<'_, Arc<ImageEngine>>) -> Result<WorkspaceSnapshot> { state.workspace() }
#[tauri::command]
pub fn image_workspace_save(workspace: ImageWorkspace, state: tauri::State<'_, Arc<ImageEngine>>) -> Result<()> { state.save_workspace(workspace) }
#[tauri::command]
pub fn image_recover(state: tauri::State<'_, Arc<ImageEngine>>) -> Result<WorkspaceSnapshot> { state.recover() }
#[tauri::command]
pub async fn image_discard(id: String, state: tauri::State<'_, Arc<ImageEngine>>) -> Result<()> {
    let engine = state.inner().clone(); tauri::async_runtime::spawn_blocking(move || engine.discard(&id)).await.map_err(|_| "image_storage")?
}
#[cfg(test)]
#[path = "workspace_tests.rs"]
mod tests;
