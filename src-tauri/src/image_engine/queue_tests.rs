use super::*;

fn fixture(engine: &ImageEngine, path: &Path) -> ImageJob {
    let mut request = super::super::tests::request(); request.model_path = path.to_string_lossy().into();
    let job = ImageJob { id: uuid::Uuid::new_v4().to_string(), request, status: "queued".into(), phase: "queued".into(), step: 0, hashed_bytes: 0, model_bytes: 4, model_sha256: Some("immutable test hash".into()), runtime: "unit fixture".into(), device: "unit fixture".into(), created_at: crate::database::now(), elapsed_ms: 0, error: None, output: None, saved_path: None, saved_binding: None, working_directory: None, log_tail: String::new(), discarded: false, started_at: None, finished_at: None, queue_position: None };
    fs::create_dir(engine.config.join(&job.id)).unwrap();
    let mut state = engine.state.lock().unwrap(); persist(&state, &job).unwrap(); state.jobs.insert(0, job.clone());
    state.queue.push_back(PreparedImage { id: job.id.clone(), model: read_locked(path).unwrap() }); job
}

#[test]
fn queued_cancellation_releases_model_pin_without_touching_running_or_other_waiters() {
    let temp = tempfile::tempdir().unwrap(); let engine = ImageEngine::new(temp.path(), temp.path().join("runtime")).unwrap();
    let a = temp.path().join("a"); let b = temp.path().join("b"); fs::write(&a, b"test").unwrap(); fs::write(&b, b"test").unwrap();
    let first = fixture(&engine, &a); let second = fixture(&engine, &b);
    let cancel = Arc::new(AtomicBool::new(false)); *engine.running.lock().unwrap() = Some(("running fixture".into(), cancel.clone()));
    assert!(OpenOptions::new().write(true).open(&a).is_err()); assert!(OpenOptions::new().write(true).open(&b).is_err());
    assert_eq!(engine.list().unwrap().iter().find(|j| j.id == second.id).unwrap().queue_position, Some(2));
    engine.cancel(&first.id).unwrap(); assert!(!cancel.load(Ordering::Relaxed));
    assert!(OpenOptions::new().write(true).open(&a).is_ok()); assert!(OpenOptions::new().write(true).open(&b).is_err());
    let jobs = engine.list().unwrap(); let cancelled = jobs.iter().find(|j| j.id == first.id).unwrap();
    assert_eq!(cancelled.status, "cancelled"); assert!(cancelled.started_at.is_none()); assert!(cancelled.finished_at.is_some());
    assert_eq!(jobs.iter().find(|j| j.id == second.id).unwrap().queue_position, Some(1));
    engine.stop(); assert!(cancel.load(Ordering::Relaxed)); assert!(OpenOptions::new().write(true).open(&b).is_ok());
}

#[test]
fn shutdown_and_crash_pause_waiters_with_original_snapshot_and_never_autostart() {
    for clean in [true, false] {
        let temp = tempfile::tempdir().unwrap(); let path = temp.path().join("model"); fs::write(&path, b"test").unwrap();
        let create = || ImageEngine::new(temp.path(), temp.path().join("runtime")).unwrap();
        let engine = create(); let job = fixture(&engine, &path);
        if clean { engine.finish_session(None, temp.path(), false).unwrap(); } drop(engine);
        let engine = create(); let restored = engine.list().unwrap().pop().unwrap();
        assert_eq!(restored.status, "paused"); assert!(restored.started_at.is_none()); assert_eq!(restored.model_sha256, job.model_sha256);
        assert_eq!(serde_json::to_value(restored.request).unwrap(), serde_json::to_value(job.request).unwrap());
        assert!(engine.state.lock().unwrap().queue.is_empty()); assert!(engine.active.lock().unwrap().is_none());
        assert_eq!(engine.workspace().unwrap().recovery_available, !clean);
    }
}

#[test]
fn queue_limit_and_failed_commit_leave_jobs_and_fifo_unchanged() {
    let temp = tempfile::tempdir().unwrap(); let engine = ImageEngine::new(temp.path(), temp.path().join("runtime")).unwrap();
    let path = temp.path().join("model"); fs::write(&path, b"test").unwrap();
    let first = fixture(&engine, &path); for _ in 1..MAX_WAITING { fixture(&engine, &path); }
    let mut extra = first.clone(); extra.id = uuid::Uuid::new_v4().to_string();
    assert_eq!(engine.enqueue_prepared(extra, read_locked(&path).unwrap(), false).err().as_deref(), Some("image_queue_full"));
    engine.state.lock().unwrap().db.execute_batch("CREATE TRIGGER fail_update BEFORE INSERT ON image_jobs BEGIN SELECT RAISE(ABORT, 'test full disk'); END;").unwrap();
    assert_eq!(engine.cancel(&first.id).err().as_deref(), Some("image_storage"));
    assert_eq!(engine.update(&first.id, true, |j| j.status = "completed".into()).err().as_deref(), Some("image_storage"));
    let state = engine.state.lock().unwrap(); assert_eq!(state.queue.len(), MAX_WAITING); assert_eq!(state.queue.front().unwrap().id, first.id);
    assert_eq!(state.jobs.iter().find(|j| j.id == first.id).unwrap().status, "queued");
    assert!(!engine.config.join("extra").exists()); assert!(engine.active.lock().unwrap().is_none());
}
