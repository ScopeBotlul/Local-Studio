use crate::{hf_auth::HfAuth, hub};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{collections::HashMap, fs::{self, File, OpenOptions}, io::{Read, Write}, path::{Path, PathBuf}, sync::{Arc, Mutex}, time::{Duration, Instant}};
use tokio::sync::watch;
use url::Url;
mod updates;
pub use updates::*;

type Result<T> = std::result::Result<T, String>;
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadFile { pub path: String, pub size: u64, pub sha256: Option<String>, pub git_sha1: Option<String>, pub downloaded: u64, pub actual_sha256: Option<String> }
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Download {
    pub id: String, pub repo: String, pub revision: String, pub license: Option<String>, pub task: Option<String>, pub files: Vec<DownloadFile>,
    pub destination: String, pub partial_directory: String, pub status: String, pub priority: i32, pub total_bytes: u64, pub downloaded_bytes: u64,
    #[cfg(test)] #[serde(skip)] pub test_url: Option<String>,
    pub bytes_per_second: u64, pub error: Option<String>, pub created_at: String, pub verify_only: bool,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan { pub id: String, pub download: Download, pub additional_bytes: u64, pub available_bytes: u64 }
struct State { db: Connection, jobs: Vec<Download>, plans: HashMap<String, (Instant, Plan)>, active: Option<(String, watch::Sender<bool>)>, stopped: bool }
pub struct Downloads { state: Mutex<State>, auth: Arc<HfAuth>, updates: Mutex<UpdateStatus> }

fn save(db: &Connection, job: &Download) -> Result<()> {
    db.execute("INSERT INTO downloads(id,json) VALUES(?1,?2) ON CONFLICT(id) DO UPDATE SET json=excluded.json", params![job.id, serde_json::to_string(job).map_err(|_| "download_storage")?]).map_err(|_| "download_storage")?; Ok(())
}
fn stopped(cancel: &watch::Receiver<bool>) -> Result<()> { if *cancel.borrow() { Err("download_interrupted".into()) } else { Ok(()) } }

pub fn valid_file(path: &str) -> bool {
    !path.is_empty() && path.len() <= 220 && path.split('/').all(|part| {
        let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
        !part.is_empty() && ![".", ".."].contains(&part) && !part.ends_with(['.', ' '])
            && !part.chars().any(|c| c.is_control() || "\\:*?\"<>|".contains(c))
            && !["CON", "PRN", "AUX", "NUL", "CONIN$", "CONOUT$"].contains(&stem.as_str())
            && !(stem.starts_with("COM") || stem.starts_with("LPT")) .then(|| &stem[3..]).is_some_and(|n| ["1","2","3","4","5","6","7","8","9","¹","²","³"].contains(&n))
    })
}
fn regular(path: &Path, directory: bool) -> Result<()> {
    let meta = fs::symlink_metadata(path).map_err(|_| "download_storage")?;
    #[cfg(windows)] { use std::os::windows::fs::MetadataExt; if meta.file_attributes() & 0x400 != 0 { return Err("download_unsafe_path".into()); } }
    if meta.file_type().is_symlink() || (directory && !meta.is_dir()) || (!directory && !meta.is_file()) { return Err("download_unsafe_path".into()); } Ok(())
}
fn directory(path: &Path) -> Result<()> {
    if path.exists() {
        for ancestor in path.ancestors() { if !ancestor.as_os_str().is_empty() { regular(ancestor, true)?; } }
        return Ok(());
    }
    let parent = path.parent().ok_or("download_unsafe_path")?;
    directory(parent)?;
    fs::create_dir(path).map_err(|_| "download_storage")?; regular(path, true)
}
fn within(root: &Path, relative: &str) -> Result<PathBuf> {
    if !valid_file(relative) { return Err("download_unsafe_path".into()); }
    directory(root)?;
    let parts: Vec<_> = relative.split('/').collect(); let mut path = root.to_path_buf();
    for part in &parts[..parts.len()-1] { path.push(part); directory(&path)?; }
    path.push(parts.last().unwrap());
    if path.exists() { regular(&path, false)?; } Ok(path)
}
fn transfer_host(url: &Url) -> bool {
    #[cfg(test)] if url.scheme()=="http" && url.host_str()==Some("127.0.0.1") { return true; }
    url.scheme() == "https" && url.port().is_none() && url.username().is_empty() && url.password().is_none()
        && url.host_str().is_some_and(|host| host == "huggingface.co" || host.ends_with(".huggingface.co") || host == "hf.co" || host.ends_with(".hf.co"))
}
fn resolve_url(job: &Download, file: &DownloadFile) -> Url {
    #[cfg(test)] if let Some(url)=&job.test_url {return Url::parse(url).unwrap();}
    let mut url = Url::parse(hub::ORIGIN).unwrap();
    { let mut path = url.path_segments_mut().unwrap(); for part in job.repo.split('/') { path.push(part); } path.push("resolve").push(&job.revision); for part in file.path.split('/') { path.push(part); } }
    url
}
fn range_start(value: &str, offset: u64, size: u64) -> bool {
    value.strip_prefix("bytes ").and_then(|v| v.split_once('/')).and_then(|(range,total)| Some((range.split_once('-')?,total)))
        .is_some_and(|((start,end),total)| start.parse::<u64>().ok() == Some(offset) && total.parse::<u64>().ok() == Some(size) && end.parse::<u64>().ok() == size.checked_sub(1))
}
async fn response(http: &reqwest::Client, mut url: Url, offset: u64, token: Option<&str>, cancel: &mut watch::Receiver<bool>) -> Result<reqwest::Response> {
    for _ in 0..10 {
        stopped(cancel)?;
        if !transfer_host(&url) { return Err("download_redirect".into()); }
        let mut request = http.get(url.clone()).header("Accept-Encoding", "identity");
        if offset > 0 { request = request.header("Range", format!("bytes={offset}-")); }
        if url.host_str() == Some("huggingface.co") { if let Some(token) = token { request = request.bearer_auth(token); } }
        let reply = tokio::select! { biased; _ = cancel.changed() => return Err("download_interrupted".into()), reply = request.send() => reply.map_err(|_| "network")? };
        if reply.status().is_redirection() {
            url = url.join(reply.headers().get("location").and_then(|v|v.to_str().ok()).ok_or("download_redirect")?).map_err(|_| "download_redirect")?;
            continue;
        }
        return match reply.status().as_u16() { 200 | 206 => Ok(reply), 401 => Err("unauthorized".into()), 403 => Err("forbidden".into()), 404 => Err("not_found".into()), 429 => Err("rate_limited".into()), _ => Err("download_http".into()) };
    }
    Err("download_redirect".into())
}
fn hash_file(path: &Path, file: &DownloadFile, cancel: &watch::Receiver<bool>) -> Result<String> {
    regular(path, false)?;
    let mut input = File::open(path).map_err(|_| "download_storage")?;
    if input.metadata().map_err(|_| "download_storage")?.len() != file.size { return Err("download_size".into()); }
    let mut sha = Sha256::new(); let mut git = sha1::Sha1::new(); git.update(format!("blob {}\0", file.size).as_bytes());
    let mut buffer = [0u8; 128*1024]; let mut size = 0u64;
    loop { stopped(cancel)?; let n = input.read(&mut buffer).map_err(|_| "download_storage")?; if n == 0 { break; } size += n as u64; sha.update(&buffer[..n]); git.update(&buffer[..n]); }
    let digest = format!("{:x}", sha.finalize());
    if size != file.size || file.actual_sha256.as_ref().is_some_and(|expected| !digest.eq_ignore_ascii_case(expected)) || file.sha256.as_ref().is_some_and(|expected| !digest.eq_ignore_ascii_case(expected))
        || (file.sha256.is_none() && file.git_sha1.as_ref().is_some_and(|expected| !format!("{:x}",git.finalize()).eq_ignore_ascii_case(expected))) { return Err("download_hash".into()); }
    Ok(digest)
}

impl Downloads {
    pub fn new(config: &Path, auth: Arc<HfAuth>) -> Result<Arc<Self>> {
        let db = Connection::open(config.join("downloads.sqlite3")).map_err(|_| "download_storage")?;
        db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; CREATE TABLE IF NOT EXISTS downloads(id TEXT PRIMARY KEY,json TEXT NOT NULL);").map_err(|_| "download_storage")?;
        let mut jobs: Vec<Download> = {
            let mut query = db.prepare("SELECT json FROM downloads ORDER BY rowid DESC").map_err(|_| "download_storage")?;
            let rows = query.query_map([], |row| row.get::<_,String>(0)).map_err(|_| "download_storage")?;
            rows.map(|value| serde_json::from_str(&value.map_err(|_| "download_storage")?).map_err(|_| "download_storage".to_string())).collect::<Result<_>>()?
        };
        for job in &mut jobs {
            if ["queued","downloading","verifying","installing","pausing","cancelling"].contains(&job.status.as_str()) { job.status = "paused".into(); job.error = Some("download_recovered".into()); }
            job.bytes_per_second = 0; save(&db, job)?;
        }
        let updates = Self::load_updates(&db)?;
        Ok(Arc::new(Self { state: Mutex::new(State { db, jobs, plans: HashMap::new(), active: None, stopped: false }), auth, updates: Mutex::new(updates) }))
    }
    pub fn list(&self) -> Result<Vec<Download>> { Ok(self.state.lock().map_err(|_| "internal")?.jobs.clone()) }
    pub fn plan(&self, repo: String, revision: String, files: Vec<String>, paths: crate::types::StoragePaths) -> Result<Plan> {
        if files.is_empty() || files.len()>2048 { return Err("download_selection".into()); }
        let token = self.auth.token()?;
        let details = hub::detail(&repo, &revision, token.as_ref().map(|v|v.as_str()))?;
        let mut selected = Vec::new(); let mut keys = std::collections::HashSet::new(); let mut total = 0u64;
        for path in files {
            if !valid_file(&path) || !keys.insert(path.to_lowercase()) { return Err("download_unsafe_path".into()); }
            let file = details.files.iter().find(|file|file.path == path).ok_or("download_selection")?;
            let size = file.size.ok_or("download_unknown_size")?; total = total.checked_add(size).ok_or("download_size")?;
            let sha = file.sha256.clone().filter(|s|s.len()==64 && s.bytes().all(|c|c.is_ascii_hexdigit()));
            let git = file.git_sha1.clone().filter(|s|s.len()==40 && s.bytes().all(|c|c.is_ascii_hexdigit()));
            if sha.is_none() && git.is_none() { return Err("download_unknown_hash".into()); }
            selected.push(DownloadFile { path, size, sha256: sha, git_sha1: git, downloaded: 0, actual_sha256: None });
        }
        selected.sort_by(|a,b|a.path.cmp(&b.path));
        for a in &selected { for b in &selected { if b.path.to_lowercase().starts_with(&(a.path.to_lowercase()+"/")) { return Err("download_unsafe_path".into()); } } }
        let id = uuid::Uuid::new_v4().to_string();
        directory(Path::new(&paths.models))?; directory(Path::new(&paths.downloads))?;
        let models = fs::canonicalize(paths.models).map_err(|_| "download_storage")?;
        let downloads = fs::canonicalize(paths.downloads).map_err(|_| "download_storage")?;
        let available = fs2::available_space(&models).map_err(|_| "download_storage")?.min(fs2::available_space(&downloads).map_err(|_| "download_storage")?);
        let additional = total.checked_mul(2).ok_or("download_size")?;
        if available < additional.saturating_add(64*1024*1024) { return Err("download_space".into()); }
        let plan = Plan { id: id.clone(), additional_bytes: additional, available_bytes: available, download: Download {
            destination: models.join(format!("hf-{id}")).to_string_lossy().into(), partial_directory: downloads.join(&id).to_string_lossy().into(), id: id.clone(),
            repo: details.model.id, revision: details.revision, license: details.model.license, task: details.model.task, files: selected, status: "queued".into(), priority: 0,
            #[cfg(test)] test_url: None,
            total_bytes: total, downloaded_bytes: 0, bytes_per_second: 0, error: None, created_at: crate::database::now(), verify_only: false,
        }};
        let mut state = self.state.lock().map_err(|_| "internal")?; state.plans.retain(|_,(at,_)|at.elapsed()<Duration::from_secs(600));
        if state.plans.len()>=64 { return Err("download_plan_limit".into()); }
        state.plans.insert(id,(Instant::now(),plan.clone())); Ok(plan)
    }
    pub fn start(self: &Arc<Self>, plan_id: &str) -> Result<Download> {
        let job = {
            let mut state = self.state.lock().map_err(|_| "internal")?;
            if state.stopped { return Err("download_closing".into()); }
            let (at,plan) = state.plans.remove(plan_id).ok_or("download_plan_expired")?;
            if at.elapsed()>Duration::from_secs(600) { return Err("download_plan_expired".into()); }
            if state.jobs.iter().any(|job|job.repo==plan.download.repo && job.revision==plan.download.revision && job.files.iter().map(|f|&f.path).eq(plan.download.files.iter().map(|f|&f.path)) && job.status!="cancelled") { return Err("download_duplicate".into()); }
            save(&state.db,&plan.download)?; state.jobs.insert(0,plan.download.clone()); plan.download
        }; self.kick(); Ok(job)
    }
    pub fn action(self: &Arc<Self>, id: &str, action: &str, priority: Option<i32>) -> Result<()> {
        {
            let mut state = self.state.lock().map_err(|_| "internal")?;
            if state.stopped { return Err("download_closing".into()); }
            let index = state.jobs.iter().position(|j|j.id==id).ok_or("download_missing")?;
            let active = state.active.as_ref().is_some_and(|(key,_)|key==id);
            let job = &mut state.jobs[index];
            match action {
                "pause" | "cancel" if active => { job.status = if action=="pause" {"pausing"} else {"cancelling"}.into(); },
                "pause" | "cancel" if !["completed","invalid"].contains(&job.status.as_str()) => { job.status = if action=="pause" {"paused"} else {"cancelled"}.into(); },
                "resume" | "retry" if !active && ["paused","failed","cancelled"].contains(&job.status.as_str()) => { job.status="queued".into(); job.error=None; },
                "restart" if !active && ["paused","failed","cancelled"].contains(&job.status.as_str()) && !job.verify_only => {
                    let root=Path::new(&job.partial_directory);
                    if root.exists() { regular(root,true)?; for (i, file) in job.files.iter_mut().enumerate() { let part=within(root,&format!("{i}.part"))?; if part.exists() { fs::remove_file(part).map_err(|_| "download_storage")?; } file.downloaded=0; file.actual_sha256=None; } }
                    job.downloaded_bytes=0; job.status="queued".into();job.error=None;
                },
                "verify" if !active && ["completed","invalid"].contains(&job.status.as_str()) => { job.verify_only=true;job.status="queued".into();job.error=None; },
                "priority" => { let value=priority.ok_or("download_priority")?; if !(0..=2).contains(&value) { return Err("download_priority".into()); } job.priority=value; },
                _ => return Err("download_state".into()),
            }
            save(&state.db,&state.jobs[index])?;
            if active && ["pause","cancel"].contains(&action) { if let Some((_,tx))=&state.active { let _=tx.send(true); } }
        }
        self.kick(); Ok(())
    }
    fn kick(self: &Arc<Self>) {
        let work = (|| -> Result<_> {
            let mut state=self.state.lock().map_err(|_| "internal")?;
            if state.stopped || state.active.is_some() { return Ok(None); }
            let next=state.jobs.iter().enumerate().filter(|(_,j)|j.status=="queued").max_by(|(_,a),(_,b)|a.priority.cmp(&b.priority).then_with(||b.created_at.cmp(&a.created_at))).map(|(i,_)|i);
            let Some(index)=next else { return Ok(None); };
            state.jobs[index].status=if state.jobs[index].verify_only {"verifying"} else {"downloading"}.into();
            save(&state.db,&state.jobs[index])?; let job=state.jobs[index].clone();
            let (tx,rx)=watch::channel(false); state.active=Some((job.id.clone(),tx)); Ok(Some((job,rx)))
        })();
        if let Ok(Some((mut job,mut cancel)))=work {
            let manager=Arc::clone(self);
            tauri::async_runtime::spawn(async move {
                let result=manager.transfer(&mut job,&mut cancel).await;
                if let Ok(mut state)=manager.state.lock() {
                    if let Some(index)=state.jobs.iter().position(|j|j.id==job.id) {
                        let desired=state.jobs[index].status.clone(); job.priority=state.jobs[index].priority; job.bytes_per_second=0;
                        job.status=match (&result,desired.as_str()) { (Ok(_),_)=>"completed", (_,"cancelling")=>"cancelled", (_,"pausing")=>"paused", (Err(_),_) if job.verify_only=>"invalid", _=>"failed" }.into();
                        job.error=result.err().filter(|e|e!="download_interrupted");
                        state.jobs[index]=job; if save(&state.db,&state.jobs[index]).is_err() { state.jobs[index].status="failed".into();state.jobs[index].error=Some("download_storage".into()); }
                    }
                    state.active=None;
                }
                manager.kick();
            });
        }
    }
    fn progress(&self, job: &Download) -> Result<()> {
        let mut state=self.state.lock().map_err(|_| "internal")?;
        let index=state.jobs.iter().position(|j|j.id==job.id).ok_or("download_missing")?;
        let desired=state.jobs[index].status.clone(); let priority=state.jobs[index].priority;
        state.jobs[index]=job.clone();state.jobs[index].priority=priority;
        if ["pausing","cancelling"].contains(&desired.as_str()) {state.jobs[index].status=desired;}
        save(&state.db,&state.jobs[index])
    }
    async fn transfer(&self, job: &mut Download, cancel: &mut watch::Receiver<bool>) -> Result<()> {
        let destination=PathBuf::from(&job.destination);
        if destination.exists() || job.verify_only {
            job.status="verifying".into();self.progress(job)?;regular(&destination,true)?;
            for file in &mut job.files { let path=within(&destination,&file.path)?; file.actual_sha256=Some(hash_file(&path,file,cancel)?);file.downloaded=file.size; }
            job.downloaded_bytes=job.total_bytes;return Ok(());
        }
        let root=PathBuf::from(&job.partial_directory);directory(&root)?;
        let http=reqwest::Client::builder().user_agent(concat!("Local-Studio/",env!("CARGO_PKG_VERSION"))).redirect(reqwest::redirect::Policy::none()).connect_timeout(Duration::from_secs(10)).read_timeout(Duration::from_secs(20)).build().map_err(|_| "network")?;
        let began=Instant::now();let mut moved=0u64;let mut checkpoint=Instant::now();
        // Disk length is authoritative after pause/crash; stale persisted progress is never used as a Range offset.
        for (i,file) in job.files.iter_mut().enumerate() { let part=within(&root,&format!("{i}.part"))?;file.downloaded=if part.exists(){fs::metadata(part).map_err(|_| "download_storage")?.len()}else{0};if file.downloaded>file.size{return Err("download_size".into());} }
        job.downloaded_bytes=job.files.iter().map(|f|f.downloaded).sum();self.progress(job)?;
        let remaining=job.total_bytes.saturating_sub(job.downloaded_bytes);
        if fs2::available_space(&root).map_err(|_| "download_storage")? < remaining.saturating_add(job.total_bytes).saturating_add(64*1024*1024) {return Err("download_space".into());}
        for index in 0..job.files.len() {
            stopped(cancel)?;
            let part=within(&root,&format!("{index}.part"))?;let offset=job.files[index].downloaded;
            if offset<job.files[index].size {
                job.status="downloading".into();self.progress(job)?;
                let auth=Arc::clone(&self.auth);let token=tauri::async_runtime::spawn_blocking(move||auth.token()).await.map_err(|_|"internal")??;
                let mut reply=response(&http,resolve_url(job,&job.files[index]),offset,token.as_ref().map(|t|t.as_str()),cancel).await?;
                if offset>0 && reply.status().as_u16()!=206 {return Err("download_resume_unsupported".into());}
                if reply.status().as_u16()==206 && !range_start(reply.headers().get("content-range").and_then(|v|v.to_str().ok()).unwrap_or(""),offset,job.files[index].size) {return Err("download_range".into());}
                if reply.content_length().is_some_and(|n|n!=job.files[index].size-offset) {return Err("download_size".into());}
                let mut output=OpenOptions::new().create(true).append(true).open(&part).map_err(|_|"download_storage")?;
                loop {
                    stopped(cancel)?;
                    let chunk=tokio::select! {biased; _=cancel.changed()=>return Err("download_interrupted".into()), chunk=reply.chunk()=>chunk.map_err(|_|"network")?};
                    let Some(chunk)=chunk else {break};stopped(cancel)?;
                    if job.files[index].downloaded+chunk.len() as u64>job.files[index].size{return Err("download_size".into());}
                    output.write_all(&chunk).map_err(|_|"download_storage")?;
                    let n=chunk.len() as u64;job.files[index].downloaded+=n;job.downloaded_bytes+=n;moved+=n;
                    job.bytes_per_second=(moved as f64/began.elapsed().as_secs_f64().max(0.01)) as u64;
                    if checkpoint.elapsed()>Duration::from_millis(250) {self.progress(job)?;checkpoint=Instant::now();}
                }
                output.sync_all().map_err(|_|"download_storage")?;
            } else if !part.exists() { File::create(&part).map_err(|_|"download_storage")?; }
            job.status="verifying".into();self.progress(job)?;
            job.files[index].actual_sha256=Some(hash_file(&part,&job.files[index],cancel)?);self.progress(job)?;
        }
        stopped(cancel)?;job.status="installing".into();job.bytes_per_second=0;self.progress(job)?;
        let parent=destination.parent().ok_or("download_unsafe_path")?;directory(parent)?;
        let staging=parent.join(format!(".pending-{}",job.id));directory(&staging)?;
        for (index,file) in job.files.iter().enumerate() {
            let source=within(&root,&format!("{index}.part"))?;let target=within(&staging,&file.path)?;
            let mut input=File::open(source).map_err(|_|"download_storage")?;let mut output=OpenOptions::new().create(true).write(true).truncate(true).open(&target).map_err(|_|"download_storage")?;
            let mut buffer=[0u8;128*1024];loop {stopped(cancel)?;let n=input.read(&mut buffer).map_err(|_|"download_storage")?;if n==0{break;}output.write_all(&buffer[..n]).map_err(|_|"download_storage")?;}
            output.sync_all().map_err(|_|"download_storage")?;drop(output);hash_file(&target,file,cancel)?;
        }
        stopped(cancel)?;
        // Same-directory rename publishes only a fully verified selection; no existing model is overwritten.
        if destination.exists(){return Err("download_destination_exists".into());}
        fs::rename(&staging,&destination).map_err(|_|"download_storage")?;
        for index in 0..job.files.len() {let path=within(&root,&format!("{index}.part"))?;let _=fs::remove_file(path);}
        let _=fs::remove_dir(&root);
        Ok(())
    }
    pub fn stop(&self) {
        if let Ok(mut state)=self.state.lock() {state.stopped=true;let active=state.active.as_ref().map(|(id,_)|id.clone());
            for index in 0..state.jobs.len() {if Some(&state.jobs[index].id)==active.as_ref(){state.jobs[index].status="pausing".into();}else if state.jobs[index].status=="queued"{state.jobs[index].status="paused".into();} let _=save(&state.db,&state.jobs[index]);}
            if let Some((_,tx))=&state.active{let _=tx.send(true);}
        }
    }
    pub async fn shutdown(&self) {self.stop();loop {if self.state.lock().map(|s|s.active.is_none()).unwrap_or(true){break;}tokio::time::sleep(Duration::from_millis(25)).await;}}
}

#[tauri::command]
pub fn download_list(manager: tauri::State<'_,Arc<Downloads>>) -> Result<Vec<Download>> {manager.list()}
#[tauri::command]
pub async fn download_plan(repo:String,revision:String,files:Vec<String>,manager:tauri::State<'_,Arc<Downloads>>,core:tauri::State<'_,Arc<crate::core::Core>>) -> Result<Plan> {
    let manager=manager.inner().clone();let core=core.inner().clone();tauri::async_runtime::spawn_blocking(move||manager.plan(repo,revision,files,core.storage_paths()?)).await.map_err(|_|"internal")?
}
#[tauri::command]
pub fn download_start(plan_id:String,manager:tauri::State<'_,Arc<Downloads>>) -> Result<Download> {manager.inner().start(&plan_id)}
#[tauri::command]
pub async fn download_action(id:String,action:String,priority:Option<i32>,manager:tauri::State<'_,Arc<Downloads>>) -> Result<()> {let manager=manager.inner().clone();tauri::async_runtime::spawn_blocking(move||manager.action(&id,&action,priority)).await.map_err(|_|"internal")?}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn windows_paths_and_redirects_cannot_escape_trusted_roots() {
        for path in ["../x","a/../../x","C:/x","a\\b","a:stream","CON.txt","x/NUL","x. ","a//b","/root","x/LPT1.txt","a/COM¹.txt"] {assert!(!valid_file(path),"{path}");}
        assert!(valid_file("weights/model-Q4_K_M.gguf"));
        for url in ["http://huggingface.co/x","https://huggingface.co.evil.test/x","https://127.0.0.1/x","https://user@hf.co/x","https://evil.test/x","https://hf.co:444/x"] {assert!(!transfer_host(&Url::parse(url).unwrap()));}
        assert!(transfer_host(&Url::parse("https://cas-bridge.xethub.hf.co/file?signature=opaque").unwrap()));
    }
    #[test] fn resume_range_requires_exact_offset_and_pinned_size() {
        assert!(range_start("bytes 10-99/100",10,100));
        for value in ["bytes 0-99/100","bytes 10-98/100","bytes 10-99/*","bytes 10-99/101","invalid"] {assert!(!range_start(value,10,100));}
    }
    #[test] fn hash_verifies_lfs_and_git_objects_and_detects_same_size_corruption() {
        let dir=tempfile::tempdir().unwrap();let path=dir.path().join("test");fs::write(&path,b"test").unwrap();let (_,cancel)=watch::channel(false);
        let mut git=sha1::Sha1::new();git.update(b"blob 4\0test");let git=format!("{:x}",git.finalize());
        let file=DownloadFile{path:"test".into(),size:4,sha256:None,git_sha1:Some(git),downloaded:4,actual_sha256:None};
        let digest=hash_file(&path,&file,&cancel).unwrap();assert_eq!(digest,format!("{:x}",Sha256::digest(b"test")));
        let lfs=DownloadFile{sha256:Some(digest),..file.clone()};assert!(hash_file(&path,&lfs,&cancel).is_ok());
        fs::write(path,b"fail").unwrap();assert_eq!(hash_file(&dir.path().join("test"),&file,&cancel).unwrap_err(),"download_hash");
    }
}

#[cfg(test)]
#[path = "download_tests.rs"]
mod integration_tests;
