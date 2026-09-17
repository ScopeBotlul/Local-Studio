use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::Mutex};
use tauri::{Manager, State, Webview, WebviewUrl, webview::{WebviewBuilder, NewWindowResponse, PageLoadEvent}};
use url::Url;

const LABEL: &str = "hf-website";
const HOME: &str = "https://huggingface.co/models";

pub struct HfBrowser {
    profile: PathBuf,
    operations: Mutex<()>,
    owner: Mutex<Option<String>>,
    metadata: Mutex<Metadata>,
    login: Mutex<Option<Login>>,
}
#[derive(Clone)]
struct Login { epoch: u64, url: Url, redirect: Url }

#[derive(Default)]
struct Metadata { title: String, loading: bool, notice: Option<String> }

#[derive(Deserialize, Clone, Copy)]
pub struct Bounds { x: f64, y: f64, width: f64, height: f64 }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BrowserStatus {
    url: String, title: String, loading: bool, notice: Option<String>, model_repo: Option<String>, visible: bool,
}

impl HfBrowser {
    pub fn new(config: &std::path::Path) -> Self {
        Self { profile: config.join("huggingface-webview"), operations: Mutex::new(()), owner: Mutex::new(None), metadata: Mutex::new(Metadata::default()), login: Mutex::new(None) }
    }
}

// The only non-HF navigation exception is the active OAuth listener's exact endpoint.
fn callback_matches(url: &Url, redirect: &Url) -> bool {
    url.scheme() == "http" && url.host_str() == Some("127.0.0.1")
        && url.port() == redirect.port() && url.path() == "/callback"
        && url.username().is_empty() && url.password().is_none() && url.fragment().is_none()
}
fn login_callback(app: &tauri::AppHandle, url: &Url) -> bool {
    app.state::<HfBrowser>().login.lock().ok().and_then(|login| login.clone())
        .is_some_and(|login| callback_matches(url, &login.redirect)
            && app.state::<std::sync::Arc<crate::hf_auth::HfAuth>>().pending_generation(login.epoch))
}
pub fn start_login(app: &tauri::AppHandle, epoch: u64, url: Url, redirect: Url) -> Result<(), String> {
    let browser = app.state::<HfBrowser>();
    let _operation = browser.operations.lock().map_err(|_| "internal")?;
    if !app.state::<std::sync::Arc<crate::hf_auth::HfAuth>>().pending_generation(epoch) { return Err("cancelled".into()); }
    *browser.login.lock().map_err(|_| "internal")? = Some(Login { epoch, url: url.clone(), redirect });
    if let Some(view) = app.get_webview(LABEL) {
        if view.navigate(url).is_err() {
            *browser.login.lock().map_err(|_| "internal")? = None;
            return Err("browser_view".into());
        }
    }
    Ok(())
}
pub fn finish_login(app: &tauri::AppHandle, epoch: u64) {
    let browser = app.state::<HfBrowser>();
    let Ok(_operation) = browser.operations.lock() else { return; };
    let removed = browser.login.lock().ok().is_some_and(|mut login| {
        if login.as_ref().is_some_and(|value| value.epoch == epoch) { login.take(); true } else { false }
    });
    if removed { if let Some(view) = app.get_webview(LABEL) { let _ = view.navigate(Url::parse(HOME).unwrap()); } }
}
pub fn cancel_login(app: &tauri::AppHandle, auth: &crate::hf_auth::HfAuth) -> Result<crate::hf_auth::AuthStatus, String> {
    let browser = app.state::<HfBrowser>();
    let _operation = browser.operations.lock().map_err(|_| "internal")?;
    let status = auth.cancel()?;
    browser.login.lock().map_err(|_| "internal")?.take();
    if let Some(view) = app.get_webview(LABEL) { let _ = view.navigate(Url::parse(HOME).unwrap()); }
    Ok(status)
}

fn local(caller: &Webview) -> Result<(), String> {
    // Defense in depth: even a future accidental capability change cannot expose this controller to the website.
    if caller.label() != "main" { return Err("browser_forbidden".into()); }
    Ok(())
}

