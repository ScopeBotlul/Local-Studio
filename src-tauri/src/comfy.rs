use crate::{
    gif_studio::GifAiRequest,
    image_engine::{ImageLora, ImageRequest, ProcessGroup},
    model_library,
};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::VecDeque,
    fs,
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

type Result<T> = std::result::Result<T, String>;
const ENDPOINT: &str = "http://127.0.0.1:8188";
const RELEASE_API: &str = "https://api.github.com/repos/Comfy-Org/ComfyUI/releases/latest";
const MAX_ARCHIVE_BYTES: u64 = 12 * 1024 * 1024 * 1024;
const AMD_COMPAT_ARGS: [&str; 3] = [
    "--use-split-cross-attention",
    "--disable-pinned-memory",
    "--disable-async-offload",
];
const AMD_LOW_MEMORY_ARGS: [&str; 3] = ["--disable-dynamic-vram", "--lowvram", "--disable-mmap"];
const AMD_LOW_MEMORY_ENV: (&str, &str) = ("COMFY_KITCHEN_DISABLE_HIP", "1");
const QWEN_IMAGE21_ENCODER: &str = "qwen3vl_8b_int8_convrot.safetensors";
const QWEN_IMAGE21_VAE: &str = "qwen_image_2.1_vae_bf16.safetensors";

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Config {
    path: Option<String>,
    dismissed: bool,
    #[serde(default)]
    lora_paths: Vec<String>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComfyStatus {
    pub installed: bool,
    path: Option<String>,
    running: bool,
    managed: bool,
    endpoint: String,
    version: Option<String>,
    error: Option<String>,
    log: Option<String>,
    dismissed: bool,
    install: ComfyInstallStatus,
    update: ComfyUpdateStatus,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WanModel {
    path: String,
    name: String,
    bytes: u64,
}
#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComfyInstallStatus {
    phase: String,
    variant: Option<String>,
    total_bytes: u64,
    received_bytes: u64,
    bytes_per_second: u64,
    error: Option<String>,
}
#[derive(Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComfyUpdateStatus {
    phase: String,
    installed_version: Option<String>,
    latest_version: Option<String>,
    error: Option<String>,
    log: Option<String>,
}
struct Process {
    child: Child,
    _group: ProcessGroup,
}
pub struct Comfy {
    config_path: PathBuf,
    managed_root: PathBuf,
    external_checkpoints: PathBuf,
    extra_model_paths: PathBuf,
    config: Mutex<Config>,
    process: Mutex<Option<Process>>,
    start_lock: Mutex<()>,
    install: Mutex<ComfyInstallStatus>,
    update: Mutex<ComfyUpdateStatus>,
    update_log: PathBuf,
    runtime_log: PathBuf,
    runtime_error: Mutex<Option<String>>,
    active_generations: AtomicUsize,
}

struct ActiveGeneration<'a>(&'a AtomicUsize);
impl Drop for ActiveGeneration<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

fn valid_root(path: &Path) -> bool {
    path.is_absolute()
        && model_library::no_links(path).is_ok()
        && path.join("python_embeded/python.exe").is_file()
        && path.join("ComfyUI/main.py").is_file()
}
fn relative_model_path(path: &Path, root: &Path) -> Option<String> {
    let root = fs::canonicalize(root).ok()?;
    let path = fs::canonicalize(path).ok()?;
    let relative = path.strip_prefix(root).ok()?.to_str()?.replace('\\', "/");
    (!relative.is_empty()).then_some(relative)
}
fn normalized_model_name(value: &str) -> String {
    value.replace('\\', "/").to_lowercase()
}
fn resolve_comfy_option(info: &Value, node: &str, input: &str, requested: &str) -> Option<String> {
    let options = info
        .get(node)?
        .pointer(&format!("/input/required/{input}/0"))?
        .as_array()?;
    options
        .iter()
        .filter_map(Value::as_str)
        .find(|value| *value == requested)
        .or_else(|| {
            let requested = normalized_model_name(requested);
            options
                .iter()
                .filter_map(Value::as_str)
                .find(|value| normalized_model_name(value) == requested)
        })
        .map(str::to_owned)
}
fn first_comfy_option(info: &Value, node: &str, input: &str, predicate: impl Fn(&str) -> bool) -> Option<String> {
    info.get(node)?
        .pointer(&format!("/input/required/{input}/0"))?
        .as_array()?
        .iter()
        .filter_map(Value::as_str)
        .find(|value| predicate(value))
        .map(str::to_owned)
}
fn uses_amd_safe_attention(path: &Path) -> bool {
    path.join("run_amd_gpu.bat").is_file()
}
fn uses_amd_low_memory_profile(total_memory: u64) -> bool {
    total_memory <= 16 * 1024 * 1024 * 1024
}
fn likely_roots(managed_root: &Path) -> Vec<PathBuf> {
    let mut choices = Vec::new();
    choices.push(managed_root.join("ComfyUI_windows_portable"));
    for drive in b'C'..=b'Z' {
        let root = PathBuf::from(format!("{}:\\", drive as char));
        choices.push(root.join("ComfyUI_windows_portable"));
        choices.push(root.join("Local/AI/ComfyUI_windows_portable"));
        choices.push(root.join("LocalAI/ComfyUI_windows_portable"));
        choices.push(root.join("AI/ComfyUI_windows_portable"));
    }
    if let Some(user) = std::env::var_os("USERPROFILE") {
        let user = PathBuf::from(user);
        choices.push(user.join("Downloads/ComfyUI_windows_portable"));
        choices.push(user.join("Desktop/ComfyUI_windows_portable"));
        choices.push(user.join("Documents/ComfyUI_windows_portable"));
        choices.push(user.join("ComfyUI_windows_portable"));
    }
    choices
}
fn detect(managed_root: &Path, deep: bool) -> Option<PathBuf> {
    if let Some(path) = likely_roots(managed_root)
        .into_iter()
        .find(|p| valid_root(p))
    {
        return fs::canonicalize(path).ok();
    }
    if !deep {
        return None;
    }
    let mut queue = VecDeque::new();
    for root in [
        std::env::var_os("USERPROFILE").map(PathBuf::from),
        Some(managed_root.to_path_buf()),
    ]
    .into_iter()
    .flatten()
    {
        for child in ["Downloads", "Desktop", "Documents", "LocalAI", "AI", ""] {
            let path = root.join(child);
            if path.is_dir() {
                queue.push_back((path, 0usize));
            }
        }
    }
    let mut visited = 0usize;
    while let Some((path, depth)) = queue.pop_front() {
        visited += 1;
        if visited > 15_000 {
            break;
        }
        if valid_root(&path) {
            return fs::canonicalize(path).ok();
        }
        if depth >= 4 {
            continue;
        }
        let Ok(entries) = fs::read_dir(&path) else {
            continue;
        };
        for entry in entries.flatten() {
            let child = entry.path();
            let Ok(metadata) = fs::symlink_metadata(&child) else {
                continue;
            };
            if metadata.is_dir() && !metadata.file_type().is_symlink() {
                let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
                if !name.starts_with('.')
                    && !["node_modules", "$recycle.bin", "windows"].contains(&name.as_str())
                {
                    queue.push_back((child, depth + 1));
                }
            }
        }
    }
    None
}
fn client(timeout: Duration) -> Result<Client> {
    Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(1))
        .timeout(timeout)
        .build()
        .map_err(|_| "comfy_connection".into())
}
fn probe_http() -> Result<Value> {
    let response = client(Duration::from_secs(2))?
        .get(format!("{ENDPOINT}/system_stats"))
        .send()
        .map_err(|_| "comfy_connection")?;
    if !response.status().is_success() {
        return Err("comfy_connection".into());
    }
    let value: Value = response.json().map_err(|_| "comfy_response")?;
    if !value.get("system").is_some_and(Value::is_object)
        || !value.get("devices").is_some_and(Value::is_array)
    {
        return Err("comfy_response".into());
    }
    Ok(value)
}

#[derive(Deserialize)]
struct ReleaseAsset {
    name: String,
    size: u64,
    digest: Option<String>,
    browser_download_url: String,
}
#[derive(Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<ReleaseAsset>,
}

