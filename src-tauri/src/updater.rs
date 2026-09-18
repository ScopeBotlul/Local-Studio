use crate::{model_library,hub};
use serde::{Serialize,Deserialize};
use sha2::{Digest,Sha256};
use base64::Engine;
use std::{fs::{self,File,OpenOptions},io::{Read,Write,Seek,SeekFrom},path::{Path,PathBuf},sync::{Arc,Mutex,atomic::{AtomicBool,Ordering}},process::{Command,Child},time::Duration};
use std::os::windows::{fs::OpenOptionsExt,process::CommandExt};
use tauri::State;
type Result<T>=std::result::Result<T,String>;
const BASE:&str="https://github.com/ScopeBotlul/Local-Studio/releases";
const MAX_PACKAGE:u64=512*1024*1024;
#[derive(Clone,Serialize,Deserialize,Debug)]
#[serde(deny_unknown_fields)]
pub struct Asset{pub url:String,pub size:u64,pub sha256:String}
#[derive(Clone,Serialize,Deserialize,Debug)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Manifest{pub schema:u32,pub version:String,pub published_at:String,pub notes:String,pub installer:Asset,pub portable:Asset}
#[derive(Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct Status{phase:String,portable:bool,latest:Option<Manifest>,received:u64,total:u64,error:Option<String>}
struct Data{status:Status,signed:Option<(Vec<u8>,Vec<u8>)>,stage:Option<PathBuf>,helper:Option<Child>}
pub struct Updater{data:Mutex<Data>,cancel:AtomicBool,root:PathBuf,install:PathBuf,portable:bool}
fn err(_:impl std::fmt::Display)->String{"update_storage".into()}
fn version(value:&str)->Result<[u32;3]>{let parts=value.split('.').map(|v|if v.is_empty()||!v.bytes().all(|b|b.is_ascii_digit())||v.len()>8||(v.len()>1&&v.starts_with('0')){Err("update_manifest".into())}else{v.parse::<u32>().map_err(|_|"update_manifest".to_string())}).collect::<Result<Vec<_>>>()?;parts.try_into().map_err(|_|"update_manifest".into())}
fn public_key()->Result<Vec<u8>>{let text=include_str!("../update-public-key.txt").trim();if text.len()!=64{return Err("update_signature".into());}(0..64).step_by(2).map(|n|u8::from_str_radix(&text[n..n+2],16).map_err(|_|"update_signature".into())).collect()}
fn verified(bytes:&[u8],signature:&[u8],key:&[u8])->Result<Manifest>{
 if bytes.len()>65536||signature.len()>256{return Err("update_manifest".into());}
 let sig=base64::engine::general_purpose::STANDARD.decode(signature).map_err(|_|"update_signature")?;
 ring::signature::UnparsedPublicKey::new(&ring::signature::ED25519,key).verify(bytes,&sig).map_err(|_|"update_signature")?;
 let m:Manifest=serde_json::from_slice(bytes).map_err(|_|"update_manifest")?;version(&m.version)?;
 if m.schema!=1||m.notes.len()>16000||m.published_at.len()>64{return Err("update_manifest".into());}
 for(kind,a)in [("setup.exe",&m.installer),("portable.zip",&m.portable)]{let expected=format!("{BASE}/download/v{}/Local-Studio-{}-hub-{kind}",m.version,m.version);if a.url!=expected||a.size==0||a.size>MAX_PACKAGE||a.sha256.len()!=64||!a.sha256.bytes().all(|b|b.is_ascii_hexdigit()){return Err("update_manifest".into());}}
 Ok(m)
}
fn client()->Result<reqwest::blocking::Client>{reqwest::blocking::Client::builder().user_agent("Local-Studio-Updater").connect_timeout(Duration::from_secs(10)).timeout(Duration::from_secs(120)).redirect(reqwest::redirect::Policy::custom(|attempt|{let u=attempt.url();if attempt.previous().len()<6&&u.scheme()=="https"&&matches!(u.host_str(),Some("github.com"|"release-assets.githubusercontent.com"|"objects.githubusercontent.com")){attempt.follow()}else{attempt.stop()}})).build().map_err(|_|"update_network".into())}
fn small(client:&reqwest::blocking::Client,url:&str,max:u64)->Result<Vec<u8>>{let response=client.get(url).send().map_err(|_|"update_network")?;if response.status()==404{return Err("update_no_release".into());}if !response.status().is_success(){return Err("update_network".into());}let mut bytes=vec![];response.take(max+1).read_to_end(&mut bytes).map_err(|_|"update_network")?;if bytes.len()as u64>max{return Err("update_manifest".into());}Ok(bytes)}
fn bounded(path:&Path,max:u64)->Result<Vec<u8>>{let file=crate::gallery::lock_file(path)?;if file.metadata().map_err(err)?.len()>max{return Err("update_storage".into());}let mut b=vec![];file.take(max+1).read_to_end(&mut b).map_err(err)?;Ok(b)}
fn digest(file:&mut File)->Result<String>{let mut hash=Sha256::new();let mut buf=vec![0;1024*1024];loop{let n=file.read(&mut buf).map_err(err)?;if n==0{break;}hash.update(&buf[..n]);}Ok(format!("{:x}",hash.finalize()))}
fn checked_installer(stage:&Path,key:&[u8])->Result<(File,Manifest,Vec<File>)>{let pins=crate::gallery::directory_guards(stage)?;let m=verified(&bounded(&stage.join("update.json"),65536)?,&bounded(&stage.join("update.sig"),256)?,key)?;if version(&m.version)?<=version(env!("CARGO_PKG_VERSION"))?{return Err("update_old".into());}let mut file=crate::gallery::lock_file(&stage.join("setup.exe"))?;if file.metadata().map_err(err)?.len()!=m.installer.size||digest(&mut file)?!=m.installer.sha256.to_lowercase(){return Err("update_integrity".into());}Ok((file,m,pins))}
impl Updater{
 pub fn new(config:&Path,install:&Path,portable:bool)->Arc<Self>{Arc::new(Self{data:Mutex::new(Data{status:Status{phase:"idle".into(),portable,latest:None,received:0,total:0,error:None},signed:None,stage:None,helper:None}),cancel:AtomicBool::new(false),root:config.join("updates"),install:install.into(),portable})}
 fn status(&self)->Result<Status>{Ok(self.data.lock().map_err(err)?.status.clone())}
 fn begin(&self,phase:&str)->Result<()>{let mut d=self.data.lock().map_err(err)?;if matches!(d.status.phase.as_str(),"checking"|"downloading"|"ready"|"installing"){return Err("update_busy".into());}d.status.phase=phase.into();d.status.error=None;d.status.received=0;d.status.total=0;self.cancel.store(false,Ordering::SeqCst);Ok(())}
 fn finish(&self,result:Result<()>)->Result<Status>{if let Err(e)=result{let mut d=self.data.lock().map_err(err)?;d.status.phase=if e=="update_cancelled"{"available"}else{"error"}.into();d.status.error=Some(e);}self.status()}
 fn check(&self)->Result<Status>{self.begin("checking")?;let result=(||{let c=client()?;let bytes=small(&c,&format!("{BASE}/latest/download/update.json"),65536)?;let sig=small(&c,&format!("{BASE}/latest/download/update.sig"),256)?;let m=verified(&bytes,&sig,&public_key()?)?;let newer=version(&m.version)?>version(env!("CARGO_PKG_VERSION"))?;let mut d=self.data.lock().map_err(err)?;d.status.phase=if newer{"available"}else{"current"}.into();d.status.latest=Some(m);d.signed=Some((bytes,sig));d.stage=None;Ok(())})();self.finish(result)}
 fn download(&self)->Result<Status>{if self.portable{return Err("update_portable".into());}let(m,bytes,sig)={let d=self.data.lock().map_err(err)?;if d.status.phase!="available"{return Err("update_state".into());}let(b,s)=d.signed.as_ref().ok_or("update_state")?;(d.status.latest.clone().ok_or("update_state")?,b.clone(),s.clone())};self.begin("downloading")?;
 let result=(||{if version(&m.version)?<=version(env!("CARGO_PKG_VERSION"))?{return Err("update_old".into());}for parent in self.root.ancestors().filter(|p|p.exists()){model_library::no_links(parent).map_err(err)?;}fs::create_dir_all(&self.root).map_err(err)?;let _root_pins=crate::gallery::directory_guards(&self.root)?;let stage=self.root.join(uuid::Uuid::new_v4().to_string());fs::create_dir(&stage).map_err(err)?;let _pins=crate::gallery::directory_guards(&stage)?;
 let mut output=OpenOptions::new().write(true).read(true).create_new(true).access_mode(0xc0010000).share_mode(1).open(stage.join("package.part")).map_err(err)?;
 let written:Result<()>=(||{let mut response=client()?.get(&m.installer.url).timeout(Duration::from_secs(1800)).send().map_err(|_|"update_network")?;if !response.status().is_success()||response.content_length().is_some_and(|s|s!=m.installer.size){return Err("update_network".into());}let mut hash=Sha256::new();let mut received=0u64;let mut buffer=vec![0;256*1024];loop{if self.cancel.load(Ordering::SeqCst){return Err("update_cancelled".into());}let n=response.read(&mut buffer).map_err(|_|"update_network")?;if n==0{break;}received+=n as u64;if received>m.installer.size{return Err("update_integrity".into());}output.write_all(&buffer[..n]).map_err(err)?;hash.update(&buffer[..n]);let mut d=self.data.lock().map_err(err)?;d.status.received=received;d.status.total=m.installer.size;}
 if received!=m.installer.size||format!("{:x}",hash.finalize())!=m.installer.sha256.to_lowercase(){return Err("update_integrity".into());}output.sync_all().map_err(err)?;crate::gallery::rename_handle(&output,&stage.join("setup.exe"))?;Ok(())})();if written.is_err(){let _=crate::gallery::delete_handle(&output);}written?;drop(output);
 for(name,b)in [("update.json",bytes),("update.sig",sig)]{let mut f=OpenOptions::new().create_new(true).write(true).open(stage.join(name)).map_err(err)?;f.write_all(&b).map_err(err)?;f.sync_all().map_err(err)?;}
 checked_installer(&stage,&public_key()?)?;let mut d=self.data.lock().map_err(err)?;d.stage=Some(stage);d.status.phase="ready".into();Ok(())})();self.finish(result)
 }
 fn arm(&self)->Result<()>{if self.portable||self.install.join("portable.marker").exists()||!self.install.join("installed.marker").is_file(){return Err("update_portable".into());}let mut d=self.data.lock().map_err(err)?;if d.status.phase!="ready"||d.helper.is_some(){return Err("update_state".into());}let stage=d.stage.as_ref().ok_or("update_state")?;let(_package,_m,_pins)=checked_installer(stage,&public_key()?)?;let exe=std::env::current_exe().map_err(err)?;let mut input=crate::gallery::lock_file(&exe)?;let hash=digest(&mut input)?;let token=uuid::Uuid::new_v4().to_string();let helper=stage.join(format!("update-helper-{token}.exe"));input.seek(SeekFrom::Start(0)).map_err(err)?;let mut output=OpenOptions::new().create_new(true).write(true).open(&helper).map_err(err)?;std::io::copy(&mut input,&mut output).map_err(err)?;output.sync_all().map_err(err)?;drop(output);let mut checked=crate::gallery::lock_file(&helper)?;if digest(&mut checked)?!=hash{return Err("update_integrity".into());}let mut child=Command::new(&helper).arg("--update-helper").arg(std::process::id().to_string()).arg(stage).arg(&self.install).arg(hash).creation_flags(0x08000000).spawn().map_err(err)?;let ready=helper.with_extension("ready");let deadline=std::time::Instant::now()+Duration::from_secs(8);while !ready.is_file(){if std::time::Instant::now()>deadline||child.try_wait().map_err(err)?.is_some(){let _=child.kill();let _=child.wait();return Err("update_storage".into());}std::thread::sleep(Duration::from_millis(40));}d.helper=Some(child);d.status.phase="installing".into();Ok(())}
 fn disarm(&self){if let Ok(mut d)=self.data.lock(){if let Some(mut child)=d.helper.take(){let _=child.kill();let _=child.wait();d.status.phase="ready".into();}}}
}
#[tauri::command]pub fn update_status(updater:State<'_,Arc<Updater>>)->Result<Status>{updater.status()}
#[tauri::command]pub async fn update_check(updater:State<'_,Arc<Updater>>)->Result<Status>{let u=updater.inner().clone();tauri::async_runtime::spawn_blocking(move||u.check()).await.map_err(err)?}
#[tauri::command]pub async fn update_download(updater:State<'_,Arc<Updater>>)->Result<Status>{let u=updater.inner().clone();tauri::async_runtime::spawn_blocking(move||u.download()).await.map_err(err)?}
#[tauri::command]pub fn update_cancel(updater:State<'_,Arc<Updater>>){updater.cancel.store(true,Ordering::SeqCst);}
#[tauri::command]pub async fn update_arm(updater:State<'_,Arc<Updater>>)->Result<()>{let u=updater.inner().clone();tauri::async_runtime::spawn_blocking(move||u.arm()).await.map_err(err)?}
#[tauri::command]pub fn update_disarm(updater:State<'_,Arc<Updater>>){updater.disarm();}
#[tauri::command]pub fn update_open_download(updater:State<'_,Arc<Updater>>)->Result<()>{let d=updater.data.lock().map_err(err)?;let m=d.status.latest.as_ref().ok_or("update_state")?;hub::open_external_https(&format!("{BASE}/tag/v{}",m.version))}

