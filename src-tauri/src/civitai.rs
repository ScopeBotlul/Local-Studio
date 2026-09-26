use crate::{core::Core, gallery};
use reqwest::blocking::{Client, Response};
use serde::Serialize;
use serde_json::Value;
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};
use tauri::{
    webview::{NewWindowResponse, PageLoadEvent, WebviewBuilder},
    Manager, State, Webview, WebviewUrl,
};
use url::Url;

type Result<T> = std::result::Result<T, String>;
const API: &str = "https://civitai.com/api/v1";
const LABEL: &str = "civitai-website";
const HOME: &str = "https://civitai.red/images";
pub struct CivitaiBrowser {
    profile: PathBuf,
    owner: Mutex<Option<String>>,
    meta: Mutex<(String, bool, Option<String>)>,
    operations: Mutex<()>,
}
impl CivitaiBrowser {
    pub fn new(config: &Path) -> Self {
        Self {
            profile: config.join("civitai-webview"),
            owner: Mutex::new(None),
            meta: Mutex::new((String::new(), false, None)),
            operations: Mutex::new(()),
        }
    }
}
#[derive(Clone, Copy, serde::Deserialize)]
pub struct Bounds {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserState {
    url: String,
    title: String,
    loading: bool,
    notice: Option<String>,
    image_id: Option<u64>,
    post_id: Option<u64>,
    visible: bool,
}
#[derive(serde::Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum BrowserAction {
    Home,
    Back,
    Forward,
    Reload,
    Visit { url: String },
    OpenExternal,
    DismissNotice,
}
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    id: u64,
    model_id: Option<u64>,
    name: String,
    version: String,
    kind: String,
    base_model: Option<String>,
    download_url: Option<String>,
    file_name: Option<String>,
    size_bytes: Option<u64>,
    sha256: Option<String>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageInfo {
    id: u64,
    prompt: String,
    negative_prompt: String,
    width: u32,
    height: u32,
    nsfw_level: String,
    image_url: String,
    resources: Vec<Resource>,
}

fn client() -> Result<Client> {
    Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(20))
        .user_agent("Local-Studio/0.28")
        .build()
        .map_err(|_| "civitai_network".into())
}
fn limited(response: Response, limit: u64) -> Result<Vec<u8>> {
    if !response.status().is_success() {
        return Err("civitai_http".into());
    }
    if response.content_length().is_some_and(|n| n > limit) {
        return Err("civitai_response".into());
    }
    let mut bytes = Vec::new();
    response
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "civitai_network")?;
    if bytes.len() as u64 > limit {
        Err("civitai_response".into())
    } else {
        Ok(bytes)
    }
}
fn image_id(input: &str) -> Result<u64> {
    if input.len() > 8192 || input.chars().any(char::is_control) {
        return Err("civitai_url".into());
    }
    let url = Url::parse(input).map_err(|_| "civitai_url")?;
    if url.scheme() != "https"
        || url.port().is_some()
        || url.username() != ""
        || url.password().is_some()
        || !matches!(url.host_str(), Some("civitai.com" | "civitai.red"))
    {
        return Err("civitai_url".into());
    }
    let parts: Vec<_> = url.path_segments().ok_or("civitai_url")?.collect();
    parts
        .windows(2)
        .find(|pair| pair[0] == "images")
        .and_then(|pair| pair[1].parse().ok())
        .ok_or("civitai_image".into())
}
fn danbooru_id(input: &str) -> Result<u64> {
    if input.len() > 8192 || input.chars().any(char::is_control) {
        return Err("tag_url".into());
    }
    let url = Url::parse(input).map_err(|_| "tag_url")?;
    if url.scheme() != "https"
        || url.port().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.host_str() != Some("danbooru.donmai.us")
    {
        return Err("tag_url".into());
    }
    let parts: Vec<_> = url.path_segments().ok_or("tag_url")?.collect();
    if parts.len() != 2 || parts[0] != "posts" {
        return Err("tag_post".into());
    }
    parts[1].parse().map_err(|_| "tag_post".into())
}
fn site(url: &Url) -> bool {
    url.scheme() == "https"
        && url.port().is_none()
        && url.username().is_empty()
        && url.password().is_none()
        && matches!(
            url.host_str(),
            Some("civitai.com" | "civitai.red" | "danbooru.donmai.us")
        )
}
fn local(caller: &Webview) -> Result<()> {
    if caller.label() == "main" {
        Ok(())
    } else {
        Err("browser_forbidden".into())
    }
}
fn rect(caller: &Webview, b: Bounds) -> Result<tauri::Rect> {
    let window = caller.window();
    let size = window
        .inner_size()
        .map_err(|_| "browser_bounds")?
        .to_logical::<f64>(window.scale_factor().map_err(|_| "browser_bounds")?);
    if [b.x, b.y, b.width, b.height].iter().any(|v| !v.is_finite())
        || b.x < 0.
        || b.y < 0.
        || b.width < 30.
        || b.height < 30.
        || b.x + b.width > size.width + 2.
        || b.y + b.height > size.height + 2.
    {
        return Err("browser_bounds".into());
    }
    Ok(tauri::Rect {
        position: tauri::LogicalPosition::new(b.x, b.y).into(),
        size: tauri::LogicalSize::new(
            b.width.min(size.width - b.x),
            b.height.min(size.height - b.y),
        )
        .into(),
    })
}
fn current(caller: &Webview, browser: &CivitaiBrowser) -> Result<BrowserState> {
    let url = caller
        .app_handle()
        .get_webview(LABEL)
        .and_then(|v| v.url().ok())
        .filter(site)
        .unwrap_or_else(|| Url::parse(HOME).unwrap());
    let meta = browser.meta.lock().map_err(|_| "internal")?;
    Ok(BrowserState {
        image_id: image_id(url.as_str()).ok(),
        post_id: danbooru_id(url.as_str()).ok(),
        url: url.into(),
        title: meta.0.clone(),
        loading: meta.1,
        notice: meta.2.clone(),
        visible: browser.owner.lock().map_err(|_| "internal")?.is_some(),
    })
}