fn package_name(variant: &str) -> Result<&'static str> {
    match variant {
        "nvidia" => Ok("ComfyUI_windows_portable_nvidia.7z"),
        "nvidia_legacy" => Ok("ComfyUI_windows_portable_nvidia_cu126.7z"),
        "amd" => Ok("ComfyUI_windows_portable_amd.7z"),
        "intel" => Ok("ComfyUI_windows_portable_intel.7z"),
        _ => Err("comfy_variant".into()),
    }
}
fn parse_version(value: &str) -> Result<(u32, u32, u32)> {
    let value = value.strip_prefix('v').unwrap_or(value);
    let parts = value.split('.').collect::<Vec<_>>();
    if parts.len() != 3
        || parts.iter().any(|part| {
            part.is_empty()
                || (part.len() > 1 && part.starts_with('0'))
                || !part.bytes().all(|b| b.is_ascii_digit())
        })
    {
        return Err("comfy_version".into());
    }
    Ok((
        parts[0].parse().map_err(|_| "comfy_version")?,
        parts[1].parse().map_err(|_| "comfy_version")?,
        parts[2].parse().map_err(|_| "comfy_version")?,
    ))
}
fn version_from_root(root: &Path) -> Option<String> {
    let bytes = fs::read(root.join("ComfyUI/comfy_version.py")).ok()?;
    if bytes.len() > 4096 {
        return None;
    }
    let text = std::str::from_utf8(&bytes).ok()?;
    for line in text.lines() {
        let Some(rest) = line.trim().strip_prefix("__version__") else {
            continue;
        };
        let Some(rest) = rest.trim_start().strip_prefix('=') else {
            continue;
        };
        let value = rest.trim().trim_matches(['\'', '"']);
        if parse_version(value).is_ok() {
            return Some(value.into());
        }
    }
    None
}
fn log_tail(path: &Path) -> Option<String> {
    let mut file = fs::File::open(path).ok()?;
    let length = file.metadata().ok()?.len();
    file.seek(SeekFrom::Start(length.saturating_sub(64 * 1024)))
        .ok()?;
    let mut bytes = Vec::new();
    file.take(64 * 1024).read_to_end(&mut bytes).ok()?;
    Some(String::from_utf8_lossy(&bytes).into_owned())
}
fn release_client() -> Result<Client> {
    Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if attempt.previous().len() >= 8 {
                return attempt.error("redirect limit");
            }
            match attempt.url().host_str() {
                Some(
                    "github.com"
                    | "release-assets.githubusercontent.com"
                    | "objects.githubusercontent.com",
                ) => attempt.follow(),
                _ => attempt.stop(),
            }
        }))
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(60 * 60 * 3))
        .build()
        .map_err(|_| "comfy_download".into())
}
fn official_asset(client: &Client, variant: &str) -> Result<(String, ReleaseAsset)> {
    let release: Release = client
        .get(RELEASE_API)
        .header("User-Agent", "Local-Studio")
        .send()
        .map_err(|_| "comfy_download")?
        .error_for_status()
        .map_err(|_| "comfy_download")?
        .json()
        .map_err(|_| "comfy_download")?;
    let wanted = package_name(variant)?;
    let asset = release
        .assets
        .into_iter()
        .find(|a| a.name == wanted)
        .ok_or("comfy_package")?;
    let url = url::Url::parse(&asset.browser_download_url).map_err(|_| "comfy_package")?;
    let expected_prefix = format!("/Comfy-Org/ComfyUI/releases/download/{}/", release.tag_name);
    let digest = asset.digest.as_deref().unwrap_or_default();
    if url.scheme() != "https"
        || url.host_str() != Some("github.com")
        || !url.path().starts_with(&expected_prefix)
        || asset.size < 64 * 1024 * 1024
        || asset.size > MAX_ARCHIVE_BYTES
        || !digest.starts_with("sha256:")
        || digest.len() != 71
        || !digest[7..].bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err("comfy_package".into());
    }
    Ok((release.tag_name, asset))
}
fn safe_archive_listing(listing: &[u8]) -> Result<()> {
    // bsdtar writes its listing through the Windows console encoding. Path separators and
    // traversal markers are ASCII in every supported code page, so lossy decoding keeps the
    // security-sensitive bytes intact while allowing legitimate non-UTF-8 package names.
    let listing = String::from_utf8_lossy(listing);
    let mut count = 0usize;
    for raw in listing.lines() {
        let entry = raw.trim().trim_end_matches(['/', '\\']).replace('\\', "/");
        if entry.is_empty() {
            continue;
        }
        count += 1;
        let parts = entry.split('/').collect::<Vec<_>>();
        let official_amd_kernel = parts.len() > 8
            && parts[0] == "ComfyUI_windows_portable"
            && parts[1..7]
                == [
                    "python_embeded",
                    "Lib",
                    "site-packages",
                    "torch",
                    "lib",
                    "aotriton.images",
                ]
            && parts.last().is_some_and(|name| {
                name.ends_with(".aks2")
                    && name.matches('*').count() == 1
                    && crate::downloads::valid_file(&name.replace('*', "＊"))
            });
        if count > 250_000
            || entry.len() > 4096
            || entry.starts_with('/')
            || entry.as_bytes().get(1) == Some(&b':')
            // The portable Python environment legitimately contains complete paths longer
            // than the 220-character model-download limit. Validate every Windows component
            // separately instead of applying that limit to the complete archive member.
            || !parts.iter().enumerate().all(|(index, part)| {
                crate::downloads::valid_file(part)
                    || (official_amd_kernel && index + 1 == parts.len())
            })
        {
            return Err("comfy_archive".into());
        }
    }
    if count == 0 {
        Err("comfy_archive".into())
    } else {
        Ok(())
    }
}
fn verify_tree(root: &Path) -> Result<()> {
    let mut queue = VecDeque::from([root.to_path_buf()]);
    let mut count = 0usize;
    while let Some(path) = queue.pop_front() {
        count += 1;
        if count > 250_000 {
            return Err("comfy_archive".into());
        }
        let metadata = fs::symlink_metadata(&path).map_err(|_| "comfy_archive")?;
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if metadata.file_attributes() & 0x400 != 0 {
                return Err("comfy_archive".into());
            }
        }
        if metadata.file_type().is_symlink() {
            return Err("comfy_archive".into());
        }
        if metadata.is_dir() {
            for child in fs::read_dir(&path).map_err(|_| "comfy_archive")? {
                queue.push_back(child.map_err(|_| "comfy_archive")?.path());
            }
        } else if !metadata.is_file() {
            return Err("comfy_archive".into());
        }
    }
    Ok(())
}
fn extracted_root(stage: &Path) -> Option<PathBuf> {
    if valid_root(stage) {
        return Some(stage.to_path_buf());
    }
    fs::read_dir(stage)
        .ok()?
        .flatten()
        .map(|e| e.path())
        .find(|p| valid_root(p))
}

