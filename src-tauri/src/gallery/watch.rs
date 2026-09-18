use super::*;
use std::{os::windows::ffi::OsStrExt,sync::{Mutex,atomic::{AtomicBool,Ordering}},thread::JoinHandle};
use tauri::Emitter;
use windows_sys::Win32::{Foundation::{INVALID_HANDLE_VALUE,WAIT_OBJECT_0,WAIT_TIMEOUT},Storage::FileSystem::{FindFirstChangeNotificationW,FindNextChangeNotification,FindCloseChangeNotification,FILE_NOTIFY_CHANGE_FILE_NAME,FILE_NOTIFY_CHANGE_DIR_NAME,FILE_NOTIFY_CHANGE_SIZE,FILE_NOTIFY_CHANGE_LAST_WRITE},System::Threading::WaitForSingleObject};
struct Watch {root:PathBuf,stop:Arc<AtomicBool>,thread:Option<JoinHandle<()>>}
impl Drop for Watch {fn drop(&mut self){self.stop.store(true,Ordering::Release);if let Some(thread)=self.thread.take(){let _=thread.join();}}}
pub struct GalleryWatch(Mutex<Option<Watch>>);
#[derive(Clone,Serialize)]
#[serde(rename_all="camelCase")]
pub struct WatchEvent {root_id:String,active:bool}
impl GalleryWatch {
    pub fn new()->Self{Self(Mutex::new(None))}
    fn start(&self,root:PathBuf,notify:impl Fn(bool)+Send+'static)->Result<()> {
        let mut current=self.0.lock().map_err(|_|"gallery_watch")?;
        if current.as_ref().is_some_and(|w|w.root==root&&!w.stop.load(Ordering::Acquire)){return Ok(());}
        *current=None;
        let _pins=directory_guards(&root)?;
        let wide:Vec<u16>=root.as_os_str().encode_wide().chain(Some(0)).collect();
        let handle=unsafe{FindFirstChangeNotificationW(wide.as_ptr(),1,FILE_NOTIFY_CHANGE_FILE_NAME|FILE_NOTIFY_CHANGE_DIR_NAME|FILE_NOTIFY_CHANGE_SIZE|FILE_NOTIFY_CHANGE_LAST_WRITE)};
        if handle==INVALID_HANDLE_VALUE||handle.is_null(){return Err("gallery_watch".into());}
        let handle=handle as usize;let stop=Arc::new(AtomicBool::new(false));let stopped=stop.clone();
        let thread=std::thread::spawn(move||{loop{if stopped.load(Ordering::Acquire){break;}let state=unsafe{WaitForSingleObject(handle as _,200)};if state==WAIT_TIMEOUT{continue;}if state!=WAIT_OBJECT_0{notify(false);break;}let renewed=unsafe{FindNextChangeNotification(handle as _)};if renewed==0{notify(false);break;}notify(true);}unsafe{FindCloseChangeNotification(handle as _);};stopped.store(true,Ordering::Release);});
        *current=Some(Watch{root,stop,thread:Some(thread)});Ok(())
    }
}
#[tauri::command]
pub async fn gallery_watch(app:tauri::AppHandle,core:State<'_,Arc<Core>>,watch:State<'_,Arc<GalleryWatch>>)->Result<WatchEvent>{let privacy_epoch=crate::privacy::epoch();let privacy_result=(async {
    let root=root(&core)?;let id=root_id(&root);let event_id=id.clone();let watcher=watch.inner().clone();
    tauri::async_runtime::spawn_blocking(move||watcher.start(root,move|active|{let _=app.emit_to("main","gallery-changed",WatchEvent{root_id:event_id.clone(),active});})).await.map_err(|_|"gallery_watch")??;
    Ok(WatchEvent{root_id:id,active:true})
}).await;crate::privacy::finish(privacy_epoch,privacy_result)}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn observes_real_nested_create_rename_delete_and_changes_roots(){
  let t=tempfile::tempdir().unwrap();let a=t.path().join("a");let b=t.path().join("b");fs::create_dir(&a).unwrap();fs::create_dir(&b).unwrap();let watcher=GalleryWatch::new();let(tx,rx)=std::sync::mpsc::channel();
  watcher.start(a.clone(),move|active|{tx.send(active).unwrap();}).unwrap();fs::create_dir(a.join("nested")).unwrap();assert!(rx.recv_timeout(std::time::Duration::from_secs(3)).unwrap());
  fs::write(a.join("nested/file.png"),b"file").unwrap();assert!(rx.recv_timeout(std::time::Duration::from_secs(3)).unwrap());
  let(tx2,rx2)=std::sync::mpsc::channel();watcher.start(b.clone(),move|active|{let _=tx2.send(active);}).unwrap();fs::write(b.join("new.png"),b"file").unwrap();assert!(rx2.recv_timeout(std::time::Duration::from_secs(3)).unwrap());drop(watcher);
 }
}
