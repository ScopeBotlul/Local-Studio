use crate::database::{now, Database};
use crate::types::{AppSnapshot, Job, Settings};
use crate::worker::{HashRequest, WorkerEvent};
use fs2::FileExt;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

struct ActiveWorker {
    id: String,
    child: Child,
}

pub struct Core {
    database: Mutex<Database>,
    database_path: PathBuf,
    log_path: PathBuf,
    log_lock: Mutex<()>,
    portable: bool,
    storage_warning: Mutex<Option<String>>,
    stopped: AtomicBool,
    active: Mutex<Option<ActiveWorker>>,
    scheduler: Mutex<Option<JoinHandle<()>>>,
    _instance_lock: File,
}

fn locked<T>(mutex: &Mutex<T>) -> Result<MutexGuard<'_, T>, String> {
    mutex
        .lock()
        .map_err(|_| "Internal state was interrupted; please restart Local Studio.".into())
}

impl Core {
    pub fn new(config: PathBuf, data: PathBuf, portable: bool) -> Result<Arc<Self>, String> {
        fs::create_dir_all(&config)
            .map_err(|e| format!("Cannot create configuration folder: {e}"))?;
        let instance_lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(config.join("session.lock"))
            .map_err(|e| e.to_string())?;
        instance_lock.try_lock_exclusive().map_err(|_| {
            "Another Local Studio instance is already using this configuration folder.".to_string()
        })?;
        let database_path = config.join("local-studio.sqlite3");
        let database = Database::open(&database_path, &crate::settings::defaults(&data))?;
        let storage_warning = crate::settings::prepare_storage(&database.settings()?).err();
        let core = Arc::new(Self {
            database: Mutex::new(database),
            database_path,
            log_path: config.join("local-studio.log"),
            log_lock: Mutex::new(()),
            portable,
            storage_warning: Mutex::new(storage_warning),
            stopped: AtomicBool::new(false),
            active: Mutex::new(None),
            scheduler: Mutex::new(None),
            _instance_lock: instance_lock,
        });
        core.log("INFO", "Application core initialized; telemetry disabled.");
        Ok(core)
    }

    pub fn snapshot(&self) -> Result<AppSnapshot, String> {
        let (settings, jobs, recovery_available) = {
            let db = locked(&self.database)?;
            (
                db.settings()?,
                db.jobs()?,
                db.meta("recovery_available")?.as_deref() == Some("true"),
            )
        };
        let mut hardware = crate::hardware::discover();
        if let Some(warning) = locked(&self.storage_warning)?.as_ref() {
            hardware.warnings.push(format!("Saved storage is unavailable. Open Settings to choose a writable data folder. {warning}"));
        }
        Ok(AppSnapshot {
            version: env!("CARGO_PKG_VERSION").into(),
            paths: crate::settings::effective_paths(&settings),
            settings,
            hardware,
            jobs,
            recovery_available,
            database_path: self.database_path.to_string_lossy().into_owned(),
            portable: self.portable,
        })
    }

    pub fn save_settings(&self, settings: Settings) -> Result<Settings, String> {
        if self.stopped.load(Ordering::SeqCst) {
            return Err("Application is closing.".into());
        }
        crate::settings::prepare_storage(&settings)?;
        let _active = locked(&self.active)?;
        if self.stopped.load(Ordering::SeqCst) {
            return Err("Application is closing.".into());
        }
        locked(&self.database)?.save_settings(&settings)?;
        *locked(&self.storage_warning)? = None;
        self.log(
            "INFO",
            "Settings persisted; changing the data root does not move existing media.",
        );
        Ok(settings)
    }

    pub fn current_settings(&self) -> Result<Settings, String> { locked(&self.database)?.settings() }

    pub fn storage_paths(&self) -> Result<crate::types::StoragePaths, String> {
        Ok(crate::settings::effective_paths(&locked(&self.database)?.settings()?))
    }

    pub fn jobs(&self) -> Result<Vec<Job>, String> {
        locked(&self.database)?.jobs()
    }

