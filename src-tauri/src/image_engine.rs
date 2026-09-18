use crate::{core::Core, model_library};
use base64::Engine;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs::{self, File, OpenOptions}, io::{Read, Write}, path::{Path, PathBuf}, process::{Command, Stdio}, sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}, thread::JoinHandle, time::{Duration, Instant}};

mod queue;
pub(crate) mod reference;
pub use reference::{ImageReference,image_reference};
mod cleanup;
pub use queue::{image_resume,image_generate_batch,BatchInfo};
mod workspace;
pub use workspace::*;

type Result<T> = std::result::Result<T, String>;
const RUNTIME: &str = include_str!("../image-runtime.json");
const MAX_IMAGE: u64 = 24 * 1024 * 1024;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImageRequest {
    #[serde(default,skip_serializing_if="is_false")] pub vae_on_cpu:bool,
    #[serde(default,skip_serializing_if="Option::is_none")] pub reference:Option<ImageReference>,
    pub model_path: String, pub prompt: String, pub negative_prompt: String,
    pub width: u32, pub height: u32, pub steps: u32, pub guidance: f32,
    pub seed: u32, pub sampler: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageJob {
    #[serde(default)] pub restricted:bool,
    #[serde(default,skip_deserializing)] pub locked:bool,
    #[serde(default)] pub batch:Option<BatchInfo>,
    #[serde(default)] pub sampling_steps:Option<u32>,
    pub id: String, pub request: ImageRequest, pub status: String, pub phase: String,
    pub step: u32, pub hashed_bytes: u64, pub model_bytes: u64,
    pub model_sha256: Option<String>, pub runtime: String, pub device: String,
    pub created_at: String, pub elapsed_ms: u64, pub error: Option<String>,
    pub output: Option<String>, pub saved_path: Option<String>, pub log_tail: String,
    #[serde(default)] pub saved_binding: Option<String>,
    #[serde(default)] pub working_directory: Option<String>,
    #[serde(default)] pub discarded: bool,
    #[serde(default)] pub started_at: Option<String>,
    #[serde(default)] pub finished_at: Option<String>,
    #[serde(default)] pub queue_position: Option<usize>,
}
#[derive(Clone,Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageProbe {
    pub ready: bool, pub family: Option<String>, pub model_bytes: Option<u64>,
    pub missing: Vec<String>, pub runtime: String, pub device: Option<String>,
    pub vram_bytes: Option<u64>, pub model_license: String, pub runtime_license: String,
}
struct State { db: Connection, jobs: Vec<ImageJob>, workspace: ImageWorkspace, recovery_available: bool, queue: std::collections::VecDeque<queue::PreparedImage> }
struct Active { thread: JoinHandle<()> }
pub struct ImageEngine { pub(crate) benchmarks: Arc<crate::benchmarks::Benchmarks>, state: Mutex<State>, active: Mutex<Option<Active>>, running: Mutex<Option<(String, Arc<AtomicBool>)>>, config: PathBuf, runtime: PathBuf, stopped: AtomicBool }