// Runs from a copy outside the installation, so Inno can replace the original EXE.
pub fn helper()->i32{let args:Vec<_>=std::env::args_os().collect();if args.len()!=6{return 2;}let stage=PathBuf::from(&args[3]);let install=PathBuf::from(&args[4]);let expected=args[5].to_string_lossy().to_string();let Ok(exe)=std::env::current_exe()else{return 2;};if !stage.is_absolute()||exe.parent()!=Some(stage.as_path())||model_library::no_links(&stage).is_err(){return 2;}if !install.is_absolute(){return 2;}let Ok(_stage_pins)=crate::gallery::directory_guards(&stage)else{return 2;};let mut restart=false;let result=(||->Result<()>{
 use windows_sys::Win32::{Foundation::CloseHandle,System::Threading::{OpenProcess,WaitForSingleObject,PROCESS_SYNCHRONIZE}};
 let pid=args[2].to_string_lossy().parse::<u32>().map_err(err)?;let process=unsafe{OpenProcess(PROCESS_SYNCHRONIZE,0,pid)};if process.is_null(){return Err("update_parent".into());}let ready=exe.with_extension("ready");let mut f=OpenOptions::new().create_new(true).write(true).open(&ready).map_err(err)?;f.write_all(b"ready").map_err(err)?;drop(f);let wait=unsafe{WaitForSingleObject(process,120000)};unsafe{CloseHandle(process);}if wait!=0{return Err("update_parent".into());}
 let _pins=crate::gallery::directory_guards(&install)?;if install.join("portable.marker").exists()||!install.join("installed.marker").is_file(){return Err("update_portable".into());}let mut old=crate::gallery::lock_file(&install.join("local-studio.exe"))?;if digest(&mut old)?!=expected{return Err("update_changed".into());}drop(old);restart=true;
 let(_file,_m,_pins)=checked_installer(&stage,&public_key()?)?;
 let status=Command::new(stage.join("setup.exe")).arg("/SILENT").arg("/SUPPRESSMSGBOXES").arg("/NORESTART").arg("/SP-").arg(format!("/DIR={}",install.display())).arg(format!("/LOG={}",stage.join("install.log").display())).creation_flags(0x08000000).status().map_err(err)?;
 if !matches!(status.code(),Some(0|3010)){return Err("update_install".into());}Ok(())})();
 let message=match &result{Ok(())=>"Update installed".into(),Err(e)=>e.clone()};if let Ok(mut file)=OpenOptions::new().create_new(true).write(true).open(stage.join("result.txt")){let _=file.write_all(message.as_bytes());}
 // Restart only the exact original destination after the attempted installation.
 if restart&&install.is_absolute()&&model_library::no_links(&install.join("local-studio.exe")).is_ok(){let _=Command::new(install.join("local-studio.exe")).creation_flags(0x08000000).spawn();}
 if result.is_ok(){0}else{1}
}

