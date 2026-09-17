use crate::hub::{bounded_json, client, send, HubResult, ORIGIN};
use oauth2::{CsrfToken, PkceCodeChallenge};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};
use std::time::{Duration, Instant};
use url::Url;
use zeroize::{Zeroize, Zeroizing};

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub username: String,
    pub display_name: String,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Session {
    access_token: String,
    refresh_token: Option<String>,
    expires_at: Option<i64>,
    method: String,
    account: Account,
}
impl Drop for Session {
    fn drop(&mut self) {
        self.access_token.zeroize();
        self.refresh_token.zeroize();
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStatus {
    pub account: Option<Account>,
    pub method: Option<String>,
    pub pending: bool,
    pub oauth_configured: bool,
    pub expired: bool,
    pub verified: bool,
    pub error: Option<String>,
}

struct State {
    session: Option<Session>,
    pending: bool,
    verified: bool,
    error: Option<String>,
}

pub struct HfAuth {
    state: Mutex<State>,
    refresh_lock: Mutex<()>,
    epoch: AtomicU64,
    credential_user: String,
}

fn client_id() -> Option<String> {
    let config: serde_json::Value =
        serde_json::from_str(include_str!("../huggingface-oauth.json")).ok()?;
    let id = config["clientId"].as_str()?;
    if id.is_empty()
        || id.len() > 256
        || !id
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"-_".contains(&c))
    {
        return None;
    }
    Some(id.into())
}

fn entry(user: &str) -> HubResult<keyring::Entry> {
    keyring::Entry::new("Local Studio Hugging Face", user).map_err(|_| "secret_store".into())
}
fn read_session(user: &str) -> HubResult<Option<Session>> {
    match entry(user)?.get_secret() {
        Ok(bytes) => serde_json::from_slice(&Zeroizing::new(bytes))
            .map(Some)
            .map_err(|_| "secret_store".into()),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(_) => Err("secret_store".into()),
    }
}
fn write_session(user: &str, session: &Session) -> HubResult<()> {
    let bytes = Zeroizing::new(serde_json::to_vec(session).map_err(|_| "secret_store")?);
    entry(user)?
        .set_secret(&bytes)
        .map_err(|_| "secret_store".into())
}
fn delete_session(user: &str) -> HubResult<()> {
    match entry(user)?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(_) => Err("secret_store".into()),
    }
}

#[derive(Deserialize)]
struct Identity {
    name: String,
    #[serde(default)]
    fullname: String,
    #[serde(rename = "type")]
    kind: String,
}
fn identity(token: &str) -> HubResult<Account> {
    let value: Identity = bounded_json(
        send(
            client()?
                .get(format!("{ORIGIN}/api/whoami-v2"))
                .bearer_auth(token),
        )?,
        128 * 1024,
    )?;
    if value.kind != "user"
        || value.name.is_empty()
        || value.name.len() > 200
        || value.fullname.len() > 500
    {
        return Err("invalid_response".into());
    }
    Ok(Account {
        username: value.name,
        display_name: value.fullname,
    })
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
    token_type: String,
}
impl Drop for TokenResponse {
    fn drop(&mut self) {
        self.access_token.zeroize();
        self.refresh_token.zeroize();
    }
}
fn token_response(response: reqwest::blocking::Response) -> HubResult<TokenResponse> {
    let tokens: TokenResponse = bounded_json(response, 64 * 1024)?;
    if !tokens.token_type.eq_ignore_ascii_case("bearer")
        || tokens.access_token.is_empty()
        || tokens.access_token.len() > 2048
    {
        return Err("invalid_response".into());
    }
    Ok(tokens)
}
fn expiry(tokens: &TokenResponse) -> Option<i64> {
    tokens.expires_in.map(|seconds| {
        chrono::Utc::now()
            .timestamp()
            .saturating_add(seconds.min(i64::MAX as u64) as i64)
    })
}