fn is_false(value:&bool)->bool{!*value}
pub(crate) fn validate(request: &ImageRequest) -> Result<()> {
    if let Some(reference)=&request.reference {reference::validate(reference,request.width,request.height)?;}
    if request.prompt.trim().is_empty() || request.prompt.len() > 4000 || request.negative_prompt.len() > 4000
        || request.prompt.contains('\0') || request.negative_prompt.contains('\0') { return Err("image_prompt".into()); }
    if ![512, 768, 1024].contains(&request.width) || ![512, 768, 1024].contains(&request.height)
        || !(1..=60).contains(&request.steps) || !request.guidance.is_finite() || !(1.0..=20.0).contains(&request.guidance)
        || !["euler", "dpm++2m"].contains(&request.sampler.as_str()) { return Err("image_parameters".into()); }
    Ok(())
}
fn persist(state: &State, job: &ImageJob) -> Result<()> {
    state.db.execute("INSERT OR REPLACE INTO image_jobs VALUES(?1,?2)", params![job.id, serde_json::to_string(job).map_err(|_| "image_storage")?]).map_err(|_| "image_storage")?; Ok(())
}
fn read_locked(path: &Path) -> Result<File> {
    use std::os::windows::fs::OpenOptionsExt;
    model_library::no_links(path).map_err(|_| "image_path")?;
    let file = OpenOptions::new().read(true).share_mode(1).custom_flags(0x00200000).open(path).map_err(|_| "image_path")?;
    use std::os::windows::fs::MetadataExt;
    let meta = file.metadata().map_err(|_| "image_path")?;
    if !meta.is_file() || meta.file_attributes() & 0x400 != 0 { return Err("image_path".into()); } Ok(file)
}
fn sha(file: &mut File, cancel: Option<&AtomicBool>, mut progress: impl FnMut(u64)) -> Result<String> {
    let mut digest = Sha256::new(); let mut buffer = vec![0; 1024 * 1024]; let mut bytes = 0;
    loop {
        if cancel.is_some_and(|c| c.load(Ordering::Relaxed)) { return Err("image_cancelled".into()); }
        let n = file.read(&mut buffer).map_err(|_| "image_path")?; if n == 0 { break; }
        digest.update(&buffer[..n]); bytes += n as u64; progress(bytes);
    }
    Ok(format!("{:x}", digest.finalize()))
}
pub(crate) fn model_parts(path: &Path) -> Result<(u64, Vec<String>)> {
    model_library::no_links(path).map_err(|_| "image_path")?;
    let mut budget = 16 * 1024 * 1024;
    if model_library::safetensors(path, &mut budget).map_err(|_| "image_structure")? != (true, false) { return Err("image_structure".into()); }
    let mut file = read_locked(path)?; let bytes = file.metadata().map_err(|_| "image_path")?.len();
    let mut length = [0;8]; file.read_exact(&mut length).map_err(|_| "image_structure")?;
    let size = u64::from_le_bytes(length); if size > 16 * 1024 * 1024 { return Err("image_structure".into()); }
    let mut header = vec![0; size as usize]; file.read_exact(&mut header).map_err(|_| "image_structure")?;
    let header: Value = serde_json::from_slice(&header).map_err(|_| "image_structure")?;
    let required = [
        ("SDXL UNet", "model.diffusion_model.input_blocks.0.0.weight", vec![320,4,3,3]),
        ("CLIP-L", "conditioner.embedders.0.transformer.text_model.embeddings.token_embedding.weight", vec![49408,768]),
        ("CLIP-G", "conditioner.embedders.1.model.token_embedding.weight", vec![49408,1280]),
        ("VAE encoder", "first_stage_model.encoder.conv_in.weight", vec![128,3,3,3]),
        ("VAE decoder", "first_stage_model.decoder.conv_out.weight", vec![3,128,3,3]),
    ];
    let missing = required.iter().filter(|(_,key,shape)| header[*key]["shape"] != serde_json::json!(shape)).map(|(name,_,_)| (*name).into()).collect();
    Ok((bytes, missing))
}
fn runtime_files(runtime: &Path) -> Result<Vec<File>> {
    model_library::no_links(runtime).map_err(|_| "image_runtime_missing")?;
    let manifest: Value = serde_json::from_str(RUNTIME).map_err(|_| "image_runtime_invalid")?;
    let hashes = manifest["files"].as_object().ok_or("image_runtime_invalid")?;
    // Prevent backend auto-discovery from loading a DLL added beside the trusted files.
    for entry in fs::read_dir(runtime).map_err(|_| "image_runtime_missing")? {
        let name = entry.map_err(|_| "image_runtime_invalid")?.file_name().to_string_lossy().to_string();
        if !hashes.contains_key(&name) { return Err("image_runtime_invalid".into()); }
    }
    let mut locks = Vec::new();
    for (name, expected) in hashes {
        let mut file = read_locked(&runtime.join(name)).map_err(|_| "image_runtime_missing")?;
        if sha(&mut file, None, |_| {})? != expected.as_str().unwrap_or("") { return Err("image_runtime_invalid".into()); }
        locks.push(file);
    }
    Ok(locks)
}
fn command(runtime: &Path) -> Command {
    let mut command = Command::new(runtime.join("sd-cli.exe"));
    command.current_dir(runtime).stdin(Stdio::null()).env_clear();
    for key in ["SystemRoot", "WINDIR", "TEMP", "TMP", "LOCALAPPDATA", "APPDATA", "USERPROFILE"] {
        if let Some(value) = std::env::var_os(key) { command.env(key, value); }
    }
    let windows = std::env::var_os("SystemRoot").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
    command.env("PATH", windows.join("System32"));
    crate::hardware::hide_console(&mut command); command
}
fn devices(runtime: &Path) -> Result<String> {
    let mut child = command(runtime).arg("--list-devices").stdout(Stdio::piped()).stderr(Stdio::null()).spawn().map_err(|_| "image_runtime_start")?;
    let _group = match ProcessGroup::attach(&child) { Ok(group) => group, Err(error) => { let _ = child.kill(); let _ = child.wait(); return Err(error); } };
    let stdout = child.stdout.take().ok_or("image_runtime_start")?;
    let reader = std::thread::spawn(move || { let mut output = String::new(); let _ = stdout.take(32 * 1024).read_to_string(&mut output); output });
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|_| "image_runtime_start")? { break status; }
        if started.elapsed() > Duration::from_secs(15) { let _ = child.kill(); let _ = child.wait(); let _ = reader.join(); return Err("image_runtime_timeout".into()); }
        std::thread::sleep(Duration::from_millis(30));
    };
    let output = reader.join().map_err(|_| "image_runtime_start")?;
    if !status.success() { return Err("image_runtime_start".into()); }
    output.lines().find(|line| line.starts_with("Vulkan") && line.contains('\t') && line.to_uppercase().contains("NVIDIA"))
        .map(str::to_owned).ok_or_else(|| "image_gpu".into())
}
pub(crate) struct ProcessGroup(windows_sys::Win32::Foundation::HANDLE);
impl ProcessGroup {
    pub(crate) fn peak_memory(&self) -> Option<u64> {
        use windows_sys::Win32::System::JobObjects::*;
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        let ok=unsafe { QueryInformationJobObject(self.0, JobObjectExtendedLimitInformation, &mut info as *mut _ as *mut _, std::mem::size_of_val(&info) as u32, std::ptr::null_mut()) };
        (ok!=0 && info.PeakJobMemoryUsed>0).then_some(info.PeakJobMemoryUsed as u64)
    }
    pub(crate) fn limit_memory(&self, bytes: usize) -> Result<()> {
        use windows_sys::Win32::System::JobObjects::*;
        let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { std::mem::zeroed() };
        info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_PROCESS_MEMORY | JOB_OBJECT_LIMIT_ACTIVE_PROCESS;
        info.BasicLimitInformation.ActiveProcessLimit = 4;
        info.ProcessMemoryLimit = bytes;
        if unsafe { SetInformationJobObject(self.0, JobObjectExtendedLimitInformation, &info as *const _ as *const _, std::mem::size_of_val(&info) as u32) } == 0 { return Err("image_worker_guard".into()); }
        Ok(())
    }
    pub(crate) fn attach(child: &std::process::Child) -> Result<Self> {
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::System::JobObjects::*;
        unsafe {
            let handle = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if handle.is_null() { return Err("image_worker_guard".into()); }
            let guard = Self(handle);
            let mut info: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
            info.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            if SetInformationJobObject(handle, JobObjectExtendedLimitInformation, &info as *const _ as *const _, std::mem::size_of_val(&info) as u32) == 0 || AssignProcessToJobObject(handle, child.as_raw_handle() as _) == 0 { return Err("image_worker_guard".into()); }
            Ok(guard)
        }
    }
}
impl Drop for ProcessGroup { fn drop(&mut self) { unsafe { windows_sys::Win32::Foundation::CloseHandle(self.0); } } }
fn sampling_progress(line: &str, steps: u32) -> Option<(u32,u32)> {
    if !line.contains("it/s") && !line.contains("s/it") { return None; }
    let fraction = line.rsplit('|').next()?.split_whitespace().next()?;
    let (step, total) = fraction.split_once('/')?;
    let step: u32 = step.parse().ok()?;
    let total=total.parse::<u32>().ok()?;
    (total>0 && total<=steps && step<=total).then_some((step,total))
}
fn png_bytes(path: &Path, width: u32, height: u32) -> Result<Vec<u8>> {
    let file = read_locked(path)?; let size = file.metadata().map_err(|_| "image_output")?.len();
    if size > MAX_IMAGE { return Err("image_output".into()); }
    let mut bytes = Vec::new(); file.take(MAX_IMAGE + 1).read_to_end(&mut bytes).map_err(|_| "image_output")?;
    let decoder = png::Decoder::new(std::io::Cursor::new(&bytes));
    let mut reader = decoder.read_info().map_err(|_| "image_output")?;
    if reader.info().width != width || reader.info().height != height || reader.output_buffer_size() > 16 * 1024 * 1024 { return Err("image_output".into()); }
    let mut pixels = vec![0; reader.output_buffer_size()]; reader.next_frame(&mut pixels).map_err(|_| "image_output")?;
    Ok(bytes)
}

