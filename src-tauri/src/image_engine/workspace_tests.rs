use super::*;
fn fixture(engine: &ImageEngine) -> ImageJob {
    let mut request = super::super::tests::request(); request.width = 2; request.height = 2;
    let id = uuid::Uuid::new_v4().to_string(); let directory = engine.config.join(&id); fs::create_dir(&directory).unwrap();
    let path = directory.join("image.png");
    let mut encoder = png::Encoder::new(File::create(&path).unwrap(), 2, 2); encoder.set_color(png::ColorType::Rgb); encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header().unwrap().write_image_data(&[120; 12]).unwrap();
    for name in ["prompt.txt", "negative.txt", "metadata.json"] { fs::write(directory.join(name), b"fixture").unwrap(); }
    let job = ImageJob { id, request, status: "completed".into(), phase: "completed".into(), step: 20, hashed_bytes: 10, model_bytes: 10, model_sha256: None, runtime: "test fixture".into(), device: "test fixture".into(), created_at: crate::database::now(), elapsed_ms: 0, error: None, output: Some(path.to_string_lossy().into()), saved_path: None, saved_binding: None, working_directory: None, log_tail: String::new(), discarded: false, started_at: None, finished_at: None, queue_position: None };
    let mut state = engine.state.lock().unwrap(); persist(&state, &job).unwrap(); state.jobs.insert(0, job.clone()); job
}

#[test]fn gallery_relocation_preserves_job_and_purge_retires_output_without_deleting_unknown_files(){
 let t=tempfile::tempdir().unwrap();let engine=ImageEngine::new(t.path(),t.path().join("absent-runtime")).unwrap();let job=fixture(&engine);let gallery=t.path().join("gallery");fs::create_dir(&gallery).unwrap();let saved=engine.save(&job.id,&gallery).unwrap();let renamed=Path::new(&saved).with_file_name("renamed.png");fs::rename(&saved,&renamed).unwrap();
 engine.relocate_gallery(&job.id,Path::new(&saved),Some(&renamed)).unwrap();engine.relocate_gallery(&job.id,Path::new(&saved),Some(&renamed)).unwrap();assert!(engine.output(&job.id).is_ok());assert!(engine.list().unwrap()[0].saved_binding.is_some());
 let unknown=Path::new(job.output.as_ref().unwrap()).with_file_name("keep.txt");fs::write(&unknown,b"unknown user content").unwrap();engine.clear_gallery_duplicate(&job.id).unwrap();assert!(engine.output(&job.id).is_ok());assert!(!Path::new(job.output.as_ref().unwrap()).exists());
 fs::remove_file(&renamed).unwrap();engine.relocate_gallery(&job.id,&renamed,None).unwrap();engine.relocate_gallery(&job.id,&renamed,None).unwrap();let retired=engine.list().unwrap().remove(0);assert!(retired.discarded);assert!(retired.output.is_none());assert!(retired.saved_path.is_none());assert_eq!(retired.request.prompt,job.request.prompt);assert!(engine.output(&job.id).is_err());assert!(engine.save(&job.id,&gallery).is_err());assert_eq!(fs::read(unknown).unwrap(),b"unknown user content");
}
fn workspace() -> ImageWorkspace {
    let mut request = super::super::tests::request(); request.prompt = "Recovery prompt".into();
    ImageWorkspace { request: Some(request.clone()), models: [("model".into(), request)].into() }
}