#[cfg(test)]mod tests{
 use super::*;use ring::signature::KeyPair;
 fn signed()->(Vec<u8>,Vec<u8>,Vec<u8>){let key=ring::signature::Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new()).unwrap();let pair=ring::signature::Ed25519KeyPair::from_pkcs8(key.as_ref()).unwrap();let version="99.0.0";let asset=|kind:&str|Asset{url:format!("{BASE}/download/v{version}/Local-Studio-{version}-hub-{kind}"),size:1,sha256:"a".repeat(64)};let m=Manifest{schema:1,version:version.into(),published_at:"2026-09-18".into(),notes:"Signed test fixture".into(),installer:asset("setup.exe"),portable:asset("portable.zip")};let bytes=serde_json::to_vec(&m).unwrap();let sig=base64::engine::general_purpose::STANDARD.encode(pair.sign(&bytes).as_ref()).into_bytes();(bytes,sig,pair.public_key().as_ref().to_vec())}
 #[test]fn signature_is_required_and_binds_every_manifest_byte(){let(b,s,k)=signed();assert_eq!(verified(&b,&s,&k).unwrap().version,"99.0.0");let mut changed=b.clone();changed[0]=b' ';assert!(verified(&changed,&s,&k).is_err());assert!(verified(&b,b"bad",&k).is_err());assert!(verified(&b,&s,&[0;32]).is_err());}
 #[test]fn versions_compare_numerically_and_reject_ambiguous_forms(){assert!(version("0.20.0").unwrap()>version("0.9.99").unwrap());for s in ["v1.0.0","1.0","1.0.0-beta","01.0.0","1.0.0.0","+1.0.0","1. 0.0"]{assert!(version(s).is_err());}}
 #[test]fn portable_never_downloads_or_arms(){let t=tempfile::tempdir().unwrap();let u=Updater::new(t.path(),t.path(),true);assert!(u.download().is_err());assert!(u.arm().is_err());assert!(!t.path().join("updates").exists());}
 fn stage(t:&Path,ver:&str,alter:impl FnOnce(&mut Manifest))->Vec<u8>{
  let key=ring::signature::Ed25519KeyPair::generate_pkcs8(&ring::rand::SystemRandom::new()).unwrap();let pair=ring::signature::Ed25519KeyPair::from_pkcs8(key.as_ref()).unwrap();
  let asset=|kind:&str|Asset{url:format!("{BASE}/download/v{ver}/Local-Studio-{ver}-hub-{kind}"),size:1,sha256:format!("{:x}",Sha256::digest(b"x"))};
  let mut m=Manifest{schema:1,version:ver.into(),published_at:"2026-09-18".into(),notes:String::new(),installer:asset("setup.exe"),portable:asset("portable.zip")};alter(&mut m);
  let bytes=serde_json::to_vec(&m).unwrap();let sig=base64::engine::general_purpose::STANDARD.encode(pair.sign(&bytes).as_ref());fs::write(t.join("update.json"),bytes).unwrap();fs::write(t.join("update.sig"),sig).unwrap();fs::write(t.join("setup.exe"),b"x").unwrap();pair.public_key().as_ref().to_vec()
 }
 #[test]fn staged_package_is_rechecked_before_execution(){let t=tempfile::tempdir().unwrap();let k=stage(t.path(),"99.0.0",|_|{});assert!(checked_installer(t.path(),&k).is_ok());fs::write(t.path().join("setup.exe"),b"y").unwrap();assert_eq!(checked_installer(t.path(),&k).unwrap_err(),"update_integrity");fs::write(t.path().join("setup.exe"),b"xx").unwrap();assert_eq!(checked_installer(t.path(),&k).unwrap_err(),"update_integrity");}
 #[test]fn signed_downgrades_and_same_version_are_rejected(){for v in ["0.0.1",env!("CARGO_PKG_VERSION")]{let t=tempfile::tempdir().unwrap();let k=stage(t.path(),v,|_|{});assert_eq!(checked_installer(t.path(),&k).unwrap_err(),"update_old");}}
 #[test]fn valid_signature_does_not_allow_foreign_urls_or_unbounded_packages(){for bad in 0..4{let t=tempfile::tempdir().unwrap();let k=stage(t.path(),"99.0.0",|m|match bad{0=>m.installer.url="https://example.com/setup.exe".into(),1=>m.installer.size=MAX_PACKAGE+1,2=>m.portable.url.push_str("?redirect=elsewhere"),_=>m.notes="x".repeat(16001)});assert_eq!(checked_installer(t.path(),&k).unwrap_err(),"update_manifest");}}
 #[test]fn operation_state_prevents_overlapping_checks_and_clears_old_progress(){let t=tempfile::tempdir().unwrap();let u=Updater::new(t.path(),t.path(),false);u.data.lock().unwrap().status.received=99;u.begin("checking").unwrap();assert_eq!(u.status().unwrap().received,0);assert_eq!(u.begin("downloading").unwrap_err(),"update_busy");u.finish(Err("update_network".into())).unwrap();assert_eq!(u.status().unwrap().phase,"error");u.begin("checking").unwrap();u.data.lock().unwrap().status.phase="ready".into();assert_eq!(u.begin("checking").unwrap_err(),"update_busy");assert_eq!(u.status().unwrap().phase,"ready");}
}
