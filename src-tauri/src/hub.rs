use reqwest::blocking::{Client, Response};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use std::io::Read;
use std::time::Duration;
use url::Url;

pub const ORIGIN: &str = "https://huggingface.co";
pub type HubResult<T> = Result<T, String>;

pub fn client() -> HubResult<Client> {
    Client::builder()
        .user_agent(concat!("Local-Studio/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        // Never forward credentials through a redirect, including same-host redirects.
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| "network".into())
}

pub fn send(request: reqwest::blocking::RequestBuilder) -> HubResult<Response> {
    let response = request.send().map_err(|_| "network".to_string())?;
    match response.status().as_u16() {
        200..=299 => Ok(response),
        401 => Err("unauthorized".into()),
        403 => Err("forbidden".into()),
        404 => Err("not_found".into()),
        429 => Err("rate_limited".into()),
        _ => Err("service".into()),
    }
}

pub fn bounded_json<T: DeserializeOwned>(response: Response, maximum: u64) -> HubResult<T> {
    let bytes = bounded_bytes(response, maximum)?;
    serde_json::from_slice(&bytes).map_err(|_| "invalid_response".into())
}

pub fn bounded_bytes(response: Response, maximum: u64) -> HubResult<zeroize::Zeroizing<Vec<u8>>> {
    if response
        .content_length()
        .is_some_and(|length| length > maximum)
    {
        return Err("response_too_large".into());
    }
    let mut bytes = zeroize::Zeroizing::new(Vec::new());
    response
        .take(maximum + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "network".to_string())?;
    if bytes.len() as u64 > maximum {
        return Err("response_too_large".into());
    }
    Ok(bytes)
}

pub fn validate_repo(repo: &str) -> HubResult<()> {
    let parts: Vec<_> = repo.split('/').collect();
    if !(1..=2).contains(&parts.len())
        || parts.iter().any(|part| {
            part.is_empty()
                || part.len() > 96
                || part.starts_with(['.', '-'])
                || part.ends_with(['.', '-'])
                || part.contains("..")
                || part.contains("--")
                || part.ends_with(".git")
                || !part
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || b"-_.".contains(&c))
        })
    {
        return Err("invalid_repo".into());
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchQuery {
    pub search: String,
    pub task: String,
    pub sort: String,
    pub cursor: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModelSummary {
    #[serde(default)] pub restricted:bool,
    pub id: String,
    pub task: Option<String>,
    pub library: Option<String>,
    pub downloads: u64,
    pub likes: u64,
    pub gated: bool,
    pub private: bool,
    pub license: Option<String>,
    pub revision: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchPage {
    pub models: Vec<ModelSummary>,
    pub next_cursor: Option<String>,
}

fn summary(v: &Value) -> HubResult<ModelSummary> {
    let id = v["id"].as_str().ok_or("invalid_response")?;
    validate_repo(id)?;
    Ok(ModelSummary {
        restricted:v["tags"].as_array().is_some_and(|tags|tags.iter().filter_map(Value::as_str).any(|tag|matches!(tag.to_ascii_lowercase().as_str(),"nsfw"|"18+"|"adult"|"not-for-all-audiences"))),
        id: id.into(),
        revision: v["sha"].as_str().map(str::to_owned),
        task: v["pipeline_tag"].as_str().map(str::to_owned),
        library: v["library_name"].as_str().map(str::to_owned),
        downloads: v["downloads"].as_u64().unwrap_or(0),
        likes: v["likes"].as_u64().unwrap_or(0),
        gated: v["gated"] == true || v["gated"].is_string(),
        private: v["private"] == true,
        license: v["cardData"]["license"]
            .as_str()
            .map(str::to_owned)
            .or_else(|| {
                v["tags"]
                    .as_array()?
                    .iter()
                    .filter_map(Value::as_str)
                    .find_map(|tag| tag.strip_prefix("license:").map(str::to_owned))
            }),
    })
}

fn search_url(query: &SearchQuery) -> HubResult<Url> {
    if query.search.len() > 200 || query.search.chars().any(char::is_control) {
        return Err("invalid_query".into());
    }
    if ![
        "",
        "text-to-image",
        "image-to-image",
        "text-to-video",
        "image-to-video",
        "text-generation",
        "image-text-to-text",
        "text-to-audio",
        "automatic-speech-recognition",
        "image-text-to-image",
        "mask-generation",
        "image-text-to-video",
        "video-to-video",
        "audio-to-audio",
        "text-to-speech",
        "image-to-text",
        "video-text-to-text",
        "image-segmentation",
        "depth-estimation",
        "object-detection",
        "audio-classification",
    ]
    .contains(&query.task.as_str())
    {
        return Err("invalid_query".into());
    }
    if !["downloads", "likes", "lastModified", "trendingScore"].contains(&query.sort.as_str()) {
        return Err("invalid_query".into());
    }
    let mut url = Url::parse(&format!("{ORIGIN}/api/models")).unwrap();
    url.query_pairs_mut()
        .append_pair("search", query.search.trim())
        .append_pair("sort", &query.sort)
        .append_pair("direction", "-1")
        .append_pair("limit", "20")
        .append_pair("full", "true");
    if !query.task.is_empty() {
        url.query_pairs_mut()
            .append_pair("pipeline_tag", &query.task);
    }
    if let Some(cursor) = &query.cursor {
        if cursor.len() > 8192 || cursor.chars().any(char::is_control) {
            return Err("invalid_query".into());
        }
        url.query_pairs_mut().append_pair("cursor", cursor);
    }
    Ok(url)
}

fn next_cursor(link: &str) -> Option<String> {
    for part in link.split(',') {
        let (target, attributes) = part.trim().split_once('>')?;
        if attributes
            .split(';')
            .any(|attribute| attribute.trim() == "rel=\"next\"")
        {
            let url = Url::parse(target.strip_prefix('<')?).ok()?;
            if url.scheme() != "https"
                || url.host_str() != Some("huggingface.co")
                || url.port().is_some()
                || !url.username().is_empty()
                || url.password().is_some()
                || url.path() != "/api/models"
            {
                return None;
            }
            return url
                .query_pairs()
                .find(|(key, value)| key == "cursor" && value.len() <= 8192)
                .map(|(_, value)| value.into_owned());
        }
    }
    None
}

pub fn search(query: SearchQuery, token: Option<&str>) -> HubResult<SearchPage> {
    let mut request = client()?.get(search_url(&query)?);
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }
    let response = send(request)?;
    let cursor = response
        .headers()
        .get("link")
        .and_then(|header| header.to_str().ok())
        .and_then(next_cursor);
    let values: Vec<Value> = bounded_json(response, 4 * 1024 * 1024)?;
    Ok(SearchPage {
        models: values
            .iter()
            .take(20)
            .map(summary)
            .collect::<HubResult<_>>()?,
        next_cursor: cursor,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelFile {
    pub path: String,
    pub size: Option<u64>,
    pub sha256: Option<String>,
    pub git_sha1: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelDetail {
    pub model: ModelSummary,
    pub revision: String,
    pub files: Vec<ModelFile>,
    pub card: Option<String>,
    pub card_error: Option<String>,
}

pub fn detail(repo: &str, revision: &str, token: Option<&str>) -> HubResult<ModelDetail> {
    detail_metadata(repo, revision, token, true)
}

pub(crate) fn detail_metadata(repo: &str, revision: &str, token: Option<&str>, include_card: bool) -> HubResult<ModelDetail> {
    validate_repo(repo)?;
    if revision.is_empty()
        || [".", ".."].contains(&revision)
        || revision.len() > 200
        || revision.chars().any(char::is_control)
    {
        return Err("invalid_revision".into());
    }
    let http = client()?;
    let mut url = Url::parse(&format!("{ORIGIN}/api/models/")).unwrap();
    {
        let mut path = url.path_segments_mut().unwrap();
        path.pop_if_empty();
        for part in repo.split('/') {
            path.push(part);
        }
        path.push("revision").push(revision);
    }
    url.query_pairs_mut().append_pair("blobs", "true");
    let authorized = |url: Url| {
        let request = http.get(url);
        if let Some(token) = token {
            request.bearer_auth(token)
        } else {
            request
        }
    };
    let v: Value = bounded_json(send(authorized(url))?, 4 * 1024 * 1024)?;
    let sha = v["sha"]
        .as_str()
        .filter(|sha| sha.len() == 40 && sha.bytes().all(|c| c.is_ascii_hexdigit()))
        .ok_or("invalid_response")?;
    let files = v["siblings"]
        .as_array()
        .ok_or("invalid_response")?
        .iter()
        .filter_map(|file| {
            Some(ModelFile {
                path: file["rfilename"].as_str()?.into(),
                size: file["size"]
                    .as_u64()
                    .or_else(|| file["lfs"]["size"].as_u64()),
                sha256: file["lfs"]["sha256"].as_str().map(str::to_owned),
                git_sha1: file["blobId"].as_str().map(str::to_owned),
            })
        })
        .collect();
    let card_url = Url::parse(&format!("{ORIGIN}/{repo}/raw/{sha}/README.md")).unwrap();
    let card = if include_card { send(authorized(card_url))
        .and_then(|response| bounded_bytes(response, 512 * 1024))
        .and_then(|bytes| String::from_utf8(bytes.to_vec()).map_err(|_| "invalid_response".into())) } else { Ok(String::new()) };
    let (card, card_error) = match card {
        Ok(card) => (Some(card), None),
        Err(error) => (None, Some(error)),
    };
    Ok(ModelDetail {
        model: summary(&v)?,
        revision: sha.into(),
        files,
        card,
        card_error,
    })
}

fn total_file_bytes(files: &[ModelFile]) -> Option<u64> {
    if files.is_empty() { return None; }
    files.iter().try_fold(0u64, |sum, file| sum.checked_add(file.size?))
}

pub fn model_size(repo: &str, revision: &str, token: Option<&str>) -> HubResult<Option<u64>> {
    let metadata = detail_metadata(repo, revision, token, false)?;
    Ok(total_file_bytes(&metadata.files))
}

// Only a prevalidated HTTPS URL reaches the Windows URL association. No shell command or PATH lookup.
pub fn open_browser(target: &str) -> HubResult<()> {
    let url = Url::parse(target).map_err(|_| "invalid_link")?;
    if url.scheme() != "https"
        || url.host_str() != Some("huggingface.co")
        || url.port().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
        || target.contains('\0')
    {
        return Err("invalid_link".into());
    }
    open_external_https(target)
}

// Native browser controller only; no IPC accepts arbitrary URLs for this helper.
pub fn open_external_https(target: &str) -> HubResult<()> {
    let url = Url::parse(target).map_err(|_| "invalid_link")?;
    if url.scheme() != "https" || url.host_str().is_none() || !url.username().is_empty()
        || url.password().is_some() || target.len() > 8192 || target.chars().any(char::is_control) {
        return Err("invalid_link".into());
    }
    let verb: Vec<u16> = "open\0".encode_utf16().collect();
    let target: Vec<u16> = target.encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: NUL-terminated UTF-16 buffers live across the synchronous call; all optional pointers are null.
    unsafe {
        use windows_sys::Win32::{
            System::Com::{
                CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE,
            },
            UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL},
        };
        let initialized = CoInitializeEx(
            std::ptr::null(),
            (COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE) as u32,
        );
        let result = ShellExecuteW(
            std::ptr::null_mut(),
            verb.as_ptr(),
            target.as_ptr(),
            std::ptr::null(),
            std::ptr::null(),
            SW_SHOWNORMAL,
        );
        if initialized >= 0 {
            CoUninitialize();
        }
        if result as isize <= 32 {
            return Err("browser".into());
        }
    }
    Ok(())
}

pub fn open_page(page: &str, repo: Option<&str>) -> HubResult<()> {
    let target = match page {
        "tokens" => format!("{ORIGIN}/settings/tokens"),
        "applications" => format!("{ORIGIN}/settings/connected-applications"),
        "home" => ORIGIN.into(),
        "model" => {
            let repo = repo.ok_or("invalid_repo")?;
            validate_repo(repo)?;
            format!("{ORIGIN}/{repo}")
        }
        _ => return Err("invalid_link".into()),
    };
    open_browser(&target)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn search_task_filters_are_sent_to_hugging_face_with_pagination() {
        for task in ["text-to-image", "image-to-image", "image-text-to-image", "mask-generation", "text-to-video", "image-to-video", "image-text-to-video", "video-to-video", "text-to-audio", "audio-to-audio", "automatic-speech-recognition", "text-to-speech", "text-generation", "image-text-to-text", "image-to-text", "video-text-to-text", "image-segmentation", "depth-estimation", "object-detection", "audio-classification"] {
            let url=search_url(&SearchQuery{search:"model & test".into(),task:task.into(),sort:"downloads".into(),cursor:Some("next+page".into())}).unwrap();
            assert_eq!(url.host_str(),Some("huggingface.co"));
            assert!(url.query_pairs().any(|(key,value)|key=="pipeline_tag"&&value==task));
            assert!(url.query_pairs().any(|(key,value)|key=="cursor"&&value=="next+page"));
            assert!(url.query_pairs().any(|(key,value)|key=="search"&&value=="model & test"));
        }
    }
    #[test]
    fn search_task_filters_reject_unknown_or_injected_tags() {
        for task in ["video", "arbitrary-code", "text-to-image&token=secret", "https://example.com"] {
            assert_eq!(search_url(&SearchQuery{search:String::new(),task:task.into(),sort:"downloads".into(),cursor:None}).unwrap_err(),"invalid_query");
        }
    }
    fn response_server(
        status: &str,
        extra: &str,
        body: &str,
    ) -> (String, std::thread::JoinHandle<()>) {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/", listener.local_addr().unwrap());
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Length: {}\r\n{extra}Connection: close\r\n\r\n{body}",
            body.len()
        );
        let handle = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            let mut request = [0; 2048];
            let _ = socket.read(&mut request);
            let _ = socket.write_all(response.as_bytes());
        });
        (url, handle)
    }
    #[test]
    fn http_errors_never_echo_remote_body_or_credentials() {
        let (url, handle) = response_server("401 Unauthorized", "", "hf_SECRET_ECHO");
        let error = send(client().unwrap().get(url).bearer_auth("synthetic"))
            .err()
            .unwrap();
        assert_eq!(error, "unauthorized");
        handle.join().unwrap();
    }
    #[test]
    fn response_limits_apply_before_parsing_remote_data() {
        let (url, handle) = response_server("200 OK", "", "1234567890");
        let response = send(client().unwrap().get(url)).unwrap();
        assert_eq!(
            bounded_json::<Value>(response, 3).unwrap_err(),
            "response_too_large"
        );
        handle.join().unwrap();
    }
    #[test]
    fn authenticated_client_refuses_redirects() {
        let (url, handle) = response_server(
            "302 Found",
            "Location: https://example.invalid/collect\r\n",
            "",
        );
        assert_eq!(
            send(client().unwrap().get(url).bearer_auth("synthetic"))
                .err()
                .unwrap(),
            "service"
        );
        handle.join().unwrap();
    }
    #[test]
    fn repository_and_query_cannot_redirect_requests() {
        for repo in [
            "../secret",
            "https://evil.com",
            "user/repo/extra",
            "foo\\bar",
            "a/%2e%2e",
            "a/repo.git",
            "a/--repo",
        ] {
            assert!(validate_repo(repo).is_err(), "{repo}");
        }
        assert!(validate_repo("openai-community/gpt2").is_ok());
        let url = search_url(&SearchQuery {
            search: "a&token=secret".into(),
            task: "".into(),
            sort: "downloads".into(),
            cursor: Some("https://evil.com".into()),
        })
        .unwrap();
        assert_eq!(url.host_str(), Some("huggingface.co"));
        assert!(!url.query_pairs().any(|(key, _)| key == "token"));
    }
    #[test]
    fn pagination_only_extracts_cursor_from_official_endpoint() {
        assert_eq!(
            next_cursor("<https://huggingface.co/api/models?cursor=abc%2Bdef>; rel=\"next\""),
            Some("abc+def".into())
        );
        for link in [
            "<https://evil.com/api/models?cursor=x>; rel=\"next\"",
            "<http://huggingface.co/api/models?cursor=x>; rel=\"next\"",
            "<https://huggingface.co/api/whoami-v2?cursor=x>; rel=\"next\"",
        ] {
            assert_eq!(next_cursor(link), None);
        }
    }
    #[test]
    fn model_size_requires_complete_metadata_and_checked_sum() {
        let file = |size| ModelFile { path: "file".into(), size, sha256: None, git_sha1: None };
        assert_eq!(total_file_bytes(&[]), None);
        assert_eq!(total_file_bytes(&[file(Some(0))]), Some(0));
        assert_eq!(total_file_bytes(&[file(Some(453864)), file(Some(807))]), Some(454671));
        assert_eq!(total_file_bytes(&[file(Some(10)), file(None)]), None);
        assert_eq!(total_file_bytes(&[file(Some(u64::MAX)), file(Some(1))]), None);
    }

    #[test]
    fn parses_license_and_gated_metadata_without_claiming_compatibility() {
        let v = serde_json::json!({"id":"org/model", "gated":"manual", "tags":["license:apache-2.0"], "downloads":42});
        let model = summary(&v).unwrap();
        assert!(model.gated);
        assert_eq!(model.license.as_deref(), Some("apache-2.0"));
        assert_eq!(model.downloads, 42);
    }
}
