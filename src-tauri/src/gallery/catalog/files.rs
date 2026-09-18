use super::*;
use std::os::windows::{ffi::OsStrExt,io::AsRawHandle};
use windows_sys::Win32::Storage::FileSystem::{SetFileInformationByHandle,FILE_RENAME_INFO,FileRenameInfo,FILE_DISPOSITION_INFO,FileDispositionInfo};
pub const TRASH:&str=".local-studio-trash";
#[derive(Clone,Deserialize,Serialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct FileTarget {pub path:String,pub file_id:String,pub version:String}
#[derive(Deserialize)]
#[serde(tag="type",rename_all="camelCase",deny_unknown_fields)]
pub enum FileAction {Rename {name:String},Move {folder:String},Trash {confirmed:bool}}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct FileRequest {pub root_id:String,pub targets:Vec<FileTarget>,pub action:FileAction}
#[derive(Serialize,Default)]
#[serde(rename_all="camelCase")]
pub struct FileReport {pub completed:Vec<String>,pub errors:Vec<String>}
#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct TrashItem {pub id:String,pub original_path:String,pub stored_path:String,pub file_id:String,pub deleted_at:String,pub bytes:u64,pub kind:String}
#[derive(Clone,Serialize,Deserialize)]
struct Intent {id:String,source:String,destination:Option<String>,file_id:String,action:String,trash:Option<TrashItem>,job:Option<String>}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct TrashList {root_id:String,pub entries:Vec<TrashItem>,total:usize}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct TrashRequest {root_id:String,ids:Vec<String>,action:String,confirmed:bool}

