use crate::{
    image_engine::{ImageRequest, ProcessGroup},
    model_library,
};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    time::{Duration, Instant},
};

type Result<T> = std::result::Result<T, String>;
const ENDPOINT: &str = "http://127.0.0.1:8188";
const DOWNLOAD:&str="https://github.com/Comfy-Org/ComfyUI/releases/latest/download/ComfyUI_windows_portable_nvidia.7z";

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
}
struct Process {
    child: Child,
    _group: ProcessGroup,
}
pub struct Comfy {
    config_path: PathBuf,
    config: Mutex<Config>,
    process: Mutex<Option<Process>>,
}

fn valid_root(path: &Path) -> bool {
    path.is_absolute()
        && model_library::no_links(path).is_ok()
        && path.join("python_embeded/python.exe").is_file()
        && path.join("ComfyUI/main.py").is_file()
}
fn detect() -> Option<PathBuf> {
    let mut choices = Vec::new();
    for drive in b'C'..=b'Z' {
        let root = PathBuf::from(format!("{}:\\", drive as char));
        choices.push(root.join("ComfyUI_windows_portable"));
        choices.push(root.join("Local/AI/ComfyUI_windows_portable"));
    }
    if let Some(user) = std::env::var_os("USERPROFILE") {
        let user = PathBuf::from(user);
        choices.push(user.join("Downloads/ComfyUI_windows_portable"));
        choices.push(user.join("ComfyUI_windows_portable"));
    }
    choices.into_iter().find(|p| valid_root(p))
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
impl Comfy {
    pub fn new(config_dir: &Path) -> Result<Arc<Self>> {
        let config_path = config_dir.join("comfy.json");
        let mut config: Config = fs::read(&config_path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        if config
            .path
            .as_ref()
            .is_none_or(|p| !valid_root(Path::new(p)))
        {
            config.path = detect().map(|p| p.to_string_lossy().into());
        }
        let this = Arc::new(Self {
            config_path,
            config: Mutex::new(config),
            process: Mutex::new(None),
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
        ComfyStatus {
            installed: config
                .path
                .as_ref()
                .is_some_and(|p| valid_root(Path::new(p))),
            path: config.path,
            running: response.is_ok(),
            managed,
            endpoint: ENDPOINT.into(),
            version: response
                .as_ref()
                .ok()
                .and_then(|v| {
                    v.pointer("/system/comfyui_version")
                        .or_else(|| v.get("comfyui_version"))
                })
                .and_then(Value::as_str)
                .map(str::to_owned),
            error: response.err().filter(|_| managed).map(|e| e.to_string()),
            dismissed: config.dismissed,
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
pub fn comfy_status(state: tauri::State<'_, Arc<Comfy>>) -> ComfyStatus {
    state.status()
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
pub fn comfy_download() {
    let _ = Command::new("explorer.exe").arg(DOWNLOAD).spawn();
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