fn hf_url(url: &Url) -> bool {
    url.scheme() == "https" && matches!(url.host_str(), Some("huggingface.co" | "hf.co"))
        && url.port().is_none() && url.username().is_empty() && url.password().is_none()
}

fn external_url(url: &Url) -> bool {
    url.scheme() == "https" && url.host_str().is_some() && url.username().is_empty() && url.password().is_none()
        && !matches!(url.host_str(), Some("localhost" | "tauri.localhost" | "ipc.localhost" | "asset.localhost"))
        && matches!(url.host(), Some(url::Host::Domain(host)) if !host.ends_with(".localhost"))
}

fn model_repo(url: &Url) -> Option<String> {
    if !hf_url(url) || url.host_str() != Some("huggingface.co") { return None; }
    let parts: Vec<_> = url.path().trim_start_matches('/').split('/').collect();
    if parts.len() < 2 || ["docs", "settings", "login", "logout", "join", "signup", "models", "spaces", "datasets", "organizations", "oauth", "api", "blog", "posts", "collections", "papers", "tasks", "enterprise", "pricing", "notifications", "search"].contains(&parts[0]) { return None; }
    let repo = format!("{}/{}", parts[0], parts[1]);
    crate::hub::validate_repo(&repo).ok()?;
    Some(repo)
}

fn checked_bounds(bounds: Bounds, width: f64, height: f64) -> Result<tauri::Rect, String> {
    if [bounds.x, bounds.y, bounds.width, bounds.height, width, height].iter().any(|v| !v.is_finite())
        || bounds.x < 0.0 || bounds.y < 0.0 || bounds.width < 30.0 || bounds.height < 30.0
        || bounds.x + bounds.width > width + 2.0 || bounds.y + bounds.height > height + 2.0 {
        return Err("browser_bounds".into());
    }
    Ok(tauri::Rect { position: tauri::LogicalPosition::new(bounds.x, bounds.y).into(), size: tauri::LogicalSize::new(bounds.width.min(width - bounds.x), bounds.height.min(height - bounds.y)).into() })
}

fn rectangle(caller: &Webview, bounds: Bounds) -> Result<tauri::Rect, String> {
    let window = caller.window();
    let size = window.inner_size().map_err(|_| "browser_bounds")?.to_logical::<f64>(window.scale_factor().map_err(|_| "browser_bounds")?);
    checked_bounds(bounds, size.width, size.height)
}

fn notice(app: &tauri::AppHandle, text: &str) {
    if let Some(browser) = app.try_state::<HfBrowser>() {
        if let Ok(mut metadata) = browser.metadata.lock() { metadata.notice = Some(text.into()); }
    }
}

fn external(app: tauri::AppHandle, url: Url) {
    if !external_url(&url) { notice(&app, "browser_blocked"); return; }
    std::thread::spawn(move || {
        if crate::hub::open_external_https(url.as_str()).is_err() { notice(&app, "browser"); }
        else { notice(&app, "browser_external"); }
    });
}

fn state(caller: &Webview, browser: &HfBrowser) -> Result<BrowserStatus, String> {
    let url = caller.app_handle().get_webview(LABEL).and_then(|view| view.url().ok()).filter(hf_url).unwrap_or_else(|| Url::parse(HOME).unwrap());
    let metadata = browser.metadata.lock().map_err(|_| "internal")?;
    Ok(BrowserStatus { model_repo: model_repo(&url), url: url.into(), title: metadata.title.clone(), loading: metadata.loading, notice: metadata.notice.clone(), visible: browser.owner.lock().map_err(|_| "internal")?.is_some() })
}

