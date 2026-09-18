use super::*;

pub(super) struct PreparedImage { pub id: String, pub model: File }
const MAX_WAITING: usize = 20;

impl ImageEngine {
    fn prepare(&self, request: &mut ImageRequest) -> Result<(ImageProbe, File, String)> {
        if self.stopped.load(Ordering::Relaxed) { return Err("image_closing".into()); }
        if self.workspace()?.recovery_available { return Err("image_recovery_pending".into()); }
        validate(request)?;
        request.model_path = fs::canonicalize(&request.model_path).map_err(|_| "image_path")?.to_string_lossy().into();
        let mut model = read_locked(Path::new(&request.model_path))?;
        let probe = self.probe(&request.model_path);
        if !probe.ready { return Err(format!("image_not_ready: {}", probe.missing.join(", "))); }
        let digest = sha(&mut model, Some(&self.stopped), |_| {})?;
        Ok((probe, model, digest))
    }
    pub(super) fn start_in(self: &Arc<Self>, mut request: ImageRequest, temporary: Option<&Path>) -> Result<ImageJob> {
        let (probe, model, digest) = self.prepare(&mut request)?;
        let job = ImageJob {
            id: uuid::Uuid::new_v4().to_string(), request, status: "queued".into(), phase: "queued".into(),
            step: 0, hashed_bytes: 0, model_bytes: probe.model_bytes.unwrap_or(0), model_sha256: Some(digest),
            runtime: probe.runtime, device: probe.device.unwrap_or_default(), created_at: crate::database::now(),
            elapsed_ms: 0, error: None, output: None, saved_path: None, saved_binding: None, working_directory: None, log_tail: String::new(), discarded: false,
            started_at: None, finished_at: None, queue_position: None,
        };
        let mut job = job;
        if let Some(temporary) = temporary {
            model_library::no_links(temporary).map_err(|_| "image_path")?;
            let outputs = temporary.join("image-results");
            fs::create_dir_all(&outputs).map_err(|_| "image_storage")?;
            model_library::no_links(&outputs).map_err(|_| "image_path")?;
            job.working_directory = Some(outputs.join(&job.id).to_string_lossy().into());
        }
        self.enqueue_prepared(job, model, false)
    }
    pub fn resume(self: &Arc<Self>, id: &str) -> Result<ImageJob> {
        let job = self.list()?.into_iter().find(|j| j.id == id && j.status == "paused").ok_or("image_not_paused")?;
        let mut request = job.request.clone();
        let (probe, model, digest) = self.prepare(&mut request)?;
        if job.model_sha256.as_deref() != Some(&digest) { return Err("image_model_changed".into()); }
        if probe.runtime != job.runtime || probe.device.as_deref() != Some(&job.device) { return Err("image_environment_changed".into()); }
        self.enqueue_prepared(job, model, true)
    }
    fn enqueue_prepared(self: &Arc<Self>, mut job: ImageJob, model: File, resume: bool) -> Result<ImageJob> {
        // Serialize thread creation with shutdown; the worker itself never takes this mutex.
        let mut active = self.active.lock().map_err(|_| "image_storage")?;
        let mut state = self.state.lock().map_err(|_| "image_storage")?;
        if self.stopped.load(Ordering::Relaxed) { return Err("image_closing".into()); }
        if state.recovery_available { return Err("image_recovery_pending".into()); }
        if state.queue.len() >= MAX_WAITING { return Err("image_queue_full".into()); }
        if resume && !state.jobs.iter().any(|j| j.id == job.id && j.status == "paused") { return Err("image_not_paused".into()); }
        if !resume {
            let directory = job_directory(&job, &self.config)?;
            model_library::no_links(directory.parent().ok_or("image_path")?).map_err(|_| "image_path")?;
            fs::create_dir(directory).map_err(|_| "image_storage")?;
        }
        job.status = "queued".into(); job.phase = "queued".into(); job.error = None;
        persist(&state, &job)?;
        if resume { *state.jobs.iter_mut().find(|j| j.id == job.id).unwrap() = job.clone(); }
        else { state.jobs.insert(0, job.clone()); }
        state.queue.push_back(PreparedImage { id: job.id.clone(), model });
        job.queue_position = Some(state.queue.len());
        drop(state);
        if active.as_ref().is_some_and(|a| a.thread.is_finished()) { if let Some(old) = active.take() { let _ = old.thread.join(); } }
        if active.is_none() {
            let weak = Arc::downgrade(self);
            let thread = std::thread::spawn(move || loop {
                let Some(engine) = weak.upgrade() else { break; };
                if engine.stopped.load(Ordering::Relaxed) { break; }
                if engine.run_next().is_err() { engine.stop(); break; }
                drop(engine);
                std::thread::sleep(Duration::from_millis(50));
            });
            *active = Some(Active { thread });
        }
        Ok(job)
    }
    fn run_next(&self) -> Result<()> {
        let next = {
            let mut state = self.state.lock().map_err(|_| "image_storage")?;
            if self.stopped.load(Ordering::Relaxed) { return Ok(()); }
            let Some(prepared) = state.queue.pop_front() else { return Ok(()); };
            let Some(index) = state.jobs.iter().position(|j| j.id == prepared.id && j.status == "queued") else { return Ok(()); };
            let mut job = state.jobs[index].clone();
            job.status = "running".into(); job.phase = "hashing".into(); job.started_at = Some(crate::database::now());
            persist(&state, &job)?;
            let cancel = Arc::new(AtomicBool::new(false));
            *self.running.lock().map_err(|_| "image_storage")? = Some((job.id.clone(), cancel.clone()));
            state.jobs[index] = job.clone();
            (job, prepared.model, cancel)
        };
        let (job, model_pin, cancel) = next;
        let directory = job_directory(&job, &self.config)?;
        let id = job.id.clone();
        let result = self.execute(job, directory, cancel, model_pin);
        *self.running.lock().map_err(|_| "image_storage")? = None;
        if result.is_err() {
            // Keep the durable running record for output recovery after a storage failure.
            let _ = self.update(&id, false, |j| { j.status = "failed".into(); j.phase = "failed".into(); j.error = Some("image_storage".into()); });
        }
        result
    }
    pub fn cancel(&self, id: &str) -> Result<()> {
        let mut state = self.state.lock().map_err(|_| "image_storage")?;
        let index = state.jobs.iter().position(|j| j.id == id).ok_or("image_missing")?;
        if ["queued", "paused"].contains(&state.jobs[index].status.as_str()) {
            let mut job = state.jobs[index].clone(); job.status = "cancelled".into(); job.phase = "cancelled".into();
            job.error = None; job.finished_at = Some(crate::database::now()); persist(&state, &job)?;
            state.jobs[index] = job; state.queue.retain(|p| p.id != id); return Ok(());
        }
        if let Some((running_id, cancel)) = self.running.lock().map_err(|_| "image_storage")?.as_ref() {
            if running_id == id { cancel.store(true, Ordering::Relaxed); }
        }
        Ok(())
    }
    pub fn stop(&self) {
        self.stopped.store(true, Ordering::Relaxed);
        if let Ok(mut state) = self.state.lock() {
            for index in 0..state.jobs.len() {
                if state.jobs[index].status == "queued" {
                    let mut job = state.jobs[index].clone(); job.status = "paused".into(); job.phase = "paused".into(); job.error = Some("image_queue_paused".into());
                    let _ = persist(&state, &job); state.jobs[index] = job;
                }
            }
            state.queue.clear();
            if let Ok(running) = self.running.lock() { if let Some((_, cancel)) = running.as_ref() { cancel.store(true, Ordering::Relaxed); } }
        }
    }
}

#[tauri::command]
pub async fn image_resume(id: String, state: tauri::State<'_, Arc<ImageEngine>>) -> Result<ImageJob> {
    let engine = state.inner().clone(); tauri::async_runtime::spawn_blocking(move || engine.resume(&id)).await.map_err(|_| "image_storage")?
}

#[cfg(test)]
#[path = "queue_tests.rs"]
mod tests;