fn job_directory(job: &ImageJob, legacy: &Path) -> Result<PathBuf> {
    if uuid::Uuid::parse_str(&job.id).map(|v| v.to_string()).ok().as_deref() != Some(&job.id) { return Err("image_path".into()); }
    let directory = job.working_directory.as_ref().map(PathBuf::from).unwrap_or_else(|| legacy.join(&job.id));
    if !directory.is_absolute() || directory.file_name().and_then(|n| n.to_str()) != Some(&job.id)
        || directory.parent().and_then(|p| p.file_name()).and_then(|n| n.to_str()) != Some("image-results")
        || directory.components().any(|c| c == std::path::Component::ParentDir) { return Err("image_path".into()); }
    Ok(directory)
}

impl ImageEngine {
    pub(crate) fn protect_known_jobs(&self)->Result<()> {
        let mut s=self.state.lock().map_err(|_|"image_storage")?;
        for n in 0..s.jobs.len(){if crate::privacy::request(&s.jobs[n].request){
            s.jobs[n].restricted=true;let job=&s.jobs[n];
            crate::privacy::protect_path(&job_directory(job,&self.config)?)?;
            if let Some(path)=&job.saved_path{if Path::new(path).is_file(){crate::privacy::mark(Path::new(path))?;}}
            persist(&s,job)?;
        }}Ok(())
    }
    pub fn new(config: &Path, runtime: PathBuf) -> Result<Arc<Self>> {
        let outputs = config.join("image-results"); fs::create_dir_all(&outputs).map_err(|_| "image_storage")?;
        let db = Connection::open(config.join("images.sqlite3")).map_err(|_| "image_storage")?;
        db.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE IF NOT EXISTS image_jobs(id TEXT PRIMARY KEY,json TEXT NOT NULL);").map_err(|_| "image_storage")?;
        let jobs = { let mut query = db.prepare("SELECT json FROM image_jobs ORDER BY json_extract(json, '$.createdAt') DESC, id DESC").map_err(|_| "image_storage")?;
            let rows = query.query_map([], |r| r.get::<_, String>(0)).map_err(|_| "image_storage")?;
            rows.map(|r| r.map_err(|_| "image_storage".to_string()).and_then(|json| serde_json::from_str::<ImageJob>(&json).map_err(|_| "image_storage".into()))).collect::<Result<Vec<_>>>()? };
        let (workspace, recovery_available) = workspace::load(&db, &jobs)?;
        let mut state = State { db, jobs, workspace, recovery_available, queue: Default::default() };
        for job in &mut state.jobs {
            if job.status == "queued" { job.status = "paused".into(); job.phase = "paused".into(); job.error = Some("image_queue_paused".into()); }
            if job.status == "running" {
                let output = job_directory(job, &outputs).ok().map(|p| p.join("image.png"));
                if let Some(output) = output.filter(|p| png_bytes(p, job.request.width, job.request.height).is_ok()) {
                    job.status = "completed".into(); job.phase = "completed".into(); job.output = Some(output.to_string_lossy().into()); job.step = job.sampling_steps.unwrap_or(job.request.steps);
                } else { job.status = "interrupted".into(); job.phase = "interrupted".into(); job.error = Some("image_interrupted".into()); }
            }
        }
        for job in &state.jobs { persist(&state, job)?; }
        Ok(Arc::new(Self { benchmarks: crate::benchmarks::Benchmarks::new(config)?, state: Mutex::new(state), active: Mutex::new(None), running: Mutex::new(None), config: outputs, runtime, stopped: AtomicBool::new(false) }))
    }
    pub fn list(&self) -> Result<Vec<ImageJob>> {
        let state = self.state.lock().map_err(|_| "image_storage")?; let mut jobs = state.jobs.clone();
        for job in &mut jobs { job.queue_position = state.queue.iter().position(|p| p.id == job.id).map(|index| index + 1); }
        Ok(jobs)
    }
    pub fn probe(&self, path: &str) -> ImageProbe {
        let mut result = ImageProbe { ready: false, family: None, model_bytes: None, missing: Vec::new(), runtime: "stable-diffusion.cpp cc515a0 · Vulkan".into(), device: None, vram_bytes: None, model_license: "unknown".into(), runtime_license: "MIT".into() };
        if path.is_empty() { result.missing.push("image_model_missing".into()); }
        else { match fs::canonicalize(path).map_err(|_| "image_path".to_string()).and_then(|path| model_parts(&path)) { Ok((bytes, missing)) => { result.model_bytes = Some(bytes); if missing.is_empty() { result.family = Some("SDXL".into()); } result.missing.extend(missing); }, Err(error) => result.missing.push(error) } }
        match runtime_files(&self.runtime) { Ok(_locks) => match devices(&self.runtime) { Ok(device) => result.device = Some(device), Err(error) => result.missing.push(error) }, Err(error) => result.missing.push(error) }
        let hardware = crate::hardware::discover();
        result.vram_bytes = hardware.gpus.iter().find(|g| g.vendor == "NVIDIA").and_then(|g| g.vram_bytes);
        if result.vram_bytes.is_some_and(|bytes| bytes < 8 * 1024 * 1024 * 1024) { result.missing.push("image_vram".into()); }
        result.ready = result.missing.is_empty(); result
    }
    fn update(&self, id: &str, save: bool, change: impl FnOnce(&mut ImageJob)) -> Result<()> {
        let mut state = self.state.lock().map_err(|_| "image_storage")?;
        let index = state.jobs.iter().position(|j| j.id == id).ok_or("image_missing")?;
        if save {
            let mut job = state.jobs[index].clone(); change(&mut job);
            persist(&state, &job)?; state.jobs[index] = job;
        } else { change(&mut state.jobs[index]); }
        Ok(())
    }
    fn execute(&self, job: ImageJob, directory: PathBuf, cancel: Arc<AtomicBool>, _model_pin: File) -> Result<()> {
        let mut started = Instant::now();
        let mut peak_memory = None;
        let result = (|| -> Result<String> {
            self.update(&job.id, true, |j|j.phase="waiting".into())?;
            let _admission=crate::resources::shared().acquire(&job.id,"image",job.model_bytes.saturating_add(1024*1024*1024),true,&cancel).map_err(|e|if e=="resource_cancelled"{"image_cancelled".to_string()}else{e})?;
            started = Instant::now();
            let _directory_pins=crate::gallery::directory_guards(&directory)?;
            let _runtime_locks = runtime_files(&self.runtime)?;
            let mut model = read_locked(Path::new(&job.request.model_path))?;
            if model_parts(Path::new(&job.request.model_path))?.1.len() != 0 { return Err("image_structure".into()); }
            let mut reported = Instant::now();
            let digest = sha(&mut model, Some(&cancel), |bytes| { if reported.elapsed() > Duration::from_millis(200) { let _ = self.update(&job.id, false, |j| j.hashed_bytes = bytes); reported = Instant::now(); } })?;
            if job.model_sha256.as_deref().is_some_and(|expected| expected != digest) { return Err("image_model_changed".into()); }
            self.update(&job.id, true, |j| { j.model_sha256 = Some(digest); j.hashed_bytes = j.model_bytes; j.phase = "loading".into(); })?;
            fs::write(directory.join("prompt.txt"), &job.request.prompt).map_err(|_| "image_storage")?;
            fs::write(directory.join("negative.txt"), &job.request.negative_prompt).map_err(|_| "image_storage")?;
            let output = directory.join("image.png"); let request = &job.request;
            let backend = job.device.split('\t').next().ok_or("image_gpu")?;
            let mut cmd = command(&self.runtime);
            let _reference_pins=reference::run_inputs(request,&directory,&mut cmd)?;
            cmd.args(["-m", &request.model_path, "--prompt-file"]).arg(directory.join("prompt.txt"))
                .arg("--negative-prompt-file").arg(directory.join("negative.txt"))
                .args(["-W", &request.width.to_string(), "-H", &request.height.to_string(), "--steps", &request.steps.to_string(), "--cfg-scale", &request.guidance.to_string(), "--seed", &request.seed.to_string(), "--sampling-method", &request.sampler, "--scheduler", "karras", "--rng", "cpu", "--backend", backend, "--auto-fit", "off", "--diffusion-fa", "-o"]).arg(&output)
                .stdout(Stdio::piped()).stderr(Stdio::piped());
            if cancel.load(Ordering::Relaxed) { return Err("image_cancelled".into()); }
            if request.vae_on_cpu {cmd.arg("--vae-on-cpu");}
            let mut child = cmd.spawn().map_err(|_| "image_runtime_start")?;
            let _group = match ProcessGroup::attach(&child) { Ok(group) => group, Err(error) => { let _ = child.kill(); let _ = child.wait(); return Err(error); } };
            let (sender, receiver) = std::sync::mpsc::sync_channel::<String>(256);
            let mut readers = Vec::new();
            let streams: Vec<Box<dyn Read + Send>> = vec![Box::new(child.stdout.take().unwrap()), Box::new(child.stderr.take().unwrap())];
            for mut stream in streams { let sender = sender.clone(); readers.push(std::thread::spawn(move || {
                let mut pending = Vec::new(); let mut data = [0;4096];
                while let Ok(n) = stream.read(&mut data) { if n == 0 { break; }
                    for byte in &data[..n] { if *byte == b'\n' || *byte == b'\r' { if !pending.is_empty() { let _ = sender.try_send(String::from_utf8_lossy(&pending).into()); pending.clear(); } } else if pending.len() < 8192 { pending.push(*byte); } }
                }
                if !pending.is_empty() { let _ = sender.try_send(String::from_utf8_lossy(&pending).into()); }
            })); }
            drop(sender);
            let mut terminal = None;
            let mut memory_error=false;
            let status = loop {
                for line in receiver.try_iter() {if line.contains("out of memory")||line.contains("cannot make enough memory available"){memory_error=true;} let _ = self.update(&job.id, false, |j| {
                    if line.contains("generating image:") { j.phase = "sampling".into(); }
                    if line.contains("decoding") && line.contains("latents") { j.phase = "decoding".into(); }
                    if let Some((step,total)) = sampling_progress(&line, request.steps) { j.phase = "sampling".into(); j.step = step; j.sampling_steps=Some(total); }
                    j.log_tail.push_str(&line); j.log_tail.push('\n');
                    if j.log_tail.len() > 64 * 1024 { let mut keep = j.log_tail.len() - 48 * 1024; while !j.log_tail.is_char_boundary(keep) { keep += 1; } j.log_tail.drain(..keep); }
                    j.elapsed_ms = started.elapsed().as_millis() as u64;
                }); }
                if cancel.load(Ordering::Relaxed) { terminal = Some("image_cancelled"); let _ = child.kill(); }
                else if started.elapsed() > Duration::from_secs(600) { terminal = Some("image_timeout"); let _ = child.kill(); }
                match child.try_wait() { Ok(Some(status)) => break status, Ok(None) => {}, Err(_) => { let _ = child.kill(); let _ = child.wait(); return Err("image_runtime_start".into()); } }
                std::thread::sleep(Duration::from_millis(40));
            };
            for reader in readers { let _ = reader.join(); }
            peak_memory = _group.peak_memory();
            if let Some(error) = terminal { return Err(error.into()); }
            if cancel.load(Ordering::Relaxed) { return Err("image_cancelled".into()); }
            if !status.success() { return Err(if memory_error{"image_memory"}else{"image_execution"}.into()); }
            png_bytes(&output, request.width, request.height)?;
            reference::compose(request,&directory,&output)?;
            Ok(output.to_string_lossy().into())
        })();
        self.update(&job.id, true, |j| {
            j.elapsed_ms = started.elapsed().as_millis() as u64; j.finished_at = Some(crate::database::now());
            match result { Ok(output) => { j.status = "completed".into(); j.phase = "completed".into(); j.step = j.sampling_steps.unwrap_or(j.request.steps); j.output = Some(output); }, Err(error) => { j.status = if error == "image_cancelled" { "cancelled" } else { "failed" }.into(); j.phase = j.status.clone(); j.error = Some(error); } }
        })?;
        if let Some(measured)=self.list()?.into_iter().find(|j|j.id==job.id&&j.status=="completed") {
            if let Err(error)=self.benchmarks.image(&measured,peak_memory){let _=self.update(&job.id,true,|j|{j.log_tail.push_str("\n");j.log_tail.push_str(&error);});}
        }
        if let Ok(jobs) = self.list() { if let Some(job) = jobs.iter().find(|j| j.id == job.id) { if let Ok(json) = serde_json::to_vec_pretty(job) { let _ = fs::write(directory.join("metadata.json"), json); } } }
        Ok(())
    }
    pub fn shutdown(&self) { self.stop(); let active = self.active.lock().ok().and_then(|mut a| a.take()); if let Some(a) = active { let _ = a.thread.join(); } }
    fn output(&self, id: &str) -> Result<String> {
        let job = self.list()?.into_iter().find(|j| j.id == id && j.status == "completed" && !j.discarded).ok_or("image_missing")?;
        let output = job.saved_path.clone().or(job.output.clone()).ok_or("image_missing")?;
        let path = Path::new(&output);
        if job.saved_path.is_none() && path != job_directory(&job, &self.config)?.join("image.png") { return Err("image_path".into()); }
        let bytes = png_bytes(path, job.request.width, job.request.height)?;
        Ok(format!("data:image/png;base64,{}", base64::engine::general_purpose::STANDARD.encode(bytes)))
    }
    pub(crate) fn project_source(&self,id:&str)->Result<(PathBuf,File,Vec<File>)> {
        let s=self.state.lock().map_err(|_|"image_storage")?;
        let job=s.jobs.iter().find(|j|j.id==id&&j.status=="completed"&&!j.discarded).ok_or("image_missing")?;
        let path=PathBuf::from(job.saved_path.as_ref().or(job.output.as_ref()).ok_or("image_missing")?);
        if job.saved_path.is_none()&&path!=job_directory(job,&self.config)?.join("image.png"){return Err("image_path".into());}
        let pins=crate::gallery::directory_guards(path.parent().ok_or("image_path")?)?;let file=crate::gallery::lock_file(&path)?;
        if job.saved_path.is_some()&&job.saved_binding.as_deref()!=Some(&crate::gallery::saved_binding(&path)?){return Err("image_missing".into());}
        png_bytes(&path,job.request.width,job.request.height)?;Ok((path,file,pins))
    }
    pub(crate) fn clear_gallery_duplicate(&self,id:&str)->Result<()> {
        let state=self.state.lock().map_err(|_|"image_storage")?;
        let job=state.jobs.iter().find(|j|j.id==id).ok_or("image_missing")?;
        if uuid::Uuid::parse_str(&job.id).is_err(){return Err("image_path".into());}
        let owned=job_directory(&job, &self.config)?.join("image.png");
        if let Some(output)=&job.output {
            if Path::new(output)!=owned{return Err("image_path".into());}
            if owned.exists(){crate::gallery::delete_owned(&owned)?;}
        }Ok(())
    }
    pub(crate) fn relocate_gallery(&self,id:&str,from:&Path,to:Option<&Path>)->Result<()> {
        let mut state=self.state.lock().map_err(|_|"image_storage")?;
        let mut job=state.jobs.iter().find(|j|j.id==id).cloned().ok_or("image_missing")?;
        let normalized=|p:&Path|p.to_string_lossy().trim_start_matches(r"\\?\").replace('/',"\\").to_lowercase();
        if to.is_none()&&job.discarded&&job.saved_path.is_none(){return Ok(());}
        if to.is_some_and(|p|job.saved_path.as_ref().is_some_and(|s|normalized(Path::new(s))==normalized(p))){return Ok(());}
        if !job.saved_path.as_ref().is_some_and(|s|normalized(Path::new(s))==normalized(from)){return Err("gallery_changed".into());}
        if let Some(to)=to {job.saved_path=Some(to.to_string_lossy().into());}else{
            if uuid::Uuid::parse_str(&job.id).is_err(){return Err("image_path".into());}
            let owned=job_directory(&job, &self.config)?.join("image.png");
            if let Some(output)=&job.output {
                if normalized(Path::new(output))!=normalized(&owned){return Err("image_path".into());}
                if owned.exists(){crate::gallery::delete_owned(&owned)?;}
            }
            job.saved_path=None;job.output=None;job.discarded=true;
        }
        persist(&state,&job)?;if let Some(current)=state.jobs.iter_mut().find(|j|j.id==id){*current=job;}Ok(())
    }
    fn save(&self, id: &str, gallery: &Path) -> Result<String> {
        let mut state = self.state.lock().map_err(|_| "image_storage")?;
        let mut job = state.jobs.iter().find(|j| j.id == id && j.status == "completed" && !j.discarded).cloned().ok_or("image_missing")?;
        if let Some(saved) = &job.saved_path { return Ok(saved.clone()); }
        model_library::no_links(gallery).map_err(|_| "image_path")?;
        let output = job.output.as_ref().ok_or("image_missing")?;
        if Path::new(output) != job_directory(&job, &self.config)?.join("image.png") { return Err("image_path".into()); }
        let bytes = png_bytes(Path::new(output), job.request.width, job.request.height)?;
        let destination = gallery.join(format!("Local-Studio-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&destination).map_err(|_| "image_storage")?;
        let image = destination.join("image.png");
        let mut file = OpenOptions::new().create_new(true).write(true).open(&image).map_err(|_| "image_storage")?;
        file.write_all(&bytes).map_err(|_| "image_storage")?; file.sync_all().map_err(|_| "image_storage")?;
        drop(file);if job.restricted{crate::privacy::mark(&image)?;}job.saved_binding=Some(crate::gallery::saved_binding(&image)?);
        job.saved_path = Some(image.to_string_lossy().into());
        let mut metadata = OpenOptions::new().create_new(true).write(true).open(destination.join("metadata.json")).map_err(|_| "image_storage")?;
        metadata.write_all(&serde_json::to_vec_pretty(&job).map_err(|_| "image_storage")?).map_err(|_| "image_storage")?;
        metadata.sync_all().map_err(|_| "image_storage")?;
        persist(&state, &job)?;
        if let Some(existing) = state.jobs.iter_mut().find(|j| j.id == id) { *existing = job.clone(); }
        Ok(job.saved_path.unwrap())
    }
}
#[tauri::command]
pub async fn image_probe(path: String, state: tauri::State<'_, Arc<ImageEngine>>) -> Result<ImageProbe> {let privacy_epoch=crate::privacy::epoch();let privacy_result=(async { let engine = state.inner().clone(); tauri::async_runtime::spawn_blocking(move || engine.probe(&path)).await.map_err(|_| "image_storage".into()) }).await;crate::privacy::finish(privacy_epoch,privacy_result)}
#[tauri::command]
pub fn image_jobs(state: tauri::State<'_, Arc<ImageEngine>>) -> Result<Vec<ImageJob>> {let privacy_epoch=crate::privacy::epoch();let privacy_result=(||{ let mut jobs=state.list()?;if crate::privacy::locked(){for j in &mut jobs{if j.restricted||crate::privacy::request(&j.request){j.locked=true;crate::privacy::redact(&mut j.request);j.output=None;j.saved_path=None;j.saved_binding=None;j.working_directory=None;j.log_tail.clear();j.error=None;}}}Ok(jobs) })();crate::privacy::finish(privacy_epoch,privacy_result)}
#[tauri::command]
pub async fn image_generate(request: ImageRequest, core: tauri::State<'_, Arc<Core>>, state: tauri::State<'_, Arc<ImageEngine>>) -> Result<ImageJob> {let privacy_epoch=crate::privacy::epoch();let privacy_result=(async { let temporary = core.storage_paths()?.temporary; let engine = state.inner().clone(); tauri::async_runtime::spawn_blocking(move || engine.start_in(request, Some(Path::new(&temporary)))).await.map_err(|_| "image_storage")? }).await;crate::privacy::finish(privacy_epoch,privacy_result)}
#[tauri::command]
pub fn image_cancel(id: String, state: tauri::State<'_, Arc<ImageEngine>>) -> Result<()> { state.cancel(&id) }
#[tauri::command]
pub async fn image_output(id: String, state: tauri::State<'_, Arc<ImageEngine>>) -> Result<String> {let privacy_epoch=crate::privacy::epoch();let privacy_result=(async { let engine = state.inner().clone(); tauri::async_runtime::spawn_blocking(move || engine.output(&id)).await.map_err(|_| "image_storage")? }).await;crate::privacy::finish(privacy_epoch,privacy_result)}
#[tauri::command]
pub async fn image_save(id: String, core: tauri::State<'_, Arc<Core>>, state: tauri::State<'_, Arc<ImageEngine>>) -> Result<String> {let privacy_epoch=crate::privacy::epoch();let privacy_result=(async { let gallery = core.storage_paths()?.gallery; let engine = state.inner().clone(); tauri::async_runtime::spawn_blocking(move || engine.save(&id, Path::new(&gallery))).await.map_err(|_| "image_storage")? }).await;crate::privacy::finish(privacy_epoch,privacy_result)}

#[cfg(test)]
mod tests {
    use super::*;
    pub(super) fn request() -> ImageRequest { ImageRequest { vae_on_cpu:false,  reference:None, model_path: "model".into(), prompt: "test".into(), negative_prompt: String::new(), width: 512, height: 512, steps: 20, guidance: 5., seed: 42, sampler: "euler".into() } }
    #[test] fn rejects_invalid_or_unbounded_parameters() { let mut r = request(); assert!(validate(&r).is_ok()); r.width=4096; assert!(validate(&r).is_err()); r.width=512; r.guidance=f32::NAN; assert!(validate(&r).is_err()); r.guidance=5.; r.sampler="--rpc-servers".into(); assert!(validate(&r).is_err()); }
    #[test] fn progress_uses_denoiser_steps_not_weight_loading() { assert_eq!(sampling_progress("|====>| 2/20 - 3.71it/s",20),Some((2,20))); assert_eq!(sampling_progress("|####| 20/20 - 2.00GB/s",20),None); assert_eq!(sampling_progress("|====>| 2/40 - 1.2s/it",20),None); assert_eq!(sampling_progress("|====>| 3/5 - 1.2s/it",6),Some((3,5))); }
    #[test] fn missing_runtime_never_claims_ready() { let dir=tempfile::tempdir().unwrap(); assert!(runtime_files(dir.path()).is_err()); }
}

#[cfg(test)]
#[path = "image_engine_tests.rs"]
mod lifecycle_tests;
