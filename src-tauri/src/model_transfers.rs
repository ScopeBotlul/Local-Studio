use super::*;
use crate::{ai::AiEngine, gallery};
use std::{
    io::{Seek, Write},
    os::windows::fs::OpenOptionsExt,
    time::Instant,
};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveFile {
    pub source: String,
    pub destination: String,
    pub bytes: u64,
    #[serde(skip_serializing)]
    binding: String,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MovePlan {
    pub id: String,
    pub model: LocalModel,
    pub destination: String,
    pub total_bytes: u64,
    pub files: Vec<MoveFile>,
    #[serde(skip_serializing)]
    created: Instant,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveJob {
    pub id: String,
    pub name: String,
    pub source: String,
    pub destination: String,
    pub status: String,
    pub phase: String,
    pub total_bytes: u64,
    pub copied_bytes: u64,
    pub error: Option<String>,
}
#[derive(Default)]
pub(super) struct TransferState {
    pub plan: Option<MovePlan>,
    pub job: Option<MoveJob>,
}
pub(super) fn load(db: &Connection) -> Result<TransferState> {
    db.execute_batch("PRAGMA synchronous=FULL; CREATE TABLE IF NOT EXISTS model_transfer(id INTEGER PRIMARY KEY CHECK(id=1),json TEXT NOT NULL)").map_err(|_|"local_storage")?;
    let mut job: Option<MoveJob> = db
        .query_row("SELECT json FROM model_transfer WHERE id=1", [], |r| {
            r.get::<_, String>(0)
        })
        .ok()
        .map(|v| serde_json::from_str(&v).map_err(|_| "local_storage"))
        .transpose()?;
    if let Some(j) = job.as_mut() {
        if j.status == "running" {
            j.status = "interrupted".into();
            j.error = Some("move_interrupted".into());
        }
    }
    Ok(TransferState { plan: None, job })
}
struct Copies {
    root: PathBuf,
    files: Vec<File>,
    directories: Vec<PathBuf>,
    pins: Vec<File>,
    keep: bool,
}
impl Drop for Copies {
    fn drop(&mut self) {
        if !self.keep {
            for f in &self.files {
                let _ = gallery::delete_handle(f);
            }
            self.files.clear();
            self.pins.clear();
            self.directories
                .sort_by_key(|p| std::cmp::Reverse(p.components().count()));
            self.directories.dedup();
            for d in &self.directories {
                let _ = fs::remove_dir(d);
            }
            let _ = fs::remove_dir(&self.root);
        }
    }
}
impl ModelLibrary {
    fn transfer_update(&self, f: impl FnOnce(&mut MoveJob)) -> Result<()> {
        let mut s = self.transfer.lock().map_err(|_| "local_storage")?;
        let j = s.job.as_mut().ok_or("move_missing")?;
        f(j);
        let json = serde_json::to_string(j).map_err(|_| "local_storage")?;
        self.state
            .lock()
            .map_err(|_| "local_storage")?
            .db
            .execute("INSERT OR REPLACE INTO model_transfer VALUES(1,?1)", [json])
            .map_err(|_| "local_storage")?;
        Ok(())
    }
    fn move_plan(&self, id: &str, destination: &Path) -> Result<MovePlan> {
        if self.maintenance.load(Ordering::SeqCst) {
            return Err("local_busy".into());
        }
        let model = self
            .snapshot()?
            .entries
            .into_iter()
            .find(|m| m.id == id && m.discovery == Discovery::Model)
            .ok_or("local_missing")?;
        if model.files.is_empty()
            || model.files.len() > 1024
            || !["checked", "recognized"].contains(&model.status.as_str())
        {
            return Err("move_recheck".into());
        }
        no_links(destination)?;
        if !destination.is_dir() {
            return Err("local_folder".into());
        }
        let destination = fs::canonicalize(destination).map_err(|_| "local_unavailable")?;
        let base = Path::new(&model.path).parent().ok_or("local_path")?;
        let id = uuid::Uuid::new_v4().to_string();
        let folder = destination.join(format!("LocalStudio-Model-{}", &id[..8]));
        let mut files = vec![];
        let mut names = HashSet::new();
        let mut total_bytes = 0u64;
        for f in &model.files {
            let path = Path::new(&f.path);
            no_links(path)?;
            let relative = path.strip_prefix(base).map_err(|_| "move_layout")?;
            let relative = relative.to_string_lossy().replace('\\', "/");
            if !relative_file(&relative) || !names.insert(relative.to_lowercase()) {
                return Err("move_layout".into());
            }
            let _pins = gallery::directory_guards(path.parent().ok_or("local_path")?)?;
            let file = gallery::lock_file(path)?;
            let bytes = file.metadata().map_err(|_| "local_unavailable")?.len();
            if Some(bytes) != f.size {
                return Err("move_changed".into());
            }
            total_bytes = total_bytes.checked_add(bytes).ok_or("local_limit")?;
            files.push(MoveFile {
                source: f.path.clone(),
                destination: folder.join(relative).to_string_lossy().into(),
                bytes,
                binding: gallery::file_binding(&file)?,
            });
        }
        if fs2::available_space(&destination).map_err(|_| "local_unavailable")?
            < total_bytes.saturating_add(64 * 1024 * 1024)
        {
            return Err("move_space".into());
        }
        let plan = MovePlan {
            id,
            model,
            destination: folder.to_string_lossy().into(),
            total_bytes,
            files,
            created: Instant::now(),
        };
        self.transfer.lock().map_err(|_| "local_storage")?.plan = Some(plan.clone());
        Ok(plan)
    }
    fn transfer(&self, plan: MovePlan, ai: &AiEngine) -> Result<()> {
        let mut sources = Vec::new();
        let mut source_pins = Vec::new();
        for info in &plan.files {
            let p = Path::new(&info.source);
            source_pins.extend(gallery::directory_guards(p.parent().ok_or("local_path")?)?);
            let f = gallery::destructive_file(p).map_err(|_| "move_in_use")?;
            if f.metadata().map_err(|_| "local_unavailable")?.len() != info.bytes
                || gallery::file_binding(&f)? != info.binding
            {
                return Err("move_changed".into());
            }
            sources.push(f);
        }
        let root = PathBuf::from(&plan.destination);
        let _destination_parent = gallery::directory_guards(root.parent().ok_or("local_path")?)?;
        if fs2::available_space(root.parent().ok_or("local_path")?)
            .map_err(|_| "local_unavailable")?
            < plan.total_bytes.saturating_add(64 * 1024 * 1024)
        {
            return Err("move_space".into());
        }
        fs::create_dir(&root).map_err(|_| "move_collision")?;
        let mut copies = Copies {
            root: root.clone(),
            files: vec![],
            directories: vec![],
            pins: gallery::directory_guards(&root)?,
            keep: false,
        };
        let mut mappings = vec![];
        let mut copied = 0u64;
        let mut progress = Instant::now();
        for (info, source) in plan.files.iter().zip(sources.iter_mut()) {
            if self.transfer_cancel.load(Ordering::SeqCst) {
                return Err("move_cancelled".into());
            }
            let target = Path::new(&info.destination);
            let parent = target.parent().ok_or("local_path")?;
            fs::create_dir_all(parent).map_err(|_| "local_storage")?;
            copies.pins.extend(gallery::directory_guards(parent)?);
            copies.directories.extend(
                parent
                    .ancestors()
                    .take_while(|p| *p != root)
                    .filter(|p| p.starts_with(&root))
                    .map(Path::to_path_buf),
            );
            let file = fs::OpenOptions::new()
                .read(true)
                .write(true)
                .access_mode(0xc0010000)
                .share_mode(0)
                .create_new(true)
                .open(target)
                .map_err(|_| "move_collision")?;
            copies.files.push(file);
            let file = copies.files.last_mut().unwrap();
            let mut hash = Sha256::new();
            let mut buffer = vec![0u8; 1024 * 1024];
            loop {
                if self.transfer_cancel.load(Ordering::SeqCst) {
                    return Err("move_cancelled".into());
                }
                let n = source.read(&mut buffer).map_err(|_| "local_unavailable")?;
                if n == 0 {
                    break;
                }
                file.write_all(&buffer[..n]).map_err(|_| "move_write")?;
                hash.update(&buffer[..n]);
                copied += n as u64;
                if progress.elapsed().as_millis() > 200 {
                    self.transfer_update(|j| {
                        j.phase = "copying".into();
                        j.copied_bytes = copied;
                    })?;
                    progress = Instant::now();
                }
            }
            file.sync_all().map_err(|_| "move_write")?;
            file.rewind().map_err(|_| "local_storage")?;
            self.transfer_update(|j| {
                j.phase = "verifying".into();
                j.copied_bytes = copied;
            })?;
            let expected = format!("{:x}", hash.finalize());
            let mut actual = Sha256::new();
            loop {
                if self.transfer_cancel.load(Ordering::SeqCst) {
                    return Err("move_cancelled".into());
                }
                let n = file.read(&mut buffer).map_err(|_| "move_verify")?;
                if n == 0 {
                    break;
                }
                actual.update(&buffer[..n]);
            }
            if expected != format!("{:x}", actual.finalize())
                || file.metadata().map_err(|_| "move_verify")?.len() != info.bytes
            {
                return Err("move_verify".into());
            }
            mappings.push((
                info.source.clone(),
                info.destination.clone(),
                info.bytes,
                expected,
                gallery::file_binding(file)?,
            ));
        }
        if self.transfer_cancel.load(Ordering::SeqCst) {
            return Err("move_cancelled".into());
        }
        self.transfer_update(|j| j.phase = "committing".into())?;
        // Once any durable reference can point here, never remove the verified destination on error.
        copies.keep = true;
        ai.relocate(&mappings)?;
        let mut next = plan.model.clone();
        next.path = mappings
            .iter()
            .find(|m| m.0 == plan.model.path)
            .ok_or("move_layout")?
            .1
            .clone();
        next.id = format!("{:x}", Sha256::digest(next.path.to_lowercase().as_bytes()));
        next.source_root = plan.destination.clone();
        next.checked_at = crate::database::now();
        next.files = plan
            .files
            .iter()
            .zip(&copies.files)
            .map(|(m, file)| LocalFile {
                path: m.destination.clone(),
                size: Some(m.bytes),
                modified: file
                    .metadata()
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
                    .map(|d| d.as_millis() as u64),
            })
            .collect();
        {
            let mut s = self.state.lock().map_err(|_| "local_storage")?;
            let tx = s.db.transaction().map_err(|_| "local_storage")?;
            tx.execute(
                "INSERT OR REPLACE INTO local_models VALUES(?1,?2)",
                params![
                    next.id,
                    serde_json::to_string(&next).map_err(|_| "local_storage")?
                ],
            )
            .map_err(|_| "local_storage")?;
            tx.execute("DELETE FROM local_models WHERE id=?1", [&plan.model.id])
                .map_err(|_| "local_storage")?;
            tx.commit().map_err(|_| "local_storage")?;
        }
        self.transfer_update(|j| j.phase = "removing_originals".into())?;
        let mut retained = false;
        for source in &sources {
            if gallery::delete_handle(source).is_err() {
                retained = true;
            }
        }
        self.transfer_update(|j| {
            j.status = "completed".into();
            j.phase = "completed".into();
            j.copied_bytes = j.total_bytes;
            if retained {
                j.error = Some("move_original_retained".into());
            }
        })?;
        Ok(())
    }
    fn begin_move(
        self: &Arc<Self>,
        id: &str,
        confirmed: bool,
        ai: Arc<AiEngine>,
    ) -> Result<MoveJob> {
        if !confirmed {
            return Err("move_confirmation".into());
        }
        {
            let s = self.state.lock().map_err(|_| "local_storage")?;
            if s.scan.status == "running" || self.maintenance.swap(true, Ordering::SeqCst) {
                return Err("local_busy".into());
            }
        }
        let prepare = (|| {
            let mut s = self.transfer.lock().map_err(|_| "local_storage")?;
            let p = s
                .plan
                .take()
                .filter(|p| p.id == id && p.created.elapsed().as_secs() < 300)
                .ok_or("move_expired")?;
            let j = MoveJob {
                id: p.id.clone(),
                name: p.model.name.clone(),
                source: p.model.path.clone(),
                destination: p.destination.clone(),
                status: "running".into(),
                phase: "checking".into(),
                total_bytes: p.total_bytes,
                copied_bytes: 0,
                error: None,
            };
            s.job = Some(j.clone());
            Ok::<_, String>((p, j))
        })();
        let (plan, job) = match prepare {
            Ok(v) => v,
            Err(e) => {
                self.maintenance.store(false, Ordering::SeqCst);
                return Err(e);
            }
        };
        if let Err(e) = self.transfer_update(|_| {}) {
            self.maintenance.store(false, Ordering::SeqCst);
            return Err(e);
        }
        self.transfer_cancel.store(false, Ordering::SeqCst);
        let library = self.clone();
        std::thread::spawn(move || {
            if let Err(error) = library.transfer(plan, &ai) {
                let _ = library.transfer_update(|j| {
                    j.status = if error == "move_cancelled" {
                        "cancelled"
                    } else {
                        "failed"
                    }
                    .into();
                    j.error = Some(error);
                });
            }
            library.maintenance.store(false, Ordering::SeqCst);
        });
        Ok(job)
    }
}
#[tauri::command]
pub async fn model_move_plan(
    id: String,
    destination: String,
    state: tauri::State<'_, Arc<ModelLibrary>>,
) -> Result<MovePlan> {
    let l = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || l.move_plan(&id, Path::new(&destination)))
        .await
        .map_err(|_| "local_storage")?
}
#[tauri::command]
pub fn model_move_start(
    plan_id: String,
    confirmed: bool,
    state: tauri::State<'_, Arc<ModelLibrary>>,
    ai: tauri::State<'_, Arc<AiEngine>>,
) -> Result<MoveJob> {
    state
        .inner()
        .begin_move(&plan_id, confirmed, ai.inner().clone())
}
#[tauri::command]
pub fn model_move_status(state: tauri::State<'_, Arc<ModelLibrary>>) -> Result<Option<MoveJob>> {
    Ok(state
        .transfer
        .lock()
        .map_err(|_| "local_storage")?
        .job
        .clone())
}
#[tauri::command]
pub fn model_move_cancel(state: tauri::State<'_, Arc<ModelLibrary>>) {
    state.transfer_cancel.store(true, Ordering::SeqCst);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn verified_move_switches_reference_only_after_copy_and_preserves_collisions() {
        let tmp = tempfile::tempdir().unwrap();
        let config = tmp.path().join("config");
        let source = tmp.path().join("source");
        let target = tmp.path().join("target");
        for p in [&config, &source, &target] {
            fs::create_dir(p).unwrap();
        }
        let path = source.join("test.safetensors");
        let header = br#"{"weight":{"dtype":"F32","shape":[1],"data_offsets":[0,4]}}"#;
        let mut bytes = (header.len() as u64).to_le_bytes().to_vec();
        bytes.extend_from_slice(header);
        bytes.extend_from_slice(&[1, 2, 3, 4]);
        fs::write(&path, &bytes).unwrap();
        let library = ModelLibrary::new(&config).unwrap();
        library.scan_folder(source.clone());
        let model = library.snapshot().unwrap().entries[0].clone();
        let ai = AiEngine::new(&config, tmp.path().into()).unwrap();
        let p = library.move_plan(&model.id, &target).unwrap();
        assert!(library.begin_move(&p.id, false, ai.clone()).is_err());
        let job = library.begin_move(&p.id, true, ai.clone()).unwrap();
        let end = Instant::now();
        while library.maintenance.load(Ordering::SeqCst) {
            assert!(end.elapsed().as_secs() < 10);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert_eq!(
            library
                .transfer
                .lock()
                .unwrap()
                .job
                .as_ref()
                .unwrap()
                .status,
            "completed"
        );
        let moved = library.snapshot().unwrap().entries[0].clone();
        assert_eq!(fs::read(&moved.path).unwrap(), bytes);
        assert!(!path.exists());
        assert_eq!(moved.source_root, job.destination);
        let p = library.move_plan(&moved.id, &source).unwrap();
        fs::create_dir(&p.destination).unwrap();
        fs::write(Path::new(&p.destination).join("keep.txt"), b"keep").unwrap();
        assert!(library.transfer(p.clone(), &ai).is_err());
        assert_eq!(fs::read(&moved.path).unwrap(), bytes);
        assert_eq!(
            fs::read(Path::new(&p.destination).join("keep.txt")).unwrap(),
            b"keep"
        );
    }
    #[test]
    fn cancelled_copy_keeps_original_and_registered_path() {
        let tmp = tempfile::tempdir().unwrap();
        let c = tmp.path().join("c");
        let s = tmp.path().join("s");
        let t = tmp.path().join("t");
        for p in [&c, &s, &t] {
            fs::create_dir(p).unwrap();
        }
        let path = s.join("weights.safetensors");
        let header = br#"{"weight":{"dtype":"U8","shape":[1000000],"data_offsets":[0,1000000]}}"#;
        let mut data = (header.len() as u64).to_le_bytes().to_vec();
        data.extend_from_slice(header);
        data.extend(vec![1u8; 1_000_000]);
        fs::write(&path, &data).unwrap();
        let l = ModelLibrary::new(&c).unwrap();
        l.scan_folder(s);
        let m = l.snapshot().unwrap().entries[0].clone();
        let p = l.move_plan(&m.id, &t).unwrap();
        l.transfer_cancel.store(true, Ordering::SeqCst);
        let ai = AiEngine::new(&c, tmp.path().into()).unwrap();
        assert_eq!(l.transfer(p.clone(), &ai).unwrap_err(), "move_cancelled");
        assert_eq!(fs::read(path).unwrap(), data);
        assert_eq!(l.snapshot().unwrap().entries[0].id, m.id);
        assert!(!Path::new(&p.destination).exists());
    }
}
