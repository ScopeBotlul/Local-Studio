use crate::{
    image_engine::{ImageRequest, ProcessGroup},
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

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Config {
    path: Option<String>,
    dismissed: bool,
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
    dismissed: bool,
    install: ComfyInstallStatus,
    update: ComfyUpdateStatus,
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
    config: Mutex<Config>,
    process: Mutex<Option<Process>>,
    install: Mutex<ComfyInstallStatus>,
    update: Mutex<ComfyUpdateStatus>,
    update_log: PathBuf,
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
        if count > 250_000
            || entry.len() > 4096
            || entry.starts_with('/')
            || entry.as_bytes().get(1) == Some(&b':')
            // The portable Python environment legitimately contains complete paths longer
            // than the 220-character model-download limit. Validate every Windows component
            // separately instead of applying that limit to the complete archive member.
            || !entry.split('/').all(crate::downloads::valid_file)
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
    pub fn new(config_dir: &Path) -> Result<Arc<Self>> {
        let config_path = config_dir.join("comfy.json");
        let update_log = config_dir.join("comfy-update.log");
        let data_root = if config_dir
            .file_name()
            .is_some_and(|name| name.eq_ignore_ascii_case("config"))
        {
            config_dir.parent().unwrap_or(config_dir).to_path_buf()
        } else {
            config_dir.join("Data")
        };
        let managed_root = data_root.join("ComfyUI");
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
            config: Mutex::new(config),
            process: Mutex::new(None),
            install: Mutex::new(ComfyInstallStatus {
                phase: "idle".into(),
                ..Default::default()
            }),
            update: Mutex::new(ComfyUpdateStatus {
                phase: "idle".into(),
                ..Default::default()
            }),
            update_log,
            active_generations: AtomicUsize::new(0),
        });
        this.save()?;
        Ok(this)
    }
    fn save(&self) -> Result<()> {
        let data = serde_json::to_vec_pretty(&*self.config.lock().map_err(|_| "comfy_storage")?)
            .map_err(|_| "comfy_storage")?;
        fs::write(&self.config_path, data).map_err(|_| "comfy_storage".into())
    }
    pub fn status(&self) -> ComfyStatus {
        if let Ok(mut process) = self.process.lock() {
            if process
                .as_mut()
                .is_some_and(|p| p.child.try_wait().ok().flatten().is_some())
            {
                *process = None;
            }
        }
        let managed = self.process.lock().ok().is_some_and(|p| p.is_some());
        let response = probe_http();
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
            running: response.is_ok(),
            managed,
            endpoint: ENDPOINT.into(),
            version,
            error: response.err().filter(|_| managed).map(|e| e.to_string()),
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
                let listing = Command::new(&tar)
                    .arg("-tf")
                    .arg(&archive)
                    .output()
                    .map_err(|_| "comfy_extract")?;
                if !listing.status.success() {
                    return Err("comfy_extract".into());
                }
                safe_archive_listing(&listing.stdout)?;
                fs::create_dir(&stage).map_err(|_| "comfy_storage")?;
                self.install_progress("installing", &variant, asset.size, received, 0, None);
                let extracted = Command::new(&tar)
                    .arg("-xf")
                    .arg(&archive)
                    .arg("-C")
                    .arg(&stage)
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
            Ok(status) => {
                self.install_progress("completed", &variant, 0, 0, 0, None);
                Ok(self.start().unwrap_or(status))
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
        if probe_http().is_ok() {
            return Ok(self.status());
        }
        let root = {
            let c = self.config.lock().map_err(|_| "comfy_storage")?;
            PathBuf::from(c.path.as_ref().ok_or("comfy_missing")?)
        };
        if !valid_root(&root) {
            return Err("comfy_path".into());
        }
        let mut command = Command::new(root.join("python_embeded/python.exe"));
        command
            .current_dir(&root)
            .arg("ComfyUI/main.py")
            .args([
                "--listen",
                "127.0.0.1",
                "--port",
                "8188",
                "--disable-auto-launch",
                "--disable-api-nodes",
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
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
        crate::hardware::hide_console(&mut command);
        let child = command.spawn().map_err(|_| "comfy_start")?;
        let group = ProcessGroup::attach(&child).map_err(|_| "comfy_start")?;
        *self.process.lock().map_err(|_| "comfy_start")? = Some(Process {
            child,
            _group: group,
        });
        for _ in 0..120 {
            if probe_http().is_ok() {
                return Ok(self.status());
            }
            std::thread::sleep(Duration::from_millis(250));
        }
        self.stop();
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
        self.model_relative(path, "checkpoints")
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
    fn model_relative(&self, path: &Path, folder: &str) -> Option<String> {
        if probe_http().is_err() {
            return None;
        }
        let root = self.config.lock().ok()?.path.clone().map(PathBuf::from)?;
        let models = fs::canonicalize(root.join("ComfyUI/models").join(folder)).ok()?;
        let path = fs::canonicalize(path).ok()?;
        let relative = path.strip_prefix(models).ok()?.to_str()?.replace('\\', "/");
        (!relative.is_empty()).then_some(relative)
    }
    pub fn generate(
        &self,
        request: &ImageRequest,
        output: &Path,
        cancel: &AtomicBool,
        mut update: impl FnMut(&str),
    ) -> Result<()> {
        if self
            .update
            .lock()
            .ok()
            .is_some_and(|state| state.phase == "updating")
        {
            return Err("comfy_update_busy".into());
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
        let checkpoint = self
            .checkpoint(Path::new(&request.model_path))
            .ok_or("comfy_model_path")?;
        let sampler = if request.sampler == "dpm++2m" {
            "dpmpp_2m"
        } else {
            "euler"
        };
        let mut workflow = json!({
         "1":{"class_type":"CheckpointLoaderSimple","inputs":{"ckpt_name":checkpoint}},
         "2":{"class_type":"CLIPTextEncode","inputs":{"text":request.prompt,"clip":["1",1]}},
         "3":{"class_type":"CLIPTextEncode","inputs":{"text":request.negative_prompt,"clip":["1",1]}},
         "4":{"class_type":"EmptyLatentImage","inputs":{"width":request.width,"height":request.height,"batch_size":1}},
         "5":{"class_type":"KSampler","inputs":{"seed":request.seed,"steps":request.steps,"cfg":request.guidance,"sampler_name":sampler,"scheduler":"karras","denoise":1.0,"model":["1",0],"positive":["2",0],"negative":["3",0],"latent_image":["4",0]}},
         "6":{"class_type":"VAEDecode","inputs":{"samples":["5",0],"vae":["1",2]}},
         "7":{"class_type":"PreviewImage","inputs":{"images":["6",0]}}
        });
        let mut source = "1".to_string();
        for (index, lora) in request.loras.iter().enumerate() {
            let name = self
                .model_relative(Path::new(&lora.path), "loras")
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
        let client = client(Duration::from_secs(20))?;
        let response = client
            .post(format!("{ENDPOINT}/prompt"))
            .json(&json!({"prompt":workflow,"client_id":uuid::Uuid::new_v4().to_string()}))
            .send()
            .map_err(|_| "comfy_connection")?;
        if !response.status().is_success() {
            return Err("comfy_workflow".into());
        }
        let id = response.json::<Value>().map_err(|_| "comfy_response")?["prompt_id"]
            .as_str()
            .ok_or("comfy_response")?
            .to_owned();
        update("processing");
        let started = Instant::now();
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
                .and_then(|r| r.json::<Value>())
                .unwrap_or(Value::Null);
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
    fn package_variants_are_an_explicit_allowlist() {
        assert_eq!(
            package_name("amd").unwrap(),
            "ComfyUI_windows_portable_amd.7z"
        );
        assert_eq!(
            package_name("nvidia_legacy").unwrap(),
            "ComfyUI_windows_portable_nvidia_cu126.7z"
        );
        assert_eq!(package_name("other"), Err("comfy_variant".into()));
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
        assert!(safe_archive_listing(b"../escape.exe\n").is_err());
        assert!(safe_archive_listing(b"C:/escape.exe\n").is_err());
        assert!(safe_archive_listing(b"ComfyUI_windows_portable/../../escape.exe\n").is_err());
        assert!(safe_archive_listing(b"ComfyUI_windows_portable/CON/file.py\n").is_err());
    }
}
