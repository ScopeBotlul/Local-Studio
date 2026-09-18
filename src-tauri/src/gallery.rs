mod watch;
pub use watch::{GalleryWatch,gallery_watch};
use crate::{core::Core, image_engine::{ImageEngine, ImageRequest}, model_library};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{fs::{self, File, OpenOptions}, io::{Read, Seek, SeekFrom, Write}, path::{Path, PathBuf}, sync::Arc, time::{SystemTime, UNIX_EPOCH}};
use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
use tauri::{Manager, State};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
mod compare;
pub use compare::gallery_compare;
mod thumbnails;
pub use thumbnails::{gallery_thumbnail,gallery_thumbnail_clear,gallery_video_thumbnail_store};
mod catalog;
pub(crate) use catalog::editor::{Operation as EditOperation,Preview as EditPreview,render_preview,render_bytes,render_rgba,encode_rgba,validate_operations};
pub(crate) use thumbnails::exclusive as editor_permit;
pub use catalog::{editor_preview,editor_export};
pub use catalog::{gallery_lineage,gallery_create_variant,gallery_set_primary,GalleryCatalog, gallery_annotate, gallery_annotate_batch, gallery_file_action,gallery_trash_list,gallery_trash_action,gallery_trash_detail};
pub(crate) use catalog::files::delete_owned;
pub(crate) use catalog::files::{destructive_file,rename_handle,delete_handle};
pub(crate) use catalog::files::FileTarget;
use catalog::{Origin,origins::{BoundOrigin,stamp},files::TRASH};
use catalog::Annotation;
use std::collections::{HashMap, BTreeMap};
type Result<T> = std::result::Result<T, String>;
const MAX_ENTRIES: usize = 20_000;
const CHUNK: u64 = 4 * 1024 * 1024;
const MAX_IMAGE: u64 = 32 * 1024 * 1024;

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Entry { pub locked:bool, pub origin: Option<Origin>, pub path: String, pub name: String, pub kind: String, pub bytes: u64, pub modified: u64, pub file_id: Option<String>, pub annotation: Annotation, pub thumbnail_version: String }
#[derive(Clone,Copy,Default,Deserialize)]
#[serde(rename_all="camelCase")]
pub enum Sort { #[default] ModifiedDesc, ModifiedAsc, NameAsc, NameDesc, SizeAsc, SizeDesc }
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Query { #[serde(default)] sort:Sort, folder: String, search: String, kind: String, recursive: bool, offset: usize, #[serde(default)] favorites_only: bool, #[serde(default)] tag: String }
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Listing { root: String, root_id: String, folder: String, folders: Vec<Entry>, entries: Vec<Entry>, total: usize, skipped: usize, limited: bool, tags: Vec<String> }
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Detail { origin: Option<Origin>, dimensions: Option<(u32,u32)>, url: String, kind: String, request: Option<ImageRequest>, job_id: Option<String> }
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult { pub(crate) imported: Vec<String>, pub(crate) errors: Vec<String> }

pub(crate) fn media(path: &Path) -> Option<(&'static str, &'static str)> {
    Some(match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "png" => ("image", "image/png"), "jpg" | "jpeg" => ("image", "image/jpeg"),
        "webp" => ("image", "image/webp"), "gif" => ("image", "image/gif"), "bmp" => ("image", "image/bmp"), "avif" => ("image", "image/avif"),
        "mp4" | "m4v" => ("video", "video/mp4"), "webm" => ("video", "video/webm"), "mov" => ("video", "video/quicktime"), "mkv" => ("video", "video/x-matroska"), "avi" => ("video", "video/x-msvideo"),
        "mp3" => ("audio", "audio/mpeg"), "wav" => ("audio", "audio/wav"), "ogg" | "opus" => ("audio", "audio/ogg"), "flac" => ("audio", "audio/flac"), "m4a" => ("audio", "audio/mp4"),
        _ => return None,
    })
}
fn relative(root: &Path, path: &Path) -> Result<String> { Ok(path.strip_prefix(root).map_err(|_| "gallery_path")?.to_str().ok_or("gallery_path")?.replace('\\', "/")) }
pub(crate) fn root_id(root: &Path) -> String { format!("{:x}", Sha256::digest(root.to_string_lossy().to_lowercase().as_bytes())) }
fn resolve(root: &Path, relative: &str) -> Result<PathBuf> {
    if relative.split('/').next().is_some_and(|p|p.eq_ignore_ascii_case(TRASH)){return Err("gallery_path".into());}
    resolve_internal(root,relative)
}
fn resolve_internal(root: &Path, relative: &str) -> Result<PathBuf> {
    if relative.len() > 32768 || relative.contains(['\\', ':', '\0']) || relative.split('/').any(|c| c == ".." || c == "." || c.ends_with([' ', '.'])) || relative.starts_with('/') { return Err("gallery_path".into()); }
    let path = root.join(relative); model_library::no_links(&path).map_err(|_| "gallery_path")?;
    let path = fs::canonicalize(path).map_err(|_| "gallery_missing")?;
    if !path.starts_with(root) { return Err("gallery_path".into()); } Ok(path)
}
pub(crate) fn root(core: &Core) -> Result<PathBuf> {
    let path = PathBuf::from(core.storage_paths()?.gallery);
    model_library::no_links(&path).map_err(|_| "gallery_path")?;
    fs::canonicalize(path).map_err(|_| "gallery_missing".into())
}
pub(crate) fn file_binding(file:&File)->Result<String>{Ok(format!("{}:{}",catalog::identity(file)?,stamp(&file.metadata().map_err(|_|"gallery_missing")?)))}
pub(crate) fn saved_binding(path:&Path)->Result<String>{file_binding(&lock_file(path)?)}
pub(crate) fn lock_file(path: &Path) -> Result<File> {
    model_library::no_links(path).map_err(|_| "gallery_path")?;
    let file = OpenOptions::new().read(true).share_mode(1).custom_flags(0x00200000).open(path).map_err(|_| "gallery_missing")?;
    let meta = file.metadata().map_err(|_| "gallery_missing")?;
    if !meta.is_file() || meta.file_attributes() & 0x400 != 0 { return Err("gallery_path".into()); } Ok(file)
}
pub(crate) fn directory_guards(path: &Path) -> Result<Vec<File>> {
    let mut guards = Vec::new();
    for ancestor in path.ancestors().collect::<Vec<_>>().into_iter().rev() {
        let file = OpenOptions::new().access_mode(0x80).share_mode(3).custom_flags(0x02200000).open(ancestor).map_err(|_| "gallery_path")?;
        let meta = file.metadata().map_err(|_| "gallery_path")?;
        if !meta.is_dir() || meta.file_attributes() & 0x400 != 0 { return Err("gallery_path".into()); }
        guards.push(file);
    } Ok(guards)
}
fn entry(root: &Path, path: &Path, kind: &str, meta: &fs::Metadata) -> Result<Entry> {
    let id = if kind == "folder" { None } else { catalog::path_identity(path).ok() };
    let thumbnail_version = thumbnails::version(root,path,meta,id.as_deref());
    Ok(Entry { locked:false, origin:None, thumbnail_version, path: relative(root, path)?, name: path.file_name().and_then(|n| n.to_str()).ok_or("gallery_path")?.into(), kind: kind.into(), bytes: meta.len(), file_id: id, annotation: Annotation::default(), modified: meta.modified().unwrap_or(SystemTime::UNIX_EPOCH).duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64 })
}
#[cfg(test)]
fn list(root: &Path, query: Query) -> Result<Listing> { list_with(root,query,&HashMap::new()) }
#[cfg(test)]
fn list_with(root: &Path, query: Query, annotations: &HashMap<String,Annotation>) -> Result<Listing> { list_full(root,query,annotations,&HashMap::new()) }
fn list_full(root: &Path, query: Query, annotations: &HashMap<String,Annotation>,origins:&HashMap<String,BoundOrigin>) -> Result<Listing> {
    if query.search.len() > 256 || query.tag.len() > 256 || !["all", "image", "video", "audio"].contains(&query.kind.as_str()) || query.offset > MAX_ENTRIES { return Err("gallery_query".into()); }
    let folder = resolve(root, &query.folder)?; let _guards = directory_guards(&folder)?;
    let mut queue = vec![(folder.clone(), 0)]; let mut entries = Vec::new(); let mut folders = Vec::new(); let mut skipped = 0; let mut visited = 0; let mut limited = false;
    let search = query.search.to_lowercase(); let tag = query.tag.to_lowercase(); let mut available_tags = BTreeMap::new();
    'scan: while let Some((dir, depth)) = queue.pop() {
        if model_library::no_links(&dir).is_err() { skipped += 1; continue; }
        let Ok(read) = fs::read_dir(&dir) else { skipped += 1; continue; };
        for next in read {
            visited += 1; if visited > MAX_ENTRIES { limited = true; break 'scan; }
            let Ok(next) = next else { skipped += 1; continue; }; let path = next.path();
            if dir==root && next.file_name().to_string_lossy().eq_ignore_ascii_case(TRASH){continue;}
            let Ok(meta) = fs::symlink_metadata(&path) else { skipped += 1; continue; };
            if meta.file_attributes() & 0x400 != 0 { skipped += 1; continue; }
            if meta.is_dir() {
                if dir == folder { if let Ok(e) = entry(root, &path, "folder", &meta) { folders.push(e); } }
                if query.recursive { if depth < 32 { queue.push((path, depth + 1)); } else { limited = true; } }
            } else if meta.is_file() {
                if let Some((kind, _)) = media(&path) {
                    if let Ok(mut e) = entry(root, &path, kind, &meta) {
                        e.origin=e.file_id.as_ref().and_then(|id|origins.get(id)).filter(|o|o.stamp==stamp(&meta)).map(|o|o.info.clone());
                        if let Some(saved) = e.file_id.as_ref().and_then(|id| annotations.get(id)) { e.annotation = saved.clone(); }
                        if crate::privacy::locked()&&crate::privacy::media(&path){
                            if !search.is_empty()||!tag.is_empty()||query.favorites_only{continue;}
                            if query.kind!="all"&&query.kind!=kind{continue;}
                            e.locked=true;e.path=format!("__restricted__/{}",e.thumbnail_version);e.name="18+".into();e.bytes=0;e.modified=0;e.file_id=None;e.annotation=Annotation::default();e.origin=None;e.thumbnail_version.clear();entries.push(e);continue;
                        }
                        for name in &e.annotation.tags { available_tags.entry(name.to_lowercase()).or_insert_with(|| name.clone()); }
                        if query.kind != "all" && query.kind != kind { continue; }
                        if query.favorites_only && !e.annotation.favorite { continue; }
                        if !tag.is_empty() && !e.annotation.tags.iter().any(|t| t.to_lowercase() == tag) { continue; }
                        if e.origin.as_ref().is_some_and(|o|o.model_name.to_lowercase().contains(&search)||o.request.prompt.to_lowercase().contains(&search)||o.request.negative_prompt.to_lowercase().contains(&search)) || e.path.to_lowercase().contains(&search) || e.annotation.tags.iter().any(|t| t.to_lowercase().contains(&search)) { entries.push(e); }
                    }
                }
            }
        }
    }
    entries.sort_by(|a,b| {
        let order=match query.sort {Sort::ModifiedDesc=>b.modified.cmp(&a.modified),Sort::ModifiedAsc=>a.modified.cmp(&b.modified),Sort::NameAsc=>a.name.to_lowercase().cmp(&b.name.to_lowercase()),Sort::NameDesc=>b.name.to_lowercase().cmp(&a.name.to_lowercase()),Sort::SizeAsc=>a.bytes.cmp(&b.bytes),Sort::SizeDesc=>b.bytes.cmp(&a.bytes)};
        order.then(a.path.cmp(&b.path))
    }); folders.sort_by_key(|e| e.name.to_lowercase());
    let total = entries.len(); let entries = entries.into_iter().skip(query.offset).take(50).collect();
    // Folder buttons are bounded separately, with a visible limit indication.
    if folders.len() > 200 { folders.truncate(200); limited = true; }
    Ok(Listing { root: root.to_string_lossy().into(), root_id: root_id(root), folder: query.folder, folders, entries, total, skipped, limited, tags: available_tags.into_values().collect() })
}
pub(crate) fn valid_name(name: &str) -> bool {
    !name.is_empty() && name.len() <= 180 && !name.contains(['/', '\\', ':', '<', '>', '"', '|', '?', '*']) && !name.chars().any(char::is_control) && !name.ends_with(['.', ' ']) && !["CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9"].contains(&name.split('.').next().unwrap_or("").to_uppercase().as_str())
}
fn publish(source: &Path, destination: &Path) -> std::io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    let source: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
    let destination: Vec<u16> = destination.as_os_str().encode_wide().chain(Some(0)).collect();
    if unsafe { windows_sys::Win32::Storage::FileSystem::MoveFileExW(source.as_ptr(), destination.as_ptr(), 0) } == 0 { Err(std::io::Error::last_os_error()) } else { Ok(()) }
}
pub(crate) fn copy_one(root: &Path, folder: &Path, source: &Path) -> Result<String> {
    let name = source.file_name().and_then(|n| n.to_str()).ok_or("gallery_name")?;
    copy_one_named(root, folder, source, name)
}
pub(crate) fn project_sources(core:&Core,expected:&str,targets:&[FileTarget])->Result<Vec<(PathBuf,File,Vec<File>)>> {
    let root=root(core)?;if root_id(&root)!=expected||targets.is_empty()||targets.len()>100{return Err("gallery_changed".into());}
    let mut result=vec![];let mut seen=std::collections::HashSet::new();
    for target in targets {
        if !seen.insert(&target.file_id){return Err("gallery_changed".into());}
        let path=resolve(&root,&target.path)?;let pins=directory_guards(path.parent().ok_or("gallery_path")?)?;let file=lock_file(&path)?;
        let meta=file.metadata().map_err(|_|"gallery_missing")?;
        if catalog::identity(&file)?!=target.file_id||thumbnails::version(&root,&path,&meta,Some(&target.file_id))!=target.version{return Err("gallery_changed".into());}
        result.push((path,file,pins));
    }Ok(result)
}
pub(crate) fn copy_one_named(root: &Path, folder: &Path, source: &Path, name: &str) -> Result<String> {
    if !valid_name(name) || media(Path::new(name)) != media(source) { return Err("gallery_name".into()); }
    if !source.is_absolute() || media(source).is_none() { return Err("gallery_format".into()); }
    let _source_guards = directory_guards(source.parent().ok_or("gallery_path")?)?;
    let mut input = lock_file(source)?;
    let size = input.metadata().map_err(|_| "gallery_missing")?.len(); if size > 8 * 1024 * 1024 * 1024 { return Err("gallery_import_size".into()); }
    let temp = folder.join(format!(".local-studio-import-{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut out = OpenOptions::new().create_new(true).write(true).open(&temp).map_err(|_| "gallery_storage")?;
        let mut digest = Sha256::new(); let mut bytes = 0; let mut buffer = vec![0; 1024 * 1024];
        loop { let n = input.read(&mut buffer).map_err(|_| "gallery_missing")?; if n == 0 { break; } out.write_all(&buffer[..n]).map_err(|_| "gallery_storage")?; digest.update(&buffer[..n]); bytes += n as u64; }
        out.sync_all().map_err(|_| "gallery_storage")?; drop(out);
        if bytes != size { return Err("gallery_storage".into()); }
        let mut verify = File::open(&temp).map_err(|_| "gallery_storage")?; let mut actual = Sha256::new();
        loop { let n = verify.read(&mut buffer).map_err(|_| "gallery_storage")?; if n == 0 { break; } actual.update(&buffer[..n]); } drop(verify);
        if actual.finalize() != digest.finalize() { return Err("gallery_storage".into()); }
        let mut destination = folder.join(name);
        // Publish without MOVEFILE_REPLACE_EXISTING. Retry unique names on races.
        for attempt in 0..5 {
            if attempt > 0 || destination.exists() { destination = folder.join(format!("{}-{}.{}", Path::new(name).file_stem().unwrap().to_string_lossy(), uuid::Uuid::new_v4(), Path::new(name).extension().unwrap().to_string_lossy())); }
            match publish(&temp, &destination) { Ok(()) => {crate::privacy::inherit(source,&destination)?;return relative(root, &destination);}, Err(_) if destination.exists() => continue, Err(_) => return Err("gallery_storage".into()) }
        } Err("gallery_storage".into())
    })();
    if result.is_err() { let _ = fs::remove_file(&temp); } result
}