impl Comfy {
    pub fn new_with_checkpoints(
        config_dir: &Path,
        checkpoints: Option<PathBuf>,
    ) -> Result<Arc<Self>> {
        let config_path = config_dir.join("comfy.json");
        let extra_model_paths = config_dir.join("comfy-extra-model-paths.yaml");
        let update_log = config_dir.join("comfy-update.log");
        let runtime_log = config_dir.join("comfy-runtime.log");
        let data_root = if config_dir
            .file_name()
            .is_some_and(|name| name.eq_ignore_ascii_case("config"))
        {
            config_dir.parent().unwrap_or(config_dir).to_path_buf()
        } else {
            config_dir.join("Data")
        };
        let managed_root = data_root.join("ComfyUI");
        let external_checkpoints = checkpoints.unwrap_or_else(|| data_root.join("Models"));
        let mut config: Config = fs::read(&config_path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        if config
            .path
            .as_ref()
            .is_none_or(|p| !valid_root(Path::new(p)))
        {
            config.path = detect(&managed_root, false).map(|p| p.to_string_lossy().into());
        }
        let this = Arc::new(Self {
            config_path,
            managed_root,
            external_checkpoints,
            extra_model_paths,
            config: Mutex::new(config),
            process: Mutex::new(None),
            start_lock: Mutex::new(()),
            install: Mutex::new(ComfyInstallStatus {
                phase: "idle".into(),
                ..Default::default()
            }),
            update: Mutex::new(ComfyUpdateStatus {
                phase: "idle".into(),
                ..Default::default()
            }),
            update_log,
            runtime_log,
            runtime_error: Mutex::new(None),
            active_generations: AtomicUsize::new(0),
        });
        this.save()?;
        Ok(this)
    }
    fn write_extra_model_paths(&self) -> Result<()> {
        model_library::no_links(&self.external_checkpoints).map_err(|_| "comfy_model_path")?;
        let root = fs::canonicalize(&self.external_checkpoints)
            .map_err(|_| "comfy_model_path")?
            .to_string_lossy()
            .trim_start_matches(r"\\?\")
            .to_string();
        if root.chars().any(char::is_control) {
            return Err("comfy_model_path".into());
        }
        let root = root.replace('\'', "''");
        let lora_paths = self.configured_lora_paths()?;
        let mut yaml = format!("local_studio:\n  checkpoints: '{root}'\n");
        for (index, lora_path) in lora_paths.iter().enumerate() {
            let lora_path = lora_path
                .to_string_lossy()
                .trim_start_matches(r"\\?\")
                .replace('\'', "''");
            yaml.push_str(&format!(
                "local_studio_loras_{index}:\n  base_path: '{lora_path}'\n  loras: '.'\n"
            ));
        }
        fs::write(&self.extra_model_paths, yaml).map_err(|_| "comfy_storage".into())
    }
    fn configured_lora_paths(&self) -> Result<Vec<PathBuf>> {
        let configured = self
            .config
            .lock()
            .map_err(|_| "comfy_storage")?
            .lora_paths
            .clone();
        let mut paths = Vec::new();
        for value in configured {
            let candidate = PathBuf::from(value);
            // A disconnected removable drive or an outdated entry must not stop
            // ComfyUI from starting. Only existing, safe directories are exposed.
            if !candidate.is_dir() || model_library::no_links(&candidate).is_err() {
                continue;
            }
            let canonical = match fs::canonicalize(candidate) {
                Ok(path) => path,
                Err(_) => continue,
            };
            if !paths.iter().any(|path: &PathBuf| path == &canonical) {
                paths.push(canonical);
            }
        }
        Ok(paths)
    }
    fn register_lora_parents(&self, loras: &[ImageLora]) -> Result<bool> {
        let mut added = Vec::new();
        for lora in loras {
            let parent = Path::new(&lora.path).parent().ok_or("image_lora_path")?;
            if !parent.is_dir() {
                return Err("image_lora_path".into());
            }
            model_library::no_links(parent).map_err(|_| "image_lora_path")?;
            let canonical = fs::canonicalize(parent).map_err(|_| "image_lora_path")?;
            if !added.iter().any(|path: &PathBuf| path == &canonical) {
                added.push(canonical);
            }
        }
        if added.is_empty() {
            return Ok(false);
        }
        let mut changed = false;
        {
            let mut config = self.config.lock().map_err(|_| "comfy_storage")?;
            for path in added {
                let value = path.to_string_lossy().into_owned();
                if !config.lora_paths.iter().any(|known| {
                    fs::canonicalize(known)
                        .ok()
                        .is_some_and(|known| known == path)
                }) {
                    config.lora_paths.push(value);
                    changed = true;
                }
            }
        }
        if changed {
            self.save()?;
            self.write_extra_model_paths()?;
        }
        Ok(changed)
    }
    fn reload_after_lora_registration(&self) -> Result<()> {
        let managed = self
            .process
            .lock()
            .ok()
            .is_some_and(|process| process.is_some());
        if probe_http().is_ok() && !managed {
            // We must not stop a ComfyUI instance that Local Studio does not own.
            return Err("comfy_lora_restart".into());
        }
        if managed {
            self.stop();
        }
        self.start().map(|_| ())
    }
    fn save(&self) -> Result<()> {
        let data = serde_json::to_vec_pretty(&*self.config.lock().map_err(|_| "comfy_storage")?)
            .map_err(|_| "comfy_storage")?;
        fs::write(&self.config_path, data).map_err(|_| "comfy_storage".into())
    }
    fn reap_runtime_exit(&self) -> bool {
        let mut exited = false;
        if let Ok(mut process) = self.process.lock() {
            if process
                .as_mut()
                .is_some_and(|p| p.child.try_wait().ok().flatten().is_some())
            {
                *process = None;
                exited = true;
            }
        }
        if exited {
            if let Ok(mut error) = self.runtime_error.lock() {
                *error = Some("comfy_runtime_exit".into());
            }
        }
        exited
            || self
                .runtime_error
                .lock()
                .ok()
                .is_some_and(|error| error.as_deref() == Some("comfy_runtime_exit"))
    }
    pub fn status(&self) -> ComfyStatus {
        self.reap_runtime_exit();
        let managed = self.process.lock().ok().is_some_and(|p| p.is_some());
        let response = probe_http();
        let running = response.is_ok();
        let config = self.config.lock().map(|c| c.clone()).unwrap_or_default();
        let installed = config
            .path
            .as_ref()
            .is_some_and(|p| valid_root(Path::new(p)));
        let version = response
            .as_ref()
            .ok()
            .and_then(|v| {
                v.pointer("/system/comfyui_version")
                    .or_else(|| v.get("comfyui_version"))
            })
            .and_then(Value::as_str)
            .map(str::to_owned)
            .or_else(|| {
                config
                    .path
                    .as_ref()
                    .and_then(|p| version_from_root(Path::new(p)))
            });
        let mut update = self.update.lock().map(|s| s.clone()).unwrap_or_default();
        update.installed_version = version.clone();
        if update.phase == "updating" {
            update.log = log_tail(&self.update_log);
        }
        ComfyStatus {
            installed,
            path: config.path,
            running,
            managed,
            endpoint: ENDPOINT.into(),
            version,
            error: if running {
                (!managed && installed).then_some("comfy_external_running".into())
            } else {
                self.runtime_error
                    .lock()
                    .ok()
                    .and_then(|error| error.clone())
                    .or_else(|| response.err().filter(|_| managed).map(|e| e.to_string()))
            },
            log: log_tail(&self.runtime_log),
            dismissed: config.dismissed,
            install: self.install.lock().map(|s| s.clone()).unwrap_or_default(),
            update,
        }
    }
    pub fn check_update(&self) -> Result<ComfyStatus> {
        if self.update.lock().map_err(|_| "comfy_storage")?.phase == "updating" {
            return Err("comfy_update_busy".into());
        }
        let current = self.status();
        if !current.installed {
            return Err("comfy_missing".into());
        }
        let installed = current.version.ok_or("comfy_version")?;
        {
            let mut update = self.update.lock().map_err(|_| "comfy_storage")?;
            *update = ComfyUpdateStatus {
                phase: "checking".into(),
                installed_version: Some(installed.clone()),
                latest_version: None,
                error: None,
                log: None,
            };
        }
        let result: Result<()> = (|| {
            let release: Release = release_client()?
                .get(RELEASE_API)
                .header("User-Agent", "Local-Studio")
                .send()
                .map_err(|_| "comfy_update_network")?
                .error_for_status()
                .map_err(|_| "comfy_update_network")?
                .json()
                .map_err(|_| "comfy_update_network")?;
            let latest = release
                .tag_name
                .strip_prefix('v')
                .ok_or("comfy_version")?
                .to_string();
            let newer = parse_version(&latest)? > parse_version(&installed)?;
            let mut update = self.update.lock().map_err(|_| "comfy_storage")?;
            update.phase = if newer { "available" } else { "current" }.into();
            update.latest_version = Some(latest);
            Ok(())
        })();
        if let Err(error) = result {
            if let Ok(mut update) = self.update.lock() {
                update.phase = "error".into();
                update.error = Some(error.clone());
            }
            return Err(error);
        }
        Ok(self.status())
    }
    fn run_update_pass(
        &self,
        python: &Path,
        script: &Path,
        repository: &Path,
        second: bool,
    ) -> Result<()> {
        let update_dir = script.parent().ok_or("comfy_update_files")?;
        verify_tree(python)?;
        verify_tree(script)?;
        model_library::no_links(repository).map_err(|_| "comfy_update_files")?;
        let log = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.update_log)
            .map_err(|_| "comfy_storage")?;
        let mut command = Command::new(python);
        command
            .current_dir(update_dir)
            .arg("-s")
            .arg(script)
            .arg(repository)
            .arg("--stable")
            .stdin(Stdio::null())
            .stdout(Stdio::from(log.try_clone().map_err(|_| "comfy_storage")?))
            .stderr(Stdio::from(log))
            .env_clear();
        if second {
            command.arg("--skip_self_update");
        }
        for key in [
            "SystemRoot",
            "WINDIR",
            "TEMP",
            "TMP",
            "LOCALAPPDATA",
            "APPDATA",
            "USERPROFILE",
            "HTTP_PROXY",
            "HTTPS_PROXY",
            "NO_PROXY",
            "SSL_CERT_FILE",
        ] {
            if let Some(value) = std::env::var_os(key) {
                command.env(key, value);
            }
        }
        crate::hardware::hide_console(&mut command);
        let mut child = command.spawn().map_err(|_| "comfy_update_start")?;
        let _group = ProcessGroup::attach(&child).map_err(|_| "comfy_update_start")?;
        let deadline = Instant::now() + Duration::from_secs(30 * 60);
        loop {
            if let Some(status) = child.try_wait().map_err(|_| "comfy_update_failed")? {
                return if status.success() {
                    Ok(())
                } else {
                    Err("comfy_update_failed".into())
                };
            }
            if Instant::now() >= deadline {
                let _ = child.kill();
                let _ = child.wait();
                return Err("comfy_update_timeout".into());
            }
            std::thread::sleep(Duration::from_millis(200));
        }
    }
    pub fn update_comfy(&self) -> Result<ComfyStatus> {
        let current = self.status();
        if !current.installed {
            return Err("comfy_missing".into());
        }
        if current.running && !current.managed {
            return Err("comfy_update_external_running".into());
        }
        let root = self
            .config
            .lock()
            .map_err(|_| "comfy_storage")?
            .path
            .clone()
            .map(PathBuf::from)
            .ok_or("comfy_missing")?;
        if !valid_root(&root) {
            return Err("comfy_path".into());
        }
        let python = root.join("python_embeded/python.exe");
        let repository = root.join("ComfyUI");
        let update_dir = root.join("update");
        let script = update_dir.join("update.py");
        if !script.is_file() {
            return Err("comfy_update_files".into());
        }
        {
            let mut update = self.update.lock().map_err(|_| "comfy_storage")?;
            if update.phase != "available" {
                return Err("comfy_update_state".into());
            }
            update.phase = "updating".into();
            update.error = None;
            update.log = None;
        }
        if self.active_generations.load(Ordering::SeqCst) != 0 {
            if let Ok(mut update) = self.update.lock() {
                update.phase = "available".into();
            }
            return Err("comfy_update_busy".into());
        }
        if fs::write(
            &self.update_log,
            b"Local Studio: official ComfyUI stable update\r\n",
        )
        .is_err()
        {
            if let Ok(mut update) = self.update.lock() {
                update.phase = "error".into();
                update.error = Some("comfy_storage".into());
            }
            return Err("comfy_storage".into());
        }
        let restart = current.managed && current.running;
        if restart {
            self.stop();
        }
        let result: Result<String> = (|| {
            self.run_update_pass(&python, &script, &repository, false)?;
            let replacement = update_dir.join("update_new.py");
            if replacement.exists() {
                verify_tree(&replacement)?;
                let metadata =
                    fs::metadata(&replacement).map_err(|_| "comfy_update_files".to_string())?;
                if metadata.len() < 10 || metadata.len() > 1024 * 1024 {
                    return Err("comfy_update_files".into());
                }
                fs::copy(&replacement, &script).map_err(|_| "comfy_update_files".to_string())?;
                fs::remove_file(&replacement).map_err(|_| "comfy_update_files".to_string())?;
                self.run_update_pass(&python, &script, &repository, true)?;
            }
            version_from_root(&root).ok_or_else(|| "comfy_version".to_string())
        })();
        match result {
            Ok(installed) => {
                if let Ok(mut update) = self.update.lock() {
                    update.phase = "current".into();
                    update.installed_version = Some(installed);
                    update.error = None;
                    update.log = log_tail(&self.update_log);
                }
                if restart {
                    let _ = self.start();
                }
                Ok(self.status())
            }
            Err(error) => {
                if let Ok(mut update) = self.update.lock() {
                    update.phase = "error".into();
                    update.error = Some(error.clone());
                    update.log = log_tail(&self.update_log);
                }
                if restart {
                    let _ = self.start();
                }
                Err(error)
            }
        }
    }
    pub fn detect_installation(&self) -> Result<ComfyStatus> {
        let found = detect(&self.managed_root, true).ok_or("comfy_not_found")?;
        self.set_path(found.to_string_lossy().into())
    }
    fn install_progress(
        &self,
        phase: &str,
        variant: &str,
        total: u64,
        received: u64,
        speed: u64,
        error: Option<String>,
    ) {
        if let Ok(mut state) = self.install.lock() {
            *state = ComfyInstallStatus {
                phase: phase.into(),
                variant: Some(variant.into()),
                total_bytes: total,
                received_bytes: received,
                bytes_per_second: speed,
                error,
            };
        }
    }
    pub fn download_install(&self, variant: String) -> Result<ComfyStatus> {
        {
            let state = self.install.lock().map_err(|_| "comfy_storage")?;
            if ["resolving", "downloading", "verifying", "installing"]
                .contains(&state.phase.as_str())
            {
                return Err("comfy_install_busy".into());
            }
        }
        self.install_progress("resolving", &variant, 0, 0, 0, None);
        let result = (|| {
            let client = release_client()?;
            let (_release, asset) = official_asset(&client, &variant)?;
            fs::create_dir_all(&self.managed_root).map_err(|_| "comfy_storage")?;
            model_library::no_links(&self.managed_root).map_err(|_| "comfy_storage")?;
            let required = asset
                .size
                .checked_mul(4)
                .ok_or("comfy_space")?
                .saturating_add(1024 * 1024 * 1024);
            if fs2::available_space(&self.managed_root).map_err(|_| "comfy_space")? < required {
                return Err("comfy_space".into());
            }
            let work = self
                .managed_root
                .join(format!(".install-{}", uuid::Uuid::new_v4()));
            fs::create_dir(&work).map_err(|_| "comfy_storage")?;
            let archive = work.join(&asset.name);
            let stage = work.join("extracted");
            let install_result = (|| {
                let mut response = client
                    .get(&asset.browser_download_url)
                    .header("User-Agent", "Local-Studio")
                    .send()
                    .map_err(|_| "comfy_download")?
                    .error_for_status()
                    .map_err(|_| "comfy_download")?;
                let final_host = response.url().host_str();
                if !matches!(
                    final_host,
                    Some(
                        "github.com"
                            | "release-assets.githubusercontent.com"
                            | "objects.githubusercontent.com"
                    )
                ) {
                    return Err("comfy_package".into());
                }
                let mut file = fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .open(&archive)
                    .map_err(|_| "comfy_storage")?;
                let mut hasher = Sha256::new();
                let mut received = 0u64;
                let started = Instant::now();
                let mut buffer = vec![0u8; 1024 * 1024];
                self.install_progress("downloading", &variant, asset.size, 0, 0, None);
                loop {
                    let read = response.read(&mut buffer).map_err(|_| "comfy_download")?;
                    if read == 0 {
                        break;
                    }
                    received = received.checked_add(read as u64).ok_or("comfy_download")?;
                    if received > asset.size {
                        return Err("comfy_package".into());
                    }
                    file.write_all(&buffer[..read])
                        .map_err(|_| "comfy_storage")?;
                    hasher.update(&buffer[..read]);
                    let speed =
                        (received as f64 / started.elapsed().as_secs_f64().max(0.001)) as u64;
                    self.install_progress(
                        "downloading",
                        &variant,
                        asset.size,
                        received,
                        speed,
                        None,
                    );
                }
                file.sync_all().map_err(|_| "comfy_storage")?;
                self.install_progress("verifying", &variant, asset.size, received, 0, None);
                let actual = format!("{:x}", hasher.finalize());
                if received != asset.size
                    || asset.digest.as_deref().map(|d| &d[7..]) != Some(actual.as_str())
                {
                    return Err("comfy_hash".into());
                }
                let system_root =
                    PathBuf::from(std::env::var_os("SystemRoot").ok_or("comfy_extract")?);
                let tar = system_root.join("System32/tar.exe");
                let mut listing_command = Command::new(&tar);
                listing_command.arg("-tf").arg(&archive);
                crate::hardware::hide_console(&mut listing_command);
                let listing = listing_command
                    .output()
                    .map_err(|_| "comfy_extract")?;
                if !listing.status.success() {
                    return Err("comfy_extract".into());
                }
                safe_archive_listing(&listing.stdout)?;
                fs::create_dir(&stage).map_err(|_| "comfy_storage")?;
                self.install_progress("installing", &variant, asset.size, received, 0, None);
                let mut extract_command = Command::new(&tar);
                extract_command
                    .arg("-xf")
                    .arg(&archive)
                    .arg("-C")
                    .arg(&stage);
                crate::hardware::hide_console(&mut extract_command);
                let extracted = extract_command
                    .status()
                    .map_err(|_| "comfy_extract")?;
                if !extracted.success() {
                    return Err("comfy_extract".into());
                }
                verify_tree(&stage)?;
                let source = extracted_root(&stage).ok_or("comfy_archive")?;
                let target = self.managed_root.join("ComfyUI_windows_portable");
                if target.exists() {
                    if valid_root(&target) {
                        return self.set_path(target.to_string_lossy().into());
                    }
                    return Err("comfy_install_exists".into());
                }
                fs::rename(&source, &target).map_err(|_| "comfy_storage")?;
                self.set_path(target.to_string_lossy().into())
            })();
            let _ = fs::remove_dir_all(&work);
            install_result
        })();
        match result {
            Ok(_) => {
                self.install_progress("completed", &variant, 0, 0, 0, None);
                match self.start() {
                    Ok(running) => Ok(running),
                    Err(_) => Ok(self.status()),
                }
            }
            Err(error) => {
                self.install_progress("failed", &variant, 0, 0, 0, Some(error.clone()));
                Err(error)
            }
        }
    }
    pub fn set_path(&self, path: String) -> Result<ComfyStatus> {
        let root = fs::canonicalize(path).map_err(|_| "comfy_path")?;
        if !valid_root(&root) {
            return Err("comfy_path".into());
        }
        {
            let mut c = self.config.lock().map_err(|_| "comfy_storage")?;
            c.path = Some(root.to_string_lossy().into());
            c.dismissed = false;
        }
        self.save()?;
        Ok(self.status())
    }
    pub fn dismiss(&self) -> Result<ComfyStatus> {
        {
            self.config.lock().map_err(|_| "comfy_storage")?.dismissed = true;
        }
        self.save()?;
        Ok(self.status())
    }
    pub fn start(&self) -> Result<ComfyStatus> {
        // Startup can take a while on handheld PCs. Serialize callers so a
        // second click or generation request cannot launch a duplicate server.
        let _start_guard = self.start_lock.lock().map_err(|_| "comfy_start")?;
        if probe_http().is_ok() {
            return if self
                .process
                .lock()
                .ok()
                .is_some_and(|process| process.is_some())
            {
                Ok(self.status())
            } else {
                Err("comfy_external_running".into())
            };
        }
        let root = {
            let c = self.config.lock().map_err(|_| "comfy_storage")?;
            PathBuf::from(c.path.as_ref().ok_or("comfy_missing")?)
        };
        if !valid_root(&root) {
            return Err("comfy_path".into());
        }
        self.write_extra_model_paths()?;
        let amd_safe_attention = uses_amd_safe_attention(&root);
        let amd_low_memory = if amd_safe_attention {
            let mut memory = sysinfo::System::new();
            memory.refresh_memory();
            uses_amd_low_memory_profile(memory.total_memory())
        } else {
            false
        };
        let log_header = if amd_low_memory {
            "Local Studio: ComfyUI runtime (AMD low-memory compatibility: split cross attention, pinned memory, async offload, dynamic VRAM, mmap and comfy-kitchen HIP disabled; lowvram enabled)\r\n"
        } else if amd_safe_attention {
            "Local Studio: ComfyUI runtime (AMD compatibility: split cross attention, pinned memory and async offload disabled)\r\n"
        } else {
            "Local Studio: ComfyUI runtime\r\n"
        };
        fs::write(&self.runtime_log, log_header).map_err(|_| "comfy_storage")?;
        if let Ok(mut error) = self.runtime_error.lock() {
            *error = None;
        }
        let log = fs::OpenOptions::new()
            .append(true)
            .open(&self.runtime_log)
            .map_err(|_| "comfy_storage")?;
        let mut command = Command::new(root.join("python_embeded/python.exe"));
        command
            .current_dir(&root)
            .arg("-s")
            .arg("ComfyUI/main.py")
            .args([
                "--windows-standalone-build",
                "--listen",
                "127.0.0.1",
                "--port",
                "8188",
                "--disable-auto-launch",
                "--disable-api-nodes",
                "--extra-model-paths-config",
            ])
            .arg(&self.extra_model_paths)
            .stdin(Stdio::null());
        if amd_safe_attention {
            command.args(AMD_COMPAT_ARGS);
        }
        if amd_low_memory {
            command.args(AMD_LOW_MEMORY_ARGS);
        }
        command
            .stdout(Stdio::from(log.try_clone().map_err(|_| "comfy_storage")?))
            .stderr(Stdio::from(log))
            .env_clear();
        for key in [
            "SystemRoot",
            "WINDIR",
            "TEMP",
            "TMP",
            "LOCALAPPDATA",
            "APPDATA",
            "USERPROFILE",
        ] {
            if let Some(v) = std::env::var_os(key) {
                command.env(key, v);
            }
        }
        if amd_low_memory {
            command.env(AMD_LOW_MEMORY_ENV.0, AMD_LOW_MEMORY_ENV.1);
        }
        crate::hardware::hide_console(&mut command);
        let mut child = command.spawn().map_err(|_| {
            if let Ok(mut error) = self.runtime_error.lock() {
                *error = Some("comfy_start".into());
            }
            "comfy_start"
        })?;
        let group = match ProcessGroup::attach(&child) {
            Ok(group) => group,
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                if let Ok(mut error) = self.runtime_error.lock() {
                    *error = Some("comfy_start".into());
                }
                return Err("comfy_start".into());
            }
        };
        *self.process.lock().map_err(|_| "comfy_start")? = Some(Process {
            child,
            _group: group,
        });
        // ComfyUI imports PyTorch, GPU backends and custom nodes before its API
        // becomes available. Give slower devices up to three minutes to finish.
        for _ in 0..720 {
            if probe_http().is_ok() {
                if let Ok(mut error) = self.runtime_error.lock() {
                    *error = None;
                }
                return Ok(self.status());
            }
            let exited = self
                .process
                .lock()
                .ok()
                .and_then(|mut process| {
                    process
                        .as_mut()
                        .and_then(|process| process.child.try_wait().ok().flatten())
                })
                .is_some();
            if exited {
                self.stop();
                if let Ok(mut error) = self.runtime_error.lock() {
                    *error = Some("comfy_start".into());
                }
                return Err("comfy_start".into());
            }
            std::thread::sleep(Duration::from_millis(250));
        }
        self.stop();
        if let Ok(mut error) = self.runtime_error.lock() {
            *error = Some("comfy_start".into());
        }
        Err("comfy_start".into())
    }
    pub fn stop(&self) {
        if let Ok(mut p) = self.process.lock() {
            if let Some(mut p) = p.take() {
                let _ = p.child.kill();
                let _ = p.child.wait();
            }
        }
    }
    pub fn checkpoint(&self, path: &Path) -> Option<String> {
        let requested = self.checkpoint_candidate(path)?;
        let info = client(Duration::from_secs(5))
            .ok()?
            .get(format!("{ENDPOINT}/object_info/CheckpointLoaderSimple"))
            .send()
            .ok()?
            .error_for_status()
            .ok()?
            .json::<Value>()
            .ok()?;
        resolve_comfy_option(
            &info,
            "CheckpointLoaderSimple",
            "ckpt_name",
            &requested,
        )
    }
    pub fn checkpoint_path(&self, path: &Path) -> bool {
        self.checkpoint_candidate(path).is_some()
    }
    /// Qwen Image 2.1 GGUF models are diffusion models, not checkpoints.  Keep this
    /// narrow on purpose: a random GGUF (for example a chat model) must never be
    /// submitted to the image engine.
    pub fn qwen_image21_path(&self, path: &Path) -> bool {
        self.qwen_image21_candidate(path).is_some()
    }
    fn qwen_image21_candidate(&self, path: &Path) -> Option<String> {
        let name = path.file_name()?.to_str()?.to_ascii_lowercase();
        if path.extension()?.to_str()?.eq_ignore_ascii_case("gguf") != true
            || !name.starts_with("qwen-image-2.1")
        {
            return None;
        }
        let root = self.config.lock().ok()?.path.clone().map(PathBuf::from)?;
        relative_model_path(path, &root.join("ComfyUI/models/diffusion_models"))
    }
    pub fn qwen_image21_missing(&self, path: &Path) -> Vec<String> {
        let Some(requested) = self.qwen_image21_candidate(path) else {
            return vec!["comfy_qwen_model_path".into()];
        };
        let Ok(client) = client(Duration::from_secs(5)) else {
            return vec!["comfy_connection".into()];
        };
        let info = |node: &str| client.get(format!("{ENDPOINT}/object_info/{node}"))
            .send().and_then(|response| response.error_for_status()).and_then(|response| response.json::<Value>());
        let Ok(unet) = info("UnetLoaderGGUF") else { return vec!["comfy_qwen_loader".into()]; };
        let Ok(clip) = info("CLIPLoader") else { return vec!["comfy_qwen_encoder".into()]; };
        let Ok(vae) = info("VAELoader") else { return vec!["comfy_qwen_vae".into()]; };
        let Ok(encode) = info("TextEncodeQwenImage21") else { return vec!["comfy_qwen_workflow".into()]; };
        let mut missing = Vec::new();
        if resolve_comfy_option(&unet, "UnetLoaderGGUF", "unet_name", &requested).is_none() { missing.push("comfy_qwen_model_unavailable".into()); }
        if resolve_comfy_option(&clip, "CLIPLoader", "clip_name", QWEN_IMAGE21_ENCODER).is_none() { missing.push("comfy_qwen_encoder".into()); }
        if resolve_comfy_option(&vae, "VAELoader", "vae_name", QWEN_IMAGE21_VAE).is_none() { missing.push("comfy_qwen_vae".into()); }
        if encode.get("TextEncodeQwenImage21").is_none() { missing.push("comfy_qwen_workflow".into()); }
        missing
    }
    fn checkpoint_candidate(&self, path: &Path) -> Option<String> {
        let root = self.config.lock().ok()?.path.clone().map(PathBuf::from)?;
        relative_model_path(path, &root.join("ComfyUI/models/checkpoints"))
            .or_else(|| relative_model_path(path, &self.external_checkpoints))
    }
    pub fn model_folder(&self, folder: &str) -> Option<PathBuf> {
        if !matches!(
            folder,
            "checkpoints" | "loras" | "vae" | "controlnet" | "upscale_models" | "embeddings"
        ) {
            return None;
        }
        let root = self.config.lock().ok()?.path.clone().map(PathBuf::from)?;
        if !valid_root(&root) {
            return None;
        }
        let models = root.join("ComfyUI/models").join(folder);
        fs::create_dir_all(&models).ok()?;
        model_library::no_links(&models).ok()?;
        fs::canonicalize(models).ok()
    }
    pub fn wan_models(&self) -> Result<Vec<WanModel>> {
        let root = self.config.lock().map_err(|_| "comfy_storage")?
            .path.clone().map(PathBuf::from).ok_or("comfy_missing")?;
        if !valid_root(&root) {
            return Err("comfy_missing".into());
        }
        let folder = root.join("ComfyUI/models/diffusion_models");
        model_library::no_links(&folder).map_err(|_| "comfy_model_path")?;
        let mut pending = vec![(folder, 0usize)];
        let mut models = Vec::new();
        while let Some((directory, depth)) = pending.pop() {
            for entry in fs::read_dir(&directory).map_err(|_| "comfy_model_path")?.flatten() {
                if models.len() >= 500 {
                    return Ok(models);
                }
                let path = entry.path();
                if model_library::no_links(&path).is_err() {
                    continue;
                }
                let Ok(metadata) = entry.metadata() else { continue };
                if metadata.is_dir() {
                    if depth < 3 { pending.push((path, depth + 1)); }
                } else if metadata.is_file() {
                    let name = entry.file_name().to_string_lossy().into_owned();
                    let extension = path.extension().and_then(|value| value.to_str()).unwrap_or("");
                    if name.to_ascii_lowercase().contains("wan")
                        && (extension.eq_ignore_ascii_case("gguf")
                            || extension.eq_ignore_ascii_case("safetensors"))
                    {
                        models.push(WanModel {
                            path: path.to_string_lossy().into_owned(),
                            name,
                            bytes: metadata.len(),
                        });
                    }
                }
            }
        }
        models.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(models)
    }
    fn model_relative(&self, path: &Path, folder: &str) -> Option<String> {
        if probe_http().is_err() {
            return None;
        }
        let root = self.config.lock().ok()?.path.clone().map(PathBuf::from)?;
        if let Some(relative) = relative_model_path(path, &root.join("ComfyUI/models").join(folder))
        {
            return Some(relative);
        }
        if folder == "loras" {
            for lora_root in self.configured_lora_paths().ok()? {
                if let Some(relative) = relative_model_path(path, &lora_root) {
                    return Some(relative);
                }
            }
        }
        (folder == "checkpoints")
            .then(|| relative_model_path(path, &self.external_checkpoints))
            .flatten()
    }
    /// Generates still frames through ComfyUI's built-in Wan image-to-video nodes.
    /// The caller owns the returned temporary PNG files and must remove them after
    /// turning them into the requested media format.
    pub fn wan_image_to_frames(&self, request: GifAiRequest) -> Result<Vec<String>> {
        if self.update.lock().ok().is_some_and(|state| state.phase == "updating") {
            return Err("comfy_update_busy".into());
        }
        if request.prompt.trim().is_empty()
            || request.prompt.len() > 8_000
            || request.negative_prompt.len() > 8_000
            || !(128..=2048).contains(&request.width)
            || !(128..=2048).contains(&request.height)
            || request.width % 16 != 0
            || request.height % 16 != 0
            || !(5..=81).contains(&request.frames)
            || (request.frames - 1) % 4 != 0
            || !(1..=50).contains(&request.steps)
            || !(0.0..=20.0).contains(&request.guidance)
        {
            return Err("gif_ai_parameters".into());
        }
        let source = fs::canonicalize(&request.source_path).map_err(|_| "gif_ai_source")?;
        model_library::no_links(&source).map_err(|_| "gif_ai_source")?;
        let source_metadata = fs::metadata(&source).map_err(|_| "gif_ai_source")?;
        let extension = source.extension().and_then(|value| value.to_str()).map(str::to_ascii_lowercase);
        if !source_metadata.is_file()
            || source_metadata.len() > 32 * 1024 * 1024
            || !matches!(extension.as_deref(), Some("png" | "jpg" | "jpeg" | "webp" | "bmp"))
        {
            return Err("gif_ai_source".into());
        }
        let model = fs::canonicalize(&request.model_path).map_err(|_| "gif_ai_model")?;
        model_library::no_links(&model).map_err(|_| "gif_ai_model")?;
        let model_name = model.file_name().and_then(|value| value.to_str()).unwrap_or("").to_ascii_lowercase();
        let root = self.config.lock().map_err(|_| "comfy_storage")?.path.clone().map(PathBuf::from).ok_or("comfy_missing")?;
        let is_gguf = model.extension().and_then(|value| value.to_str()).is_some_and(|value| value.eq_ignore_ascii_case("gguf"));
        let is_safetensors = model.extension().and_then(|value| value.to_str()).is_some_and(|value| value.eq_ignore_ascii_case("safetensors"));
        if !valid_root(&root)
            || !(is_safetensors || is_gguf)
            || !model_name.contains("wan")
        {
            return Err("gif_ai_model".into());
        }
        let model = relative_model_path(&model, &root.join("ComfyUI/models/diffusion_models")).ok_or("gif_ai_model")?;
        let input = root.join("ComfyUI/input");
        fs::create_dir_all(&input).map_err(|_| "comfy_storage")?;
        model_library::no_links(&input).map_err(|_| "comfy_storage")?;
        let source_name = format!("local-studio-gif-{}.{}", uuid::Uuid::new_v4(), extension.unwrap_or_else(|| "png".into()));
        let imported = input.join(&source_name);
        fs::copy(&source, &imported).map_err(|_| "gif_ai_source")?;
        let cleanup_import = |path: &Path| { let _ = fs::remove_file(path); };
        let result = (|| {
            self.active_generations.fetch_add(1, Ordering::SeqCst);
            let _active = ActiveGeneration(&self.active_generations);
            let client = client(Duration::from_secs(20))?;
            let object = |node: &str| client.get(format!("{ENDPOINT}/object_info/{node}"))
                .send().and_then(|response| response.error_for_status()).and_then(|response| response.json::<Value>())
                .map_err(|_| "comfy_connection");
            let unet_node = if is_gguf { "UnetLoaderGGUF" } else { "UNETLoader" };
            let unet = object(unet_node)?;
            let clip = object("CLIPLoader")?;
            let vae = object("VAELoader")?;
            let sampler_info = object("KSampler")?;
            let _wan = object("WanImageToVideo")?;
            let _load_image = object("LoadImage")?;
            let _encode = object("CLIPTextEncode")?;
            let _preview = object("PreviewImage")?;
            let unet_name = resolve_comfy_option(&unet, unet_node, "unet_name", &model).ok_or("gif_ai_model_unavailable")?;
            let clip_name = first_comfy_option(&clip, "CLIPLoader", "clip_name", |value| value.to_ascii_lowercase().contains("umt5")).ok_or("gif_ai_encoder")?;
            let clip_type = resolve_comfy_option(&clip, "CLIPLoader", "type", "wan").ok_or("gif_ai_workflow")?;
            let vae_name = first_comfy_option(&vae, "VAELoader", "vae_name", |value| value.to_ascii_lowercase().contains("wan")).ok_or("gif_ai_vae")?;
            let sampler = resolve_comfy_option(&sampler_info, "KSampler", "sampler_name", "euler")
                .or_else(|| first_comfy_option(&sampler_info, "KSampler", "sampler_name", |_| true)).ok_or("gif_ai_workflow")?;
            let scheduler = resolve_comfy_option(&sampler_info, "KSampler", "scheduler", "simple")
                .or_else(|| first_comfy_option(&sampler_info, "KSampler", "scheduler", |_| true)).ok_or("gif_ai_workflow")?;
            let loader_inputs = if is_gguf { json!({"unet_name":unet_name}) } else { json!({"unet_name":unet_name,"weight_dtype":"default"}) };
            let workflow = json!({
                "1":{"class_type":unet_node,"inputs":loader_inputs},
                "2":{"class_type":"CLIPLoader","inputs":{"clip_name":clip_name,"type":clip_type,"device":"default"}},
                "3":{"class_type":"VAELoader","inputs":{"vae_name":vae_name}},
                "4":{"class_type":"CLIPTextEncode","inputs":{"text":request.prompt,"clip":["2",0]}},
                "5":{"class_type":"CLIPTextEncode","inputs":{"text":request.negative_prompt,"clip":["2",0]}},
                "6":{"class_type":"LoadImage","inputs":{"image":source_name}},
                "7":{"class_type":"WanImageToVideo","inputs":{"positive":["4",0],"negative":["5",0],"vae":["3",0],"width":request.width,"height":request.height,"length":request.frames,"batch_size":1,"start_image":["6",0]}},
                "8":{"class_type":"KSampler","inputs":{"seed":request.seed,"steps":request.steps,"cfg":request.guidance,"sampler_name":sampler,"scheduler":scheduler,"denoise":1.0,"model":["1",0],"positive":["7",0],"negative":["7",1],"latent_image":["7",2]}},
                "9":{"class_type":"VAEDecode","inputs":{"samples":["8",0],"vae":["3",0]}},
                "10":{"class_type":"PreviewImage","inputs":{"images":["9",0]}}
            });
            let response = client.post(format!("{ENDPOINT}/prompt")).json(&json!({"prompt":workflow,"client_id":uuid::Uuid::new_v4().to_string()})).send().map_err(|_| "comfy_connection")?;
            if !response.status().is_success() { return Err("gif_ai_workflow".into()); }
            let id = response.json::<Value>().map_err(|_| "comfy_response")?["prompt_id"].as_str().ok_or("comfy_response")?.to_owned();
            let started = Instant::now();
            loop {
                if started.elapsed() > Duration::from_secs(900) { let _ = client.post(format!("{ENDPOINT}/interrupt")).send(); return Err("gif_ai_timeout".into()); }
                let history = client.get(format!("{ENDPOINT}/history/{id}")).send().and_then(|response| response.error_for_status()).and_then(|response| response.json::<Value>()).map_err(|_| "comfy_connection")?;
                if let Some(entry) = history.get(&id) {
                    if entry.pointer("/status/status_str").and_then(Value::as_str) == Some("error") || entry.pointer("/status/completed").and_then(Value::as_bool) == Some(false) { return Err("gif_ai_execution".into()); }
                    if let Some(images) = entry.pointer("/outputs/10/images").and_then(Value::as_array) {
                        if images.is_empty() || images.len() > 81 { return Err("gif_ai_output".into()); }
                        let mut frames = Vec::with_capacity(images.len());
                        for (index, image) in images.iter().enumerate() {
                            let filename = image["filename"].as_str().ok_or("comfy_response")?;
                            let subfolder = image["subfolder"].as_str().unwrap_or("");
                            let kind = image["type"].as_str().unwrap_or("output");
                            let response = client.get(format!("{ENDPOINT}/view")).query(&[("filename",filename),("subfolder",subfolder),("type",kind)]).send().map_err(|_| "comfy_connection")?;
                            if !response.status().is_success() { return Err("gif_ai_output".into()); }
                            let frame = input.join(format!("local-studio-gif-frame-{}-{index}.png", uuid::Uuid::new_v4()));
                            let mut output = fs::OpenOptions::new().create_new(true).write(true).open(&frame).map_err(|_| "gif_ai_output")?;
                            let copied = std::io::copy(&mut response.take(24 * 1024 * 1024 + 1), &mut output).map_err(|_| "gif_ai_output")?;
                            if copied > 24 * 1024 * 1024 { let _ = fs::remove_file(&frame); return Err("gif_ai_output".into()); }
                            output.sync_all().map_err(|_| "gif_ai_output")?;
                            frames.push(frame.to_string_lossy().into_owned());
                        }
                        return Ok(frames);
                    }
                }
                std::thread::sleep(Duration::from_millis(350));
            }
        })();
        cleanup_import(&imported);
        result
    }
    pub fn generate(
        &self,
        request: &ImageRequest,
        output: &Path,
        cancel: &AtomicBool,
        mut update: impl FnMut(&str),
        mut log_detail: impl FnMut(&str),
    ) -> Result<()> {
        if self
            .update
            .lock()
            .ok()
            .is_some_and(|state| state.phase == "updating")
        {
            return Err("comfy_update_busy".into());
        }
        if self.qwen_image21_path(Path::new(&request.model_path)) {
            return self.generate_qwen_image21(request, output, cancel, update, log_detail);
        }
        if self.register_lora_parents(&request.loras)? {
            self.reload_after_lora_registration()?;
        }
        self.active_generations.fetch_add(1, Ordering::SeqCst);
        let _active = ActiveGeneration(&self.active_generations);
        if self
            .update
            .lock()
            .ok()
            .is_some_and(|state| state.phase == "updating")
        {
            return Err("comfy_update_busy".into());
        }
        let requested_checkpoint = self
            .checkpoint(Path::new(&request.model_path))
            .ok_or("comfy_model_path")?;
        let requested_sampler = if request.sampler == "dpm++2m" {
            "dpmpp_2m"
        } else {
            "euler"
        };
        let client = client(Duration::from_secs(20))?;
        let loader_info = client
            .get(format!("{ENDPOINT}/object_info/CheckpointLoaderSimple"))
            .send()
            .and_then(|response| response.error_for_status())
            .and_then(|response| response.json::<Value>())
            .map_err(|_| "comfy_connection")?;
        let checkpoint = resolve_comfy_option(
            &loader_info,
            "CheckpointLoaderSimple",
            "ckpt_name",
            &requested_checkpoint,
        )
        .ok_or("comfy_checkpoint_unavailable")?;
        let sampler_info = client
            .get(format!("{ENDPOINT}/object_info/KSampler"))
            .send()
            .and_then(|response| response.error_for_status())
            .and_then(|response| response.json::<Value>())
            .map_err(|_| "comfy_connection")?;
        let sampler = resolve_comfy_option(
            &sampler_info,
            "KSampler",
            "sampler_name",
            requested_sampler,
        )
        .ok_or("comfy_workflow")?;
        let scheduler =
            resolve_comfy_option(&sampler_info, "KSampler", "scheduler", "karras")
                .ok_or("comfy_workflow")?;
        let mut workflow = json!({
         "1":{"class_type":"CheckpointLoaderSimple","inputs":{"ckpt_name":checkpoint}},
         "2":{"class_type":"CLIPTextEncode","inputs":{"text":request.prompt,"clip":["1",1]}},
         "3":{"class_type":"CLIPTextEncode","inputs":{"text":request.negative_prompt,"clip":["1",1]}},
         "4":{"class_type":"EmptyLatentImage","inputs":{"width":request.width,"height":request.height,"batch_size":1}},
         "5":{"class_type":"KSampler","inputs":{"seed":request.seed,"steps":request.steps,"cfg":request.guidance,"sampler_name":sampler,"scheduler":scheduler,"denoise":1.0,"model":["1",0],"positive":["2",0],"negative":["3",0],"latent_image":["4",0]}},
         "6":{"class_type":"VAEDecode","inputs":{"samples":["5",0],"vae":["1",2]}},
         "7":{"class_type":"PreviewImage","inputs":{"images":["6",0]}}
        });
        let mut source = "1".to_string();
        let lora_info = if request.loras.is_empty() {
            None
        } else {
            Some(
                client
                    .get(format!("{ENDPOINT}/object_info/LoraLoader"))
                    .send()
                    .and_then(|response| response.error_for_status())
                    .and_then(|response| response.json::<Value>())
                    .map_err(|_| "comfy_connection")?,
            )
        };
        for (index, lora) in request.loras.iter().enumerate() {
            let requested_name = self
                .model_relative(Path::new(&lora.path), "loras")
                .ok_or("image_lora_path")?;
            let name = resolve_comfy_option(
                lora_info.as_ref().ok_or("image_lora_path")?,
                "LoraLoader",
                "lora_name",
                &requested_name,
            )
            .ok_or("image_lora_path")?;
            let node = (8 + index).to_string();
            workflow[&node] = json!({
                "class_type":"LoraLoader",
                "inputs":{
                    "model":[source,0],
                    "clip":[source,1],
                    "lora_name":name,
                    "strength_model":lora.strength,
                    "strength_clip":lora.strength
                }
            });
            source = node;
        }
        workflow["2"]["inputs"]["clip"] = json!([source, 1]);
        workflow["3"]["inputs"]["clip"] = json!([source, 1]);
        workflow["5"]["inputs"]["model"] = json!([source, 0]);
        let response = client
            .post(format!("{ENDPOINT}/prompt"))
            .json(&json!({"prompt":workflow,"client_id":uuid::Uuid::new_v4().to_string()}))
            .send()
            .map_err(|_| "comfy_connection")?;
        if !response.status().is_success() {
            let status = response.status();
            let mut body = String::new();
            let _ = response.take(16 * 1024).read_to_string(&mut body);
            let body: String = body
                .chars()
                .map(|character| if character.is_control() { ' ' } else { character })
                .collect();
            log_detail(&format!("ComfyUI HTTP {status}: {body}"));
            if let Ok(mut log) = fs::OpenOptions::new().append(true).open(&self.runtime_log) {
                let _ = writeln!(log, "Local Studio: ComfyUI rejected workflow ({status}): {body}");
            }
            if body.contains("ckpt_name") && body.contains("not in") {
                return Err("comfy_checkpoint_unavailable".into());
            }
            return Err("comfy_workflow".into());
        }
        let id = response.json::<Value>().map_err(|_| "comfy_response")?["prompt_id"]
            .as_str()
            .ok_or("comfy_response")?
            .to_owned();
        update("processing");
        let started = Instant::now();
        let mut connection_failures = 0u8;
        loop {
            if cancel.load(Ordering::Relaxed) {
                let _ = client.post(format!("{ENDPOINT}/interrupt")).send();
                return Err("image_cancelled".into());
            }
            if started.elapsed() > Duration::from_secs(900) {
                let _ = client.post(format!("{ENDPOINT}/interrupt")).send();
                return Err("image_timeout".into());
            }
            let history = client
                .get(format!("{ENDPOINT}/history/{id}"))
                .send()
                .and_then(|r| r.error_for_status())
                .and_then(|r| r.json::<Value>());
            let history = match history {
                Ok(history) => {
                    connection_failures = 0;
                    history
                }
                Err(_) => {
                    if self.reap_runtime_exit() {
                        log_detail("ComfyUI process exited during image generation.");
                        return Err("comfy_runtime_exit".into());
                    }
                    connection_failures = connection_failures.saturating_add(1);
                    if connection_failures >= 3 {
                        log_detail("ComfyUI API stopped responding during image generation.");
                        return Err("comfy_connection".into());
                    }
                    std::thread::sleep(Duration::from_millis(250));
                    continue;
                }
            };
            if let Some(entry) = history.get(&id) {
                if entry.pointer("/status/status_str").and_then(Value::as_str) == Some("error")
                    || entry.pointer("/status/completed").and_then(Value::as_bool) == Some(false)
                {
                    return Err("comfy_execution".into());
                }
                if let Some(image) = entry.pointer("/outputs/7/images/0") {
                    let filename = image["filename"].as_str().ok_or("comfy_response")?;
                    let subfolder = image["subfolder"].as_str().unwrap_or("");
                    let kind = image["type"].as_str().unwrap_or("output");
                    let response = client
                        .get(format!("{ENDPOINT}/view"))
                        .query(&[
                            ("filename", filename),
                            ("subfolder", subfolder),
                            ("type", kind),
                        ])
                        .send()
                        .map_err(|_| "comfy_connection")?;
                    if !response.status().is_success() {
                        return Err("comfy_response".into());
                    }
                    let mut file = fs::OpenOptions::new()
                        .create_new(true)
                        .write(true)
                        .open(output)
                        .map_err(|_| "image_storage")?;
                    let mut limited = response.take(24 * 1024 * 1024 + 1);
                    let copied =
                        std::io::copy(&mut limited, &mut file).map_err(|_| "image_storage")?;
                    if copied > 24 * 1024 * 1024 {
                        return Err("image_output".into());
                    }
                    file.sync_all().map_err(|_| "image_storage")?;
                    return Ok(());
                }
            }
            std::thread::sleep(Duration::from_millis(250));
        }
    }
    fn generate_qwen_image21(
        &self,
        request: &ImageRequest,
        output: &Path,
        cancel: &AtomicBool,
        mut update: impl FnMut(&str),
        mut log_detail: impl FnMut(&str),
    ) -> Result<()> {
        if !request.loras.is_empty() {
            return Err("comfy_qwen_lora".into());
        }
        let missing = self.qwen_image21_missing(Path::new(&request.model_path));
        if let Some(error) = missing.first() {
            return Err(error.clone());
        }
        self.active_generations.fetch_add(1, Ordering::SeqCst);
        let _active = ActiveGeneration(&self.active_generations);
        let requested = self.qwen_image21_candidate(Path::new(&request.model_path)).ok_or("comfy_qwen_model_path")?;
        let client = client(Duration::from_secs(20))?;
        let object = |node: &str| client.get(format!("{ENDPOINT}/object_info/{node}"))
            .send().and_then(|response| response.error_for_status()).and_then(|response| response.json::<Value>())
            .map_err(|_| "comfy_connection");
        let unet = object("UnetLoaderGGUF")?;
        let clip = object("CLIPLoader")?;
        let vae = object("VAELoader")?;
        let sampler_info = object("KSampler")?;
        let model = resolve_comfy_option(&unet, "UnetLoaderGGUF", "unet_name", &requested).ok_or("comfy_qwen_model_unavailable")?;
        let encoder = resolve_comfy_option(&clip, "CLIPLoader", "clip_name", QWEN_IMAGE21_ENCODER).ok_or("comfy_qwen_encoder")?;
        let vae = resolve_comfy_option(&vae, "VAELoader", "vae_name", QWEN_IMAGE21_VAE).ok_or("comfy_qwen_vae")?;
        let sampler = resolve_comfy_option(&sampler_info, "KSampler", "sampler_name", if request.sampler == "dpm++2m" { "dpmpp_2m" } else { "euler" }).ok_or("comfy_qwen_workflow")?;
        let scheduler = resolve_comfy_option(&sampler_info, "KSampler", "scheduler", "simple").ok_or("comfy_qwen_workflow")?;
        // TextEncodeQwenImage21 supplies the correctly shaped latent itself. Its
        // autogrow image input is deliberately empty for text-to-image.
        let workflow = json!({
            "1":{"class_type":"UnetLoaderGGUF","inputs":{"unet_name":model}},
            "2":{"class_type":"CLIPLoader","inputs":{"clip_name":encoder,"type":"qwen_image","device":"default"}},
            "3":{"class_type":"VAELoader","inputs":{"vae_name":vae}},
            "4":{"class_type":"TextEncodeQwenImage21","inputs":{"clip":["2",0],"prompt":request.prompt,"negative_prompt":request.negative_prompt,"vae":["3",0],"resolution":request.width.max(request.height),"images":{}}},
            "5":{"class_type":"ModelSamplingAuraFlow","inputs":{"model":["1",0],"shift":3.1,"sampling":"flow"}},
            "6":{"class_type":"KSampler","inputs":{"seed":request.seed,"steps":request.steps,"cfg":request.guidance,"sampler_name":sampler,"scheduler":scheduler,"denoise":1.0,"model":["5",0],"positive":["4",0],"negative":["4",1],"latent_image":["4",2]}},
            "7":{"class_type":"VAEDecode","inputs":{"samples":["6",0],"vae":["3",0]}},
            "8":{"class_type":"PreviewImage","inputs":{"images":["7",0]}}
        });
        self.submit_and_wait(&client, workflow, "8", output, cancel, &mut update, &mut log_detail)
    }
    fn submit_and_wait(
        &self, client: &Client, workflow: Value, output_node: &str, output: &Path, cancel: &AtomicBool,
        update: &mut impl FnMut(&str), log_detail: &mut impl FnMut(&str),
    ) -> Result<()> {
        let response = client.post(format!("{ENDPOINT}/prompt"))
            .json(&json!({"prompt":workflow,"client_id":uuid::Uuid::new_v4().to_string()})).send().map_err(|_| "comfy_connection")?;
        if !response.status().is_success() {
            let status = response.status(); let mut body = String::new(); let _ = response.take(16 * 1024).read_to_string(&mut body);
            let body: String = body.chars().map(|c| if c.is_control() { ' ' } else { c }).collect();
            log_detail(&format!("ComfyUI HTTP {status}: {body}"));
            if let Ok(mut log) = fs::OpenOptions::new().append(true).open(&self.runtime_log) { let _ = writeln!(log, "Local Studio: ComfyUI rejected workflow ({status}): {body}"); }
            return Err("comfy_workflow".into());
        }
        let id = response.json::<Value>().map_err(|_| "comfy_response")?["prompt_id"].as_str().ok_or("comfy_response")?.to_owned();
        update("processing"); let started = Instant::now(); let mut failures = 0u8;
        loop {
            if cancel.load(Ordering::Relaxed) { let _ = client.post(format!("{ENDPOINT}/interrupt")).send(); return Err("image_cancelled".into()); }
            if started.elapsed() > Duration::from_secs(900) { let _ = client.post(format!("{ENDPOINT}/interrupt")).send(); return Err("image_timeout".into()); }
            let history = client.get(format!("{ENDPOINT}/history/{id}")).send().and_then(|r| r.error_for_status()).and_then(|r| r.json::<Value>());
            let history = match history { Ok(history) => { failures=0; history }, Err(_) => { if self.reap_runtime_exit() { log_detail("ComfyUI process exited during image generation."); return Err("comfy_runtime_exit".into()); } failures=failures.saturating_add(1); if failures>=3 { log_detail("ComfyUI API stopped responding during image generation."); return Err("comfy_connection".into()); } std::thread::sleep(Duration::from_millis(250)); continue; } };
            if let Some(entry) = history.get(&id) {
                if entry.pointer("/status/status_str").and_then(Value::as_str)==Some("error") || entry.pointer("/status/completed").and_then(Value::as_bool)==Some(false) { return Err("comfy_execution".into()); }
                let pointer = format!("/outputs/{output_node}/images/0");
                if let Some(image) = entry.pointer(&pointer) {
                    let filename=image["filename"].as_str().ok_or("comfy_response")?; let subfolder=image["subfolder"].as_str().unwrap_or(""); let kind=image["type"].as_str().unwrap_or("output");
                    let response=client.get(format!("{ENDPOINT}/view")).query(&[("filename",filename),("subfolder",subfolder),("type",kind)]).send().map_err(|_| "comfy_connection")?;
                    if !response.status().is_success(){return Err("comfy_response".into());}
                    let mut file=fs::OpenOptions::new().create_new(true).write(true).open(output).map_err(|_| "image_storage")?; let mut limited=response.take(24*1024*1024+1); let copied=std::io::copy(&mut limited,&mut file).map_err(|_| "image_storage")?; if copied>24*1024*1024{return Err("image_output".into());} file.sync_all().map_err(|_| "image_storage")?; return Ok(());
                }
            }
            std::thread::sleep(Duration::from_millis(250));
        }
    }
}
impl Drop for Comfy {
    fn drop(&mut self) {
        self.stop();
    }
}

#[tauri::command]
pub async fn comfy_status(state: tauri::State<'_, Arc<Comfy>>) -> Result<ComfyStatus> {
    let s = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || s.status())
        .await
        .map_err(|_| "comfy_connection".into())
}
#[tauri::command]
pub async fn comfy_set_path(
    path: String,
    state: tauri::State<'_, Arc<Comfy>>,
) -> Result<ComfyStatus> {
    let s = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || s.set_path(path))
        .await
        .map_err(|_| "comfy_storage")?
}
#[tauri::command]
pub async fn comfy_detect(state: tauri::State<'_, Arc<Comfy>>) -> Result<ComfyStatus> {
    let s = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || s.detect_installation())
        .await
        .map_err(|_| "comfy_storage")?
}
#[tauri::command]
pub async fn comfy_start(state: tauri::State<'_, Arc<Comfy>>) -> Result<ComfyStatus> {
    let s = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || s.start())
        .await
        .map_err(|_| "comfy_start")?
}
#[tauri::command]
pub fn comfy_stop(state: tauri::State<'_, Arc<Comfy>>) -> ComfyStatus {
    state.stop();
    state.status()
}
#[tauri::command]
pub fn comfy_dismiss(state: tauri::State<'_, Arc<Comfy>>) -> Result<ComfyStatus> {
    state.dismiss()
}
#[tauri::command]
pub async fn comfy_download(
    variant: String,
    state: tauri::State<'_, Arc<Comfy>>,
) -> Result<ComfyStatus> {
    let s = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || s.download_install(variant))
        .await
        .map_err(|_| "comfy_download")?
}
#[tauri::command]
pub async fn comfy_update_check(state: tauri::State<'_, Arc<Comfy>>) -> Result<ComfyStatus> {
    let s = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || s.check_update())
        .await
        .map_err(|_| "comfy_update_network")?
}
#[tauri::command]
pub async fn comfy_update(state: tauri::State<'_, Arc<Comfy>>) -> Result<ComfyStatus> {
    let s = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || s.update_comfy())
        .await
        .map_err(|_| "comfy_update_failed")?
}
#[tauri::command]
pub fn comfy_open_updater(state: tauri::State<'_, Arc<Comfy>>) -> Result<()> {
    let root = state
        .config
        .lock()
        .map_err(|_| "comfy_storage")?
        .path
        .clone()
        .map(PathBuf::from)
        .ok_or("comfy_missing")?;
    if !valid_root(&root) {
        return Err("comfy_path".into());
    }
    let update = root.join("update");
    model_library::no_links(&update).map_err(|_| "comfy_path")?;
    let explorer =
        PathBuf::from(std::env::var_os("SystemRoot").ok_or("comfy_path")?).join("explorer.exe");
    Command::new(explorer)
        .arg(update)
        .spawn()
        .map_err(|_| "comfy_path")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wan_catalog_lists_only_video_diffusion_models() {
        let temp = tempfile::tempdir().unwrap();
        let portable = temp.path().join("ComfyUI_windows_portable");
        let models = portable.join("ComfyUI/models/diffusion_models");
        fs::create_dir_all(portable.join("python_embeded")).unwrap();
        fs::create_dir_all(&models).unwrap();
        fs::write(portable.join("python_embeded/python.exe"), b"").unwrap();
        fs::write(portable.join("ComfyUI/main.py"), b"").unwrap();
        fs::write(models.join("wan_i2v.safetensors"), b"video").unwrap();
        fs::write(models.join("wan_i2v.gguf"), b"video").unwrap();
        fs::write(models.join("other_model.safetensors"), b"image").unwrap();
        fs::write(models.join("wan_notes.txt"), b"not a model").unwrap();
        let config = temp.path().join("config");
        fs::create_dir_all(&config).unwrap();
        fs::write(config.join("comfy.json"), serde_json::to_vec(&Config {
            path: Some(portable.to_string_lossy().into_owned()),
            ..Config::default()
        }).unwrap()).unwrap();
        let comfy = Comfy::new_with_checkpoints(&config, None).unwrap();
        let found = comfy.wan_models().unwrap();
        assert_eq!(found.len(), 2);
        assert!(found.iter().all(|model| model.name.starts_with("wan_i2v.")));
        assert!(found.iter().all(|model| model.bytes == 5));
    }

    #[test]
    fn package_variants_are_an_explicit_allowlist() {
        assert_eq!(
            package_name("amd").unwrap(),
            "ComfyUI_windows_portable_amd.7z"
        );
        assert!(uses_amd_low_memory_profile(12 * 1024 * 1024 * 1024));
        assert!(uses_amd_low_memory_profile(16 * 1024 * 1024 * 1024));
        assert!(!uses_amd_low_memory_profile(32 * 1024 * 1024 * 1024));
        assert_eq!(
            AMD_LOW_MEMORY_ARGS,
            ["--disable-dynamic-vram", "--lowvram", "--disable-mmap"]
        );
        assert_eq!(AMD_LOW_MEMORY_ENV, ("COMFY_KITCHEN_DISABLE_HIP", "1"));
        assert_eq!(
            package_name("nvidia_legacy").unwrap(),
            "ComfyUI_windows_portable_nvidia_cu126.7z"
        );
        assert_eq!(package_name("other"), Err("comfy_variant".into()));
    }

    #[test]
    fn amd_portable_installations_use_the_safe_attention_backend() {
        let temp = tempfile::tempdir().unwrap();
        assert!(!uses_amd_safe_attention(temp.path()));
        fs::write(temp.path().join("run_amd_gpu.bat"), b"rem official launcher").unwrap();
        assert!(uses_amd_safe_attention(temp.path()));
        assert_eq!(
            AMD_COMPAT_ARGS,
            [
                "--use-split-cross-attention",
                "--disable-pinned-memory",
                "--disable-async-offload"
            ]
        );
    }

    #[test]
    fn managed_comfy_config_exposes_local_studio_checkpoints_without_copying() {
        let temp = tempfile::tempdir().unwrap();
        let config = temp.path().join("config");
        let models = temp.path().join("Model's");
        let nested = models.join("hf-download");
        fs::create_dir_all(&config).unwrap();
        fs::create_dir_all(&nested).unwrap();
        let checkpoint = nested.join("wai.safetensors");
        fs::write(&checkpoint, b"model").unwrap();
        let comfy = Comfy::new_with_checkpoints(&config, Some(models.clone())).unwrap();

        comfy.write_extra_model_paths().unwrap();
        let yaml = fs::read_to_string(config.join("comfy-extra-model-paths.yaml")).unwrap();
        assert!(yaml.contains("local_studio:"));
        let expected = fs::canonicalize(&models)
            .unwrap()
            .to_string_lossy()
            .trim_start_matches(r"\\?\")
            .replace('\'', "''");
        assert_eq!(
            yaml,
            format!("local_studio:\n  checkpoints: '{expected}'\n")
        );
        assert_eq!(
            relative_model_path(&checkpoint, &models).as_deref(),
            Some("hf-download/wai.safetensors")
        );
        assert_eq!(
            comfy.checkpoint_candidate(&checkpoint).as_deref(),
            Some("hf-download/wai.safetensors")
        );
        assert!(relative_model_path(&checkpoint, &config).is_none());
        assert!(!comfy.checkpoint_path(&config.join("foreign.safetensors")));
    }

    #[test]
    fn extra_model_paths_register_safe_external_lora_folders() {
        let temp = tempfile::tempdir().unwrap();
        let config = temp.path().join("config");
        let models = temp.path().join("models");
        let lora_dir = temp.path().join("any user chosen lora folder");
        fs::create_dir_all(&config).unwrap();
        fs::create_dir_all(&models).unwrap();
        fs::create_dir_all(&lora_dir).unwrap();
        let lora = lora_dir.join("clove.safetensors");
        fs::write(&lora, b"lora").unwrap();
        let comfy = Comfy::new_with_checkpoints(&config, Some(models)).unwrap();

        assert!(comfy
            .register_lora_parents(&[ImageLora {
                path: lora.to_string_lossy().into_owned(),
                strength: 1.0,
                sha256: None,
            }])
            .unwrap());
        assert!(!comfy
            .register_lora_parents(&[ImageLora {
                path: lora.to_string_lossy().into_owned(),
                strength: 1.0,
                sha256: None,
            }])
            .unwrap());

        let yaml = fs::read_to_string(config.join("comfy-extra-model-paths.yaml")).unwrap();
        assert!(yaml.contains("local_studio_loras_0:"));
        assert!(yaml.contains("loras: '.'"));
        assert_eq!(
            comfy.configured_lora_paths().unwrap(),
            vec![fs::canonicalize(lora_dir).unwrap()]
        );
    }

    #[test]
    fn workflow_options_use_the_exact_names_reported_by_comfyui() {
        let info = json!({
            "CheckpointLoaderSimple": {"input":{"required":{"ckpt_name":[["nested\\wai.safetensors"]]}}},
            "KSampler": {"input":{"required":{
                "sampler_name":[["euler", "dpmpp_2m"]],
                "scheduler":[["normal", "karras"]]
            }}}
        });
        assert_eq!(
            resolve_comfy_option(
                &info,
                "CheckpointLoaderSimple",
                "ckpt_name",
                "nested/wai.safetensors"
            )
            .as_deref(),
            Some("nested\\wai.safetensors")
        );
        assert_eq!(
            resolve_comfy_option(&info, "KSampler", "scheduler", "karras").as_deref(),
            Some("karras")
        );
        assert!(resolve_comfy_option(&info, "KSampler", "sampler_name", "missing").is_none());
    }

    #[test]
    fn versions_are_read_as_data_and_compared_numerically() {
        assert!(parse_version("v0.37.0").unwrap() > parse_version("0.9.99").unwrap());
        for invalid in ["0.37", "0.37.0-beta", "00.37.0", "version 0.37.0"] {
            assert!(parse_version(invalid).is_err());
        }
        let temp = tempfile::tempdir().unwrap();
        let folder = temp.path().join("ComfyUI");
        fs::create_dir(&folder).unwrap();
        fs::write(
            folder.join("comfy_version.py"),
            b"# metadata\n__version__ = \"0.37.0\"\n",
        )
        .unwrap();
        assert_eq!(version_from_root(temp.path()).as_deref(), Some("0.37.0"));
    }

    #[test]
    fn archive_listing_rejects_paths_outside_the_install_root() {
        assert!(safe_archive_listing(b"ComfyUI_windows_portable/ComfyUI/main.py\n").is_ok());
        let long_safe = format!(
            "ComfyUI_windows_portable/python_embeded/Lib/site-packages/{}/{}/module.py\n",
            "long-package-directory".repeat(6),
            "nested-directory".repeat(5)
        );
        assert!(long_safe.len() > 220);
        assert!(safe_archive_listing(long_safe.as_bytes()).is_ok());
        assert!(safe_archive_listing(
            b"ComfyUI_windows_portable/python_embeded/Lib/site-packages/name-\x96/module.py\n"
        )
        .is_ok());
        assert!(safe_archive_listing(b"ComfyUI_windows_portable/python_embeded/Lib/site-packages/torch/lib/aotriton.images/amd-gfx11xx/flash/attn_fwd/FONLY__*bf16@16_128_F_F_0_0___gfx11xx.aks2\n").is_ok());
        assert!(safe_archive_listing(b"../escape.exe\n").is_err());
        assert!(safe_archive_listing(b"C:/escape.exe\n").is_err());
        assert!(safe_archive_listing(b"ComfyUI_windows_portable/../../escape.exe\n").is_err());
        assert!(safe_archive_listing(b"ComfyUI_windows_portable/CON/file.py\n").is_err());
        assert!(
            safe_archive_listing(b"ComfyUI_windows_portable/ComfyUI/custom_nodes/*.py\n").is_err()
        );
        assert!(safe_archive_listing(b"ComfyUI_windows_portable/python_embeded/Lib/site-packages/torch/lib/aotriton.images/amd-gfx11xx/flash/attn_fwd/bad**kernel.aks2\n").is_err());
    }
}