    pub fn enqueue(&self, path: &str) -> Result<Job, String> {
        if !Path::new(path).is_absolute() {
            return Err("Select an absolute file path.".into());
        }
        let path =
            fs::canonicalize(path).map_err(|e| format!("Cannot access selected file: {e}"))?;
        if !path.is_file() {
            return Err("Select a regular file to calculate its SHA-256 digest.".into());
        }
        File::open(&path).map_err(|e| format!("Selected file is not readable: {e}"))?;
        let _active = locked(&self.active)?;
        if self.stopped.load(Ordering::SeqCst) {
            return Err("Application is closing.".into());
        }
        let job = locked(&self.database)?.insert_job(&path)?;
        self.log("INFO", &format!("Job {} queued (SHA-256).", job.id));
        Ok(job)
    }

    pub fn cancel(&self, id: &str) -> Result<(), String> {
        let mut active = locked(&self.active)?;
        let db = locked(&self.database)?;
        let job = db.job(id)?.ok_or("Job not found.")?;
        if !["queued", "running"].contains(&job.status.as_str()) {
            return Ok(());
        }
        if let Some(worker) = active.as_mut().filter(|worker| worker.id == id) {
            // Kill and reap before acknowledging cancellation. Dropping Child alone would orphan it.
            match worker.child.try_wait().map_err(|e| e.to_string())? {
                None => worker
                    .child
                    .kill()
                    .map_err(|e| format!("Could not stop worker: {e}"))?,
                Some(_) => {}
            }
            worker
                .child
                .wait()
                .map_err(|e| format!("Could not reap stopped worker: {e}"))?;
        }
        db.finish(id, "cancelled", None, None)?;
        self.log("INFO", &format!("Job {id} cancelled; worker stopped."));
        Ok(())
    }

    pub fn dismiss_recovery(&self) -> Result<(), String> {
        locked(&self.database)?.set_meta("recovery_available", "false")
    }

