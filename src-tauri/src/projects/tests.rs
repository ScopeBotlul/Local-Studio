use super::*;
fn request() -> ImageRequest {
    ImageRequest { vae_on_cpu:false,  reference:None,
        model_path: String::new(),
        prompt: "Project prompt ü".into(),
        negative_prompt: "noise".into(),
        width: 512,
        height: 768,
        steps: 25,
        guidance: 5.,
        seed: 42,
        sampler: "euler".into(),
    }
}

#[test]
fn rename_removed_media_and_multiple_recovery_points_survive_save_and_restart() {
    let t = tempfile::tempdir().unwrap();
    let p = Projects::new(t.path()).unwrap();
    let original = p
        .new_project(t.path(), "first".into(), Some(request()), false)
        .unwrap();
    let source = t.path().join("media.png");
    fs::write(&source, b"passive fixture").unwrap();
    let loaded = p.add(vec![source.to_string_lossy().into()]).unwrap();
    let asset = loaded.assets[0].clone();
    p.checkpoint().unwrap();
    let point = p.history().unwrap()[0].id;
    let renamed = p.rename(&original.id, "renamed".into()).unwrap();
    assert_eq!(renamed.name, "renamed");
    assert!(p.rename("stale", "wrong".into()).is_err());
    p.remove(&asset.id).unwrap();
    let target = t.path().join("undo.localstudio");
    p.save(&target).unwrap();
    let mut file = File::open(&target).unwrap();
    assert_eq!(archive(&mut file).unwrap().len(), 1);
    drop(file);
    drop(p);
    let p = Projects::new(t.path()).unwrap();
    p.recover().unwrap();
    assert_eq!(p.snapshot().unwrap().unwrap().removed.len(), 1);
    p.restore_media(&asset.id).unwrap();
    assert_eq!(p.snapshot().unwrap().unwrap().assets.len(), 1);
    assert!(p.restore_point(point, false).is_err());
    let restored = p.restore_point(point, true).unwrap();
    assert_eq!(restored.name, "first");
    assert!(restored.dirty);
    assert_eq!(restored.assets.len(), 1);
    p.remove(&asset.id).unwrap();
    fs::write(owned(&restored, &asset).unwrap(), b"corrupt").unwrap();
    assert!(p.restore_media(&asset.id).is_err());
    assert!(p.snapshot().unwrap().unwrap().assets.is_empty());
    assert_eq!(fs::read(source).unwrap(), b"passive fixture");
}

#[test]
fn project_cleanup_protects_active_and_last_three_points_and_rejects_replacement() {
    let t = tempfile::tempdir().unwrap();
    let p = Projects::new(t.path()).unwrap();
    let source = t.path().join("media.wav");
    fs::write(&source, b"media").unwrap();
    let mut sessions = vec![];
    for n in 0..6 {
        p.new_project(t.path(), format!("project {n}"), None, true)
            .unwrap();
        let project = p.add(vec![source.to_string_lossy().into()]).unwrap();
        p.checkpoint().unwrap();
        sessions.push(project);
    }
    let old = crate::projects::history::now() + 1;
    let inventory = p.cleanup_inventory(old).unwrap();
    assert!(!inventory.files.is_empty());
    assert!(inventory
        .files
        .iter()
        .all(|f| f.owner != sessions.last().unwrap().id));
    let protected: Vec<String> = {
        let s = p.state.lock().unwrap();
        let mut stmt =
            s.db.prepare("SELECT project_id FROM project_history ORDER BY id DESC LIMIT 3")
                .unwrap();
        let result = stmt
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<std::result::Result<_, _>>()
            .unwrap();
        result
    };
    assert!(inventory
        .files
        .iter()
        .all(|f| !protected.contains(&f.owner)));
    let changed = inventory.files[0].clone();
    fs::write(&changed.path, b"external replacement bytes").unwrap();
    assert!(p.cleanup_file(&changed, old).is_err());
    assert!(Path::new(&changed.path).exists());
    let candidate = inventory
        .files
        .iter()
        .find(|f| f.owner != changed.owner)
        .unwrap();
    let unknown = Path::new(&candidate.path)
        .parent()
        .unwrap()
        .join("personal.txt");
    fs::write(&unknown, b"keep").unwrap();
    assert!(p.cleanup_file(candidate, old).unwrap());
    assert!(!Path::new(&candidate.path).exists());
    assert!(unknown.exists());
    assert!(source.exists());
    assert!(
        p.history()
            .unwrap()
            .iter()
            .filter(|point| point.protected)
            .count()
            >= 3
    );
}