pub(crate) fn destructive_file(path:&Path)->Result<File> {
    model_library::no_links(path).map_err(|_|"gallery_path")?;
    let file=OpenOptions::new().access_mode(0x80010000).share_mode(1).custom_flags(0x00200000).open(path).map_err(|_|"gallery_locked")?;
    let meta=file.metadata().map_err(|_|"gallery_missing")?;
    if !meta.is_file() || meta.file_attributes()&0x400!=0 {return Err("gallery_path".into());}Ok(file)
}
pub(crate) fn rename_handle(file:&File,to:&Path)->Result<()> {
    let name:Vec<u16>=to.as_os_str().encode_wide().chain(Some(0)).collect();
    let bytes=std::mem::offset_of!(FILE_RENAME_INFO,FileName)+name.len()*2;
    let mut buffer=vec![0usize;bytes.div_ceil(std::mem::size_of::<usize>())];
    let info=buffer.as_mut_ptr() as *mut FILE_RENAME_INFO;
    unsafe {(*info).FileNameLength=((name.len()-1)*2) as u32;std::ptr::copy_nonoverlapping(name.as_ptr(),std::ptr::addr_of_mut!((*info).FileName).cast::<u16>(),name.len());
        if SetFileInformationByHandle(file.as_raw_handle() as _,FileRenameInfo,info as _,bytes as u32)==0 {return Err("gallery_move".into());}}
    Ok(())
}
pub(crate) fn delete_handle(file:&File)->Result<()> {let info=FILE_DISPOSITION_INFO{DeleteFile:true};if unsafe {SetFileInformationByHandle(file.as_raw_handle() as _,FileDispositionInfo,&info as *const _ as _,std::mem::size_of_val(&info) as u32)}==0 {return Err("gallery_locked".into());}Ok(())}
pub(crate) fn delete_owned(path:&Path)->Result<()> {let _guards=directory_guards(path.parent().ok_or("gallery_path")?)?;let file=destructive_file(path)?;delete_handle(&file)}
fn internal(root:&Path,relative:&str)->Result<PathBuf> {resolve_internal(root,relative)}
fn destination(root:&Path,relative:&str)->Result<PathBuf> {
    let p=Path::new(relative);let name=p.file_name().and_then(|n|n.to_str()).filter(|n|valid_name(n)).ok_or("gallery_name")?;
    let parent=p.parent().and_then(|p|p.to_str()).ok_or("gallery_path")?;
    Ok(internal(root,parent)?.join(name))
}
fn matches(root:&Path,path:&str,id:&str)->bool {internal(root,path).ok().and_then(|p|path_identity(&p).ok()).as_deref()==Some(id)}
impl GalleryCatalog {
    fn journal(&self,root:&Path,intent:&Intent)->Result<()> {
        self.db.lock().map_err(|_|"gallery_storage")?.execute("INSERT INTO file_journal(root,id,json) VALUES(?1,?2,?3)",params![root_id(root),intent.id,serde_json::to_string(intent).map_err(|_|"gallery_storage")?]).map_err(|_|"gallery_storage")?;Ok(())
    }
    fn finish_intent(&self,root:&Path,intent:&Intent,images:&ImageEngine)->Result<()> {
        if let Some(job)=&intent.job {images.relocate_gallery(job,&root.join(&intent.source),intent.destination.as_ref().map(|p|root.join(p)).as_deref())?;}
        let mut db=self.db.lock().map_err(|_|"gallery_storage")?;let tx=db.transaction().map_err(|_|"gallery_storage")?;
        if intent.action=="trash" {let item=intent.trash.as_ref().ok_or("gallery_storage")?;tx.execute("INSERT OR REPLACE INTO trash(root,id,json) VALUES(?1,?2,?3)",params![root_id(root),item.id,serde_json::to_string(item).map_err(|_|"gallery_storage")?]).map_err(|_|"gallery_storage")?;}
        if intent.action=="restore"||intent.action=="purge" {tx.execute("DELETE FROM trash WHERE root=?1 AND id=?2",params![root_id(root),intent.trash.as_ref().ok_or("gallery_storage")?.id]).map_err(|_|"gallery_storage")?;}
        tx.execute("DELETE FROM file_journal WHERE root=?1 AND id=?2",params![root_id(root),intent.id]).map_err(|_|"gallery_storage")?;tx.commit().map_err(|_|"gallery_storage".into())
    }
    fn recover_files(&self,root:&Path,images:&ImageEngine)->Result<()> {
        let intents:Vec<Intent>={let db=self.db.lock().map_err(|_|"gallery_storage")?;let mut stmt=db.prepare("SELECT json FROM file_journal WHERE root=?1").map_err(|_|"gallery_storage")?;let rows=stmt.query_map([root_id(root)],|r|r.get::<_,String>(0)).map_err(|_|"gallery_storage")?;let mut out=Vec::new();for row in rows {out.push(serde_json::from_str(&row.map_err(|_|"gallery_storage")?).map_err(|_|"gallery_storage")?);}out};
        for intent in intents {
            let old=matches(root,&intent.source,&intent.file_id);let new=intent.destination.as_ref().is_some_and(|p|matches(root,p,&intent.file_id));
            if (!old && new) || (intent.action=="purge" && !root.join(&intent.source).exists()) {self.finish_intent(root,&intent,images)?;}
            else if old && !new {self.db.lock().map_err(|_|"gallery_storage")?.execute("DELETE FROM file_journal WHERE root=?1 AND id=?2",params![root_id(root),intent.id]).map_err(|_|"gallery_storage")?;}
            else {return Err("gallery_recovery".into());}
        }Ok(())
    }
    pub(in crate::gallery) fn recover(&self,root:&Path,images:&ImageEngine)->Result<()> {let _gate=self.files_gate.lock().map_err(|_|"gallery_storage")?;self.recover_files(root,images)}
    fn apply_intent(&self,root:&Path,intent:&Intent,file:File,images:&ImageEngine)->Result<()> {
        self.journal(root,intent)?;
        let result=if let Some(to)=&intent.destination {rename_handle(&file,&destination(root,to)?)} else {
            // Remove the app-owned temporary duplicate before final deletion, while
            // the selected gallery file is still safely retained in the trash.
            if let Some(job)=&intent.job {images.clear_gallery_duplicate(job)?;}delete_handle(&file)
        };
        drop(file);
        if result.is_err() {self.recover_files(root,images)?;return result;}
        self.finish_intent(root,intent,images).map_err(|_|"gallery_recovery".into())
    }
    fn file_action(&self,root:&Path,request:FileRequest,images:&ImageEngine)->Result<FileReport> {
        let _gate=self.files_gate.lock().map_err(|_|"gallery_storage")?;self.recover_files(root,images)?;
        if request.root_id!=root_id(root){return Err("gallery_changed".into());}
        if request.targets.is_empty()||request.targets.len()>50 {return Err("gallery_batch_limit".into());}
        if matches!(request.action,FileAction::Trash{confirmed:false}) {return Err("gallery_confirmation".into());}
        if matches!(request.action,FileAction::Rename{..}) && request.targets.len()!=1 {return Err("gallery_batch_limit".into());}
        self.sync_origins(root,images.list()?)?;
        let _root_guards=directory_guards(root)?;
        if matches!(request.action,FileAction::Trash{..}) {let path=root.join(TRASH);if !path.exists(){fs::create_dir(&path).map_err(|_|"gallery_storage")?;}directory_guards(&path)?;}
        let mut seen=HashSet::new();let mut destinations=HashSet::new();let mut prepared=Vec::new();
        for target in request.targets {
            if !seen.insert(target.file_id.clone()){return Err("gallery_batch_duplicate".into());}
            let source=resolve(root,&target.path)?;let _kind=media(&source).ok_or("gallery_format")?;
            let guards=directory_guards(source.parent().ok_or("gallery_path")?)?;let file=destructive_file(&source)?;let meta=file.metadata().map_err(|_|"gallery_missing")?;
            if identity(&file)?!=target.file_id || thumbnails::version(root,&source,&meta,Some(&target.file_id))!=target.version{return Err("gallery_changed".into());}
            let id=uuid::Uuid::new_v4().to_string();let mut item=None;
            let to=match &request.action {
                FileAction::Rename{name}=>{if !valid_name(name)||Path::new(name).extension().map(|s|s.to_ascii_lowercase())!=source.extension().map(|s|s.to_ascii_lowercase()){return Err("gallery_name".into());}source.parent().unwrap().join(name)},
                FileAction::Move{folder}=>resolve(root,folder)?.join(source.file_name().unwrap()),
                FileAction::Trash{..}=>{let stored=format!("{TRASH}/{id}.{}",source.extension().unwrap().to_string_lossy());item=Some(TrashItem{id:id.clone(),original_path:target.path.clone(),stored_path:stored.clone(),file_id:target.file_id.clone(),deleted_at:crate::database::now(),bytes:meta.len(),kind:_kind.0.into()});root.join(stored)},
            };
            if to==source {return Err("gallery_same_path".into());}
            let to_guards=directory_guards(to.parent().ok_or("gallery_path")?)?;
            if to.exists()||!destinations.insert(to.to_string_lossy().to_lowercase()){return Err("gallery_collision".into());}
            let intent=Intent{id,source:target.path,destination:Some(relative(root,&to)?),file_id:target.file_id.clone(),action:if item.is_some(){"trash"}else{"move"}.into(),trash:item,job:self.origin_job(root,&target.file_id)?};
            prepared.push((intent,file,guards,to_guards));
        }
        let mut report=FileReport::default();
        for (intent,file,_from,_to) in prepared {let path=intent.source.clone();match self.apply_intent(root,&intent,file,images){Ok(())=>report.completed.push(path),Err(error)=>{report.errors.push(format!("{path}: {error}"));break;}}}
        Ok(report)
    }
    #[cfg(test)]
    fn trash_all(&self,root:&Path)->Result<Vec<TrashItem>> {
        let db=self.db.lock().map_err(|_|"gallery_storage")?;let mut stmt=db.prepare("SELECT json FROM trash WHERE root=?1 ORDER BY json_extract(json,'$.deletedAt') DESC,id").map_err(|_|"gallery_storage")?;
        let rows=stmt.query_map([root_id(root)],|r|r.get::<_,String>(0)).map_err(|_|"gallery_storage")?;let mut items=Vec::new();for row in rows {items.push(serde_json::from_str(&row.map_err(|_|"gallery_storage")?).map_err(|_|"gallery_storage")?);}Ok(items)
    }
    fn trash_item(&self,root:&Path,id:&str)->Result<TrashItem>{
        let json:String=self.db.lock().map_err(|_|"gallery_storage")?.query_row("SELECT json FROM trash WHERE root=?1 AND id=?2",params![root_id(root),id],|r|r.get(0)).map_err(|_|"gallery_missing")?;
        serde_json::from_str(&json).map_err(|_|"gallery_storage".into())
    }
    fn trash_page(&self,root:&Path,offset:usize)->Result<TrashList>{
        if offset>1_000_000{return Err("gallery_query".into());}
        let db=self.db.lock().map_err(|_|"gallery_storage")?;
        let total:i64=db.query_row("SELECT count(*) FROM trash WHERE root=?1",[root_id(root)],|r|r.get(0)).map_err(|_|"gallery_storage")?;
        let mut stmt=db.prepare("SELECT json FROM trash WHERE root=?1 ORDER BY json_extract(json,'$.deletedAt') DESC,id LIMIT 50 OFFSET ?2").map_err(|_|"gallery_storage")?;
        let rows=stmt.query_map(params![root_id(root),offset as i64],|r|r.get::<_,String>(0)).map_err(|_|"gallery_storage")?;let mut entries=Vec::new();for row in rows{entries.push(serde_json::from_str(&row.map_err(|_|"gallery_storage")?).map_err(|_|"gallery_storage")?);}
        Ok(TrashList{root_id:root_id(root),total:total as usize,entries})
    }
    fn trash_action(&self,root:&Path,request:TrashRequest,images:&ImageEngine)->Result<FileReport> {
        let _gate=self.files_gate.lock().map_err(|_|"gallery_storage")?;self.recover_files(root,images)?;
        if request.root_id!=root_id(root){return Err("gallery_changed".into());}
        if !["restore","purge"].contains(&request.action.as_str()){return Err("gallery_query".into());}
        if request.action=="purge"&&!request.confirmed{return Err("gallery_confirmation".into());}
        if request.ids.is_empty()||request.ids.len()>50{return Err("gallery_batch_limit".into());}
        let mut unique=HashSet::new();let mut prepared=Vec::new();let mut destinations=HashSet::new();
        for id in request.ids {
            if !unique.insert(id.clone()){return Err("gallery_batch_duplicate".into());}
            let item=self.trash_item(root,&id)?;
            let from=internal(root,&item.stored_path)?;let guards=directory_guards(from.parent().ok_or("gallery_path")?)?;let file=destructive_file(&from)?;
            if identity(&file)?!=item.file_id {return Err("gallery_changed".into());}
            let mut to_guards=Vec::new();let dest=if request.action=="restore" {
                // Existing original folder is required; never follow or create an untrusted tree.
                let target=destination(root,&item.original_path)?;to_guards=directory_guards(target.parent().ok_or("gallery_path")?)?;
                if target.exists()||!destinations.insert(target.to_string_lossy().to_lowercase()){return Err("gallery_collision".into());}Some(item.original_path.clone())
            } else {None};
            let intent=Intent{id:uuid::Uuid::new_v4().to_string(),source:item.stored_path.clone(),destination:dest,file_id:item.file_id.clone(),action:request.action.clone(),job:self.origin_job(root,&item.file_id)?,trash:Some(item)};
            prepared.push((intent,file,guards,to_guards));
        }
        let mut report=FileReport::default();for (intent,file,_from,_to) in prepared {let name=intent.trash.as_ref().unwrap().original_path.clone();match self.apply_intent(root,&intent,file,images){Ok(())=>report.completed.push(name),Err(error)=>{report.errors.push(format!("{name}: {error}"));break;}}}Ok(report)
    }
}
#[tauri::command]
pub async fn gallery_file_action(request:FileRequest,core:State<'_,Arc<Core>>,catalog:State<'_,Arc<GalleryCatalog>>,images:State<'_,Arc<ImageEngine>>)->Result<FileReport>{let privacy_epoch=crate::privacy::epoch();let privacy_result=(async {let permit=thumbnails::exclusive().await?;let root=root(&core)?;let cat=catalog.inner().clone();let images=images.inner().clone();tauri::async_runtime::spawn_blocking(move||{let _permit=permit;cat.file_action(&root,request,&images)}).await.map_err(|_|"gallery_storage")?}).await;crate::privacy::finish(privacy_epoch,privacy_result)}
#[tauri::command]
pub async fn gallery_trash_list(offset:usize,core:State<'_,Arc<Core>>,catalog:State<'_,Arc<GalleryCatalog>>,images:State<'_,Arc<ImageEngine>>)->Result<TrashList>{let privacy_epoch=crate::privacy::epoch();let privacy_result=(async {let root=root(&core)?;let cat=catalog.inner().clone();let images=images.inner().clone();tauri::async_runtime::spawn_blocking(move||{cat.recover(&root,&images)?;cat.trash_page(&root,offset)}).await.map_err(|_|"gallery_storage")?}).await;crate::privacy::finish(privacy_epoch,privacy_result)}
#[tauri::command]
pub async fn gallery_trash_action(request:TrashRequest,core:State<'_,Arc<Core>>,catalog:State<'_,Arc<GalleryCatalog>>,images:State<'_,Arc<ImageEngine>>)->Result<FileReport>{let privacy_epoch=crate::privacy::epoch();let privacy_result=(async {let permit=thumbnails::exclusive().await?;let root=root(&core)?;let cache=PathBuf::from(core.storage_paths()?.cache);let cat=catalog.inner().clone();let images=images.inner().clone();tauri::async_runtime::spawn_blocking(move||{let _permit=permit;if request.action=="purge"&&request.confirmed {thumbnails::clear(&cache)?;}cat.trash_action(&root,request,&images)}).await.map_err(|_|"gallery_storage")?}).await;crate::privacy::finish(privacy_epoch,privacy_result)}
#[tauri::command]
pub async fn gallery_trash_detail(id:String,core:State<'_,Arc<Core>>,catalog:State<'_,Arc<GalleryCatalog>>)->Result<Detail>{let privacy_epoch=crate::privacy::epoch();let privacy_result=(async {let root=root(&core)?;let item=catalog.trash_item(&root,&id)?;let path=internal(&root,&item.stored_path)?;if path_identity(&path)?!=item.file_id{return Err("gallery_changed".into());}Ok(Detail{url:format!("http://gallery.localhost/{}/{}",root_id(&root),URL_SAFE_NO_PAD.encode(item.stored_path.as_bytes())),kind:item.kind,request:None,job_id:None,origin:None,dimensions:None})}).await;crate::privacy::finish(privacy_epoch,privacy_result)}
pub(in crate::gallery) fn allows_trash(cat:&GalleryCatalog,root:&Path,path:&str)->bool {registered_trash_id(cat,root,path).is_some_and(|id|matches(root,path,&id))}

pub(in crate::gallery) fn registered_trash_id(cat:&GalleryCatalog,root:&Path,path:&str)->Option<String>{cat.db.lock().ok()?.query_row("SELECT json_extract(json,'$.fileId') FROM trash WHERE root=?1 AND json_extract(json,'$.storedPath')=?2",params![root_id(root),path],|r|r.get(0)).ok()}

#[cfg(test)]
#[path="files_tests.rs"]
mod tests;
