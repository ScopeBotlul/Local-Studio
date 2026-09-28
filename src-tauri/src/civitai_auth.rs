use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{path::Path, sync::Mutex};
use zeroize::Zeroizing;

type Result<T> = std::result::Result<T, String>;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status { pub connected: bool }

pub struct CivitaiAuth { user: String, connected: Mutex<bool> }

impl CivitaiAuth {
 pub fn new(config:&Path)->Self { let user=format!("profile-{:x}",Sha256::digest(config.to_string_lossy().to_lowercase().as_bytes())); let connected=entry(&user).ok().and_then(|e|e.get_password().ok()).is_some(); Self{user,connected:Mutex::new(connected)} }
 pub fn status(&self)->Result<Status>{Ok(Status{connected:*self.connected.lock().map_err(|_|"secret_store")?})}
 pub fn connect(&self, token:String)->Result<Status>{let token=Zeroizing::new(token);let token=token.trim();if !(16..=2048).contains(&token.len())||token.chars().any(char::is_control){return Err("civitai_token".into())} entry(&self.user)?.set_password(token).map_err(|_|"secret_store")?;*self.connected.lock().map_err(|_|"secret_store")?=true;self.status()}
 pub fn logout(&self)->Result<Status>{match entry(&self.user)?.delete_credential(){Ok(())|Err(keyring::Error::NoEntry)=>{},Err(_)=>return Err("secret_store".into())};*self.connected.lock().map_err(|_|"secret_store")?=false;self.status()}
}
fn entry(user:&str)->Result<keyring::Entry>{keyring::Entry::new("Local Studio Civitai",user).map_err(|_|"secret_store".into())}
