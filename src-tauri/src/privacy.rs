//! Local content lock. This is an application access boundary, not file encryption.
use ring::{pbkdf2, rand::{SecureRandom, SystemRandom}};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::{num::NonZeroU32, path::Path, sync::{Arc, Mutex, OnceLock}, time::Instant};
use tauri::{Emitter, Manager};
use zeroize::Zeroizing;
mod detection;

type Result<T> = std::result::Result<T, String>;
const ITERATIONS: u32 = 600_000;
static SHARED: OnceLock<Arc<Privacy>> = OnceLock::new();
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all="camelCase", deny_unknown_fields)]
struct Config { kind: String, salt: Vec<u8>, hash: Vec<u8>, iterations: u32, auto_lock: bool, enabled: bool, remembered: bool, failures: u32, retry_at: i64 }
struct State { db: Connection, config: Option<Config>, unlocked: bool, epoch: u64, retry: Option<Instant> }
pub struct Privacy(Mutex<State>);
#[derive(Clone, Serialize)]
#[serde(rename_all="camelCase")]
pub struct Status { configured: bool, enabled: bool, locked: bool, auto_lock: bool, kind: String, epoch: u64, retry_seconds: u64, pub models: Vec<String>, pub has_protected: bool, pub project_locked: bool }
fn err(_: impl std::fmt::Display)->String { "privacy_storage".into() }
fn now()->i64 { chrono::Utc::now().timestamp() }
fn key(path:&Path)->String { let canonical=std::fs::canonicalize(path).ok();let path=canonical.as_deref().unwrap_or(path);path.to_string_lossy().trim_start_matches(r"\\?\").replace('/',"\\").trim_end_matches('\\').to_lowercase() }
fn file_id(path:&Path)->Option<String> { let f=crate::model_library::safe_file(path).ok()?; let binding=crate::gallery::file_binding(&f).ok()?;Some(binding.split(':').next()?.to_string()) }
impl Privacy {
    pub fn new(config:&Path)->Result<Arc<Self>> {
        let db=Connection::open(config.join("privacy.sqlite3")).map_err(err)?;
        db.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE IF NOT EXISTS privacy_config(id INTEGER PRIMARY KEY CHECK(id=1),json TEXT NOT NULL); CREATE TABLE IF NOT EXISTS protected_models(path TEXT PRIMARY KEY,identity TEXT); CREATE TABLE IF NOT EXISTS model_classification_overrides(path TEXT PRIMARY KEY,identity TEXT,restricted INTEGER NOT NULL); CREATE TABLE IF NOT EXISTS protected_media(identity TEXT PRIMARY KEY); CREATE TABLE IF NOT EXISTS protected_paths(path TEXT PRIMARY KEY);").map_err(err)?;
        let raw:Option<String>=db.query_row("SELECT json FROM privacy_config WHERE id=1",[],|r|r.get(0)).optional().map_err(err)?;
        let config:Option<Config>=raw.map(|s|serde_json::from_str(&s).map_err(err)).transpose()?;
        if config.as_ref().is_some_and(|c|c.salt.len()!=32||c.hash.len()!=32||c.iterations!=ITERATIONS||!["pin","password"].contains(&c.kind.as_str())){return Err("privacy_storage".into());}
        let unlocked=config.as_ref().is_some_and(|c|c.enabled&&!c.auto_lock&&c.remembered);
        Ok(Arc::new(Self(Mutex::new(State{db,config,unlocked,epoch:0,retry:None}))))
    }
    fn persist(s:&State)->Result<()> { if let Some(c)=&s.config{s.db.execute("INSERT OR REPLACE INTO privacy_config VALUES(1,?1)",[serde_json::to_string(c).map_err(err)?]).map_err(err)?;} Ok(()) }
    fn locked_state(s:&State)->bool { !s.unlocked||s.config.as_ref().is_none_or(|c|!c.enabled) }
    fn status(&self)->Result<Status> {
        let s=self.0.lock().map_err(err)?; let c=s.config.as_ref();
        let mut stmt=s.db.prepare("SELECT path FROM protected_models ORDER BY path").map_err(err)?;
        let models=stmt.query_map([],|r|r.get(0)).map_err(err)?.collect::<std::result::Result<Vec<String>,_>>().map_err(err)?;
        let count:i64=s.db.query_row("SELECT (SELECT count(*) FROM protected_media)+(SELECT count(*) FROM protected_paths)",[],|r|r.get(0)).map_err(err)?;
        Ok(Status{configured:c.is_some(),enabled:c.is_some_and(|c|c.enabled),locked:Self::locked_state(&s),auto_lock:c.is_none_or(|c|c.auto_lock),kind:c.map(|c|c.kind.clone()).unwrap_or_else(||"pin".into()),epoch:s.epoch,retry_seconds:c.map(|c|(c.retry_at-now()).max(0) as u64).unwrap_or(0),has_protected:count>0||!models.is_empty(),models,project_locked:false})
    }
    fn credential(kind:&str,secret:&str)->Result<()> {
        let valid=match kind {"pin"=>(6..=12).contains(&secret.len())&&secret.bytes().all(|b|b.is_ascii_digit()),"password"=>(8..=128).contains(&secret.chars().count())&&secret.len()<=512&&!secret.chars().any(char::is_control),_=>false};
        if valid{Ok(())}else{Err("privacy_credential".into())}
    }
    fn verify(s:&mut State,secret:&str)->Result<()> {
        let c=s.config.as_mut().ok_or("privacy_setup")?;
        if secret.len()>512{return Err("privacy_credential".into());}
        if c.retry_at>now()||s.retry.is_some_and(|v|v>Instant::now()){return Err("privacy_retry".into());}
        if pbkdf2::verify(pbkdf2::PBKDF2_HMAC_SHA256,NonZeroU32::new(c.iterations).unwrap(),&c.salt,secret.as_bytes(),&c.hash).is_err(){
            c.failures=c.failures.saturating_add(1);let seconds=(1u64<<c.failures.min(6)).min(60);c.retry_at=now()+seconds as i64;s.retry=Some(Instant::now()+std::time::Duration::from_secs(seconds));Self::persist(s)?;return Err("privacy_wrong".into());
        }
        c.failures=0;c.retry_at=0;s.retry=None;Ok(())
    }
    fn setup(&self,kind:String,secret:Zeroizing<String>,adult:bool)->Result<()> {
        if !adult{return Err("privacy_age".into());}Self::credential(&kind,&secret)?;
        let mut s=self.0.lock().map_err(err)?;if s.config.is_some(){return Err("privacy_configured".into());}
        let mut salt=vec![0;32];SystemRandom::new().fill(&mut salt).map_err(err)?;let mut hash=vec![0;32];pbkdf2::derive(pbkdf2::PBKDF2_HMAC_SHA256,NonZeroU32::new(ITERATIONS).unwrap(),&salt,secret.as_bytes(),&mut hash);
        s.config=Some(Config{kind,salt,hash,iterations:ITERATIONS,auto_lock:true,enabled:true,remembered:true,failures:0,retry_at:0});Self::persist(&s)?;s.unlocked=true;s.epoch+=1;Ok(())
    }
    fn unlock(&self,secret:Zeroizing<String>)->Result<()> {
        let mut s=self.0.lock().map_err(err)?;Self::verify(&mut s,&secret)?;let c=s.config.as_mut().unwrap();c.enabled=true;c.remembered=true;Self::persist(&s)?;s.unlocked=true;s.epoch+=1;Ok(())
    }
    fn lock(&self,disable:bool)->Result<()> {
        let mut s=self.0.lock().map_err(err)?;s.unlocked=false;s.epoch+=1;if let Some(c)=s.config.as_mut(){c.remembered=false;if disable{c.enabled=false;}}Self::persist(&s)
    }
    fn options(&self,secret:Zeroizing<String>,auto_lock:bool,new_secret:Option<Zeroizing<String>>,kind:Option<String>)->Result<()> {
        let mut s=self.0.lock().map_err(err)?;Self::verify(&mut s,&secret)?;
        if let Some(new)=new_secret{let kind=kind.ok_or("privacy_credential")?;Self::credential(&kind,&new)?;let c=s.config.as_mut().unwrap();SystemRandom::new().fill(&mut c.salt).map_err(err)?;pbkdf2::derive(pbkdf2::PBKDF2_HMAC_SHA256,NonZeroU32::new(ITERATIONS).unwrap(),&c.salt,new.as_bytes(),&mut c.hash);c.kind=kind;}
        s.config.as_mut().unwrap().auto_lock=auto_lock;Self::persist(&s)?;s.epoch+=1;Ok(())
    }
    pub fn model(&self,path:&Path)->bool {
        let id=file_id(path);let Ok(s)=self.0.lock()else{return true;};
        let manual = s.db.query_row("SELECT restricted FROM model_classification_overrides WHERE (path=?1 AND identity=?2) OR (?2 IS NOT NULL AND identity=?2) LIMIT 1",params![key(path),id],|r|r.get::<_,bool>(0)).optional();
        match manual { Ok(Some(value)) => return value, Err(_) => return true, _ => {} }
        let registered = s.db.query_row("SELECT EXISTS(SELECT 1 FROM protected_models WHERE path=?1 OR (?2 IS NOT NULL AND identity=?2))",params![key(path),id],|r|r.get::<_,bool>(0)).unwrap_or(true);
        if registered { return true; }
        drop(s);
        if detection::detect(path) {
            if let Ok(s)=self.0.lock() { let _=s.db.execute("INSERT OR IGNORE INTO protected_models VALUES(?1,?2)",params![key(path),id]); }
            return true;
        }
        false
    }
    pub fn set_model(&self,path:&Path,restricted:bool)->Result<()> {
        let id=file_id(path).ok_or("privacy_model")?;
        if !path.extension().and_then(|s|s.to_str()).is_some_and(|s|["safetensors","gguf","onnx","bin","pt","ckpt"].contains(&s.to_ascii_lowercase().as_str())){return Err("privacy_model".into());}
        let mut s=self.0.lock().map_err(err)?;if Self::locked_state(&s){return Err("privacy_locked".into());}
        s.db.execute("DELETE FROM model_classification_overrides WHERE identity=?1",[&id]).map_err(err)?;
        s.db.execute("INSERT OR REPLACE INTO model_classification_overrides VALUES(?1,?2,?3)",params![key(path),id,restricted]).map_err(err)?;
        if restricted{s.db.execute("INSERT OR REPLACE INTO protected_models VALUES(?1,?2)",params![key(path),id]).map_err(err)?;}else{s.db.execute("DELETE FROM protected_models WHERE path=?1 OR identity=?2",params![key(path),id]).map_err(err)?;}s.epoch+=1;Ok(())
    }
    pub fn mark_path(&self,path:&Path)->Result<()> { let s=self.0.lock().map_err(err)?;s.db.execute("INSERT OR IGNORE INTO protected_paths VALUES(?1)",[key(path)]).map_err(err)?;Ok(()) }
    pub fn mark_file(&self,path:&Path)->Result<()> {
        let id=file_id(path).ok_or("privacy_storage")?;let s=self.0.lock().map_err(err)?;s.db.execute("INSERT OR IGNORE INTO protected_media VALUES(?1)",[id]).map_err(err)?;Ok(())
    }
    pub fn media(&self,path:&Path)->bool {
        let id=file_id(path);let Ok(s)=self.0.lock()else{return true;};let k=key(path);
        s.db.query_row("SELECT EXISTS(SELECT 1 FROM protected_media WHERE identity=?1) OR EXISTS(SELECT 1 FROM protected_paths WHERE path=?2 OR substr(?2,1,length(path)+1)=path||char(92))",params![id,k],|r|r.get(0)).unwrap_or(true)
    }
}
pub fn install(p:Arc<Privacy>)->Result<()> {SHARED.set(p).map_err(|_|"privacy_storage".into())}
pub fn shared()->Option<&'static Arc<Privacy>> {SHARED.get()}
pub fn locked()->bool {shared().is_some_and(|p|p.0.lock().map(|s|Privacy::locked_state(&s)).unwrap_or(true))}
pub fn epoch()->u64 {shared().and_then(|p|p.0.lock().ok().map(|s|s.epoch)).unwrap_or(0)}
pub fn finish<T>(at:u64,result:Result<T>)->Result<T>{if epoch()!=at&&locked(){Err("privacy_locked".into())}else{result}}
pub fn model(path:&Path)->bool {shared().is_some_and(|p|p.model(path))}
pub fn media(path:&Path)->bool {shared().is_some_and(|p|p.media(path))}
pub fn check(path:&Path)->Result<()> {if locked()&&(media(path)||model(path)){Err("privacy_locked".into())}else{Ok(())}}
pub fn mark(path:&Path)->Result<()> {if let Some(p)=shared(){p.mark_file(path)?;}Ok(())}
pub fn register_model(path:&Path)->Result<()> {if let Some(p)=shared(){let s=p.0.lock().map_err(err)?;s.db.execute("INSERT OR IGNORE INTO protected_models VALUES(?1,?2)",params![key(path),file_id(path)]).map_err(err)?;}Ok(())}
pub fn protect_path(path:&Path)->Result<()> {if let Some(p)=shared(){p.mark_path(path)?;}Ok(())}
pub fn inherit(source:&Path,target:&Path)->Result<()> {if media(source){mark(target)?;}Ok(())}
pub fn request(r:&crate::image_engine::ImageRequest)->bool {model(Path::new(&r.model_path))||r.reference.as_ref().is_some_and(|r|media(Path::new(&r.path))||r.mask.as_ref().is_some_and(|m|media(Path::new(&m.path))))}
pub fn check_request(r:&crate::image_engine::ImageRequest)->Result<()> {if locked()&&request(r){Err("privacy_locked".into())}else{Ok(())}}
pub fn redact(r:&mut crate::image_engine::ImageRequest){r.prompt.clear();r.negative_prompt.clear();r.reference=None;r.width=512;r.height=512;r.steps=25;r.guidance=5.;r.seed=42;r.sampler="euler".into();r.vae_on_cpu=false;}
pub fn has_protected()->bool {shared().is_some_and(|p|p.status().map(|s|s.has_protected).unwrap_or(true))}
fn notify(app:&tauri::AppHandle)->Result<Status>{let p=app.state::<Arc<Privacy>>();let mut status=p.status()?;status.project_locked=locked()&&app.state::<Arc<crate::projects::Projects>>().privacy_restricted()?;app.emit_to("main","privacy-changed",&status).map_err(err)?;Ok(status)}
#[tauri::command]
pub fn privacy_status(app:tauri::AppHandle)->Result<Status>{let mut s=app.state::<Arc<Privacy>>().status()?;s.project_locked=locked()&&app.state::<Arc<crate::projects::Projects>>().privacy_restricted()?;Ok(s)}
#[tauri::command]
pub async fn privacy_setup(kind:String,secret:String,adult:bool,app:tauri::AppHandle)->Result<Status>{let p=app.state::<Arc<Privacy>>().inner().clone();let secret=Zeroizing::new(secret);tauri::async_runtime::spawn_blocking(move||p.setup(kind,secret,adult)).await.map_err(err)??;notify(&app)}
#[tauri::command]
pub async fn privacy_unlock(secret:String,app:tauri::AppHandle)->Result<Status>{let p=app.state::<Arc<Privacy>>().inner().clone();let secret=Zeroizing::new(secret);let result=tauri::async_runtime::spawn_blocking(move||p.unlock(secret)).await.map_err(err)?;let status=notify(&app)?;result?;Ok(status)}
#[tauri::command]
pub fn privacy_lock(disable:bool,app:tauri::AppHandle)->Result<Status>{app.state::<Arc<Privacy>>().lock(disable)?;notify(&app)}
#[tauri::command]
pub async fn privacy_options(secret:String,auto_lock:bool,new_secret:Option<String>,kind:Option<String>,app:tauri::AppHandle)->Result<Status>{let p=app.state::<Arc<Privacy>>().inner().clone();let secret=Zeroizing::new(secret);let new_secret=new_secret.map(Zeroizing::new);tauri::async_runtime::spawn_blocking(move||p.options(secret,auto_lock,new_secret,kind)).await.map_err(err)??;notify(&app)}
#[tauri::command]
pub async fn privacy_model_set(path:String,restricted:bool,app:tauri::AppHandle)->Result<Status>{let p=app.state::<Arc<Privacy>>().inner().clone();let images=app.state::<Arc<crate::image_engine::ImageEngine>>().inner().clone();let ai=app.state::<Arc<crate::ai::AiEngine>>().inner().clone();tauri::async_runtime::spawn_blocking(move||{p.set_model(Path::new(&path),restricted)?;if restricted{images.protect_known_jobs()?;ai.protect_chat()?;}Ok::<_,String>(())}).await.map_err(err)??;notify(&app)}
#[tauri::command]
pub async fn privacy_model_status(path:String)->Result<bool>{if path.len()>32768{return Err("privacy_model".into());}tauri::async_runtime::spawn_blocking(move||Ok(model(Path::new(&path)))).await.map_err(err)?}

mod boundary;
pub use boundary::guard;
pub fn dispatch(f:impl Fn(tauri::ipc::Invoke)->bool,invoke:tauri::ipc::Invoke)->bool{f(invoke)}
#[cfg(test)] mod tests;