fn danbooru_tag_text(value: &Value, id: u64) -> Result<String> {
    if value["id"].as_u64() != Some(id) {
        return Err("tag_post".into());
    }
    let fields = [
        ("Character", "tag_string_character"),
        ("General", "tag_string_general"),
        ("Copyright", "tag_string_copyright"),
        ("Artist", "tag_string_artist"),
        ("Metadata", "tag_string_meta"),
    ];
    let mut output = String::new();
    for (heading, field) in fields {
        let tags = value[field].as_str().unwrap_or("");
        if tags.is_empty() {
            continue;
        }
        output.push_str(heading);
        output.push('\n');
        for tag in tags.split_whitespace().take(1500) {
            output.push_str(tag);
            output.push('\n');
        }
    }
    if output.is_empty() || output.len() > 64 * 1024 {
        return Err("tag_response".into());
    }
    Ok(output)
}
fn danbooru_tags(id: u64) -> Result<String> {
    let url = format!("https://danbooru.donmai.us/posts/{id}.json");
    let response = client()?.get(url).send().map_err(|_| "tag_network")?;
    let value: Value =
        serde_json::from_slice(&limited(response, 1024 * 1024)?).map_err(|_| "tag_response")?;
    danbooru_tag_text(&value, id)
}
fn api_json(path: &str) -> Result<Value> {
    let url = format!("{API}/{path}");
    let response = client()?.get(url).send().map_err(|_| "civitai_network")?;
    serde_json::from_slice(&limited(response, 2 * 1024 * 1024)?)
        .map_err(|_| "civitai_response".into())
}
fn version(id: u64) -> Result<Resource> {
    let value = api_json(&format!("model-versions/{id}"))?;
    let primary = value["files"].as_array().and_then(|files| {
        files
            .iter()
            .find(|file| file["primary"].as_bool() == Some(true))
            .or_else(|| files.first())
    });
    let sha = primary
        .and_then(|f| f.pointer("/hashes/SHA256").and_then(Value::as_str))
        .filter(|v| v.len() == 64 && v.bytes().all(|c| c.is_ascii_hexdigit()))
        .map(str::to_owned);
    Ok(Resource {
        id,
        model_id: value["modelId"].as_u64(),
        name: value
            .pointer("/model/name")
            .and_then(Value::as_str)
            .unwrap_or("Civitai resource")
            .chars()
            .take(200)
            .collect(),
        version: value["name"]
            .as_str()
            .unwrap_or("")
            .chars()
            .take(200)
            .collect(),
        kind: value
            .pointer("/model/type")
            .and_then(Value::as_str)
            .unwrap_or("Unknown")
            .chars()
            .take(80)
            .collect(),
        base_model: value["baseModel"]
            .as_str()
            .map(|v| v.chars().take(100).collect()),
        download_url: value["downloadUrl"]
            .as_str()
            .filter(|url| url.starts_with("https://civitai.com/api/download/models/"))
            .map(str::to_owned),
        file_name: primary
            .and_then(|f| f["name"].as_str())
            .map(|v| v.chars().take(180).collect()),
        size_bytes: primary
            .and_then(|f| f["sizeKB"].as_f64())
            .map(|kb| (kb * 1024.0) as u64),
        sha256: sha,
    })
}