#[tauri::command]
pub async fn hf_browser_mount(caller: Webview, owner: String, bounds: Bounds, browser: State<'_, HfBrowser>) -> Result<BrowserStatus, String> {
    local(&caller)?;
    if uuid::Uuid::parse_str(&owner).is_err() { return Err("browser_owner".into()); }
    // Async Tauri command: WebView2 child creation deadlocks on the UI thread.
    let _operation = browser.operations.lock().map_err(|_| "internal")?;
    let rect = rectangle(&caller, bounds)?;
    if let Some(view) = caller.app_handle().get_webview(LABEL) {
        view.set_bounds(rect).map_err(|_| "browser_bounds")?;
        view.show().map_err(|_| "browser_view")?;
    } else {
        std::fs::create_dir_all(&browser.profile).map_err(|_| "browser_profile")?;
        let nav_app = caller.app_handle().clone();
        let popup_app = nav_app.clone();
        let builder = WebviewBuilder::new(LABEL, WebviewUrl::External(browser.login.lock().map_err(|_| "internal")?.as_ref().map(|login| login.url.clone()).unwrap_or_else(|| Url::parse(HOME).unwrap())))
            .data_directory(browser.profile.clone())
            .disable_drag_drop_handler()
            .zoom_hotkeys_enabled(false)
            .devtools(false)
            .on_navigation(move |url| {
                if hf_url(url) || login_callback(&nav_app, url) { true } else { external(nav_app.clone(), url.clone()); false }
            })
            .on_new_window(move |url, _features| {
                if hf_url(&url) || login_callback(&popup_app, &url) {
                    let app = popup_app.clone();
                    std::thread::spawn(move || { if let Some(view) = app.get_webview(LABEL) { let _ = view.navigate(url); } });
                } else { external(popup_app.clone(), url); }
                // Never create an ungoverned child/popup WebView.
                NewWindowResponse::Deny
            })
            .on_document_title_changed(|view, title| {
                if let Some(browser) = view.app_handle().try_state::<HfBrowser>() {
                    if let Ok(mut metadata) = browser.metadata.lock() { metadata.title = title.chars().take(300).collect(); }
                }
            })
            .on_page_load(|view, payload| {
                if let Some(browser) = view.app_handle().try_state::<HfBrowser>() {
                    if let Ok(mut metadata) = browser.metadata.lock() { metadata.loading = matches!(payload.event(), PageLoadEvent::Started); }
                }
            })
            .on_download(|view, _event| { notice(view.app_handle(), "browser_download"); false });
        caller.window().add_child(builder, rect.position, rect.size).map_err(|_| "browser_view")?;
    }
    *browser.owner.lock().map_err(|_| "internal")? = Some(owner);
    state(&caller, &browser)
}

#[tauri::command]
pub async fn hf_browser_layout(caller: Webview, owner: String, bounds: Bounds, browser: State<'_, HfBrowser>) -> Result<(), String> {
    local(&caller)?;
    let _operation = browser.operations.lock().map_err(|_| "internal")?;
    if browser.owner.lock().map_err(|_| "internal")?.as_deref() != Some(&owner) { return Ok(()); }
    let rect = rectangle(&caller, bounds)?;
    if let Some(view) = caller.app_handle().get_webview(LABEL) { view.set_bounds(rect).map_err(|_| "browser_bounds")?; }
    Ok(())
}

#[tauri::command]
pub async fn hf_browser_hide(caller: Webview, owner: String, browser: State<'_, HfBrowser>) -> Result<(), String> {
    local(&caller)?;
    let _operation = browser.operations.lock().map_err(|_| "internal")?;
    if browser.owner.lock().map_err(|_| "internal")?.as_deref() != Some(&owner) { return Ok(()); }
    if let Some(view) = caller.app_handle().get_webview(LABEL) { view.hide().map_err(|_| "browser_view")?; }
    *browser.owner.lock().map_err(|_| "internal")? = None;
    Ok(())
}

