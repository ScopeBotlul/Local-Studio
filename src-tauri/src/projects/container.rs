use super::*;
use std::os::windows::fs::OpenOptionsExt;

#[derive(Serialize, Deserialize)]
struct SaveIntent {
    target: String,
    temporary: String,
    backup: String,
    previous: Option<String>,
    next: String,
    project: Project,
}
fn pending(s: &State) -> Result<bool> {
    s.db.query_row(
        "SELECT EXISTS(SELECT 1 FROM project_save_intent)",
        [],
        |r| r.get(0),
    )
    .map_err(err)
}
pub(super) fn no_intent(s: &State) -> Result<()> {
    if pending(s)? {
        Err("project_save_recovery".into())
    } else {
        Ok(())
    }
}
fn file_hash(path: &str) -> Option<String> {
    gallery::lock_file(Path::new(path))
        .ok()
        .and_then(|mut f| digest(&mut f).ok())
}
fn commit(s: &mut State, p: Project) -> Result<()> {
    let tx = s.db.transaction().map_err(err)?;
    if let Some(old) = &s.project {
        history::checkpoint(&tx, old, history::now())?;
    }
    history::register(&tx, &p, history::now())?;
    recent::remember(&tx,&p)?;
    tx.execute(
        "INSERT OR REPLACE INTO project_session VALUES(1,?1)",
        [serde_json::to_string(&Some(&p)).map_err(err)?],
    )
    .map_err(err)?;
    tx.execute("DELETE FROM project_save_intent", [])
        .map_err(err)?;
    tx.commit().map_err(err)?;
    s.project = Some(p);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interrupted_save_restores_backup_or_commits_published_file_without_overwriting_collision() {
        for phase in 0..3 {
            let t = tempfile::tempdir().unwrap();
            let p = Projects::new(t.path()).unwrap();
            let mut project = p
                .new_project(t.path(), "journal".into(), None, false)
                .unwrap();
            let target = t.path().join("project.localstudio");
            let backup = t.path().join(".local-studio-project-test.bak");
            let temporary = t.path().join(".local-studio-project-test.tmp");
            fs::write(&backup, b"previous").unwrap();
            fs::write(&temporary, b"next").unwrap();
            if phase == 1 {
                fs::write(&target, b"next").unwrap();
            } else if phase == 2 {
                fs::write(&target, b"foreign collision").unwrap();
            }
            project.dirty = false;
            project.path = Some(target.to_string_lossy().into());
            project.version = Some(file_hash(temporary.to_str().unwrap()).unwrap());
            let intent = SaveIntent {
                target: target.to_string_lossy().into(),
                temporary: temporary.to_string_lossy().into(),
                backup: backup.to_string_lossy().into(),
                previous: file_hash(backup.to_str().unwrap()),
                next: project.version.clone().unwrap(),
                project,
            };
            let mut s = p.state.lock().unwrap();
            s.db.execute(
                "INSERT INTO project_save_intent VALUES(1,?1)",
                [serde_json::to_string(&intent).unwrap()],
            )
            .unwrap();
            let result = recover_save(&mut s);
            if phase == 2 {
                assert!(result.is_err());
                assert_eq!(fs::read(&target).unwrap(), b"foreign collision");
                assert!(backup.exists());
                assert!(pending(&s).unwrap());
            } else {
                result.unwrap();
                assert_eq!(
                    fs::read(&target).unwrap(),
                    if phase == 0 {
                        b"previous".as_slice()
                    } else {
                        b"next".as_slice()
                    }
                );
                assert_eq!(s.project.as_ref().unwrap().dirty, phase == 0);
                assert!(!pending(&s).unwrap());
                assert!(!temporary.exists());
                assert!(!backup.exists());
            }
        }
    }
}
pub(super) fn recover_save(s: &mut State) -> Result<()> {
    let json: Option<String> =
        s.db.query_row("SELECT json FROM project_save_intent WHERE id=1", [], |r| {
            r.get(0)
        })
        .optional()
        .map_err(err)?;
    let Some(json) = json else {
        return Ok(());
    };
    let intent: SaveIntent = serde_json::from_str(&json).map_err(err)?;
    let target = Path::new(&intent.target);
    let parent = target.parent().ok_or("project_path")?;
    let _guards = gallery::directory_guards(parent)?;
    for path in [&intent.temporary, &intent.backup] {
        let p = Path::new(path);
        if p.parent() != Some(parent)
            || !p
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .starts_with(".local-studio-project-")
        {
            return Err("project_path".into());
        }
    }
    let current = file_hash(&intent.target);
    if current.as_deref() == Some(&intent.next) {
        commit(s, intent.project)?;
    } else if current == intent.previous && (current.is_some() || !target.exists()) {
        s.db.execute("DELETE FROM project_save_intent", [])
            .map_err(err)?;
    } else if !target.exists()
        && intent.previous.is_some()
        && file_hash(&intent.backup) == intent.previous
    {
        let file = gallery::destructive_file(Path::new(&intent.backup))?;
        gallery::rename_handle(&file, target)?;
        s.db.execute("DELETE FROM project_save_intent", [])
            .map_err(err)?;
    } else {
        return Err("project_save_recovery".into());
    }
    // Remove only verified staging/backup bytes named by our durable journal.
    if file_hash(&intent.temporary).as_deref() == Some(&intent.next) {
        let _ = gallery::delete_owned(Path::new(&intent.temporary));
    }
    if intent.previous.is_some() && file_hash(&intent.backup) == intent.previous {
        let _ = gallery::delete_owned(Path::new(&intent.backup));
    }
    Ok(())
}
impl Projects {
    pub(super) fn save(&self, path: &Path) -> Result<Project> {
        self.ensure_reference_asset()?;
        let mut s = self.state.lock().map_err(err)?;
        no_intent(&s)?;
        let mut p = Self::require(&s)?;
        if !path.is_absolute()
            || path
                .components()
                .any(|c| c == std::path::Component::ParentDir)
            || !path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(gallery::valid_name)
            || path
                .extension()
                .and_then(|e| e.to_str())
                .is_none_or(|e| !e.eq_ignore_ascii_case("localstudio"))
        {
            return Err("project_path".into());
        }
        let parent = path.parent().ok_or("project_path")?;
        let _guards = gallery::directory_guards(parent)?;
        let mut previous = None;
        let mut existing = None;
        if path.exists() {
            let same = p
                .path
                .as_ref()
                .is_some_and(|old| fs::canonicalize(old).ok() == fs::canonicalize(path).ok());
            if !same {
                return Err("project_collision".into());
            }
            let mut file = gallery::destructive_file(path)?;
            let hash = digest(&mut file)?;
            if p.version.as_deref() != Some(&hash) {
                return Err("project_changed".into());
            }
            previous = Some(hash);
            existing = Some(file);
        }
        let mut request = p.request.clone();
        if let Some(r) = &mut request {
            if !r.model_path.is_empty() {
                let model = Path::new(&r.model_path);
                if model
                    .extension()
                    .and_then(|e| e.to_str())
                    .is_none_or(|e| !e.eq_ignore_ascii_case("safetensors"))
                {
                    return Err("project_model".into());
                }
                let mut file = gallery::lock_file(model)?;
                p.model = Some(ModelReference {
                    name: model.file_name().unwrap().to_string_lossy().into(),
                    sha256: digest(&mut file)?,
                    source: "local".into(),
                });
            }
            r.model_path.clear();
            if let Some(reference)=r.reference.as_mut(){
                let asset=p.assets.iter().find(|a|a.sha256==reference.sha256&&a.kind=="image").ok_or("project_manifest")?;
                reference.path=asset.archive_name.clone();
                if let Some(mask)=reference.mask.as_mut(){mask.path=p.assets.iter().find(|a|a.sha256==mask.sha256&&a.kind=="image").ok_or("project_manifest")?.archive_name.clone();}
            }
        }
        if let Some(reference)=request.as_ref().and_then(|r|r.reference.as_ref()) {
            let asset=p.assets.iter().find(|a|a.archive_name==reference.path).ok_or("project_manifest")?;
            let stored=owned(&p,asset)?.to_string_lossy().into_owned();
            p.request.as_mut().unwrap().reference.as_mut().unwrap().path=stored;
            if let Some(mask)=reference.mask.as_ref(){let asset=p.assets.iter().find(|a|a.archive_name==mask.path).ok_or("project_manifest")?;let stored=owned(&p,asset)?.to_string_lossy().into_owned();p.request.as_mut().unwrap().reference.as_mut().unwrap().mask.as_mut().unwrap().path=stored;}
        }
        let m = Manifest {
            creative:p.creative.clone(),
            format: "local-studio".into(),
            version: if request.as_ref().is_some_and(|r|r.reference.is_some()||r.vae_on_cpu){4}else if p.creative.is_some(){3}else if p.assets.iter().any(|a| !a.edit.is_empty()) {2} else {1},
            name: p.name.clone(),
            request,
            model: p.model.clone(),
            assets: p.assets.clone(),
        };
        validate(&m)?;
        let temporary = parent.join(format!(".local-studio-project-{}.tmp", uuid()));
        let backup = parent.join(format!(".local-studio-project-{}.bak", uuid()));
        let mut output = OpenOptions::new()
            .read(true)
            .write(true)
            .access_mode(0xC0010000)
            .share_mode(1)
            .create_new(true)
            .open(&temporary)
            .map_err(err)?;
        let result = (|| {
            let mut writer = ZipWriter::new(&mut output);
            let options = SimpleFileOptions::default()
                .compression_method(CompressionMethod::Deflated)
                .compression_level(Some(3));
            writer.start_file("project.json", options).map_err(err)?;
            writer
                .write_all(&serde_json::to_vec(&m).map_err(err)?)
                .map_err(err)?;
            for a in &p.assets {
                let path = owned(&p, a)?;
                let _pins = gallery::directory_guards(path.parent().ok_or("project_path")?)?;
                let mut input = gallery::lock_file(&path)?;
                let compress = matches!(
                    Path::new(&a.name)
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("")
                        .to_lowercase()
                        .as_str(),
                    "wav" | "bmp"
                );
                writer
                    .start_file(
                        &a.archive_name,
                        if compress {
                            options
                        } else {
                            options
                                .compression_method(CompressionMethod::Stored)
                                .compression_level(None)
                        },
                    )
                    .map_err(err)?;
                let (bytes, hash) = transfer(&mut input, &mut writer, a.bytes)?;
                if bytes != a.bytes || hash != a.sha256 {
                    return Err("project_changed".into());
                }
            }
            writer.finish().map_err(err)?.sync_all().map_err(err)?;
            output.rewind().map_err(err)?;
            let hash = digest(&mut output)?;
            p.path = Some(path.to_string_lossy().into());
            p.version = Some(hash.clone());
            p.dirty = false;
            let intent = SaveIntent {
                target: path.to_string_lossy().into(),
                temporary: temporary.to_string_lossy().into(),
                backup: backup.to_string_lossy().into(),
                previous: previous.clone(),
                next: hash,
                project: p.clone(),
            };
            s.db.execute(
                "INSERT INTO project_save_intent VALUES(1,?1)",
                [serde_json::to_string(&intent).map_err(err)?],
            )
            .map_err(err)?;
            if let Some(file) = &existing {
                gallery::rename_handle(file, &backup)?;
            }
            gallery::rename_handle(&output, path)?;
            commit(&mut s, p.clone()).map_err(|_| "project_save_recovery")?;
            if let Some(file) = &existing {
                let _ = gallery::delete_handle(file);
            }
            Ok(p.clone())
        })();
        drop(output);
        drop(existing);
        if result.is_err() {
            if pending(&s)? {
                let _ = recover_save(&mut s);
            } else {
                let _ = gallery::delete_owned(&temporary);
            }
        }
        result
    }
}
