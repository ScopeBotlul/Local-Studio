use super::*;
use image::{DynamicImage, ImageDecoder, ImageFormat, ImageReader, Limits};
use rusqlite::{params, Connection, OptionalExtension};
use std::{io::{BufReader, Cursor}, sync::OnceLock};
use tokio::sync::Semaphore;

const EDGE: u32 = 320;
const MAX_SOURCE: u64 = 64 * 1024 * 1024;
const MAX_PIXELS: u64 = 32_000_000;
const MAX_DECODED: u64 = 128 * 1024 * 1024;
const MAX_THUMB: usize = 512 * 1024;
const CACHE_BYTES: i64 = 64 * 1024 * 1024;
const CACHE_ENTRIES: i64 = 2000;
pub(super) async fn exclusive()->Result<tokio::sync::OwnedSemaphorePermit>{slots().acquire_many_owned(2).await.map_err(|_|"gallery_thumbnail".into())}
static SLOTS: OnceLock<Arc<Semaphore>> = OnceLock::new();
fn slots() -> Arc<Semaphore> { SLOTS.get_or_init(|| Arc::new(Semaphore::new(2))).clone() }

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ThumbnailQuery { root_id: String, path: String, version: String }
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail { data_url: String, width: u32, height: u32, cached: bool, cache_stored: bool }