#[tauri::command]
pub fn hf_browser_state(caller: Webview, browser: State<'_, HfBrowser>) -> Result<BrowserStatus, String> { local(&caller)?; state(&caller, &browser) }

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum BrowserAction { Home, Back, Forward, Reload, Visit { url: String }, OpenExternal, DismissNotice }

#[tauri::command]
pub async fn hf_browser_action(caller: Webview, owner: String, action: BrowserAction, browser: State<'_, HfBrowser>) -> Result<(), String> {
    local(&caller)?;
    let _operation = browser.operations.lock().map_err(|_| "internal")?;
    if browser.owner.lock().map_err(|_| "internal")?.as_deref() != Some(&owner) { return Err("browser_owner".into()); }
    let view = caller.app_handle().get_webview(LABEL).ok_or("browser_view")?;
    match action {
        BrowserAction::Home => view.navigate(Url::parse(HOME).unwrap()),
        BrowserAction::Back => view.eval("history.back()"),
        BrowserAction::Forward => view.eval("history.forward()"),
        BrowserAction::Reload => view.reload(),
        BrowserAction::Visit { url } => {
            if url.len() > 8192 || url.chars().any(char::is_control) { return Err("invalid_link".into()); }
            let url = Url::parse(&url).map_err(|_| "invalid_link")?;
            if !hf_url(&url) { return Err("browser_hf_only".into()); }
            view.navigate(url)
        },
        BrowserAction::OpenExternal => {
            let url = view.url().map_err(|_| "browser_view")?;
            if !hf_url(&url) { return Err("invalid_link".into()); }
            return crate::hub::open_external_https(url.as_str());
        },
        BrowserAction::DismissNotice => { browser.metadata.lock().map_err(|_| "internal")?.notice = None; return Ok(()); },
    }.map_err(|_| "browser_view".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn oauth_exception_only_matches_exact_temporary_loopback_endpoint() {
        let redirect = Url::parse("http://127.0.0.1:43210/callback").unwrap();
        assert!(callback_matches(&Url::parse("http://127.0.0.1:43210/callback?state=a&code=b").unwrap(), &redirect));
        for url in ["http://127.0.0.1:43211/callback", "http://localhost:43210/callback", "https://127.0.0.1:43210/callback", "http://127.0.0.1:43210/other", "http://user@127.0.0.1:43210/callback", "http://127.0.0.1:43210/callback#fragment"] {
            assert!(!callback_matches(&Url::parse(url).unwrap(), &redirect), "{url}");
        }
    }
    #[test]
    fn navigation_keeps_only_exact_https_hf_domains_inside() {
        for url in ["https://huggingface.co/login", "https://hf.co/org/repo"] { assert!(hf_url(&Url::parse(url).unwrap())); }
        for url in ["http://huggingface.co", "https://huggingface.co.evil.test", "https://user@huggingface.co", "https://huggingface.co:123/", "tauri://localhost", "file:///C:/Windows", "javascript:alert(1)", "https://example.org/"] { assert!(!hf_url(&Url::parse(url).unwrap()), "{url}"); }
        for url in ["https://localhost/", "https://127.0.0.1/", "https://[::1]/", "https://ipc.localhost/", "file:///C:/Windows", "javascript:alert(1)", "https://user:secret@example.org/"] { assert!(!external_url(&Url::parse(url).unwrap()), "{url}"); }
        assert!(external_url(&Url::parse("https://example.org/paper").unwrap()));
    }
    #[test]
    fn model_recognition_rejects_site_routes_and_unsafe_repository_paths() {
        assert_eq!(model_repo(&Url::parse("https://huggingface.co/openai-community/gpt2/tree/main").unwrap()), Some("openai-community/gpt2".into()));
        for url in ["https://huggingface.co/docs/hub", "https://huggingface.co/settings/tokens", "https://huggingface.co/spaces/user", "https://huggingface.co/user/%2Fsecret", "https://example.org/user/repo"] { assert_eq!(model_repo(&Url::parse(url).unwrap()), None); }
    }
    #[test]
    fn browser_bounds_cannot_cover_outside_parent_or_use_nonfinite_sizes() {
        let valid = Bounds { x: 230.0, y: 200.0, width: 700.0, height: 450.0 };
        assert!(checked_bounds(valid, 1000.0, 720.0).is_ok());
        assert!(checked_bounds(Bounds { x: -1.0, ..valid }, 1000.0, 720.0).is_err());
        assert!(checked_bounds(Bounds { width: 2000.0, ..valid }, 1000.0, 720.0).is_err());
        assert!(checked_bounds(Bounds { height: f64::NAN, ..valid }, 1000.0, 720.0).is_err());
    }
    #[test]
    fn privileged_capability_is_scoped_to_local_main_webview_only() {
        let capability: serde_json::Value = serde_json::from_str(include_str!("../capabilities/main-local.json")).unwrap();
        assert_eq!(capability["webviews"], serde_json::json!(["main"]));
        assert!(capability.get("windows").is_none()); assert!(capability.get("remote").is_none()); assert_eq!(capability["local"], true);
    }
}
