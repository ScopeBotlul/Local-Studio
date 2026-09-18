use super::*;

fn weights(path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let header = br#"{"weight":{"dtype":"F32","shape":[2],"data_offsets":[0,8]}}"#;
    let mut data = (header.len() as u64).to_le_bytes().to_vec(); data.extend_from_slice(header); data.extend_from_slice(&[0; 8]);
    fs::write(path, data).unwrap();
}
fn manager(path: &Path) -> Arc<ModelLibrary> { let config = path.join("config"); fs::create_dir_all(&config).unwrap(); ModelLibrary::new(&config).unwrap() }

#[test]
fn validates_real_safetensors_structure_without_loading_tensors() {
    let dir = tempfile::tempdir().unwrap(); let path = dir.path().join("model.safetensors"); weights(&path);
    let entry = inspect(&path, "test"); assert_eq!(entry.status, "checked"); assert_eq!(entry.completeness, "container"); assert_eq!(entry.kind, "candidate");
    let mut bytes = fs::read(&path).unwrap(); bytes.pop(); fs::write(&path, bytes).unwrap();
    assert_eq!(inspect(&path, "test").status, "invalid");
    fs::write(&path, u64::MAX.to_le_bytes()).unwrap(); assert_eq!(inspect(&path, "test").status, "invalid");
    fs::write(&path, (MAX_HEADER + 1).to_le_bytes()).unwrap();
    fs::OpenOptions::new().write(true).open(&path).unwrap().set_len(MAX_HEADER + 9).unwrap();
    assert_eq!(inspect(&path, "test").status, "unverified");
}
#[test]
fn rejects_tensor_shape_mismatch_and_overlapping_ranges() {
    let dir = tempfile::tempdir().unwrap(); let path = dir.path().join("invalid.safetensors");
    for header in [r#"{"w":{"dtype":"F32","shape":[20],"data_offsets":[0,8]}}"#, r#"{"a":{"dtype":"F32","shape":[1],"data_offsets":[0,4]},"b":{"dtype":"F32","shape":[1],"data_offsets":[0,4]}}"#, r#"{"w":{"dtype":"F32","shape":[2],"data_offsets":[0,8]},"w":{"dtype":"F32","shape":[2],"data_offsets":[0,8]}}"#] {
        let mut data = (header.len() as u64).to_le_bytes().to_vec(); data.extend_from_slice(header.as_bytes()); data.extend_from_slice(&[0; 8]); fs::write(&path, data).unwrap();
        assert_eq!(inspect(&path, "test").status, "invalid");
    }
}
#[test]
fn index_detects_missing_shards_and_cannot_escape_folder() {
    let dir = tempfile::tempdir().unwrap(); weights(&dir.path().join("one.safetensors"));
    let index = dir.path().join("model.safetensors.index.json");
    fs::write(&index, r#"{"weight_map":{"a":"one.safetensors","b":"two.safetensors"}}"#).unwrap();
    let entry = inspect(&index, "test"); assert_eq!(entry.status, "incomplete"); assert_eq!(entry.files.len(), 3);
    weights(&dir.path().join("two.safetensors")); assert_eq!(inspect(&index, "test").status, "checked");
    for target in ["../escape.safetensors", "C:/escape.safetensors", "/root/model.safetensors", "a\\model.safetensors"] {
        fs::write(&index, serde_json::json!({"weight_map":{"a":target}}).to_string()).unwrap();
        let entry = inspect(&index, "test"); assert_eq!(entry.status, "invalid"); assert_eq!(entry.files.len(), 1);
    }
}
#[test]
fn scan_persists_deduplicates_and_groups_shards_without_writing_source() {
    let dir = tempfile::tempdir().unwrap(); let source = dir.path().join("source"); weights(&source.join("one.safetensors")); weights(&source.join("two.safetensors"));
    fs::write(source.join("model.safetensors.index.json"), r#"{"weight_map":{"a":"one.safetensors","b":"two.safetensors"}}"#).unwrap();
    let original = fs::read(source.join("one.safetensors")).unwrap();
    let manager = manager(dir.path()); manager.scan_folder(source.clone()); manager.scan_folder(source.clone());
    let snapshot = manager.snapshot().unwrap(); assert_eq!(snapshot.entries.len(), 1); assert_eq!(snapshot.entries[0].format, "safetensors-index"); assert_eq!(snapshot.scan.imported, 1);
    let id = snapshot.entries[0].id.clone(); drop(manager);
    let reopened = ModelLibrary::new(&dir.path().join("config")).unwrap(); assert_eq!(reopened.snapshot().unwrap().entries[0].id, id);
    reopened.forget(&id).unwrap(); assert!(reopened.snapshot().unwrap().entries.is_empty()); assert_eq!(fs::read(source.join("one.safetensors")).unwrap(), original);
}
#[test]
fn detects_removed_and_changed_files_and_rechecks_explicitly() {
    let dir = tempfile::tempdir().unwrap(); let source = dir.path().join("source"); let file = source.join("model.safetensors"); weights(&file);
    let manager = manager(dir.path()); manager.scan_folder(source); let id = manager.snapshot().unwrap().entries[0].id.clone();
    fs::write(&file, b"broken").unwrap(); assert_eq!(manager.snapshot().unwrap().entries[0].status, "changed");
    manager.recheck(&id).unwrap(); assert_eq!(manager.snapshot().unwrap().entries[0].status, "invalid");
    fs::remove_file(&file).unwrap(); assert_eq!(manager.snapshot().unwrap().entries[0].status, "missing");
}
#[test]
fn cancellation_keeps_previous_library_and_never_imports_partial_scan() {
    let dir = tempfile::tempdir().unwrap(); let source = dir.path().join("source"); weights(&source.join("first.safetensors"));
    let manager = manager(dir.path()); manager.scan_folder(source.clone()); weights(&source.join("second.safetensors"));
    manager.stop(); manager.scan_folder(source);
    let snapshot = manager.snapshot().unwrap(); assert_eq!(snapshot.scan.status, "cancelled"); assert_eq!(snapshot.entries.len(), 1); assert_eq!(snapshot.scan.imported, 0);
}
#[test]
fn gguf_and_pickle_candidates_do_not_claim_completeness_or_execute_code() {
    let dir = tempfile::tempdir().unwrap(); let path = dir.path().join("test.gguf"); let mut header = b"GGUF".to_vec(); header.extend_from_slice(&3u32.to_le_bytes()); header.extend_from_slice(&[0; 16]); fs::write(&path, header).unwrap();
    let entry = inspect(&path, "test"); assert_eq!(entry.status, "recognized"); assert_eq!(entry.completeness, "unknown");
    let pickle = dir.path().join("model.ckpt"); fs::write(&pickle, b"cos\nsystem\n(S'echo forbidden'\ntR.").unwrap();
    assert_eq!(inspect(&pickle, "test").status, "unverified");
    assert!(!local_path(Path::new("relative"))); assert!(!local_path(Path::new(r"\\server\share\model"))); assert!(!relative_file("../x"));
}
#[test]
fn lora_metadata_is_a_component_not_a_base_model() {
    let dir = tempfile::tempdir().unwrap(); let path = dir.path().join("adapter.safetensors");
    let header = br#"{"lora_A.weight":{"dtype":"F32","shape":[1],"data_offsets":[0,4]}}"#;
    let mut data = (header.len() as u64).to_le_bytes().to_vec(); data.extend_from_slice(header); data.extend_from_slice(&[0; 4]); fs::write(&path, data).unwrap();
    assert_eq!(inspect(&path, "test").kind, "component");
}

#[test]
fn full_scan_processes_multiple_roots_and_reports_unavailable_roots() {
    let dir = tempfile::tempdir().unwrap(); let first = dir.path().join("drive-a"); let second = dir.path().join("drive-b");
    weights(&first.join("a.safetensors")); weights(&second.join("nested/b.safetensors"));
    let missing = dir.path().join("unavailable-drive"); let manager = manager(dir.path());
    manager.scan_roots(vec![first.clone(), missing, second.clone()], ScanMode::Full, ScanMode::Full.limits());
    let state = manager.snapshot().unwrap(); assert_eq!(state.entries.len(), 2); assert_eq!(state.scan.roots_finished, 3);
    assert_eq!(state.scan.mode, ScanMode::Full); assert_eq!(state.scan.status, "completed"); assert!(!state.scan.truncated);
    assert!(state.scan.notes.iter().any(|n| n.code == "local_unavailable"));
    manager.scan_folder(first); assert_eq!(manager.snapshot().unwrap().entries.len(), 2);
    assert!(second.join("nested/b.safetensors").exists());
}
#[test]
fn full_scan_limits_and_depth_skips_are_explicit_and_cancel_preserves_previous_results() {
    let dir = tempfile::tempdir().unwrap(); let first = dir.path().join("first"); let second = dir.path().join("second");
    weights(&first.join("a.safetensors")); weights(&second.join("nested/b.safetensors"));
    let manager = manager(dir.path());
    manager.scan_roots(vec![first.clone(), second.clone()], ScanMode::Full, ScanLimits { entries: 1, ..ScanMode::Full.limits() });
    let state = manager.snapshot().unwrap(); assert!(state.scan.truncated); assert_eq!(state.entries.len(), 1);
    assert!(state.scan.roots_finished < state.scan.roots.len());
    manager.scan_roots(vec![second.clone(), first.clone()], ScanMode::Full, ScanLimits { depth: 0, ..ScanMode::Full.limits() });
    let state = manager.snapshot().unwrap(); assert!(state.scan.truncated); assert_eq!(state.scan.roots_finished, 2);
    assert!(state.scan.notes.iter().any(|n| n.code == "local_limit"));
    manager.stop(); manager.scan_roots(vec![first, second], ScanMode::Full, ScanMode::Full.limits());
    let state = manager.snapshot().unwrap(); assert_eq!(state.scan.status, "cancelled"); assert_eq!(state.entries.len(), 1);
}
#[test]
fn full_scan_discovery_excludes_network_drives_and_old_scan_state_migrates() {
    for kind in [2, 3, 5, 6] { assert!(local_drive_kind(kind)); }
    for kind in [0, 1, 4, 7, u32::MAX] { assert!(!local_drive_kind(kind)); }
    let roots = full_scan_roots().unwrap(); assert!(!roots.is_empty());
    assert!(roots.iter().all(|p| local_path(p) && p.parent().is_none()));
    let dir = tempfile::tempdir().unwrap(); let manager = manager(dir.path());
    manager.state.lock().unwrap().db.execute("INSERT OR REPLACE INTO model_scan VALUES(1,?1)", [r#"{"status":"running","root":"old","visited":0,"found":0,"imported":0,"skipped":0,"truncated":false,"notes":[]}"#]).unwrap();
    drop(manager);
    let reopened = ModelLibrary::new(&dir.path().join("config")).unwrap(); let state = reopened.snapshot().unwrap();
    assert_eq!(state.scan.status, "interrupted"); assert_eq!(state.scan.mode, ScanMode::Folder); assert!(state.scan.roots.is_empty());
}

#[test]
fn quick_search_includes_library_caches_and_common_folders_without_overlaps() {
    let dir = tempfile::tempdir().unwrap(); let profile = dir.path().join("profile"); let model = dir.path().join("models");
    fs::create_dir_all(model.join("assistant")).unwrap(); fs::create_dir_all(profile.join(".cache/huggingface/hub")).unwrap();
    let candidates = quick_candidates(&profile, &[dir.path().to_path_buf()], &model, vec![model.join("assistant"), model.clone()]);
    let roots = unique_roots(candidates);
    assert!(roots.contains(&model)); assert!(roots.contains(&profile.join(".cache/huggingface/hub")));
    assert!(!roots.contains(&model.join("assistant"))); assert_eq!(roots.iter().filter(|p| **p == model).count(), 1);
}
#[test]
#[ignore = "Requires Windows Developer Mode or symlink privilege; path boundaries tested separately"]
fn hf_snapshot_links_allow_only_local_same_repository_blob_files() {
    let dir = tempfile::tempdir().unwrap(); let repo = dir.path().join("models--test--model");
    let snapshot = repo.join("snapshots").join("a".repeat(40)); fs::create_dir_all(&snapshot).unwrap();
    let blob = repo.join("blobs").join("b".repeat(64)); weights(&blob);
    let link = snapshot.join("model.safetensors");
    std::os::windows::fs::symlink_file(&blob, &link).unwrap();
    assert_eq!(inspect(&link, "cache").status, "checked");
    let manager = manager(dir.path()); manager.scan_folder(repo); assert_eq!(manager.snapshot().unwrap().entries.len(), 1);
    let outside = dir.path().join("outside.safetensors"); weights(&outside);
    let escape = snapshot.join("escape.safetensors"); std::os::windows::fs::symlink_file(&outside, &escape).unwrap();
    assert!(physical_file(&escape).is_err());
    let arbitrary = dir.path().join("arbitrary.safetensors"); std::os::windows::fs::symlink_file(&blob, &arbitrary).unwrap();
    assert!(physical_file(&arbitrary).is_err());
}

#[test]
fn hf_cache_path_boundaries_reject_other_repositories_and_non_hash_blobs() {
    let repo = PathBuf::from(r"C:\cache\models--owner--model");
    let snapshot = repo.join("snapshots").join("a".repeat(40)); let blobs = repo.join("blobs");
    assert_eq!(snapshot_repository(&snapshot.join("nested/model.safetensors")).unwrap(), repo);
    assert!(snapshot_repository(&repo.join("snapshots/main/model.safetensors")).is_err());
    assert!(snapshot_repository(Path::new(r"C:\arbitrary\model.safetensors")).is_err());
    assert!(same_repository_blob(&blobs.join("b".repeat(64)), &blobs));
    assert!(same_repository_blob(&blobs.join("c".repeat(40)), &blobs));
    assert!(!same_repository_blob(&blobs.join("not-a-hash"), &blobs));
    assert!(!same_repository_blob(&repo.join("other").join("b".repeat(64)), &blobs));
    assert!(!same_repository_blob(&blobs.join("nested").join("b".repeat(64)), &blobs));
}

#[test]
fn automatic_scan_excludes_dependency_files_but_manual_folder_remains_available() {
    let dir = tempfile::tempdir().unwrap(); let source = dir.path().join("source");
    weights(&source.join("models/real.safetensors"));
    weights(&source.join("site-packages/test/example.safetensors"));
    weights(&source.join(".artifacts/test.safetensors"));
    fs::write(source.join("python-path.pth"), b"import site; configure_python_path()\n").unwrap();
    fs::write(source.join("tutor.pt"), b"This is a Portuguese Vim tutorial").unwrap();
    fs::write(source.join("models/unknown.onnx"), b"unconfirmed onnx candidate").unwrap();
    let manager = manager(dir.path());
    manager.scan_roots(vec![source.clone()], ScanMode::Full, ScanMode::Full.limits());
    let state = manager.snapshot().unwrap();
    assert_eq!(state.entries.len(), 2);
    assert_eq!(state.entries.iter().find(|e| e.name == "real.safetensors").unwrap().discovery, Discovery::Model);
    assert_eq!(state.entries.iter().find(|e| e.name == "unknown.onnx").unwrap().discovery, Discovery::Candidate);
    assert!(state.scan.notes.iter().any(|n| n.code == "local_plain_text"));
    assert!(state.scan.notes.iter().any(|n| n.code == "local_dependency_files"));
    manager.scan_folder(source.join("site-packages/test"));
    assert_eq!(manager.snapshot().unwrap().entries.iter().filter(|e| e.discovery == Discovery::Model).count(), 2);
    assert_eq!(fs::read(source.join("python-path.pth")).unwrap(), b"import site; configure_python_path()\n");
}
#[test]
fn exclusion_matches_directory_components_and_preserves_normal_model_locations() {
    for path in [r"C:\Program Files\Office\AI\word.onnx", r"C:\Windows\model.onnx", r"D:\$Recycle.Bin\model.gguf", r"D:\AI\venv\Lib\site-packages\fixture.safetensors"] {
        assert!(excluded_location(Path::new(path)).is_some(), "{path}");
    }
    for path in [r"D:\LocalAI\ComfyUI\models\checkpoints\model.safetensors", r"C:\Users\owner\.cache\huggingface\hub\model.safetensors", r"D:\models\not_site-packages\small-lora.safetensors", r"C:\Users\owner\Windows\model.gguf"] {
        assert!(excluded_location(Path::new(path)).is_none(), "{path}");
    }
}
#[test]
fn legacy_migration_reclassifies_without_removing_records_or_touching_sources() {
    let dir = tempfile::tempdir().unwrap(); let source = dir.path().join("source"); weights(&source.join("model.safetensors"));
    fs::write(source.join("python.pth"), b"import python_path\n").unwrap();
    fs::write(source.join("unconfirmed.ckpt"), [0x80, 0x02, 0x00]).unwrap();
    let engine = manager(dir.path());
    let original = fs::read(source.join("model.safetensors")).unwrap();
    { let state = engine.state.lock().unwrap();
      for name in ["model.safetensors", "python.pth", "unconfirmed.ckpt"] {
        let entry = inspect(&source.join(name), source.to_str().unwrap());
        let mut json = serde_json::to_value(&entry).unwrap();
        for key in ["discovery", "discoveryReason", "scanMode"] { json.as_object_mut().unwrap().remove(key); }
        state.db.execute("INSERT INTO local_models VALUES(?1,?2)", params![entry.id, json.to_string()]).unwrap();
      }
      state.db.pragma_update(None, "user_version", 0).unwrap(); }
    drop(engine);
    let engine = manager(dir.path()); let state = engine.snapshot().unwrap(); assert_eq!(state.entries.len(), 3);
    assert_eq!(state.entries.iter().filter(|e| e.discovery == Discovery::Model).count(), 1);
    assert_eq!(state.entries.iter().filter(|e| e.discovery == Discovery::Candidate).count(), 1);
    assert_eq!(state.entries.iter().filter(|e| e.discovery == Discovery::Excluded).count(), 1);
    let model = state.entries.iter().find(|e| e.discovery == Discovery::Model).unwrap();
    fs::write(source.join("model.safetensors"), b"damaged").unwrap(); engine.recheck(&model.id).unwrap();
    let rechecked = engine.snapshot().unwrap(); let model = rechecked.entries.iter().find(|e| e.id == model.id).unwrap();
    assert_eq!(model.discovery, Discovery::Model); assert_eq!(model.status, "invalid");
    fs::write(source.join("model.safetensors"), original).unwrap();
    assert_eq!(fs::read(source.join("python.pth")).unwrap(), b"import python_path\n");
    drop(engine); assert_eq!(manager(dir.path()).snapshot().unwrap().entries.len(), 3);
}

#[test]
fn valid_tensor_cache_is_not_automatically_presented_as_model_weights() {
    let dir = tempfile::tempdir().unwrap(); let path = dir.path().join("cached.safetensors");
    let header = br#"{"latents":{"dtype":"F32","shape":[1],"data_offsets":[0,4]}}"#;
    let mut bytes = (header.len() as u64).to_le_bytes().to_vec(); bytes.extend_from_slice(header); bytes.extend_from_slice(&[0;4]); fs::write(&path, bytes).unwrap();
    let entry = inspect(&path, dir.path().to_str().unwrap()); assert_eq!(entry.status, "checked");
    assert_eq!(entry.discovery, Discovery::Candidate); assert_eq!(entry.discovery_reason, "local_tensor_data");
    weights(&path); assert_eq!(inspect(&path, "test").discovery, Discovery::Model);
}