impl HfAuth {
    pub fn new(config: &Path) -> Arc<Self> {
        // Profiles (including isolated tests) never share credentials. No secrets in SQLite.
        let canonical = config
            .canonicalize()
            .unwrap_or_else(|_| config.to_path_buf());
        let user = format!(
            "profile-{:x}",
            Sha256::digest(canonical.to_string_lossy().to_lowercase().as_bytes())
        );
        let (session, error) = match read_session(&user) {
            Ok(session) => (session, None),
            Err(error) => (None, Some(error)),
        };
        Arc::new(Self {
            state: Mutex::new(State {
                session,
                pending: false,
                verified: false,
                error,
            }),
            refresh_lock: Mutex::new(()),
            epoch: AtomicU64::new(0),
            credential_user: user,
        })
    }
    pub fn status(&self) -> HubResult<AuthStatus> {
        let state = self.state.lock().map_err(|_| "internal")?;
        Ok(AuthStatus {
            account: state.session.as_ref().map(|s| s.account.clone()),
            method: state.session.as_ref().map(|s| s.method.clone()),
            pending: state.pending,
            oauth_configured: client_id().is_some(),
            verified: state.verified,
            error: state.error.clone(),
            expired: state
                .session
                .as_ref()
                .and_then(|s| s.expires_at)
                .is_some_and(|at| at <= chrono::Utc::now().timestamp()),
        })
    }
    fn begin(&self) -> HubResult<u64> {
        let mut state = self.state.lock().map_err(|_| "internal")?;
        if state.pending {
            return Err("busy".into());
        }
        if state.session.is_some() {
            return Err("already_connected".into());
        }
        state.pending = true;
        state.error = None;
        Ok(self.epoch.fetch_add(1, Ordering::SeqCst) + 1)
    }
    fn finish(&self, epoch: u64, result: HubResult<Session>) -> HubResult<()> {
        let mut state = self.state.lock().map_err(|_| "internal")?;
        if self.epoch.load(Ordering::SeqCst) != epoch {
            return Err("cancelled".into());
        }
        state.pending = false;
        let result = result.and_then(|session| {
            write_session(&self.credential_user, &session)?;
            Ok(session)
        });
        match result {
            Ok(session) => {
                state.session = Some(session);
                state.verified = true;
                state.error = None;
                Ok(())
            }
            Err(error) => {
                state.error = Some(error.clone());
                Err(error)
            }
        }
    }
    pub fn connect_token(&self, token: String) -> HubResult<AuthStatus> {
        let token = Zeroizing::new(token);
        let token = token.trim();
        if !(16..=2048).contains(&token.len())
            || !token.starts_with("hf_")
            || !token
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || c == b'_')
        {
            return Err("invalid_token".into());
        }
        let epoch = self.begin()?;
        let result = identity(token).map(|account| Session {
            access_token: token.into(),
            refresh_token: None,
            expires_at: None,
            method: "token".into(),
            account,
        });
        self.finish(epoch, result)?;
        self.status()
    }
    pub fn cancel(&self) -> HubResult<AuthStatus> {
        let mut state = self.state.lock().map_err(|_| "internal")?;
        self.epoch.fetch_add(1, Ordering::SeqCst);
        state.pending = false;
        state.error = None;
        drop(state);
        self.status()
    }
    pub fn logout(&self) -> HubResult<AuthStatus> {
        let mut state = self.state.lock().map_err(|_| "internal")?;
        // Hold state through deletion: an old in-flight login can never repopulate it.
        delete_session(&self.credential_user)?;
        self.epoch.fetch_add(1, Ordering::SeqCst);
        state.session = None;
        state.pending = false;
        state.verified = false;
        state.error = None;
        drop(state);
        self.status()
    }
    pub fn token(&self) -> HubResult<Option<Zeroizing<String>>> {
        let _refresh = self.refresh_lock.lock().map_err(|_| "internal")?;
        let (session, epoch) = {
            let state = self.state.lock().map_err(|_| "internal")?;
            (state.session.clone(), self.epoch.load(Ordering::SeqCst))
        };
        let Some(mut session) = session else {
            return Ok(None);
        };
        if session
            .expires_at
            .is_some_and(|at| at <= chrono::Utc::now().timestamp() + 60)
        {
            let refresh = session.refresh_token.as_ref().ok_or("expired")?;
            let id = client_id().ok_or("oauth_not_configured")?;
            let result = send(client()?.post(format!("{ORIGIN}/oauth/token")).form(&[
                ("grant_type", "refresh_token"),
                ("client_id", &id),
                ("refresh_token", refresh),
            ]))
            .and_then(token_response);
            let tokens = match result {
                Ok(tokens) => tokens,
                Err(error) => {
                    let mut state = self.state.lock().map_err(|_| "internal")?;
                    if self.epoch.load(Ordering::SeqCst) == epoch {
                        state.error = Some(error.clone());
                        state.verified = false;
                    }
                    return Err(error);
                }
            };
            session.access_token.zeroize();
            session.access_token = tokens.access_token.clone();
            if tokens.refresh_token.is_some() {
                session.refresh_token.zeroize();
                session.refresh_token = tokens.refresh_token.clone();
            }
            session.expires_at = expiry(&tokens);
            let mut state = self.state.lock().map_err(|_| "internal")?;
            if self.epoch.load(Ordering::SeqCst) != epoch {
                return Err("cancelled".into());
            }
            write_session(&self.credential_user, &session)?;
            state.session = Some(session.clone());
            state.error = None;
        }
        Ok(Some(Zeroizing::new(session.access_token.clone())))
    }
    pub fn verify(&self) -> HubResult<AuthStatus> {
        let epoch = self.epoch.load(Ordering::SeqCst);
        let token = self.token()?.ok_or("not_connected")?;
        let result = identity(&token);
        let mut state = self.state.lock().map_err(|_| "internal")?;
        if self.epoch.load(Ordering::SeqCst) != epoch {
            return Err("cancelled".into());
        }
        match result {
            Ok(account) => {
                if let Some(session) = state.session.as_mut() {
                    session.account = account;
                    write_session(&self.credential_user, session)?;
                }
                state.verified = true;
                state.error = None;
            }
            Err(error) => {
                state.verified = false;
                state.error = Some(error.clone());
                return Err(error);
            }
        }
        drop(state);
        self.status()
    }
    pub fn pending_generation(&self, epoch: u64) -> bool {
        self.state.lock().is_ok_and(|state| state.pending && self.epoch.load(Ordering::SeqCst) == epoch)
    }
    pub fn start_login(self: &Arc<Self>, app: tauri::AppHandle) -> HubResult<AuthStatus> {
        let id = client_id().ok_or("oauth_not_configured")?;
        let epoch = self.begin()?;
        let result = (|| {
            let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
                .map_err(|_| "callback_bind")?;
            listener
                .set_nonblocking(true)
                .map_err(|_| "callback_bind")?;
            let authority = format!(
                "127.0.0.1:{}",
                listener.local_addr().map_err(|_| "callback_bind")?.port()
            );
            let redirect = format!("http://{authority}/callback");
            let (challenge, verifier) = PkceCodeChallenge::new_random_sha256();
            let state = CsrfToken::new_random();
            let mut url = Url::parse(&format!("{ORIGIN}/oauth/authorize")).unwrap();
            url.query_pairs_mut()
                .append_pair("client_id", &id)
                .append_pair("redirect_uri", &redirect)
                .append_pair("response_type", "code")
                .append_pair("scope", "openid profile read-repos")
                .append_pair("state", state.secret())
                .append_pair("code_challenge", challenge.as_str())
                .append_pair("code_challenge_method", "S256");
            crate::hf_browser::start_login(&app, epoch, url, Url::parse(&redirect).map_err(|_| "invalid_callback")?)?;
            let auth = Arc::clone(self);
            std::thread::spawn(move || {
                let result = wait_callback(&listener, &authority, state.secret(), || {
                    auth.epoch.load(Ordering::SeqCst) != epoch
                })
                .and_then(|code| {
                    let code = Zeroizing::new(code);
                    if auth.epoch.load(Ordering::SeqCst) != epoch {
                        return Err("cancelled".into());
                    }
                    let tokens = token_response(send(
                        client()?.post(format!("{ORIGIN}/oauth/token")).form(&[
                            ("grant_type", "authorization_code"),
                            ("client_id", &id),
                            ("code", &code),
                            ("redirect_uri", &redirect),
                            ("code_verifier", verifier.secret()),
                        ]),
                    )?)?;
                    let account = identity(&tokens.access_token)?;
                    Ok(Session {
                        access_token: tokens.access_token.clone(),
                        refresh_token: tokens.refresh_token.clone(),
                        expires_at: expiry(&tokens),
                        method: "oauth".into(),
                        account,
                    })
                });
                let _ = auth.finish(epoch, result);
                crate::hf_browser::finish_login(&app, epoch);
            });
            Ok(())
        })();
        if let Err(error) = result {
            self.finish(epoch, Err(error))?;
        }
        self.status()
    }
}