// Include the full Windows timestamp, not the millisecond UI timestamp.
pub(super) fn version(root: &Path, path: &Path, meta: &fs::Metadata, id: Option<&str>) -> String {
    let value = format!("thumb-v1|{}|{}|{}|{}|{}|{}", root_id(root), path.to_string_lossy(), id.unwrap_or(""), meta.len(), meta.last_write_time(), meta.creation_time());
    format!("{:x}", Sha256::digest(value.as_bytes()))
}
fn decode(file: File) -> Result<(Vec<u8>,u32,u32)> {
    let mut reader = ImageReader::new(BufReader::new(file)).with_guessed_format().map_err(|_| "gallery_thumbnail")?;
    if !matches!(reader.format(),Some(ImageFormat::Png|ImageFormat::Jpeg|ImageFormat::WebP|ImageFormat::Gif|ImageFormat::Bmp)) { return Err("gallery_thumbnail".into()); }
    let mut limits=Limits::default(); limits.max_image_width=Some(16384); limits.max_image_height=Some(16384); limits.max_alloc=Some(MAX_DECODED);
    reader.limits(limits);
    let mut decoder=reader.into_decoder().map_err(|_| "gallery_thumbnail")?;
    let (width,height)=decoder.dimensions();
    // Check output size explicitly: codec allocation limits alone are best effort.
    if width==0 || height==0 || u64::from(width)*u64::from(height)>MAX_PIXELS || decoder.total_bytes()>MAX_DECODED { return Err("gallery_thumbnail_limit".into()); }
    let orientation=decoder.orientation().map_err(|_| "gallery_thumbnail")?;
    let decoded=DynamicImage::from_decoder(decoder).map_err(|_| "gallery_thumbnail")?;
    // Resize before applying orientation to avoid another full-sized allocation.
    let mut small=if width<=EDGE && height<=EDGE { decoded } else { decoded.thumbnail(EDGE,EDGE) };
    small.apply_orientation(orientation);
    let (width,height)=(small.width(),small.height());
    let mut bytes=Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(small.to_rgba8()).write_to(&mut bytes,ImageFormat::Png).map_err(|_| "gallery_thumbnail")?;
    let bytes=bytes.into_inner(); if bytes.len()>MAX_THUMB { return Err("gallery_thumbnail_limit".into()); }
    Ok((bytes,width,height))
}
fn cache(path: &Path) -> Result<(Connection,Vec<File>,File)> {
    let guards=directory_guards(path)?;
    let db_path=path.join("gallery-thumbnails-v1.sqlite3");
    // Create exclusively; keep the existing non-reparse file pinned while SQLite opens it.
    match OpenOptions::new().write(true).create_new(true).open(&db_path) { Ok(_) => {}, Err(e) if e.kind()==std::io::ErrorKind::AlreadyExists=>{}, Err(_)=>return Err("gallery_thumbnail_cache".into()) }
    model_library::no_links(&db_path).map_err(|_| "gallery_thumbnail_cache")?;
    let guard=OpenOptions::new().access_mode(0x80).share_mode(3).custom_flags(0x00200000).open(&db_path).map_err(|_| "gallery_thumbnail_cache")?;
    if guard.metadata().map_err(|_| "gallery_thumbnail_cache")?.file_attributes() & 0x400 != 0 { return Err("gallery_thumbnail_cache".into()); }
    for suffix in ["-journal","-wal","-shm"] { let sidecar=path.join(format!("gallery-thumbnails-v1.sqlite3{suffix}")); if sidecar.exists() { model_library::no_links(&sidecar).map_err(|_| "gallery_thumbnail_cache")?; } }
    let db=Connection::open(db_path).map_err(|_| "gallery_thumbnail_cache")?;
    db.busy_timeout(std::time::Duration::from_secs(5)).map_err(|_| "gallery_thumbnail_cache")?;
    db.execute_batch("CREATE TABLE IF NOT EXISTS thumbnails(key TEXT PRIMARY KEY, png BLOB NOT NULL, width INTEGER NOT NULL, height INTEGER NOT NULL, digest TEXT NOT NULL, used INTEGER NOT NULL);").map_err(|_| "gallery_thumbnail_cache")?;
    Ok((db,guards,guard))
}
fn get(db: &Connection,key: &str) -> Result<Option<(Vec<u8>,u32,u32)>> {
    let row: Option<(Vec<u8>,u32,u32,String)>=db.query_row("SELECT png,width,height,digest FROM thumbnails WHERE key=?1 AND length(png)<=?2",params![key,MAX_THUMB as i64],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional().map_err(|_| "gallery_thumbnail_cache")?;
    if let Some((bytes,width,height,digest))=row {
        if width>0 && height>0 && width<=EDGE && height<=EDGE && bytes.starts_with(b"\x89PNG\r\n\x1a\n") && format!("{:x}",Sha256::digest(&bytes))==digest {
            db.execute("UPDATE thumbnails SET used=?2 WHERE key=?1",params![key,chrono::Utc::now().timestamp_millis()]).map_err(|_| "gallery_thumbnail_cache")?;
            return Ok(Some((bytes,width,height)));
        }
    } Ok(None)
}
fn put(db: &mut Connection,key: &str,bytes: &[u8],width: u32,height: u32,budget: i64,count_limit: i64) -> Result<()> {
    let tx=db.transaction().map_err(|_| "gallery_thumbnail_cache")?;
    tx.execute("INSERT OR REPLACE INTO thumbnails VALUES(?1,?2,?3,?4,?5,?6)",params![key,bytes,width,height,format!("{:x}",Sha256::digest(bytes)),chrono::Utc::now().timestamp_millis()]).map_err(|_| "gallery_thumbnail_cache")?;
    loop {
        let (size,count):(i64,i64)=tx.query_row("SELECT coalesce(sum(length(png)),0),count(*) FROM thumbnails",[],|r|Ok((r.get(0)?,r.get(1)?))).map_err(|_| "gallery_thumbnail_cache")?;
        if size<=budget && count<=count_limit { break; }
        tx.execute("DELETE FROM thumbnails WHERE key=(SELECT key FROM thumbnails ORDER BY used,key LIMIT 1)",[]).map_err(|_| "gallery_thumbnail_cache")?;
    }
    tx.commit().map_err(|_| "gallery_thumbnail_cache".into())
}
fn thumbnail(root: &Path,cache_path: &Path,query: ThumbnailQuery) -> Result<Thumbnail> {
    if query.root_id!=root_id(root) { return Err("gallery_changed".into()); }
    let target=resolve(root,&query.path)?;
    let kind=media(&target).map(|(kind,_)|kind);
    if !matches!(kind,Some("image"|"video")) { return Err("gallery_thumbnail".into()); }
    let _guards=directory_guards(target.parent().ok_or("gallery_path")?)?;
    let file=lock_file(&target)?; let meta=file.metadata().map_err(|_| "gallery_missing")?;
    let id=catalog::identity(&file).ok();
    let key=version(root,&target,&meta,id.as_deref());
    if query.version!=key { return Err("gallery_changed".into()); }
    if kind==Some("image") && meta.len()>MAX_SOURCE { return Err("gallery_thumbnail_limit".into()); }
    let mut store=cache(cache_path).ok();
    if let Some((db,_,_))=&store {
        if let Ok(Some((bytes,width,height)))=get(db,&key) { return Ok(result(bytes,width,height,true,true)); }
    }
    if kind==Some("video") { return Err("gallery_video_frame".into()); }
    let (bytes,width,height)=decode(file)?;
    let stored=store.as_mut().is_some_and(|(db,_,_)|put(db,&key,&bytes,width,height,CACHE_BYTES,CACHE_ENTRIES).is_ok());
    Ok(result(bytes,width,height,false,stored))
}
fn result(bytes: Vec<u8>,width: u32,height: u32,cached: bool,cache_stored: bool) -> Thumbnail {
    Thumbnail { data_url:format!("data:image/png;base64,{}",base64::engine::general_purpose::STANDARD.encode(bytes)),width,height,cached,cache_stored }
}
pub(super) fn clear(path: &Path) -> Result<()> {
    let (db,_guards,_file)=cache(path)?;
    db.execute_batch("DELETE FROM thumbnails; VACUUM;").map_err(|_| "gallery_thumbnail_cache".into())
}
#[tauri::command]
pub async fn gallery_thumbnail(query: ThumbnailQuery,core: State<'_,Arc<Core>>) -> Result<Thumbnail> {
    let permit=slots().acquire_owned().await.map_err(|_| "gallery_thumbnail")?;
    let root=root(&core)?; let path=PathBuf::from(core.storage_paths()?.cache);
    tauri::async_runtime::spawn_blocking(move || { let _permit=permit; thumbnail(&root,&path,query) }).await.map_err(|_| "gallery_thumbnail")?
}
#[tauri::command]
pub async fn gallery_thumbnail_clear(core: State<'_,Arc<Core>>) -> Result<()> {
    let permit=slots().acquire_many_owned(2).await.map_err(|_| "gallery_thumbnail")?;
    let path=PathBuf::from(core.storage_paths()?.cache);
    tauri::async_runtime::spawn_blocking(move || { let _permit=permit; clear(&path) }).await.map_err(|_| "gallery_thumbnail_cache")?
}

#[cfg(test)]
#[path = "thumbnail_tests.rs"]
mod tests;

fn store_video(root:&Path,cache_path:&Path,query:ThumbnailQuery,png:String)->Result<Thumbnail>{
    if png.len()>MAX_THUMB*2 || query.root_id!=root_id(root){return Err("gallery_thumbnail_limit".into());}
    let target=resolve(root,&query.path)?;
    if media(&target).map(|m|m.0)!=Some("video"){return Err("gallery_thumbnail".into());}
    let _pins=directory_guards(target.parent().ok_or("gallery_path")?)?;let file=lock_file(&target)?;let meta=file.metadata().map_err(|_|"gallery_missing")?;
    if version(root,&target,&meta,catalog::identity(&file).ok().as_deref())!=query.version{return Err("gallery_changed".into());}
    let bytes=base64::engine::general_purpose::STANDARD.decode(png).map_err(|_|"gallery_thumbnail")?;
    if bytes.len()>MAX_THUMB{return Err("gallery_thumbnail_limit".into());}
    let mut reader=ImageReader::with_format(Cursor::new(&bytes),ImageFormat::Png);let mut limits=Limits::default();limits.max_image_width=Some(EDGE);limits.max_image_height=Some(EDGE);limits.max_alloc=Some(2*1024*1024);reader.limits(limits);
    let image=reader.decode().map_err(|_|"gallery_thumbnail")?;let (width,height)=(image.width(),image.height());if width==0||height==0{return Err("gallery_thumbnail".into());}
    let stored=cache(cache_path).is_ok_and(|(mut db,_pins,_file)|put(&mut db,&query.version,&bytes,width,height,CACHE_BYTES,CACHE_ENTRIES).is_ok());
    Ok(result(bytes,width,height,false,stored))
}
#[tauri::command]
pub async fn gallery_video_thumbnail_store(query:ThumbnailQuery,png:String,core:State<'_,Arc<Core>>)->Result<Thumbnail>{
    let permit=slots().acquire_owned().await.map_err(|_|"gallery_thumbnail")?;let root=root(&core)?;let path=PathBuf::from(core.storage_paths()?.cache);
    tauri::async_runtime::spawn_blocking(move||{let _permit=permit;store_video(&root,&path,query,png)}).await.map_err(|_|"gallery_thumbnail")?
}
#[cfg(test)]mod video_tests {
 use super::*;
 fn query(root:&Path)->ThumbnailQuery{let path=root.join("video.webm");let file=lock_file(&path).unwrap();ThumbnailQuery{root_id:root_id(root),path:"video.webm".into(),version:version(root,&path,&file.metadata().unwrap(),catalog::identity(&file).ok().as_deref())}}
 #[test]fn video_frames_cache_but_changed_sources_invalid_png_and_oversized_frames_are_rejected(){let t=tempfile::tempdir().unwrap();let root=fs::canonicalize(t.path()).unwrap();let cache=root.join("cache");fs::create_dir(&cache).unwrap();fs::write(root.join("video.webm"),b"passive video fixture").unwrap();assert!(thumbnail(&root,&cache,query(&root)).is_err());let mut bytes=Cursor::new(Vec::new());DynamicImage::new_rgb8(160,90).write_to(&mut bytes,ImageFormat::Png).unwrap();let encoded=base64::engine::general_purpose::STANDARD.encode(bytes.get_ref());assert!(store_video(&root,&cache,query(&root),encoded.clone()).unwrap().cache_stored);let hit=thumbnail(&root,&cache,query(&root)).unwrap();assert!(hit.cached);assert_eq!((hit.width,hit.height),(160,90));let old=query(&root);fs::write(root.join("video.webm"),b"changed fixture bytes").unwrap();assert!(store_video(&root,&cache,old,encoded).is_err());assert!(store_video(&root,&cache,query(&root),"invalid PNG".into()).is_err());let mut bytes=Cursor::new(Vec::new());DynamicImage::new_rgb8(321,1).write_to(&mut bytes,ImageFormat::Png).unwrap();assert!(store_video(&root,&cache,query(&root),base64::engine::general_purpose::STANDARD.encode(bytes.get_ref())).is_err());}
}