    pub fn start_scheduler(self: &Arc<Self>) {
        let core = self.clone();
        let handle = std::thread::spawn(move || {
            while !core.stopped.load(Ordering::SeqCst) {
                if let Err(error) = core.run_next() {
                    core.stop_failed_worker(&error);
                    core.log("ERROR", &error);
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        });
        if let Ok(mut scheduler) = self.scheduler.lock() {
            *scheduler = Some(handle);
        }
    }

    fn run_next(&self) -> Result<(), String> {
        let (job, receiver) = {
            // Same lock order as cancel/shutdown: active first, then database.
            let mut active = locked(&self.active)?;
            if self.stopped.load(Ordering::SeqCst) {
                return Ok(());
            }
            let db = locked(&self.database)?;
            let Some(job) = db.next_job()? else {
                return Ok(());
            };
            db.start_job(&job.id)?;
            match Self::spawn_worker(&job) {
                Ok((child, receiver)) => {
                    *active = Some(ActiveWorker {
                        id: job.id.clone(),
                        child,
                    });
                    (job, receiver)
                }
                Err(error) => {
                    db.finish(&job.id, "failed", None, Some(&error))?;
                    return Err(error);
                }
            }
        };
        self.log(
            "INFO",
            &format!("Job {} started in isolated process.", job.id),
        );
        let mut result = None;
        let mut failure = None;
        let mut last_message = Instant::now();
        let mut ready = false;
        loop {
            if self.stopped.load(Ordering::SeqCst) {
                break;
            }
            if locked(&self.database)?
                .job(&job.id)?
                .is_some_and(|j| j.status == "cancelled")
            {
                break;
            }
            match receiver.recv_timeout(Duration::from_millis(200)) {
                Ok(Ok(event)) => {
                    last_message = Instant::now();
                    match event {
                        WorkerEvent::Ready { protocol: 1 } => ready = true,
                        WorkerEvent::Ready { .. } => {
                            failure = Some("Worker protocol mismatch.".into());
                            break;
                        }
                        WorkerEvent::Progress {
                            bytes_read,
                            total_bytes,
                        } if ready && bytes_read <= total_bytes => {
                            let progress = if total_bytes == 0 {
                                0.0
                            } else {
                                bytes_read as f64 / total_bytes as f64
                            };
                            locked(&self.database)?.progress(&job.id, progress)?;
                        }
                        WorkerEvent::Completed {
                            sha256,
                            bytes_read: _,
                        } if ready
                            && sha256.len() == 64
                            && sha256.bytes().all(|b| b.is_ascii_hexdigit()) =>
                        {
                            result = Some(sha256)
                        }
                        WorkerEvent::Failed { message } => {
                            failure = Some(message);
                            break;
                        }
                        _ => {
                            failure = Some("Worker emitted invalid output.".into());
                            break;
                        }
                    }
                }
                Ok(Err(error)) => {
                    failure = Some(error);
                    break;
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => break,
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    if last_message.elapsed() > Duration::from_secs(30) {
                        failure = Some("Worker did not respond for 30 seconds and was stopped. Check whether the file or drive is accessible.".into());
                        break;
                    }
                }
            }
        }
        let exit_success = {
            let mut active = locked(&self.active)?;
            if let Some(mut worker) = active.take() {
                let status = worker.child.try_wait().map_err(|e| e.to_string())?;
                if status.is_none() {
                    // Output EOF can precede process teardown by a few milliseconds.
                    if result.is_some() && failure.is_none() && !self.stopped.load(Ordering::SeqCst)
                    {
                        let deadline = Instant::now() + Duration::from_secs(2);
                        while worker
                            .child
                            .try_wait()
                            .map_err(|e| e.to_string())?
                            .is_none()
                            && Instant::now() < deadline
                        {
                            std::thread::sleep(Duration::from_millis(10));
                        }
                    }
                    if worker
                        .child
                        .try_wait()
                        .map_err(|e| e.to_string())?
                        .is_none()
                    {
                        let _ = worker.child.kill();
                    }
                }
                worker
                    .child
                    .wait()
                    .map(|status| status.success())
                    .unwrap_or(false)
            } else {
                false
            }
        };
        let db = locked(&self.database)?;
        if let Some(error) = failure {
            db.finish(&job.id, "failed", None, Some(&error))?;
            self.log("ERROR", &format!("Job {} failed: {error}", job.id));
        } else if let Some(result) = result.filter(|_| exit_success) {
            db.finish(&job.id, "completed", Some(&result), None)?;
            self.log("INFO", &format!("Job {} completed.", job.id));
        } else {
            db.finish(
                &job.id,
                "failed",
                None,
                Some("Worker exited without a complete result."),
            )?;
        }
        Ok(())
    }

    fn spawn_worker(
        job: &Job,
    ) -> Result<(Child, mpsc::Receiver<Result<WorkerEvent, String>>), String> {
        let executable = std::env::current_exe().map_err(|e| e.to_string())?;
        let mut command = Command::new(executable);
        command
            .arg("--worker")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        crate::hardware::hide_console(&mut command);
        let mut child = command
            .spawn()
            .map_err(|e| format!("Cannot start isolated worker: {e}"))?;
        let request = HashRequest {
            protocol: 1,
            path: job.input_path.clone(),
        };
        let send_result = (|| -> Result<(), String> {
            let stdin = child
                .stdin
                .as_mut()
                .ok_or("Worker input pipe unavailable.")?;
            serde_json::to_writer(&mut *stdin, &request).map_err(|e| e.to_string())?;
            stdin
                .write_all(b"\n")
                .and_then(|_| stdin.flush())
                .map_err(|e| e.to_string())
        })();
        if let Err(error) = send_result {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }
        let stdout = match child.stdout.take() {
            Some(stdout) => stdout,
            None => {
                let _ = child.kill();
                let _ = child.wait();
                return Err("Worker output pipe unavailable.".into());
            }
        };
        let (sender, receiver) = mpsc::channel();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let event = line.map_err(|e| e.to_string()).and_then(|line| {
                    serde_json::from_str(&line).map_err(|e| format!("Invalid worker response: {e}"))
                });
                let failed = event.is_err();
                if sender.send(event).is_err() || failed {
                    break;
                }
            }
        });
        Ok((child, receiver))
    }

    fn stop_failed_worker(&self, error: &str) {
        let mut active = self
            .active
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if let Some(mut worker) = active.take() {
            let _ = worker.child.kill();
            let _ = worker.child.wait();
            if let Ok(db) = self.database.lock() {
                let _ = db.finish(&worker.id, "failed", None, Some(error));
            }
        }
    }
    pub fn shutdown(&self) -> Result<(), String> {
        self.stopped.store(true, Ordering::SeqCst);
        {
            let mut active = locked(&self.active)?;
            if let Some(mut worker) = active.take() {
                if worker
                    .child
                    .try_wait()
                    .map_err(|e| e.to_string())?
                    .is_none()
                {
                    worker.child.kill().map_err(|e| e.to_string())?;
                }
                worker.child.wait().map_err(|e| e.to_string())?;
            }
            locked(&self.database)?.clean_shutdown()?;
        }
        if let Some(scheduler) = locked(&self.scheduler)?.take() {
            let _ = scheduler.join();
        }
        self.log("INFO", "Clean shutdown; all worker processes stopped.");
        Ok(())
    }

    fn log(&self, level: &str, message: &str) {
        let Ok(_guard) = self.log_lock.lock() else {
            return;
        };
        if fs::metadata(&self.log_path).is_ok_and(|m| m.len() > 1024 * 1024) {
            let previous = self.log_path.with_extension("previous.log");
            let _ = fs::remove_file(&previous);
            let _ = fs::rename(&self.log_path, previous);
        }
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
        {
            let _ = writeln!(
                file,
                "{} [{level}] {}",
                now(),
                message.replace(['\r', '\n'], " ")
            );
        }
    }

    pub fn logs(&self) -> Result<String, String> {
        let _guard = locked(&self.log_lock)?;
        let mut file = match File::open(&self.log_path) {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(String::new()),
            Err(error) => return Err(error.to_string()),
        };
        let length = file.metadata().map_err(|e| e.to_string())?.len();
        let offset = length.saturating_sub(128 * 1024);
        file.seek(SeekFrom::Start(offset))
            .map_err(|e| e.to_string())?;
        let mut data = Vec::new();
        file.read_to_end(&mut data).map_err(|e| e.to_string())?;
        let mut text = String::from_utf8_lossy(&data).into_owned();
        if offset > 0 {
            text = text
                .split_once('\n')
                .map(|(_, rest)| rest.to_string())
                .unwrap_or_default();
        }
        Ok(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unavailable_saved_data_root_keeps_settings_repairable() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config");
        let data = dir.path().join("data");
        let unavailable = dir.path().join("not-a-directory");
        fs::write(&unavailable, b"occupied path").unwrap();
        let first = Core::new(config.clone(), data.clone(), false).unwrap();
        let mut settings = locked(&first.database).unwrap().settings().unwrap();
        settings.data_root = unavailable.to_string_lossy().into_owned();
        locked(&first.database)
            .unwrap()
            .save_settings(&settings)
            .unwrap();
        first.shutdown().unwrap();
        drop(first);
        let restarted = Core::new(config, data.clone(), false).unwrap();
        assert!(locked(&restarted.storage_warning).unwrap().is_some());
        assert_eq!(
            locked(&restarted.database)
                .unwrap()
                .settings()
                .unwrap()
                .data_root,
            settings.data_root
        );
        settings.data_root = data.to_string_lossy().into_owned();
        restarted.save_settings(settings).unwrap();
        assert!(locked(&restarted.storage_warning).unwrap().is_none());
        restarted.shutdown().unwrap();
    }
    #[test]
    fn rejects_second_instance_and_can_restart_after_release() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path().join("config");
        let data = dir.path().join("data");
        let first = Core::new(config.clone(), data.clone(), false).unwrap();
        assert!(Core::new(config.clone(), data.clone(), false).is_err());
        first.shutdown().unwrap();
        drop(first);
        assert!(Core::new(config, data, false).is_ok());
    }
}