fn lora_search(query: &str, base_model: &str) -> Result<Vec<Resource>> {
    let query = query.trim();
    let base_model = base_model.trim();
    if query.is_empty()
        || query.len() > 120
        || base_model.len() > 100
        || query.chars().any(char::is_control)
        || base_model.chars().any(char::is_control)
    {
        return Err("civitai_query".into());
    }
    let mut url = Url::parse(&format!("{API}/models")).map_err(|_| "civitai_network")?;
    {
        let mut pairs = url.query_pairs_mut();
        pairs
            .append_pair("limit", "20")
            .append_pair("types", "LORA")
            .append_pair("sort", "Most Downloaded")
            .append_pair("nsfw", "false")
            .append_pair("query", query);
        if !base_model.is_empty() {
            pairs.append_pair("baseModels", base_model);
        }
    }
    let response = client()?.get(url).send().map_err(|_| "civitai_network")?;
    let value: Value = serde_json::from_slice(&limited(response, 2 * 1024 * 1024)?)
        .map_err(|_| "civitai_response")?;
    let mut result = Vec::new();
    for model in value["items"].as_array().into_iter().flatten().take(20) {
        let Some(model_id) = model["id"].as_u64() else {
            continue;
        };
        let Some(version) = model["modelVersions"].as_array().and_then(|v| v.first()) else {
            continue;
        };
        let Some(id) = version["id"].as_u64() else {
            continue;
        };
        let primary = version["files"].as_array().and_then(|files| {
            files
                .iter()
                .find(|file| file["primary"].as_bool() == Some(true))
                .or_else(|| files.first())
        });
        result.push(Resource {
            id,
            model_id: Some(model_id),
            name: model["name"]
                .as_str()
                .unwrap_or("Civitai LoRA")
                .chars()
                .take(200)
                .collect(),
            version: version["name"]
                .as_str()
                .unwrap_or("")
                .chars()
                .take(200)
                .collect(),
            kind: "LORA".into(),
            base_model: version["baseModel"]
                .as_str()
                .map(|v| v.chars().take(100).collect()),
            download_url: version["downloadUrl"]
                .as_str()
                .filter(|url| url.starts_with("https://civitai.com/api/download/models/"))
                .map(str::to_owned),
            file_name: primary
                .and_then(|f| f["name"].as_str())
                .map(|v| v.chars().take(180).collect()),
            size_bytes: primary
                .and_then(|f| f["sizeKB"].as_f64())
                .map(|kb| (kb * 1024.0) as u64),
            sha256: primary
                .and_then(|f| f.pointer("/hashes/SHA256").and_then(Value::as_str))
                .filter(|v| v.len() == 64 && v.bytes().all(|c| c.is_ascii_hexdigit()))
                .map(str::to_owned),
        });
    }
    Ok(result)
}
fn info(id: u64) -> Result<ImageInfo> {
    let value = api_json(&format!("images?imageId={id}&withMeta=true&limit=1"))?;
    let image = value["items"]
        .as_array()
        .and_then(|items| items.first())
        .ok_or("civitai_image")?;
    if image["id"].as_u64() != Some(id) {
        return Err("civitai_image".into());
    }
    let meta = &image["meta"];
    let mut ids = image["modelVersionIds"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_u64)
        .collect::<Vec<_>>();
    if let Some(resources) = meta["civitaiResources"].as_array() {
        ids.extend(
            resources
                .iter()
                .filter_map(|r| r["modelVersionId"].as_u64().or_else(|| r["id"].as_u64())),
        );
    }
    ids.sort_unstable();
    ids.dedup();
    ids.truncate(16);
    let resources = ids.into_iter().filter_map(|id| version(id).ok()).collect();
    let nsfw = image["nsfwLevel"].as_str().unwrap_or("Unknown").to_string();
    if !matches!(nsfw.as_str(), "None" | "Unknown") && crate::privacy::locked() {
        return Err("privacy_locked".into());
    }
    Ok(ImageInfo {
        id,
        prompt: meta["prompt"]
            .as_str()
            .unwrap_or("")
            .chars()
            .take(4000)
            .collect(),
        negative_prompt: meta["negativePrompt"]
            .as_str()
            .or_else(|| meta["negative_prompt"].as_str())
            .unwrap_or("")
            .chars()
            .take(4000)
            .collect(),
        width: image["width"]
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .unwrap_or(0),
        height: image["height"]
            .as_u64()
            .and_then(|n| u32::try_from(n).ok())
            .unwrap_or(0),
        nsfw_level: nsfw,
        image_url: image["url"].as_str().ok_or("civitai_image")?.to_string(),
        resources,
    })
}
fn image_host(url: &Url) -> bool {
    url.scheme() == "https"
        && url.port().is_none()
        && url.username().is_empty()
        && url.password().is_none()
        && url
            .host_str()
            .is_some_and(|h| h == "civitai.com" || h.ends_with(".civitai.com"))
}
fn download_image(info: &ImageInfo, folder: &str, root: &Path) -> Result<String> {
    let mut url = Url::parse(&info.image_url).map_err(|_| "civitai_image")?;
    let http = client()?;
    let response = loop {
        if !image_host(&url) {
            return Err("civitai_image".into());
        }
        let response = http
            .get(url.clone())
            .send()
            .map_err(|_| "civitai_network")?;
        if response.status().is_redirection() {
            url = url
                .join(
                    response
                        .headers()
                        .get("location")
                        .and_then(|h| h.to_str().ok())
                        .ok_or("civitai_image")?,
                )
                .map_err(|_| "civitai_image")?;
            continue;
        }
        break response;
    };
    let content_type = response
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let extension = if content_type.starts_with("image/png") {
        "png"
    } else if content_type.starts_with("image/webp") {
        "webp"
    } else if content_type.starts_with("image/jpeg") {
        "jpg"
    } else {
        return Err("civitai_image".into());
    };
    let bytes = limited(response, 24 * 1024 * 1024)?;
    let image = image::load_from_memory(&bytes).map_err(|_| "civitai_image")?;
    if image.width() == 0 || image.height() == 0 || image.width() > 8192 || image.height() > 8192 {
        return Err("civitai_image".into());
    }
    let destination = gallery::resolve(root, folder)?;
    let _guards = gallery::directory_guards(&destination)?;
    let output = destination.join(format!("civitai-{}.{}", info.id, extension));
    if output.exists() {
        return Ok(output.to_string_lossy().into());
    }
    let temp = destination.join(format!(".civitai-{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut file = File::options()
            .write(true)
            .create_new(true)
            .open(&temp)
            .map_err(|_| "gallery_storage")?;
        file.write_all(&bytes)
            .and_then(|_| file.sync_all())
            .map_err(|_| "gallery_storage")?;
        gallery::publish(&temp, &output).map_err(|_| "gallery_storage")?;
        if !matches!(info.nsfw_level.as_str(), "None" | "Unknown") {
            crate::privacy::mark(&output)?;
        }
        Ok(output.to_string_lossy().into())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
    result
}

#[tauri::command]
pub async fn civitai_image_info(url: String) -> Result<ImageInfo> {
    tauri::async_runtime::spawn_blocking(move || info(image_id(&url)?))
        .await
        .map_err(|_| "civitai_network")?
}
#[tauri::command]
pub async fn civitai_lora_search(query: String, base_model: String) -> Result<Vec<Resource>> {
    tauri::async_runtime::spawn_blocking(move || lora_search(&query, &base_model))
        .await
        .map_err(|_| "civitai_network")?
}
#[tauri::command]
pub async fn danbooru_post_tags(url: String) -> Result<String> {
    if crate::privacy::locked() {
        return Err("privacy_locked".into());
    }
    tauri::async_runtime::spawn_blocking(move || danbooru_tags(danbooru_id(&url)?))
        .await
        .map_err(|_| "tag_network")?
}
#[tauri::command]
pub async fn civitai_save_reference(
    url: String,
    folder: String,
    core: State<'_, Arc<Core>>,
) -> Result<String> {
    let epoch = crate::privacy::epoch();
    let result = (async {
        let root =
            fs::canonicalize(core.storage_paths()?.gallery).map_err(|_| "gallery_missing")?;
        tauri::async_runtime::spawn_blocking(move || {
            let info = info(image_id(&url)?)?;
            download_image(&info, &folder, &root)
        })
        .await
        .map_err(|_| "civitai_network")?
    })
    .await;
    crate::privacy::finish(epoch, result)
}
#[tauri::command]
pub fn civitai_open_download(url: String) -> Result<()> {
    let parsed = Url::parse(&url).map_err(|_| "civitai_url")?;
    if parsed.scheme() != "https"
        || parsed.host_str() != Some("civitai.com")
        || !parsed.path().starts_with("/api/download/models/")
        || parsed.port().is_some()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
    {
        return Err("civitai_url".into());
    }
    crate::hub::open_external_https(&url)
}

#[tauri::command]
pub async fn civitai_browser_mount(
    caller: Webview,
    owner: String,
    bounds: Bounds,
    browser: State<'_, CivitaiBrowser>,
) -> Result<BrowserState> {
    local(&caller)?;
    if crate::privacy::locked() {
        return Err("privacy_locked".into());
    }
    if uuid::Uuid::parse_str(&owner).is_err() {
        return Err("browser_owner".into());
    }
    let _operation = browser.operations.lock().map_err(|_| "internal")?;
    let area = rect(&caller, bounds)?;
    if let Some(view) = caller.app_handle().get_webview(LABEL) {
        view.set_bounds(area).map_err(|_| "browser_bounds")?;
        view.show().map_err(|_| "browser_view")?;
    } else {
        fs::create_dir_all(&browser.profile).map_err(|_| "browser_profile")?;
        let nav = caller.app_handle().clone();
        let popup = nav.clone();
        let builder = WebviewBuilder::new(LABEL, WebviewUrl::External(Url::parse(HOME).unwrap()))
            .data_directory(browser.profile.clone())
            .disable_drag_drop_handler()
            .zoom_hotkeys_enabled(false)
            .devtools(false)
            .on_navigation(move |url| {
                if site(url) {
                    true
                } else {
                    let link = url.to_string();
                    std::thread::spawn(move || {
                        let _ = crate::hub::open_external_https(&link);
                    });
                    false
                }
            })
            .on_new_window(move |url, _| {
                if site(&url) {
                    let app = popup.clone();
                    std::thread::spawn(move || {
                        if let Some(view) = app.get_webview(LABEL) {
                            let _ = view.navigate(url);
                        }
                    });
                } else {
                    let link = url.to_string();
                    std::thread::spawn(move || {
                        let _ = crate::hub::open_external_https(&link);
                    });
                }
                NewWindowResponse::Deny
            })
            .on_document_title_changed(|view, title| {
                if let Some(state) = view.app_handle().try_state::<CivitaiBrowser>() {
                    if let Ok(mut meta) = state.meta.lock() {
                        meta.0 = title.chars().take(300).collect();
                    }
                }
            })
            .on_page_load(|view, payload| {
                if let Some(state) = view.app_handle().try_state::<CivitaiBrowser>() {
                    if let Ok(mut meta) = state.meta.lock() {
                        meta.1 = matches!(payload.event(), PageLoadEvent::Started);
                    }
                }
            })
            .on_download(|view, _| {
                if let Some(state) = view.app_handle().try_state::<CivitaiBrowser>() {
                    if let Ok(mut meta) = state.meta.lock() {
                        meta.2 = Some("browser_download".into());
                    }
                }
                false
            });
        caller
            .window()
            .add_child(builder, area.position, area.size)
            .map_err(|_| "browser_view")?;
    }
    *browser.owner.lock().map_err(|_| "internal")? = Some(owner);
    current(&caller, &browser)
}
#[tauri::command]
pub async fn civitai_browser_layout(
    caller: Webview,
    owner: String,
    bounds: Bounds,
    browser: State<'_, CivitaiBrowser>,
) -> Result<()> {
    local(&caller)?;
    let _operation = browser.operations.lock().map_err(|_| "internal")?;
    if browser.owner.lock().map_err(|_| "internal")?.as_deref() != Some(&owner) {
        return Ok(());
    }
    if let Some(view) = caller.app_handle().get_webview(LABEL) {
        view.set_bounds(rect(&caller, bounds)?)
            .map_err(|_| "browser_bounds")?;
    }
    Ok(())
}
#[tauri::command]
pub async fn civitai_browser_hide(
    caller: Webview,
    owner: String,
    browser: State<'_, CivitaiBrowser>,
) -> Result<()> {
    local(&caller)?;
    let _operation = browser.operations.lock().map_err(|_| "internal")?;
    if browser.owner.lock().map_err(|_| "internal")?.as_deref() != Some(&owner) {
        return Ok(());
    }
    if let Some(view) = caller.app_handle().get_webview(LABEL) {
        view.hide().map_err(|_| "browser_view")?;
    }
    *browser.owner.lock().map_err(|_| "internal")? = None;
    Ok(())
}
#[tauri::command]
pub fn civitai_browser_state(
    caller: Webview,
    browser: State<'_, CivitaiBrowser>,
) -> Result<BrowserState> {
    local(&caller)?;
    if crate::privacy::locked() {
        if let Some(view) = caller.app_handle().get_webview(LABEL) {
            let _ = view.hide();
        }
        return Err("privacy_locked".into());
    }
    current(&caller, &browser)
}
#[tauri::command]
pub async fn civitai_browser_action(
    caller: Webview,
    owner: String,
    action: BrowserAction,
    browser: State<'_, CivitaiBrowser>,
) -> Result<()> {
    local(&caller)?;
    if crate::privacy::locked() {
        return Err("privacy_locked".into());
    }
    let _operation = browser.operations.lock().map_err(|_| "internal")?;
    if browser.owner.lock().map_err(|_| "internal")?.as_deref() != Some(&owner) {
        return Err("browser_owner".into());
    }
    let view = caller
        .app_handle()
        .get_webview(LABEL)
        .ok_or("browser_view")?;
    match action {
        BrowserAction::Home => view.navigate(Url::parse(HOME).unwrap()),
        BrowserAction::Back => view.eval("history.back()"),
        BrowserAction::Forward => view.eval("history.forward()"),
        BrowserAction::Reload => view.reload(),
        BrowserAction::Visit { url } => {
            let parsed = Url::parse(&url).map_err(|_| "civitai_url")?;
            if !site(&parsed) {
                return Err("civitai_url".into());
            }
            view.navigate(parsed)
        }
        BrowserAction::OpenExternal => {
            let url = view.url().map_err(|_| "browser_view")?;
            if !site(&url) {
                return Err("civitai_url".into());
            }
            return crate::hub::open_external_https(url.as_str());
        }
        BrowserAction::DismissNotice => {
            browser.meta.lock().map_err(|_| "internal")?.2 = None;
            return Ok(());
        }
    }
    .map_err(|_| "browser_view".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepts_only_exact_image_pages() {
        assert_eq!(image_id("https://civitai.red/images/123?x=1").unwrap(), 123);
        for url in [
            "http://civitai.red/images/1",
            "https://civitai.red.evil.test/images/1",
            "https://civitai.red/models/1",
            "javascript:alert(1)",
        ] {
            assert!(image_id(url).is_err(), "{url}");
        }
    }

    #[test]
    fn accepts_only_exact_danbooru_posts() {
        assert_eq!(
            danbooru_id("https://danbooru.donmai.us/posts/42").unwrap(),
            42
        );
        for url in [
            "http://danbooru.donmai.us/posts/42",
            "https://evil.example/posts/42",
            "https://danbooru.donmai.us/posts/42/extra",
        ] {
            assert!(danbooru_id(url).is_err());
        }
    }

    #[test]
    fn danbooru_fields_become_bounded_sectioned_tag_text() {
        let value = serde_json::json!({
            "id": 42,
            "tag_string_character": "cartethyia_(wuthering_waves)",
            "tag_string_general": "1girl green_hair",
            "tag_string_meta": "highres"
        });
        assert_eq!(
            danbooru_tag_text(&value, 42).unwrap(),
            "Character\ncartethyia_(wuthering_waves)\nGeneral\n1girl\ngreen_hair\nMetadata\nhighres\n"
        );
        assert_eq!(danbooru_tag_text(&value, 41).unwrap_err(), "tag_post");
    }

    #[test]
    fn lora_search_rejects_empty_and_unbounded_queries_before_network() {
        assert_eq!(lora_search("", "SDXL 1.0").unwrap_err(), "civitai_query");
        assert_eq!(
            lora_search(&"x".repeat(121), "").unwrap_err(),
            "civitai_query"
        );
    }
}