#[tauri::command]
pub async fn gallery_list(query: Query, core: State<'_, Arc<Core>>, catalog: State<'_,Arc<GalleryCatalog>>, images:State<'_,Arc<ImageEngine>>) -> Result<Listing> {let privacy_epoch=crate::privacy::epoch();let privacy_result=(async {
    let root = root(&core)?; let catalog = catalog.inner().clone();let images=images.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {catalog.recover(&root,&images)?;let origins=catalog.sync_origins(&root,images.list()?)?;list_full(&root,query,&catalog.snapshot(&root)?,&origins)}).await.map_err(|_| "gallery_storage")?
}).await;crate::privacy::finish(privacy_epoch,privacy_result)}
#[tauri::command]
pub async fn gallery_detail(path: String, core: State<'_, Arc<Core>>, images: State<'_, Arc<ImageEngine>>,catalog:State<'_,Arc<GalleryCatalog>>) -> Result<Detail> {let privacy_epoch=crate::privacy::epoch();let privacy_result=(async {
    let root = root(&core)?; let images = images.inner().clone();let catalog=catalog.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        catalog.recover(&root,&images)?;let origins=catalog.sync_origins(&root,images.list()?)?;
        let target = resolve(&root, &path)?; crate::privacy::check(&target)?; let file = lock_file(&target)?;let meta=file.metadata().map_err(|_|"gallery_missing")?; let (kind, _) = media(&target).ok_or("gallery_format")?;
        let origin=catalog::identity(&file).ok().and_then(|id|origins.get(&id)).filter(|o|o.stamp==stamp(&meta)).map(|o|o.info.clone());
        let dimensions=if kind=="image"&&meta.len()<=64*1024*1024 {image::ImageReader::new(std::io::BufReader::new(file)).with_guessed_format().ok().and_then(|mut r|{let mut limits=image::Limits::default();limits.max_alloc=Some(128*1024*1024);limits.max_image_width=Some(16384);limits.max_image_height=Some(16384);r.limits(limits);r.into_dimensions().ok()})}else{None};
        Ok(Detail { url: format!("http://gallery.localhost/{}/{}", root_id(&root), URL_SAFE_NO_PAD.encode(path.as_bytes())), kind: kind.into(), request: origin.as_ref().map(|o|o.request.clone()),job_id:origin.as_ref().map(|o|o.job_id.clone()),origin,dimensions })
    }).await.map_err(|_| "gallery_storage")?
}).await;crate::privacy::finish(privacy_epoch,privacy_result)}
#[tauri::command]
pub async fn gallery_import(folder: String, sources: Vec<String>, core: State<'_, Arc<Core>>) -> Result<ImportResult> {let privacy_epoch=crate::privacy::epoch();let privacy_result=(async {
    if sources.is_empty() || sources.len() > 100 { return Err("gallery_import_limit".into()); }
    let root = root(&core)?;
    tauri::async_runtime::spawn_blocking(move || {
        let folder = resolve(&root, &folder)?; let _guards = directory_guards(&folder)?;
        let mut result = ImportResult { imported: Vec::new(), errors: Vec::new() };
        for source in sources { match copy_one(&root, &folder, Path::new(&source)) { Ok(path) => result.imported.push(path), Err(error) => result.errors.push(format!("{}: {error}", Path::new(&source).file_name().unwrap_or_default().to_string_lossy())) } } Ok(result)
    }).await.map_err(|_| "gallery_storage")?
}).await;crate::privacy::finish(privacy_epoch,privacy_result)}
#[tauri::command]
pub async fn gallery_create_folder(folder: String, name: String, core: State<'_, Arc<Core>>) -> Result<()> {let privacy_epoch=crate::privacy::epoch();let privacy_result=(async {
    if !valid_name(&name) || name.eq_ignore_ascii_case(TRASH) { return Err("gallery_name".into()); } let root = root(&core)?;
    tauri::async_runtime::spawn_blocking(move || { let folder = resolve(&root, &folder)?; let _guards = directory_guards(&folder)?; fs::create_dir(folder.join(name)).map_err(|_| "gallery_storage".into()) }).await.map_err(|_| "gallery_storage")?
}).await;crate::privacy::finish(privacy_epoch,privacy_result)}
#[tauri::command]
pub async fn gallery_open_folder(folder: String, core: State<'_, Arc<Core>>) -> Result<()> {let privacy_epoch=crate::privacy::epoch();let privacy_result=(async {
    let root = root(&core)?;
    tauri::async_runtime::spawn_blocking(move || {
        let target = resolve(&root, &folder)?; let _guards = directory_guards(&target)?;
        let exe = PathBuf::from(std::env::var_os("SystemRoot").ok_or("gallery_storage")?).join("explorer.exe");
        std::process::Command::new(exe).arg(target.to_string_lossy().trim_start_matches(r"\\?\")).spawn().map_err(|_| "gallery_storage")?; Ok(())
    }).await.map_err(|_| "gallery_storage")?
}).await;crate::privacy::finish(privacy_epoch,privacy_result)}

