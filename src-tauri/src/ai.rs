use crate::{core::Core,downloads::Downloads,gallery,image_engine::{ImageEngine,ImageRequest,ProcessGroup},model_library::ModelLibrary,projects::{Projects,VideoEngine}};
use rusqlite::{Connection,OptionalExtension};
use serde::{Deserialize,Serialize};
use serde_json::{Value,json};
use sha2::{Digest,Sha256};
use std::{fs::{self,File},io::{Read,Seek},path::{Path,PathBuf},process::{Command,Stdio},sync::{Arc,Mutex,atomic::{AtomicBool,Ordering}},thread,time::{Duration,Instant}};
mod chat;
mod speech;
pub use chat::*;
pub use speech::*;
type Result<T>=std::result::Result<T,String>;
fn err(e:impl std::fmt::Display)->String{e.to_string()}
fn uuid()->String{uuid::Uuid::new_v4().to_string()}

#[derive(Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct AiModel{pub id:String,pub name:String,pub kind:String,pub path:String,pub bytes:u64,pub sha256:String,pub binding:String,pub catalog_id:Option<String>}
#[derive(Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub struct CatalogModel{pub id:String,pub name:String,pub kind:String,pub repo:String,pub revision:String,pub file:String,pub bytes:u64,pub sha256:String,pub license:String}
pub struct AiEngine{
 db:Mutex<Connection>,runtime:PathBuf,
 chat:Mutex<ChatState>,endpoint:Mutex<Option<chat::Endpoint>>,
 load_busy:AtomicBool,chat_busy:AtomicBool,load_cancel:AtomicBool,chat_cancel:AtomicBool,
 speech:Mutex<Vec<SpeechJob>>,speech_busy:AtomicBool,speech_cancel:AtomicBool,stopped:AtomicBool,
}
impl AiEngine{
 pub fn new(config:&Path,runtime:PathBuf)->Result<Arc<Self>>{
  let db=Connection::open(config.join("ai.sqlite3")).map_err(err)?;
  db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; CREATE TABLE IF NOT EXISTS ai_models(id TEXT PRIMARY KEY,json TEXT NOT NULL); CREATE TABLE IF NOT EXISTS ai_state(id TEXT PRIMARY KEY,json TEXT NOT NULL); CREATE TABLE IF NOT EXISTS speech_jobs(id TEXT PRIMARY KEY,json TEXT NOT NULL);").map_err(err)?;
  let stored:Option<String>=db.query_row("SELECT json FROM ai_state WHERE id='chat'",[],|r|r.get(0)).optional().map_err(err)?;
  let mut chat:ChatState=stored.map(|s|serde_json::from_str(&s).map_err(err)).transpose()?.unwrap_or_default();
  if ["loading","running"].contains(&chat.phase.as_str()){chat.error=Some("ai_interrupted".into());}chat.phase="unloaded".into();chat.model_id=None;
  let speech={let mut q=db.prepare("SELECT json FROM speech_jobs ORDER BY rowid DESC LIMIT 30").map_err(err)?;let mut jobs=vec![];for row in q.query_map([],|r|r.get::<_,String>(0)).map_err(err)?{let mut j:SpeechJob=serde_json::from_str(&row.map_err(err)?).map_err(err)?;if j.status=="running"{j.status="interrupted".into();j.error=Some("ai_interrupted".into());}jobs.push(j);}jobs};
  Ok(Arc::new(Self{db:Mutex::new(db),runtime,chat:Mutex::new(chat),endpoint:Mutex::new(None),load_busy:AtomicBool::new(false),chat_busy:AtomicBool::new(false),load_cancel:AtomicBool::new(false),chat_cancel:AtomicBool::new(false),speech:Mutex::new(speech),speech_busy:AtomicBool::new(false),speech_cancel:AtomicBool::new(false),stopped:AtomicBool::new(false)}))
 }
 pub fn stop(&self){self.stopped.store(true,Ordering::SeqCst);self.load_cancel.store(true,Ordering::SeqCst);self.chat_cancel.store(true,Ordering::SeqCst);self.speech_cancel.store(true,Ordering::SeqCst);}
 fn models(&self)->Result<Vec<AiModel>>{let db=self.db.lock().map_err(err)?;let mut q=db.prepare("SELECT json FROM ai_models ORDER BY rowid").map_err(err)?;let rows=q.query_map([],|r|r.get::<_,String>(0)).map_err(err)?;rows.map(|s|serde_json::from_str(&s.map_err(err)?).map_err(err)).collect()}
 fn model(&self,id:&str,kind:&str)->Result<(AiModel,Vec<File>)>{
  let m=self.models()?.into_iter().find(|m|m.id==id&&m.kind==kind).ok_or("ai_model_missing")?;let path=Path::new(&m.path);let mut pins=gallery::directory_guards(path.parent().ok_or("ai_model_changed")?)?;let mut file=gallery::lock_file(path)?;
  if file.metadata().map_err(err)?.len()!=m.bytes||gallery::file_binding(&file)?!=m.binding||digest(&mut file)?!=m.sha256{return Err("ai_model_changed".into());}pins.push(file);Ok((m,pins))
 }
 fn import(&self,path:String,kind:String)->Result<AiModel>{
  if !["chat","speech"].contains(&kind.as_str()){return Err("ai_parameters".into());}let path=PathBuf::from(path);crate::model_library::no_links(&path)?;let _pins=gallery::directory_guards(path.parent().ok_or("ai_model_invalid")?)?;let mut f=gallery::lock_file(&path)?;let bytes=f.metadata().map_err(err)?.len();if !(1_000_000..=16_000_000_000).contains(&bytes){return Err("ai_model_invalid".into());}let mut magic=[0;4];f.read_exact(&mut magic).map_err(err)?;
  if kind=="chat"&&magic!=*b"GGUF"||kind=="speech"&&magic!=*b"lmgg"{return Err("ai_model_invalid".into());}f.rewind().map_err(err)?;let sha256=digest(&mut f)?;let binding=gallery::file_binding(&f)?;let path=fs::canonicalize(path).map_err(err)?.to_string_lossy().to_string();
  if let Some(m)=self.models()?.into_iter().find(|m|m.path==path&&m.kind==kind&&m.sha256==sha256&&m.binding==binding){return Ok(m);}
  let catalog=catalog()?.into_iter().find(|m|m.kind==kind&&m.sha256==sha256&&m.bytes==bytes);
  let model=AiModel{id:uuid(),name:catalog.as_ref().map(|m|m.name.clone()).unwrap_or_else(||Path::new(&path).file_name().unwrap().to_string_lossy().into()),kind,path,bytes,sha256,binding,catalog_id:catalog.map(|m|m.id)};
  self.db.lock().map_err(err)?.execute("INSERT INTO ai_models VALUES(?1,?2)",rusqlite::params![model.id,serde_json::to_string(&model).map_err(err)?]).map_err(err)?;Ok(model)
 }
 fn runtime(&self,kind:&str)->Result<(PathBuf,Vec<File>)>{
  let text=match kind{"assistant"=>include_str!("../assistant-runtime.json"),"speech"=>include_str!("../speech-runtime.json"),_=>return Err("ai_runtime".into())};let manifest:Value=serde_json::from_str(text).map_err(err)?;let dir=self.runtime.join(format!("{kind}-runtime"));
  #[cfg(debug_assertions)]let dir=if dir.is_dir(){dir}else{Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../.tools/{kind}-runtime"))};
  let files=manifest["files"].as_object().ok_or("ai_runtime")?;let mut pins=gallery::directory_guards(&dir).map_err(|_|"ai_runtime")?;
  for entry in fs::read_dir(&dir).map_err(|_|"ai_runtime")?{let name=entry.map_err(err)?.file_name().to_string_lossy().to_string();if !files.contains_key(&name){return Err("ai_runtime_changed".into());}}
  for(name,hash)in files{if name.contains(['/', '\\', ':']){return Err("ai_runtime".into());}let mut f=gallery::lock_file(&dir.join(name))?;if digest(&mut f)?!=hash.as_str().unwrap_or(""){return Err("ai_runtime_changed".into());}pins.push(f);}Ok((dir,pins))
 }
}
fn catalog()->Result<Vec<CatalogModel>>{serde_json::from_str(include_str!("../ai-models.json")).map_err(err)}
fn digest(f:&mut File)->Result<String>{let mut hash=Sha256::new();let mut b=vec![0;1024*1024];loop{let n=f.read(&mut b).map_err(err)?;if n==0{break;}hash.update(&b[..n]);}Ok(format!("{:x}",hash.finalize()))}
fn command(exe:&Path)->Command{let mut c=Command::new(exe);c.stdin(Stdio::null()).env_clear();for key in ["SystemRoot","WINDIR","TEMP","TMP","LOCALAPPDATA","APPDATA","USERPROFILE"]{if let Some(v)=std::env::var_os(key){c.env(key,v);}}c.env("PATH",PathBuf::from(std::env::var_os("SystemRoot").unwrap_or("C:\\Windows".into())).join("System32"));crate::hardware::hide_console(&mut c);c}
#[tauri::command]pub fn ai_catalog()->Result<Vec<CatalogModel>>{catalog()}
#[tauri::command]pub fn ai_models(engine:tauri::State<'_,Arc<AiEngine>>)->Result<Vec<AiModel>>{engine.models()}
#[tauri::command]pub async fn ai_import(path:String,kind:String,engine:tauri::State<'_,Arc<AiEngine>>)->Result<AiModel>{let e=engine.inner().clone();tauri::async_runtime::spawn_blocking(move||e.import(path,kind)).await.map_err(err)?}
#[tauri::command]pub async fn ai_download_plan(catalog_id:String,downloads:tauri::State<'_,Arc<Downloads>>,core:tauri::State<'_,Arc<Core>>)->Result<crate::downloads::Plan>{
 let m=catalog()?.into_iter().find(|m|m.id==catalog_id).ok_or("ai_model_missing")?;let d=downloads.inner().clone();let mut paths=core.storage_paths()?;paths.models=if m.kind=="chat"{paths.assistant_models.clone()}else{paths.vision_models.clone()};
 tauri::async_runtime::spawn_blocking(move||{let p=d.plan(m.repo,m.revision,vec![m.file.clone()],paths)?;if !p.download.files.iter().any(|f|f.path==m.file&&f.sha256.as_deref()==Some(&m.sha256)&&f.size==m.bytes){return Err("ai_model_changed".into());}Ok(p)}).await.map_err(err)?
}
#[tauri::command]pub async fn ai_adopt_download(download_id:String,downloads:tauri::State<'_,Arc<Downloads>>,engine:tauri::State<'_,Arc<AiEngine>>)->Result<AiModel>{
 let d=downloads.list()?.into_iter().find(|d|d.id==download_id&&d.status=="completed").ok_or("ai_download_pending")?;let m=catalog()?.into_iter().find(|m|m.repo==d.repo&&m.revision==d.revision).ok_or("ai_model_missing")?;let file=d.files.iter().find(|f|f.path==m.file&&f.actual_sha256.as_deref()==Some(&m.sha256)).ok_or("ai_model_changed")?;let path=Path::new(&d.destination).join(&file.path).to_string_lossy().to_string();let e=engine.inner().clone();tauri::async_runtime::spawn_blocking(move||e.import(path,m.kind)).await.map_err(err)?
}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn model_registry_rechecks_changed_bytes_and_survives_restart(){
  let tmp=tempfile::tempdir().unwrap();let path=tmp.path().join("fixture.gguf");let mut bytes=vec![0;1_000_000];bytes[..4].copy_from_slice(b"GGUF");fs::write(&path,&bytes).unwrap();let engine=AiEngine::new(tmp.path(),tmp.path().into()).unwrap();
  let model=engine.import(path.to_string_lossy().into(),"chat".into()).unwrap();assert!(model.catalog_id.is_none());assert_eq!(engine.import(path.to_string_lossy().into(),"chat".into()).unwrap().id,model.id);drop(engine.model(&model.id,"chat").unwrap());
  bytes[999_999]=1;fs::write(&path,&bytes).unwrap();assert!(engine.model(&model.id,"chat").is_err());assert!(engine.import(path.to_string_lossy().into(),"speech".into()).is_err());drop(engine);
  let restored=AiEngine::new(tmp.path(),tmp.path().into()).unwrap();assert_eq!(restored.models().unwrap()[0].sha256,model.sha256);assert_eq!(restored.chat.lock().unwrap().phase,"unloaded");assert!(restored.model(&model.id,"chat").is_err());
 }
}