#[test]
fn recovery_restored_after_cleanup_preview_is_protected_again() {
    let t = tempfile::tempdir().unwrap();
    let p = Projects::new(t.path()).unwrap();
    let source = t.path().join("media.png");
    fs::write(&source, b"fixture").unwrap();
    p.new_project(t.path(), "old".into(), None, true).unwrap();
    p.add(vec![source.to_string_lossy().into()]).unwrap();
    p.checkpoint().unwrap();
    let point = p.history().unwrap()[0].id;
    for n in 0..5 {
        p.new_project(t.path(), format!("new {n}"), None, true)
            .unwrap();
        p.checkpoint().unwrap();
    }
    let cutoff = history::now() + 1;
    let before = p.cleanup_inventory(cutoff).unwrap();
    assert_eq!(before.files.len(), 1);
    p.restore_point(point, true).unwrap();
    assert!(!p.cleanup_file(&before.files[0], cutoff).unwrap());
    assert!(Path::new(&before.files[0].path).exists());
}
#[test]
fn container_roundtrip_models_stay_external_and_modified_archive_is_not_overwritten() {
    let t = tempfile::tempdir().unwrap();
    let p = Projects::new(t.path()).unwrap();
    let mut r = request();
    let model = t.path().join("model.safetensors");
    fs::write(&model, b"model reference fixture, never executed").unwrap();
    r.model_path = model.to_string_lossy().into();
    p.new_project(t.path(), "Test ü".into(), Some(r), false)
        .unwrap();
    let source = t.path().join("image.png");
    fs::write(&source, b"passive media fixture").unwrap();
    p.add(vec![source.to_string_lossy().into()]).unwrap();
    let target = t.path().join("test.localstudio");
    let saved = p.save(&target).unwrap();
    assert!(!saved.dirty);
    assert!(saved.model.is_some());
    let mut f = File::open(&target).unwrap();
    let mut z = archive(&mut f).unwrap();
    assert_eq!(z.len(), 2);
    let m = read_manifest(&mut z).unwrap();
    assert!(m.request.unwrap().model_path.is_empty());
    assert!(z.file_names().all(|n| !n.ends_with("safetensors")));
    drop(z);
    drop(f);
    fs::remove_file(&source).unwrap();
    p.close(false).unwrap();
    let loaded = p.open(&target, t.path(), false).unwrap();
    assert_eq!(loaded.assets.len(), 1);
    assert_eq!(
        fs::read(owned(&loaded, &loaded.assets[0]).unwrap()).unwrap(),
        b"passive media fixture"
    );
    assert!(loaded.request.unwrap().model_path.is_empty());
    assert!(p.relink(&source).is_err());
    p.relink(&model).unwrap();
    let again = p.save(&target).unwrap();
    assert!(!again.dirty);
    fs::write(&target, b"external change").unwrap();
    assert!(p.save(&target).is_err());
    assert_eq!(fs::read(&target).unwrap(), b"external change");
}
#[test]
fn invalid_archives_never_replace_session_or_write_traversal_paths() {
    let t = tempfile::tempdir().unwrap();
    let p = Projects::new(t.path()).unwrap();
    let original = p
        .new_project(t.path(), "original".into(), None, false)
        .unwrap();
    let archive_path = t.path().join("bad.localstudio");
    let mut zip = ZipWriter::new(File::create(&archive_path).unwrap());
    zip.start_file(
        "../escape.png",
        SimpleFileOptions::default().compression_method(CompressionMethod::Stored),
    )
    .unwrap();
    zip.write_all(b"bad").unwrap();
    zip.finish().unwrap();
    assert!(p.open(&archive_path, t.path(), true).is_err());
    assert_eq!(p.snapshot().unwrap().unwrap().id, original.id);
    assert!(!t.path().join("escape.png").exists());
    let bad = t.path().join("model.gguf");
    fs::write(&bad, b"weights").unwrap();
    assert!(p.add(vec![bad.to_string_lossy().into()]).is_err());
    assert!(p.snapshot().unwrap().unwrap().assets.is_empty());
    let target = t.path().join("existing.localstudio");
    fs::write(&target, b"existing").unwrap();
    assert!(p.save(&target).is_err());
    assert_eq!(fs::read(target).unwrap(), b"existing");
    assert!(p.update("stale-project", Some(request())).is_err());
    assert!(p.close(false).is_err());
}
#[test]
fn removal_rewrites_container_without_deleting_source_and_session_recovers() {
    let t = tempfile::tempdir().unwrap();
    let p = Projects::new(t.path()).unwrap();
    p.new_project(t.path(), "project".into(), Some(request()), false)
        .unwrap();
    let source = t.path().join("sound.wav");
    fs::write(&source, [0; 4096]).unwrap();
    let project = p.add(vec![source.to_string_lossy().into()]).unwrap();
    let target = t.path().join("project.localstudio");
    p.save(&target).unwrap();
    p.remove(&project.assets[0].id).unwrap();
    p.save(&target).unwrap();
    let mut f = File::open(&target).unwrap();
    assert_eq!(archive(&mut f).unwrap().len(), 1);
    assert!(source.exists());
    drop(p);
    let p = Projects::new(t.path()).unwrap();
    assert!(p.snapshot().unwrap().unwrap().recovery);
    let recovered = p.recover().unwrap();
    assert!(!recovered.recovery);
    assert_eq!(recovered.request.unwrap().prompt, "Project prompt ü");
}

