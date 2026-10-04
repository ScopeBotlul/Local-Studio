//! Video and GIF use the same isolated Wan generators. This adapter only adds
//! typed video jobs, cancellation, verified FFmpeg encoding and local results.
use crate::{
    comfy::Comfy,
    core::Core,
    gallery,
    gif_studio::GifAiRequest,
    image_engine::{ImageEngine, NativeVideoRequest, ProcessGroup},
    model_library,
    projects::VideoEngine,
};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant, UNIX_EPOCH},
};
use tauri::State;
type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VideoGenerationRequest {
    engine: String,
    mode: String,
    source_path: Option<String>,
    model_path: String,
    encoder_path: Option<String>,
    vae_path: Option<String>,
    prompt: String,
    negative_prompt: String,
    width: u32,
    height: u32,
    frames: u32,
    fps: u32,
    steps: u32,
    guidance: f32,
    seed: u32,
}
impl VideoGenerationRequest {
    fn validate(&self) -> Result<()> {
        if !matches!(self.engine.as_str(), "comfy" | "vulkan")
            || !matches!(self.mode.as_str(), "image" | "prompt")
            || !(1..=50).contains(&self.fps)
            || !(128..=2048).contains(&self.width)
            || self.width % 32 != 0
            || !(128..=2048).contains(&self.height)
            || self.height % 32 != 0
            || !(5..=81).contains(&self.frames)
            || (self.frames - 1) % 4 != 0
            || !(1..=50).contains(&self.steps)
            || !self.guidance.is_finite()
            || !(0.0..=20.0).contains(&self.guidance)
            || self.prompt.len() > 8000
            || self.negative_prompt.len() > 8000
        {
            return Err("video_generation_parameters".into());
        }
        if self.mode == "image"
            && self
                .source_path
                .as_deref()
                .is_none_or(|p| p.trim().is_empty())
        {
            return Err("gif_ai_source".into());
        }
        if self.mode == "prompt" && (self.prompt.trim().is_empty() || self.source_path.is_some()) {
            return Err("gif_ai_prompt".into());
        }
        Ok(())
    }
    fn wan(&self) -> GifAiRequest {
        GifAiRequest {
            source_path: self.source_path.clone(),
            model_path: self.model_path.clone(),
            prompt: self.prompt.clone(),
            negative_prompt: self.negative_prompt.clone(),
            width: self.width,
            height: self.height,
            frames: self.frames,
            steps: self.steps,
            guidance: self.guidance,
            seed: self.seed,
            delay_ms: 1000 / self.fps,
            looped: false,
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoGenerationStatus {
    id: String,
    phase: String,
    error: Option<String>,
    elapsed_seconds: u64,
}
#[derive(Default)]
pub struct VideoGeneration {
    state: Mutex<Option<(VideoGenerationStatus, Instant)>>,
    cancel: AtomicBool,
    database: Option<Mutex<rusqlite::Connection>>,
}
impl VideoGeneration {
    pub fn new(config: &Path) -> Result<Arc<Self>> {
        let db = rusqlite::Connection::open(config.join("video-generation.sqlite3"))
            .map_err(|_| "video_generation_storage")?;
        db.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE IF NOT EXISTS video_generation_jobs(id TEXT PRIMARY KEY, request TEXT NOT NULL, status TEXT NOT NULL);").map_err(|_| "video_generation_storage")?;
        let saved: Option<String> = db
            .query_row(
                "SELECT status FROM video_generation_jobs ORDER BY rowid DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .ok();
        let state = saved
            .and_then(|text| serde_json::from_str::<VideoGenerationStatus>(&text).ok())
            .map(|mut job| {
                if matches!(job.phase.as_str(), "generating" | "encoding" | "cancelling") {
                    job.phase = "interrupted".into();
                    job.error = Some("video_generation_interrupted".into());
                }
                (job, Instant::now())
            });
        Ok(Arc::new(Self {
            state: Mutex::new(state),
            cancel: AtomicBool::new(false),
            database: Some(Mutex::new(db)),
        }))
    }
    fn record(&self, job: &VideoGenerationStatus, request: &VideoGenerationRequest) -> Result<()> {
        if let Some(database) = self.database.as_ref() {
            database
                .lock()
                .map_err(|_| "video_generation_storage")?
                .execute(
                    "INSERT INTO video_generation_jobs VALUES(?1,?2,?3)",
                    rusqlite::params![
                        job.id,
                        serde_json::to_string(request).map_err(|_| "video_generation_storage")?,
                        serde_json::to_string(job).map_err(|_| "video_generation_storage")?
                    ],
                )
                .map_err(|_| "video_generation_storage")?;
        }
        Ok(())
    }
    pub fn stop(&self) {
        self.cancel.store(true, Ordering::SeqCst);
    }
    fn phase(&self, phase: &str, error: Option<String>) {
        if let Ok(mut state) = self.state.lock() {
            if let Some((job, started)) = state.as_mut() {
                job.phase = phase.into();
                job.error = error;
                job.elapsed_seconds = started.elapsed().as_secs();
                if let Some(database) = self.database.as_ref() {
                    if let Ok(db) = database.lock() {
                        let _ = db.execute(
                            "UPDATE video_generation_jobs SET status=?1 WHERE id=?2",
                            rusqlite::params![
                                serde_json::to_string(job).unwrap_or_default(),
                                job.id
                            ],
                        );
                    }
                }
            }
        }
    }
    fn status(&self) -> Result<Option<VideoGenerationStatus>> {
        Ok(self
            .state
            .lock()
            .map_err(|_| "video_generation_storage")?
            .as_ref()
            .map(|(job, start)| {
                let mut job = job.clone();
                if matches!(job.phase.as_str(), "generating" | "encoding" | "cancelling") {
                    job.elapsed_seconds = start.elapsed().as_secs();
                }
                job
            }))
    }
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoPending {
    id: String,
    bytes: u64,
    created_at: u64,
    request: Option<VideoGenerationRequest>,
}
fn root(temporary: &Path) -> Result<PathBuf> {
    let root = temporary.join("video-results");
    fs::create_dir_all(&root).map_err(|_| "video_generation_storage")?;
    model_library::no_links(&root)?;
    fs::canonicalize(root).map_err(|_| "video_generation_storage".into())
}
fn path(root: &Path, id: &str) -> Result<PathBuf> {
    if uuid::Uuid::parse_str(id)
        .map_err(|_| "video_generation_missing")?
        .to_string()
        != id
    {
        return Err("video_generation_missing".into());
    }
    model_library::no_links(root)?;
    let path = root.join(format!("{id}.mp4"));
    if path.exists() {
        model_library::no_links(&path)?;
    }
    Ok(path)
}
fn info(root: &Path, id: String) -> Result<VideoPending> {
    let file = gallery::lock_file(&path(root, &id)?)?;
    let metadata = file.metadata().map_err(|_| "video_generation_missing")?;
    let request = read_request(&root.join(format!("{id}.json"))).ok();
    Ok(VideoPending {
        id,
        bytes: metadata.len(),
        request,
        created_at: metadata
            .modified()
            .ok()
            .and_then(|v| v.duration_since(UNIX_EPOCH).ok())
            .map_or(0, |v| v.as_millis() as u64),
    })
}
fn read_request(path: &Path) -> Result<VideoGenerationRequest> {
    let file = gallery::lock_file(path)?;
    if file
        .metadata()
        .map_err(|_| "video_generation_storage")?
        .len()
        > 32 * 1024
    {
        return Err("video_generation_storage".into());
    }
    serde_json::from_reader(file).map_err(|_| "video_generation_storage".into())
}
fn list(root: &Path) -> Result<Vec<VideoPending>> {
    let mut items = vec![];
    for entry in fs::read_dir(root)
        .map_err(|_| "video_generation_storage")?
        .flatten()
        .take(1000)
    {
        if entry.path().extension().and_then(|p| p.to_str()) != Some("mp4") {
            continue;
        }
        if let Some(id) = entry.path().file_stem().and_then(|p| p.to_str()) {
            if let Ok(item) = info(root, id.into()) {
                items.push(item);
            }
        }
    }
    items.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    items.truncate(100);
    Ok(items)
}

fn encode(
    runtime: &Path,
    frames: &[String],
    directory: &Path,
    output: &Path,
    request: &VideoGenerationRequest,
    cancel: &AtomicBool,
) -> Result<()> {
    if frames.len() != request.frames as usize {
        return Err("gif_vulkan_output".into());
    }
    let mut locks = Vec::with_capacity(frames.len());
    for (index, source) in frames.iter().enumerate() {
        let mut file = gallery::lock_file(Path::new(source))?;
        let image = image::ImageReader::new(std::io::BufReader::new(
            file.try_clone().map_err(|_| "video_generation_encode")?,
        ))
        .with_guessed_format()
        .map_err(|_| "video_generation_encode")?
        .into_dimensions()
        .map_err(|_| "video_generation_encode")?;
        if image != (request.width, request.height) {
            return Err("video_generation_dimensions".into());
        }
        use std::io::Seek;
        file.rewind().map_err(|_| "video_generation_encode")?;
        let mut destination = File::options()
            .write(true)
            .create_new(true)
            .open(directory.join(format!("frame-{index:03}.png")))
            .map_err(|_| "video_generation_encode")?;
        std::io::copy(&mut file, &mut destination).map_err(|_| "video_generation_encode")?;
        locks.push(file);
    }
    let log = File::options()
        .write(true)
        .create_new(true)
        .open(directory.join("encode.log"))
        .map_err(|_| "video_generation_encode")?;
    let mut command = Command::new(runtime.join("ffmpeg.exe"));
    command
        .creation_flags(0x08000000)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(log));
    command
        .args([
            "-hide_banner",
            "-nostdin",
            "-v",
            "error",
            "-n",
            "-protocol_whitelist",
            "file,pipe",
            "-framerate",
        ])
        .arg(request.fps.to_string())
        .args(["-start_number", "0", "-i"])
        .arg(directory.join("frame-%03d.png"))
        .args(["-frames:v"])
        .arg(request.frames.to_string())
        .args(["-an", "-map_metadata", "-1", "-metadata"])
        .arg(format!(
            "comment={}",
            serde_json::to_string(request).map_err(|_| "video_generation_storage")?
        ))
        .args([
            "-c:v",
            "libopenh264",
            "-b:v",
            "8M",
            "-pix_fmt",
            "yuv420p",
            "-movflags",
            "+faststart",
            "-threads",
            "2",
            "-f",
            "mp4",
        ])
        .arg(output);
    let mut child = command.spawn().map_err(|_| "video_runtime")?;
    let _group = match ProcessGroup::attach(&child) {
        Ok(group) => group,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error);
        }
    };
    loop {
        if cancel.load(Ordering::SeqCst) {
            let _ = child.kill();
            let _ = child.wait();
            return Err("video_generation_cancelled".into());
        }
        if let Some(status) = child.try_wait().map_err(|_| "video_generation_encode")? {
            if !status.success() {
                return Err("video_generation_encode".into());
            }
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let probe = crate::projects::creative::video::probe(runtime, output)?;
    if !probe.video
        || probe.width != request.width
        || probe.height != request.height
        || (probe.duration - f64::from(request.frames) / f64::from(request.fps)).abs() > 0.15
    {
        return Err("video_generation_encode".into());
    }
    Ok(())
}

#[tauri::command]
pub async fn video_generation_create(
    request: VideoGenerationRequest,
    core: State<'_, Arc<Core>>,
    comfy: State<'_, Arc<Comfy>>,
    images: State<'_, Arc<ImageEngine>>,
    video: State<'_, Arc<VideoEngine>>,
    generation: State<'_, Arc<VideoGeneration>>,
) -> Result<VideoPending> {
    request.validate()?;
    let temporary = PathBuf::from(core.storage_paths()?.temporary);
    let root = root(&temporary)?;
    let comfy = comfy.inner().clone();
    let images = images.inner().clone();
    let video = video.inner().clone();
    let generation = generation.inner().clone();
    let id = uuid::Uuid::new_v4().to_string();
    {
        let mut state = generation
            .state
            .lock()
            .map_err(|_| "video_generation_storage")?;
        if state.as_ref().is_some_and(|(j, _)| {
            matches!(j.phase.as_str(), "generating" | "encoding" | "cancelling")
        }) {
            return Err("video_generation_busy".into());
        }
        generation.cancel.store(false, Ordering::SeqCst);
        let job = VideoGenerationStatus {
            id: id.clone(),
            phase: "generating".into(),
            error: None,
            elapsed_seconds: 0,
        };
        generation.record(&job, &request)?;
        *state = Some((job, Instant::now()));
    }
    let tracker = generation.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let directory = root.join(format!(".work-{id}"));
        let staging = root.join(format!(".{id}.tmp"));
        let output = path(&root, &id)?;
        let result = (|| {
            fs::create_dir(&directory).map_err(|_| "video_generation_storage")?;
            let directory = fs::canonicalize(&directory).map_err(|_| "video_generation_storage")?;
            if !directory.starts_with(&root) {
                return Err("video_generation_storage".into());
            }
            let _pins = gallery::directory_guards(&directory)?;
            let (runtime, _runtime) = video.runtime()?;
            let frames = if request.engine == "comfy" {
                comfy.wan_frames(request.wan(), &generation.cancel)?
            } else {
                images.generate_vulkan_video_cancellable(
                    NativeVideoRequest {
                        source_path: request.source_path.clone(),
                        model_path: request.model_path.clone(),
                        encoder_path: request.encoder_path.clone().ok_or("gif_vulkan_encoder")?,
                        vae_path: request.vae_path.clone().ok_or("gif_vulkan_vae")?,
                        prompt: request.prompt.clone(),
                        negative_prompt: request.negative_prompt.clone(),
                        width: request.width,
                        height: request.height,
                        frames: request.frames,
                        steps: request.steps,
                        guidance: request.guidance,
                        seed: request.seed,
                        fps: request.fps,
                    },
                    &directory,
                    &generation.cancel,
                )?
            };
            generation.phase("encoding", None);
            let encoded = encode(
                &runtime,
                &frames,
                &directory,
                &staging,
                &request,
                &generation.cancel,
            );
            // Only the ComfyUI adapter returns files outside our private work directory.
            if request.engine == "comfy" {
                for frame in frames {
                    let _ = fs::remove_file(frame);
                }
            }
            encoded?;
            if generation.cancel.load(Ordering::SeqCst) {
                return Err("video_generation_cancelled".into());
            }
            let metadata = root.join(format!("{id}.json"));
            let mut file = File::options()
                .write(true)
                .create_new(true)
                .open(&metadata)
                .map_err(|_| "video_generation_storage")?;
            use std::io::Write;
            file.write_all(&serde_json::to_vec(&request).map_err(|_| "video_generation_storage")?)
                .map_err(|_| "video_generation_storage")?;
            file.sync_all().map_err(|_| "video_generation_storage")?;
            drop(file);
            gallery::publish(&staging, &output).map_err(|_| "gallery_storage")?;
            info(&root, id.clone())
        })();
        if directory.starts_with(&root) && directory != root {
            let _ = fs::remove_dir_all(&directory);
        }
        if result.is_err() {
            let _ = fs::remove_file(&staging);
            let _ = fs::remove_file(root.join(format!("{id}.json")));
        }
        generation.phase(
            if result.is_ok() {
                "done"
            } else if generation.cancel.load(Ordering::SeqCst) {
                "cancelled"
            } else {
                "failed"
            },
            result.as_ref().err().cloned(),
        );
        result
    })
    .await;
    match result {
        Ok(result) => result,
        Err(_) => {
            tracker.phase("failed", Some("video_generation_storage".into()));
            Err("video_generation_storage".into())
        }
    }
}
#[tauri::command]
pub fn video_generation_status(
    generation: State<'_, Arc<VideoGeneration>>,
) -> Result<Option<VideoGenerationStatus>> {
    generation.status()
}
#[tauri::command]
pub fn video_generation_cancel(generation: State<'_, Arc<VideoGeneration>>) -> Result<()> {
    let mut state = generation
        .state
        .lock()
        .map_err(|_| "video_generation_storage")?;
    if let Some((job, _)) = state.as_mut() {
        if matches!(job.phase.as_str(), "generating" | "encoding" | "cancelling") {
            generation.stop();
            job.phase = "cancelling".into();
        }
    }
    Ok(())
}
#[tauri::command]
pub async fn video_pending_list(core: State<'_, Arc<Core>>) -> Result<Vec<VideoPending>> {
    let root = root(Path::new(&core.storage_paths()?.temporary))?;
    tauri::async_runtime::spawn_blocking(move || list(&root))
        .await
        .map_err(|_| "video_generation_storage")?
}
#[tauri::command]
pub async fn video_pending_preview(id: String, core: State<'_, Arc<Core>>) -> Result<String> {
    let root = root(Path::new(&core.storage_paths()?.temporary))?;
    tauri::async_runtime::spawn_blocking(move || {
        let mut file = gallery::lock_file(&path(&root, &id)?)?;
        if file
            .metadata()
            .map_err(|_| "video_generation_missing")?
            .len()
            > 128 * 1024 * 1024
        {
            return Err("video_generation_preview_large".into());
        }
        use std::io::Read;
        let mut bytes = vec![];
        file.read_to_end(&mut bytes)
            .map_err(|_| "video_generation_missing")?;
        Ok(format!(
            "data:video/mp4;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(bytes)
        ))
    })
    .await
    .map_err(|_| "video_generation_storage")?
}
#[tauri::command]
pub async fn video_pending_save(
    id: String,
    folder: String,
    core: State<'_, Arc<Core>>,
) -> Result<String> {
    let root = root(Path::new(&core.storage_paths()?.temporary))?;
    let gallery_root = gallery::root(&core)?;
    tauri::async_runtime::spawn_blocking(move || {
        let source = path(&root, &id)?;
        let mut input = gallery::lock_file(&source)?;
        let destination = if folder.is_empty() {
            gallery_root
        } else {
            gallery::resolve(&gallery_root, &folder)?
        };
        let _pins = gallery::directory_guards(&destination)?;
        let output = destination.join(format!("Local-Studio-{id}.mp4"));
        let staging = destination.join(format!(".local-studio-{id}.tmp"));
        let result = (|| {
            let mut file = File::options()
                .write(true)
                .create_new(true)
                .open(&staging)
                .map_err(|_| "gallery_storage")?;
            std::io::copy(&mut input, &mut file).map_err(|_| "gallery_storage")?;
            file.sync_all().map_err(|_| "gallery_storage")?;
            drop(file);
            let metadata = root.join(format!("{id}.json"));
            gallery::publish(&staging, &output).map_err(|_| "gallery_storage")?;
            drop(input);
            let _ = fs::remove_file(source);
            let _ = fs::remove_file(metadata);
            Ok(output.to_string_lossy().into_owned())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&staging);
        }
        result
    })
    .await
    .map_err(|_| "video_generation_storage")?
}
#[tauri::command]
pub async fn video_result_info(
    path: String,
    core: State<'_, Arc<Core>>,
    video: State<'_, Arc<VideoEngine>>,
) -> Result<Option<VideoGenerationRequest>> {
    let root = gallery::root(&core)?;
    let video = video.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let source = gallery::resolve(&root, &path)?;
        if source.extension().and_then(|s| s.to_str()) != Some("mp4") {
            return Ok(None);
        }
        let _file = gallery::lock_file(&source)?;
        let (runtime, _pins) = video.runtime()?;
        result_request(&runtime, &source)
    })
    .await
    .map_err(|_| "video_generation_storage")?
}
fn result_request(runtime: &Path, source: &Path) -> Result<Option<VideoGenerationRequest>> {
    let mut command = Command::new(runtime.join("ffprobe.exe"));
    command.creation_flags(0x08000000).stdin(Stdio::null());
    command
        .args([
            "-v",
            "error",
            "-protocol_whitelist",
            "file,pipe",
            "-format_whitelist",
            "mov",
            "-show_entries",
            "format_tags=comment",
            "-of",
            "json",
        ])
        .arg(source);
    let bytes = crate::projects::creative::video::capture(command)?;
    let json: serde_json::Value =
        serde_json::from_slice(&bytes).map_err(|_| "video_generation_storage")?;
    Ok(json
        .pointer("/format/tags/comment")
        .and_then(|v| v.as_str())
        .filter(|text| text.len() <= 32768)
        .and_then(|text| serde_json::from_str(text).ok()))
}
#[tauri::command]
pub async fn video_pending_discard(id: String, core: State<'_, Arc<Core>>) -> Result<()> {
    let root = root(Path::new(&core.storage_paths()?.temporary))?;
    tauri::async_runtime::spawn_blocking(move || {
        fs::remove_file(path(&root, &id)?).map_err(|_| "video_generation_missing")?;
        let _ = fs::remove_file(root.join(format!("{id}.json")));
        Ok(())
    })
    .await
    .map_err(|_| "video_generation_storage")?
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request() -> VideoGenerationRequest {
        VideoGenerationRequest {
            engine: "comfy".into(),
            mode: "prompt".into(),
            source_path: None,
            model_path: "wan2.1_t2v.safetensors".into(),
            encoder_path: None,
            vae_path: None,
            prompt: "a red square moving right".into(),
            negative_prompt: "".into(),
            width: 128,
            height: 128,
            frames: 5,
            fps: 10,
            steps: 1,
            guidance: 6.0,
            seed: 42,
        }
    }
    #[test]
    fn validates_explicit_modes_and_video_limits() {
        let mut r = request();
        assert!(r.validate().is_ok());
        r.mode = "image".into();
        assert_eq!(r.validate().unwrap_err(), "gif_ai_source");
        r.source_path = Some("source.png".into());
        r.prompt.clear();
        assert!(r.validate().is_ok());
        r.mode = "prompt".into();
        assert_eq!(r.validate().unwrap_err(), "gif_ai_prompt");
        r = request();
        r.fps = 0;
        assert!(r.validate().is_err());
        r = request();
        r.width = 144;
        assert!(r.validate().is_err());
        r = request();
        r.frames = 6;
        assert!(r.validate().is_err());
        r = request();
        r.guidance = f32::NAN;
        assert!(r.validate().is_err());
    }
    #[test]
    fn pending_ids_cannot_escape_and_cancel_is_local() {
        let temp = tempfile::tempdir().unwrap();
        let root = root(temp.path()).unwrap();
        assert!(path(&root, "../outside").is_err());
        assert!(path(&root, "/etc/passwd").is_err());
        let state = VideoGeneration::default();
        state.stop();
        assert!(state.cancel.load(Ordering::SeqCst));
        assert!(state.status().unwrap().is_none());
        assert!(list(&root).unwrap().is_empty());
    }
    #[test]
    fn interrupted_jobs_are_recovered_from_local_sqlite() {
        let temp = tempfile::tempdir().unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        {
            let state = VideoGeneration::new(temp.path()).unwrap();
            let job = VideoGenerationStatus {
                id: id.clone(),
                phase: "generating".into(),
                error: None,
                elapsed_seconds: 0,
            };
            state.record(&job, &request()).unwrap();
        }
        let state = VideoGeneration::new(temp.path()).unwrap();
        let job = state.status().unwrap().unwrap();
        assert_eq!(job.id, id);
        assert_eq!(job.phase, "interrupted");
        assert_eq!(job.error.as_deref(), Some("video_generation_interrupted"));
    }
    #[test]
    fn encodes_real_mp4_with_exact_duration_and_portable_metadata() {
        let temp = tempfile::tempdir().unwrap();
        let config = temp.path().join("config");
        fs::create_dir(&config).unwrap();
        let runtime = fs::canonicalize(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.tools/video-runtime"),
        )
        .unwrap();
        let engine = VideoEngine::new(&config, runtime).unwrap();
        let (runtime, _pins) = engine.runtime().unwrap();
        let frames = (0..5)
            .map(|index| {
                let path = temp.path().join(format!("source-{index}.png"));
                image::RgbaImage::from_pixel(128, 128, image::Rgba([index * 40, 100, 50, 255]))
                    .save(&path)
                    .unwrap();
                path.to_string_lossy().into_owned()
            })
            .collect::<Vec<_>>();
        let work = temp.path().join("work");
        fs::create_dir(&work).unwrap();
        let r = request();
        let output = temp.path().join("output.mp4");
        let encoded = encode(
            &runtime,
            &frames,
            &work,
            &output,
            &r,
            &AtomicBool::new(false),
        );
        assert!(
            encoded.is_ok(),
            "{encoded:?}: {}",
            fs::read_to_string(work.join("encode.log")).unwrap_or_default()
        );
        let probe = crate::projects::creative::video::probe(&runtime, &output).unwrap();
        assert_eq!((probe.width, probe.height), (128, 128));
        assert!((probe.duration - 0.5).abs() < 0.01);
        let metadata = result_request(&runtime, &output).unwrap().unwrap();
        assert_eq!(metadata.seed, 42);
        assert_eq!(metadata.prompt, r.prompt);
        let renamed = temp.path().join("renamed.mp4");
        fs::rename(&output, &renamed).unwrap();
        assert_eq!(
            result_request(&runtime, &renamed).unwrap().unwrap().seed,
            42
        );
        let pending_root = root(temp.path()).unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        fs::copy(&renamed, path(&pending_root, &id).unwrap()).unwrap();
        assert_eq!(list(&pending_root).unwrap()[0].id, id);
        let work = temp.path().join("cancelled");
        fs::create_dir(&work).unwrap();
        assert_eq!(
            encode(
                &runtime,
                &frames,
                &work,
                &temp.path().join("cancelled.mp4"),
                &r,
                &AtomicBool::new(true)
            )
            .unwrap_err(),
            "video_generation_cancelled"
        );
    }
}
