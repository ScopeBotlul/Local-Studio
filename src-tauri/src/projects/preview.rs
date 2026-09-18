use super::*;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use tauri::{
    http::{Request, Response},
    Manager,
};

pub(super) fn response(
    project: Option<&Project>,
    label: &str,
    req: Request<Vec<u8>>,
) -> Response<Vec<u8>> {
    let denied = || {
        Response::builder()
            .status(404)
            .header("Cache-Control", "no-store")
            .header("Access-Control-Allow-Origin", "http://tauri.localhost")
            .body(Vec::new())
            .unwrap()
    };
    let Some(p) = project.filter(|p| !p.recovery) else {
        return denied();
    };
    let parts: Vec<_> = req
        .uri()
        .path()
        .trim_start_matches('/')
        .split('/')
        .collect();
    if label != "main" || parts.len() != 2 || parts[0] != p.id {
        return denied();
    }
    let Some(a) = p.assets.iter().find(|a| a.id == parts[1]) else {
        return denied();
    };
    let Ok(path) = owned(p, a) else {
        return denied();
    };
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return denied();
    };
    let Ok(_guards) = gallery::directory_guards(Path::new(&p.directory)) else {
        return denied();
    };
    let Ok(root) = fs::canonicalize(&p.directory) else {
        return denied();
    };
    let (mut parts, body) = req.into_parts();
    let Ok(uri) = format!(
        "/{}/{}",
        gallery::root_id(&root),
        URL_SAFE_NO_PAD.encode(name)
    )
    .parse() else {
        return denied();
    };
    parts.uri = uri;
    gallery::response(&root, label, Request::from_parts(parts, body))
}
pub fn protocol(
    ctx: tauri::UriSchemeContext<'_, tauri::Wry>,
    req: Request<Vec<u8>>,
    responder: tauri::UriSchemeResponder,
) {
    let projects = ctx.app_handle().state::<Arc<Projects>>().inner().clone();
    let label = ctx.webview_label().to_owned();
    tauri::async_runtime::spawn_blocking(move || {
        let result = match projects.state.lock() {
            Ok(s) => response(s.project.as_ref(), &label, req),
            Err(_) => Response::builder().status(503).body(Vec::new()).unwrap(),
        };
        responder.respond(result);
    });
}