#[test]
fn checksum_failures_and_unsupported_versions_leave_active_project_intact() {
    let t = tempfile::tempdir().unwrap();
    let p = Projects::new(t.path()).unwrap();
    let original = p
        .new_project(t.path(), "original".into(), None, false)
        .unwrap();
    let id = uuid();
    let a = Asset {
        edit:vec![],
        id: id.clone(),
        name: "image.png".into(),
        kind: "image".into(),
        bytes: 4,
        sha256: "0".repeat(64),
        archive_name: format!("media/{id}.png"),
    };
    let mut m = Manifest {
        creative:None,
        format: "local-studio".into(),
        version: 1,
        name: "corrupt".into(),
        request: None,
        model: None,
        assets: vec![a.clone()],
    };
    for version in [1, 999] {
        m.version = version;
        let target = t.path().join(format!("bad-{version}.localstudio"));
        let mut z = ZipWriter::new(File::create(&target).unwrap());
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        z.start_file("project.json", options).unwrap();
        z.write_all(&serde_json::to_vec(&m).unwrap()).unwrap();
        z.start_file(&a.archive_name, options).unwrap();
        z.write_all(b"oops").unwrap();
        z.finish().unwrap();
        assert!(p.open(&target, t.path(), true).is_err());
        assert_eq!(p.snapshot().unwrap().unwrap().id, original.id);
    }
    m.version = 1;
    m.assets[0].bytes = MAX_BYTES + 1;
    assert!(validate(&m).is_err());
    m.assets[0] = a.clone();
    m.assets[0].edit = vec![gallery::EditOperation::Adjust{brightness:10,contrast:0,saturation:0,temperature:0}];
    assert!(validate(&m).is_err(), "v1 must not silently accept a recipe");
    m.version = 2;
    assert!(validate(&m).is_ok());
    m.assets[0].kind = "audio".into();
    m.assets[0].name = "audio.wav".into();
    m.assets[0].archive_name = format!("media/{id}.wav");
    assert!(validate(&m).is_err(), "image operations cannot target audio");
    m.assets[0] = a.clone();
    m.assets[0].edit = vec![gallery::EditOperation::Adjust{brightness:101,contrast:0,saturation:0,temperature:0}];
    assert!(validate(&m).is_err());
    let mut json=serde_json::to_value(&m).unwrap();
    json["assets"][0]["edit"]=serde_json::json!([{"type":"execute","code":"untrusted"}]);
    assert!(serde_json::from_value::<Manifest>(json).is_err());
    m.assets[0] = a.clone();
    m.assets.push(a);
    assert!(validate(&m).is_err());
}

#[test]
fn project_preview_only_serves_current_manifest_assets_to_main_window() {
    use tauri::http::Request;
    let t = tempfile::tempdir().unwrap();
    let p = Projects::new(t.path()).unwrap();
    p.new_project(t.path(), "preview".into(), None, false)
        .unwrap();
    let source = t.path().join("audio.wav");
    fs::write(&source, b"123456789").unwrap();
    let project = p.add(vec![source.to_string_lossy().into()]).unwrap();
    let url = format!("/{}/{}", project.id, project.assets[0].id);
    let req = || {
        Request::builder()
            .uri(&url)
            .header("Range", "bytes=1-3")
            .body(vec![])
            .unwrap()
    };
    let response = preview::response(Some(&project), "main", req());
    assert_eq!(response.status(), 206);
    assert_eq!(response.body(), b"234");
    assert_eq!(
        preview::response(Some(&project), "hf-browser", req()).status(),
        404
    );
    let mut recovered = project.clone();
    recovered.recovery = true;
    assert_eq!(
        preview::response(Some(&recovered), "main", req()).status(),
        404
    );
    let removed = p.remove(&project.assets[0].id).unwrap();
    assert_eq!(
        preview::response(Some(&removed), "main", req()).status(),
        404
    );
    assert!(owned(&project, &project.assets[0]).unwrap().exists());
}
#[test]
fn recent_projects_track_only_successful_save_and_open_and_do_not_delete_files(){
 let t=tempfile::tempdir().unwrap();let p=Projects::new(t.path()).unwrap();p.new_project(t.path(),"Recent".into(),None,false).unwrap();let target=t.path().join("recent.localstudio");p.save(&target).unwrap();p.close(false).unwrap();p.open(&target,t.path(),false).unwrap();let s=p.state.lock().unwrap();let count:i64=s.db.query_row("SELECT count(*) FROM recent_projects",[],|r|r.get(0)).unwrap();assert_eq!(count,1);s.db.execute("DELETE FROM recent_projects",[]).unwrap();assert!(target.exists());
}
