use super::*;
use rusqlite::params;

pub(super) fn now() -> i64 {
    chrono::Utc::now().timestamp()
}
pub(super) fn initialize(db: &Connection) -> Result<()> {
    db.execute_batch("CREATE TABLE IF NOT EXISTS project_history(id INTEGER PRIMARY KEY AUTOINCREMENT, at INTEGER NOT NULL, project_id TEXT NOT NULL, json TEXT NOT NULL);
        CREATE TABLE IF NOT EXISTS project_locations(id TEXT PRIMARY KEY, directory TEXT NOT NULL, touched INTEGER NOT NULL);
        CREATE TABLE IF NOT EXISTS project_files(path TEXT PRIMARY KEY, project_id TEXT NOT NULL, binding TEXT NOT NULL, bytes INTEGER NOT NULL);").map_err(err)
}
pub(super) fn register(db: &Connection, p: &Project, at: i64) -> Result<()> {
    db.execute("INSERT INTO project_locations VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET touched=excluded.touched", params![p.id,p.directory,at]).map_err(err)?;
    for a in p.assets.iter().chain(&p.removed) {
        let path = owned(p, a)?;
        let exists: bool = db
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM project_files WHERE path=?1)",
                [path.to_string_lossy().as_ref()],
                |r| r.get(0),
            )
            .map_err(err)?;
        if exists {
            continue;
        }
        // Preserve the original identity; replacements never become cleanup candidates.
        if let Ok(binding) = gallery::saved_binding(&path) {
            db.execute(
                "INSERT OR IGNORE INTO project_files VALUES(?1,?2,?3,?4)",
                params![path.to_string_lossy(), p.id, binding, a.bytes],
            )
            .map_err(err)?;
        }
    }
    Ok(())
}
pub(super) fn checkpoint(db: &Connection, p: &Project, at: i64) -> Result<()> {
    db.execute(
        "INSERT INTO project_history(at,project_id,json) VALUES(?1,?2,?3)",
        params![at, p.id, serde_json::to_string(p).map_err(err)?],
    )
    .map_err(err)?;
    // Keep twenty usable snapshots. Their media stay registered until safe age-based cleanup.
    db.execute("DELETE FROM project_history WHERE id NOT IN (SELECT id FROM project_history ORDER BY id DESC LIMIT 20)",[]).map_err(err)?;
    Ok(())
}
pub(super) fn persist(s: &mut State, p: Option<Project>) -> Result<()> {
    let at = now();
    let tx = s.db.transaction().map_err(err)?;
    if let Some(old) = &s.project {
        let changed =
            serde_json::to_value(old).map_err(err)? != serde_json::to_value(&p).map_err(err)?;
        if changed {
            let boundary = p.as_ref().is_none_or(|new| {
                new.id != old.id
                    || new.name != old.name
                    || new
                        .assets
                        .iter()
                        .map(|a| &a.id)
                        .ne(old.assets.iter().map(|a| &a.id))
            });
            let last: i64 = tx
                .query_row(
                    "SELECT COALESCE(MAX(at),0) FROM project_history WHERE project_id=?1",
                    [&old.id],
                    |r| r.get(0),
                )
                .map_err(err)?;
            if boundary || at - last >= 60 {
                checkpoint(&tx, old, at)?;
            }
        }
        register(&tx, old, at)?;
    }
    if let Some(p) = &p {
        register(&tx, p, at)?;
        if !p.dirty && s.project.as_ref().is_none_or(|old|old.id!=p.id){recent::remember(&tx,p)?;}
    }
    tx.execute(
        "INSERT OR REPLACE INTO project_session VALUES(1,?1)",
        [serde_json::to_string(&p).map_err(err)?],
    )
    .map_err(err)?;
    tx.commit().map_err(err)?;
    s.project = p;
    Ok(())
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Point {
    pub(super) id: i64,
    at: i64,
    name: String,
    media: usize,
    prompt: String,
    pub(super) protected: bool,
}
impl Projects {
    pub(super) fn history(&self) -> Result<Vec<Point>> {
        let s = self.state.lock().map_err(err)?;
        let mut stmt =
            s.db.prepare("SELECT id,at,json FROM project_history ORDER BY id DESC LIMIT 20")
                .map_err(err)?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, i64>(0)?,
                    r.get::<_, i64>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })
            .map_err(err)?;
        let mut result = vec![];
        for row in rows {
            let (id, at, json) = row.map_err(err)?;
            let p: Project = serde_json::from_str(&json).map_err(err)?;
            result.push(Point {
                id,
                at,
                name: p.name,
                media: p.assets.len(),
                prompt: p
                    .request
                    .map(|r| r.prompt.chars().take(100).collect())
                    .unwrap_or_default(),
                protected: result.len() < 3,
            });
        }
        Ok(result)
    }
    pub(super) fn checkpoint(&self) -> Result<()> {
        let s = self.state.lock().map_err(err)?;
        let p = Self::require(&s)?;
        checkpoint(&s.db, &p, now())
    }
    pub(super) fn restore_point(&self, id: i64, confirmed: bool) -> Result<Project> {
        let mut s = self.state.lock().map_err(err)?;
        container::no_intent(&s)?;
        if s.project.as_ref().is_some_and(|p| p.dirty) && !confirmed {
            return Err("project_unsaved".into());
        }
        let json: String =
            s.db.query_row("SELECT json FROM project_history WHERE id=?1", [id], |r| {
                r.get(0)
            })
            .map_err(|_| "project_history_missing")?;
        let mut p: Project = serde_json::from_str(&json).map_err(err)?;
        let _guards = gallery::directory_guards(Path::new(&p.directory))?;
        for a in &p.assets {
            let mut file = gallery::lock_file(&owned(&p, a)?)?;
            if digest(&mut file)? != a.sha256 {
                return Err("project_hash".into());
            }
        }
        if let Some(current) = s.project.as_ref().filter(|current| current.id == p.id) {
            p.path = current.path.clone();
            p.version = current.version.clone();
        }
        p.recovery = false;
        p.dirty = true;
        persist(&mut s, Some(p.clone()))?;
        Ok(p)
    }
    pub(super) fn rename(&self, id: &str, name: String) -> Result<Project> {
        if name.trim().is_empty() || name.len() > 180 || name.chars().any(char::is_control) {
            return Err("project_name".into());
        }
        let mut s = self.state.lock().map_err(err)?;
        let mut p = Self::require(&s)?;
        if p.id != id {
            return Err("project_changed".into());
        }
        if p.name != name {
            p.name = name;
            p.dirty = true;
            persist(&mut s, Some(p.clone()))?;
        }
        Ok(p)
    }
    pub(super) fn restore_media(&self, id: &str) -> Result<Project> {
        let mut s = self.state.lock().map_err(err)?;
        let mut p = Self::require(&s)?;
        let index = p
            .removed
            .iter()
            .position(|a| a.id == id)
            .ok_or("project_media")?;
        let a = p.removed[index].clone();
        if p.assets.len() >= MAX_MEDIA
            || p.assets
                .iter()
                .map(|a| a.bytes)
                .sum::<u64>()
                .saturating_add(a.bytes)
                > MAX_BYTES
        {
            return Err("project_limit".into());
        }
        let path = owned(&p, &a)?;
        let _guards = gallery::directory_guards(path.parent().ok_or("project_path")?)?;
        let mut file = gallery::lock_file(&path)?;
        if digest(&mut file)? != a.sha256 {
            return Err("project_hash".into());
        }
        p.removed.remove(index);
        p.assets.push(a);
        p.dirty = true;
        persist(&mut s, Some(p.clone()))?;
        Ok(p)
    }
}
#[tauri::command]
pub async fn project_history(state: tauri::State<'_, Arc<Projects>>) -> Result<Vec<Point>> {
    let p = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || p.history())
        .await
        .map_err(err)?
}
#[tauri::command]
pub async fn project_checkpoint(state: tauri::State<'_, Arc<Projects>>) -> Result<()> {
    let p = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || p.checkpoint())
        .await
        .map_err(err)?
}
#[tauri::command]
pub async fn project_restore_point(
    id: i64,
    confirmed: bool,
    state: tauri::State<'_, Arc<Projects>>,
) -> Result<Project> {
    let p = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || p.restore_point(id, confirmed))
        .await
        .map_err(err)?
}
#[tauri::command]
pub async fn project_rename(
    id: String,
    name: String,
    state: tauri::State<'_, Arc<Projects>>,
) -> Result<Project> {
    let p = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || p.rename(&id, name))
        .await
        .map_err(err)?
}
#[tauri::command]
pub async fn project_restore_media(
    id: String,
    state: tauri::State<'_, Arc<Projects>>,
) -> Result<Project> {
    let p = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || p.restore_media(&id))
        .await
        .map_err(err)?
}