#[test]
fn cleanup_never_deletes_unsaved_or_active_results_and_verifies_saved_duplicate() {
    let t=tempfile::tempdir().unwrap();let engine=ImageEngine::new(t.path(),t.path().join("runtime")).unwrap();let job=fixture(&engine);
    let directory=engine.config.join(&job.id);fs::write(directory.join("prompt.txt"),&job.request.prompt).unwrap();fs::write(directory.join("negative.txt"),&job.request.negative_prompt).unwrap();fs::write(directory.join("metadata.json"),serde_json::to_vec(&job).unwrap()).unwrap();fs::write(directory.join("unknown.txt"),b"keep").unwrap();
    engine.update(&job.id,true,|j|j.created_at="2000-01-01T00:00:00Z".into()).unwrap();
    let cutoff=978307200;assert!(engine.cleanup_inventory(cutoff).unwrap().files.is_empty());
    let gallery=t.path().join("gallery");fs::create_dir(&gallery).unwrap();let saved=engine.save(&job.id,&gallery).unwrap();
    let inventory=engine.cleanup_inventory(cutoff).unwrap();assert_eq!(inventory.files.len(),4);
    let image=inventory.files.iter().find(|c|c.path.ends_with("image.png")).unwrap();let saved_bytes=fs::read(&saved).unwrap();
    assert!(engine.cleanup_file(image,cutoff).unwrap());assert!(!Path::new(&image.path).exists());assert_eq!(fs::read(&saved).unwrap(),saved_bytes);assert!(engine.output(&job.id).is_ok());assert!(directory.join("unknown.txt").exists());
    let second=fixture(&engine);engine.update(&second.id,true,|j|j.created_at="2000-01-01T00:00:00Z".into()).unwrap();let saved=engine.save(&second.id,&gallery).unwrap();let candidate=engine.cleanup_inventory(cutoff).unwrap().files.into_iter().find(|c|c.owner==second.id&&c.path.ends_with("image.png")).unwrap();
    fs::write(saved,b"external change").unwrap();assert!(engine.cleanup_file(&candidate,cutoff).is_err());assert!(Path::new(&candidate.path).exists());
    engine.update(&second.id,true,|j|j.status="paused".into()).unwrap();assert!(!engine.cleanup_file(&candidate,cutoff).unwrap());
}
#[test]
fn custom_output_location_survives_restart_recovery_save_and_scoped_discard() {
    let temp = tempfile::tempdir().unwrap(); let config = temp.path().join("config"); fs::create_dir(&config).unwrap();
    let create = || ImageEngine::new(&config, temp.path().join("runtime")).unwrap();
    let engine = create(); let job = fixture(&engine);
    let external = temp.path().join("separate-temp").join("image-results").join(&job.id);
    fs::create_dir_all(external.parent().unwrap()).unwrap(); fs::rename(engine.config.join(&job.id), &external).unwrap();
    engine.update(&job.id, true, |j| { j.working_directory = Some(external.to_string_lossy().into()); j.output = None; j.status = "running".into(); }).unwrap();
    drop(engine); let engine = create();
    assert_eq!(engine.list().unwrap()[0].status, "completed"); assert!(engine.output(&job.id).is_ok());
    let gallery = temp.path().join("gallery"); fs::create_dir(&gallery).unwrap();
    let saved = engine.save(&job.id, &gallery).unwrap(); assert!(Path::new(&saved).is_file());
    fs::write(external.join("unknown.txt"), b"keep").unwrap(); engine.clear_gallery_duplicate(&job.id).unwrap();
    assert!(!external.join("image.png").exists()); assert!(external.join("unknown.txt").exists()); assert!(Path::new(&saved).exists());
    let unsaved = fixture(&engine); let moved = external.parent().unwrap().join(&unsaved.id);
    fs::rename(engine.config.join(&unsaved.id), &moved).unwrap();
    engine.update(&unsaved.id, true, |j| { j.working_directory = Some(moved.to_string_lossy().into()); j.output = Some(moved.join("image.png").to_string_lossy().into()); }).unwrap();
    engine.discard(&unsaved.id).unwrap(); assert!(!moved.exists()); assert!(Path::new(&saved).exists());
}
#[test]
fn all_unsaved_results_survive_restart_and_exit_refuses_silent_loss() {
    let temp = tempfile::tempdir().unwrap();
    let create = || ImageEngine::new(temp.path(), temp.path().join("runtime")).unwrap();
    let engine = create(); for _ in 0..61 { fixture(&engine); } drop(engine);
    let engine = create(); assert_eq!(engine.workspace().unwrap().unsaved, 61);
    assert!(engine.workspace().unwrap().recovery_available);
    assert!(engine.finish_session(None, temp.path(), false).is_err());
    assert!(!engine.stopped.load(Ordering::Relaxed)); assert_eq!(engine.workspace().unwrap().unsaved, 61);
    let gallery = temp.path().join("gallery"); fs::create_dir(&gallery).unwrap();
    engine.finish_session(Some(ImageExitAction::Save), &gallery, false).unwrap();
    assert_eq!(fs::read_dir(&gallery).unwrap().count(), 61);
    for job in engine.list().unwrap() { assert!(Path::new(job.saved_path.as_ref().unwrap()).is_file()); assert!(engine.discard(&job.id).is_err()); }
    drop(engine); let engine = create(); assert_eq!(engine.workspace().unwrap().unsaved, 0); assert!(!engine.workspace().unwrap().recovery_available);
}
#[test]
fn failed_save_keeps_drafts_and_does_not_mark_clean_or_leave_engine_stopped() {
    let temp = tempfile::tempdir().unwrap(); let engine = ImageEngine::new(temp.path(), temp.path().join("runtime")).unwrap();
    let job = fixture(&engine); let blocked = temp.path().join("gallery"); fs::write(&blocked, b"not a directory").unwrap();
    assert!(engine.finish_session(Some(ImageExitAction::Save), &blocked, false).is_err());
    assert!(Path::new(job.output.as_ref().unwrap()).is_file()); assert!(!engine.stopped.load(Ordering::Relaxed));
    assert!(engine.list().unwrap()[0].saved_path.is_none()); drop(engine);
    let restored = ImageEngine::new(temp.path(), temp.path().join("runtime")).unwrap(); assert!(restored.workspace().unwrap().recovery_available);
}
#[test]
fn discard_is_scoped_retryable_and_never_removes_unknown_or_gallery_files() {
    use std::os::windows::fs::OpenOptionsExt;
    let temp = tempfile::tempdir().unwrap(); let engine = ImageEngine::new(temp.path(), temp.path().join("runtime")).unwrap();
    let job = fixture(&engine); let image = Path::new(job.output.as_ref().unwrap());
    let unknown = image.parent().unwrap().join("user-file.txt"); fs::write(&unknown, b"keep me").unwrap();
    let guard = OpenOptions::new().read(true).share_mode(1).open(image).unwrap();
    assert!(engine.discard(&job.id).is_err()); assert!(engine.list().unwrap()[0].discarded); assert!(engine.output(&job.id).is_err());
    drop(guard); engine.discard(&job.id).unwrap(); engine.discard(&job.id).unwrap();
    assert!(!image.exists()); assert!(unknown.is_file()); assert_eq!(engine.workspace().unwrap().unsaved, 0);
    assert!(engine.discard("../outside").is_err());
    let outside = temp.path().join("outside.png"); fs::write(&outside, b"protected").unwrap();
    let forged = fixture(&engine); let _ = engine.update(&forged.id, true, |j| j.output = Some(outside.to_string_lossy().into()));
    assert!(engine.discard(&forged.id).is_err()); assert_eq!(fs::read(&outside).unwrap(), b"protected");
}
#[test]
fn crash_workspace_is_offered_until_recovered_and_clean_start_obeys_preference() {
    let temp = tempfile::tempdir().unwrap(); let create = || ImageEngine::new(temp.path(), temp.path().join("runtime")).unwrap();
    let engine = create(); engine.save_workspace(workspace()).unwrap(); drop(engine);
    let engine = create(); assert!(engine.workspace().unwrap().recovery_available);
    assert!(engine.save_workspace(ImageWorkspace::default()).is_err());
    assert_eq!(engine.recover().unwrap().workspace.request.unwrap().prompt, "Recovery prompt");
    engine.finish_session(None, temp.path(), true).unwrap(); drop(engine);
    let engine = create(); assert!(!engine.workspace().unwrap().recovery_available); assert!(engine.workspace().unwrap().workspace.request.is_some());
    engine.finish_session(None, temp.path(), false).unwrap(); drop(engine);
    let engine = create(); let restored = engine.workspace().unwrap(); assert!(restored.workspace.request.is_none()); assert_eq!(restored.workspace.models.len(), 1); assert!(!restored.recovery_available);
}
#[test]
fn crash_after_valid_png_before_job_commit_recovers_actual_output() {
    let temp = tempfile::tempdir().unwrap(); let create = || ImageEngine::new(temp.path(), temp.path().join("runtime")).unwrap();
    let engine = create(); let job = fixture(&engine);
    let _ = engine.update(&job.id, true, |j| { j.status = "running".into(); j.output = None; }); drop(engine);
    let engine = create(); let recovered = &engine.list().unwrap()[0]; assert_eq!(recovered.status, "completed"); assert!(engine.output(&job.id).unwrap().starts_with("data:image/png;base64,"));
}

