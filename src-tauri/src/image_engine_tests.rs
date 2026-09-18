use super::*;

#[test]
fn restored_history_keeps_creation_order_after_updates_and_recovers_running_jobs() {
    let directory = tempfile::tempdir().unwrap();
    let create = || ImageEngine::new(directory.path(), directory.path().join("absent-runtime")).unwrap();
    let engine = create();
    let make = |id: &str, created: &str, status: &str| ImageJob { restricted:false,locked:false,batch:None,sampling_steps:None,
        id: id.into(), request: super::tests::request(), status: status.into(), phase: status.into(),
        step: 0, hashed_bytes: 0, model_bytes: 10, model_sha256: None, runtime: "test fixture".into(), device: "test fixture".into(),
        created_at: created.into(), elapsed_ms: 0, error: None, output: None, saved_path: None, saved_binding: None, working_directory: None, log_tail: String::new(), discarded: false, started_at: None, finished_at: None, queue_position: None,
    };
    let older = make("older", "2026-09-17T10:00:00.000Z", "completed");
    let newer = make("newer", "2026-09-17T10:01:00.000Z", "running");
    { let state = engine.state.lock().unwrap(); persist(&state, &newer).unwrap(); persist(&state, &older).unwrap(); }
    drop(engine);
    for _ in 0..3 {
        let engine = create(); let jobs = engine.list().unwrap();
        assert_eq!(jobs.iter().map(|j| j.id.as_str()).collect::<Vec<_>>(), vec!["newer", "older"]);
        assert_eq!(jobs[0].status, "interrupted"); assert_eq!(jobs[1].status, "completed");
        assert!(engine.active.lock().unwrap().is_none());
    }
}

#[test]
fn image_output_requires_real_decodable_png_and_exact_requested_dimensions() {
    let directory = tempfile::tempdir().unwrap(); let path = directory.path().join("image.png");
    let mut encoded = Vec::new();
    { let mut encoder = png::Encoder::new(&mut encoded, 2, 2); encoder.set_color(png::ColorType::Rgb); encoder.set_depth(png::BitDepth::Eight);
      let mut writer = encoder.write_header().unwrap(); writer.write_image_data(&[128; 12]).unwrap(); }
    fs::write(&path, &encoded).unwrap();
    assert_eq!(png_bytes(&path, 2, 2).unwrap(), encoded); assert!(png_bytes(&path, 512, 512).is_err());
    let idat = encoded.windows(4).position(|bytes| bytes == b"IDAT").unwrap(); encoded[idat + 5] ^= 0xff;
    fs::write(&path, &encoded).unwrap(); assert!(png_bytes(&path, 2, 2).is_err());
}