fn range(value: Option<&str>, size: u64, image: bool) -> Result<(u64,u64,bool)> {
    if size == 0 { return Err("range".into()); }
    if let Some(value) = value {
        let (start,end) = value.strip_prefix("bytes=").ok_or("range")?.split_once('-').ok_or("range")?;
        if value.contains(',') { return Err("range".into()); }
        let (start,end) = if start.is_empty() { let n: u64 = end.parse().map_err(|_| "range")?; if n == 0 { return Err("range".into()); } (size.saturating_sub(n),size-1) }
        else { (start.parse::<u64>().map_err(|_| "range")?, if end.is_empty() { size-1 } else { end.parse::<u64>().map_err(|_| "range")?.min(size-1) }) };
        if start >= size || end < start { return Err("range".into()); } Ok((start, end.min(start.saturating_add(CHUNK-1)), true))
    } else if image { if size > MAX_IMAGE { return Err("size".into()); } Ok((0,size-1,false)) }
    else { Ok((0,(size-1).min(CHUNK-1),size>CHUNK)) }
}
pub fn response(root: &Path, label: &str, req: tauri::http::Request<Vec<u8>>) -> tauri::http::Response<Vec<u8>> {
    response_catalog(root,label,req,None)
}
fn response_catalog(root:&Path,label:&str,req:tauri::http::Request<Vec<u8>>,catalog:Option<&GalleryCatalog>)->tauri::http::Response<Vec<u8>> {
    use tauri::http::Response;
    let denied = |status| Response::builder().status(status).header("Cache-Control", "no-store").body(Vec::new()).unwrap();
    if label != "main" { return denied(403); }
    if req.method() != "GET" && req.method() != "HEAD" { return denied(405); }
    let parts: Vec<_> = req.uri().path().trim_start_matches('/').split('/').collect();
    if parts.len() != 2 || parts[0] != root_id(root) { return denied(403); }
    let path = URL_SAFE_NO_PAD.decode(parts[1]).ok().and_then(|p| String::from_utf8(p).ok()).and_then(|p| {if p.split('/').next().is_some_and(|s|s.eq_ignore_ascii_case(TRASH)) && catalog.is_some_and(|c|catalog::files::allows_trash(c,root,&p)){resolve_internal(root,&p).ok()}else{resolve(root,&p).ok()}});
    let Some(path) = path else { return denied(403); };
    if crate::privacy::check(&path).is_err(){return denied(403);}
    let Some((kind,mime)) = media(&path) else { return denied(415); };
    let Ok(_guards) = directory_guards(path.parent().unwrap()) else { return denied(403); };
    let Ok(mut file) = lock_file(&path) else { return denied(404); };
    let Ok(meta) = file.metadata() else { return denied(404); }; let size = meta.len();
    if let Ok(rel)=relative(root,&path) {if rel.split('/').next().is_some_and(|p|p.eq_ignore_ascii_case(TRASH)) && !catalog.is_some_and(|cat|catalog::files::registered_trash_id(cat,root,&rel).is_some_and(|id|catalog::identity(&file).ok().as_deref()==Some(id.as_str()))){return denied(403);}}
    if let Some(query)=req.uri().query() {
        let Some(expected)=query.strip_prefix("v=").filter(|s|s.len()==64&&s.bytes().all(|b|b.is_ascii_hexdigit())) else {return denied(400);};
        let id=catalog::identity(&file).ok();if thumbnails::version(root,&path,&meta,id.as_deref())!=expected{return denied(409);}
    }
    let Ok((start,end,partial)) = range(req.headers().get("Range").and_then(|v| v.to_str().ok()), size, kind == "image") else { return Response::builder().status(416).header("Content-Range", format!("bytes */{size}")).body(Vec::new()).unwrap(); };
    let mut bytes = Vec::new();
    if req.method() == "GET" && (file.seek(SeekFrom::Start(start)).is_err() || file.take(end-start+1).read_to_end(&mut bytes).is_err() || bytes.len() as u64 != end-start+1) { return denied(500); }
    if crate::privacy::check(&path).is_err(){return denied(403);}
    let mut response = Response::builder().status(if partial {206} else {200}).header("Content-Type",mime).header("Content-Length",(end-start+1).to_string()).header("Accept-Ranges","bytes").header("Cache-Control","no-store").header("X-Content-Type-Options","nosniff").header("Content-Security-Policy","default-src 'none'; sandbox").header("Access-Control-Allow-Origin","http://tauri.localhost");
    if partial { response = response.header("Content-Range", format!("bytes {start}-{end}/{size}")); }
    response.body(bytes).unwrap()
}
pub fn protocol(ctx: tauri::UriSchemeContext<'_, tauri::Wry>, req: tauri::http::Request<Vec<u8>>, responder: tauri::UriSchemeResponder) {
    let catalog=ctx.app_handle().state::<Arc<GalleryCatalog>>().inner().clone();let label = ctx.webview_label().to_owned(); let core = ctx.app_handle().state::<Arc<Core>>().inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let result = match root(&core) { Ok(root) => response_catalog(&root,&label,req,Some(&catalog)), Err(_) => tauri::http::Response::builder().status(404).body(Vec::new()).unwrap() }; responder.respond(result);
    });
}

#[cfg(test)]
#[path = "gallery_tests.rs"]
mod tests;

pub(crate) fn assistant_search(core:&Core,catalog:&GalleryCatalog,images:&ImageEngine,search:String)->Result<serde_json::Value>{
 let root=root(core)?;let origins=catalog.sync_origins(&root,images.list()?)?;let query=Query{sort:Sort::ModifiedDesc,folder:String::new(),search,kind:"all".into(),recursive:true,offset:0,favorites_only:false,tag:String::new()};let result=list_full(&root,query,&catalog.snapshot(&root)?,&origins)?;
 Ok(serde_json::json!({"total":result.total,"limited":result.limited,"entries":result.entries.into_iter().filter(|e|!e.locked).take(20).map(|e|serde_json::json!({"name":e.name,"kind":e.kind,"bytes":e.bytes,"tags":e.annotation.tags,"path":e.path})).collect::<Vec<_>>()}))
}