#[test]
fn explicit_keep_retains_unsaved_results_only_with_restore_enabled() {
    let temp = tempfile::tempdir().unwrap(); let create = || ImageEngine::new(temp.path(), temp.path().join("runtime")).unwrap();
    let engine = create(); let job = fixture(&engine); engine.save_workspace(workspace()).unwrap();
    assert!(engine.finish_session(Some(ImageExitAction::Keep), temp.path(), false).is_err());
    engine.finish_session(Some(ImageExitAction::Keep), temp.path(), true).unwrap(); drop(engine);
    let engine = create(); let snapshot = engine.workspace().unwrap();
    assert!(!snapshot.recovery_available); assert_eq!(snapshot.unsaved, 1); assert!(snapshot.workspace.request.is_some());
    assert!(engine.output(&job.id).is_ok());
}

#[test]
fn corrupt_job_records_are_not_silently_dropped_and_whitespace_prompts_remain_bounded() {
    let temp = tempfile::tempdir().unwrap(); let engine = ImageEngine::new(temp.path(), temp.path().join("runtime")).unwrap();
    let mut oversized = workspace(); oversized.request.as_mut().unwrap().prompt = " ".repeat(4001);
    assert!(engine.save_workspace(oversized).is_err());
    engine.state.lock().unwrap().db.execute("INSERT INTO image_jobs VALUES('bad','{}')", []).unwrap(); drop(engine);
    assert!(ImageEngine::new(temp.path(), temp.path().join("runtime")).is_err());
}