fn parse_callback(request: &str, authority: &str, expected_state: &str) -> HubResult<String> {
    let mut lines = request.split("\r\n");
    let first = lines.next().ok_or("invalid_callback")?;
    let parts: Vec<_> = first.split(' ').collect();
    if parts.len() != 3
        || parts[0] != "GET"
        || parts[2] != "HTTP/1.1"
        || !parts[1].starts_with("/callback?")
    {
        return Err("invalid_callback".into());
    }
    let hosts: Vec<_> = lines
        .filter_map(|line| line.split_once(':'))
        .filter(|(key, _)| key.eq_ignore_ascii_case("host"))
        .collect();
    if hosts.len() != 1 || hosts[0].1.trim() != authority {
        return Err("invalid_callback".into());
    }
    let url =
        Url::parse(&format!("http://{authority}{}", parts[1])).map_err(|_| "invalid_callback")?;
    if url.path() != "/callback" || url.fragment().is_some() {
        return Err("invalid_callback".into());
    }
    let values = |key: &str| {
        url.query_pairs()
            .filter(|(name, _)| name == key)
            .map(|(_, value)| value.into_owned())
            .collect::<Vec<_>>()
    };
    let states = values("state");
    if states.len() != 1 || states[0] != expected_state {
        return Err("invalid_callback".into());
    }
    if !values("error").is_empty() {
        return Err("authorization_denied".into());
    }
    let codes = values("code");
    if codes.len() != 1 || codes[0].is_empty() || codes[0].len() > 4096 {
        return Err("invalid_callback".into());
    }
    Ok(codes[0].clone())
}

fn respond(stream: &mut TcpStream, success: bool) {
    let body = if success {
        "<!doctype html><meta charset=utf-8><title>Local Studio</title><h1>Local Studio</h1><p>Antwort empfangen. Bitte zu Local Studio zurueckkehren. / Response received. Return to Local Studio to check your connection.</p>"
    } else {
        "<!doctype html><meta charset=utf-8><title>Local Studio</title><p>Invalid or declined login callback. Return to Local Studio.</p>"
    };
    let status = if success { "200 OK" } else { "400 Bad Request" };
    let _ = write!(stream, "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nContent-Security-Policy: default-src 'none'; frame-ancestors 'none'\r\nReferrer-Policy: no-referrer\r\nConnection: close\r\n\r\n{body}", body.len());
}

fn wait_callback(
    listener: &TcpListener,
    authority: &str,
    state: &str,
    cancelled: impl Fn() -> bool,
) -> HubResult<String> {
    let deadline = Instant::now() + Duration::from_secs(180);
    while Instant::now() < deadline {
        if cancelled() {
            return Err("cancelled".into());
        }
        match listener.accept() {
            Ok((mut stream, peer)) => {
                if !peer.ip().is_loopback() {
                    continue;
                }
                let _ = stream.set_read_timeout(Some(Duration::from_millis(250)));
                let _ = stream.set_write_timeout(Some(Duration::from_millis(250)));
                let mut bytes = Zeroizing::new(Vec::new());
                let until = Instant::now() + Duration::from_secs(2);
                let mut chunk = [0u8; 1024];
                while bytes.len() < 8192 && Instant::now() < until && !cancelled() {
                    match stream.read(&mut chunk) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => bytes.extend_from_slice(&chunk[..n]),
                    }
                    if bytes.windows(4).any(|w| w == b"\r\n\r\n") {
                        break;
                    }
                }
                chunk.zeroize();
                let result = std::str::from_utf8(&bytes)
                    .map_err(|_| "invalid_callback".into())
                    .and_then(|request| {
                        if !request.ends_with("\r\n\r\n") {
                            return Err("invalid_callback".into());
                        }
                        parse_callback(request, authority, state)
                    });
                respond(&mut stream, result.is_ok());
                match result {
                    Ok(code) => return Ok(code),
                    Err(error) if error == "authorization_denied" => return Err(error),
                    _ => {}
                }
            }
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(50))
            }
            Err(_) => return Err("callback_bind".into()),
        }
    }
    Err("login_timeout".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn callback_requires_matching_host_state_and_single_code() {
        let valid = "GET /callback?state=secret&code=abc HTTP/1.1\r\nHost: 127.0.0.1:42\r\n\r\n";
        assert_eq!(
            parse_callback(valid, "127.0.0.1:42", "secret").unwrap(),
            "abc"
        );
        for invalid in [
            valid.replace("state=secret", "state=wrong"),
            valid.replace("code=abc", "code=abc&code=def"),
            valid.replace("state=secret", "state=secret&state=secret"),
            valid.replace("Host: 127.0.0.1:42", "Host: evil.com"),
            valid.replace("GET ", "POST "),
        ] {
            assert!(parse_callback(&invalid, "127.0.0.1:42", "secret").is_err());
        }
        let denial = valid.replace("code=abc", "error=access_denied");
        assert_eq!(
            parse_callback(&denial, "127.0.0.1:42", "secret").unwrap_err(),
            "authorization_denied"
        );
    }
    #[test]
    fn real_loopback_ignores_forged_callback_then_accepts_valid_one() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = std::thread::spawn(move || {
            wait_callback(&listener, &addr.to_string(), "expected", || false)
        });
        for state in ["forged", "expected"] {
            let mut stream = TcpStream::connect(addr).unwrap();
            write!(
                stream,
                "GET /callback?state={state}&code=only-code HTTP/1.1\r\nHost: {addr}\r\n\r\n"
            )
            .unwrap();
            let mut response = String::new();
            stream.read_to_string(&mut response).unwrap();
            assert!(response.starts_with(if state == "expected" {
                "HTTP/1.1 200"
            } else {
                "HTTP/1.1 400"
            }));
        }
        assert_eq!(handle.join().unwrap().unwrap(), "only-code");
    }
    #[test]
    fn windows_secret_store_roundtrip_and_cancelled_login_cannot_restore_account() {
        let dir = tempfile::tempdir().unwrap();
        let auth = HfAuth::new(dir.path());
        assert!(auth.status().unwrap().account.is_none());
        let make_session = || Session {
            access_token: "synthetic-not-a-real-token".into(),
            refresh_token: None,
            expires_at: None,
            method: "test".into(),
            account: Account {
                username: "test".into(),
                display_name: "Test".into(),
            },
        };
        let epoch = auth.begin().unwrap();
        auth.cancel().unwrap();
        assert_eq!(
            auth.finish(epoch, Ok(make_session())).unwrap_err(),
            "cancelled"
        );
        assert!(read_session(&auth.credential_user).unwrap().is_none());
        let epoch = auth.begin().unwrap();
        auth.finish(epoch, Ok(make_session())).unwrap();
        let reopened = HfAuth::new(dir.path());
        assert_eq!(
            reopened.token().unwrap().unwrap().as_str(),
            "synthetic-not-a-real-token"
        );
        assert!(!serde_json::to_string(&reopened.status().unwrap())
            .unwrap()
            .contains("synthetic"));
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
        reopened.logout().unwrap();
        assert!(read_session(&auth.credential_user).unwrap().is_none());
    }
}
