pub mod creative;
pub use creative::{video_frame,media_prepare,media_status,media_cancel,media_info,caption_read,caption_write,project_creative_save,canvas_preview,canvas_export,canvas_export_mask,video_probe,video_start,video_jobs,video_cancel,VideoEngine};
mod editor;
mod image_reference;
pub use editor::{project_editor_preview,project_editor_save,project_editor_export,project_add_edit};
// Local, passive project containers. Archive names never become filesystem paths.
mod recent;
pub use recent::{project_recent,project_forget_recent};
use crate::{core::Core, gallery, image_engine::ImageRequest};
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use zip::{write::SimpleFileOptions, CompressionMethod, ZipArchive, ZipWriter};
type Result<T> = std::result::Result<T, String>;
const MAX_MEDIA: usize = 100;
const MAX_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_MANIFEST: u64 = 1024 * 1024;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelReference {
    pub name: String,
    pub sha256: String,
    pub source: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Asset {
    #[serde(default,skip_serializing_if="std::ops::Not::not")] pub restricted:bool,
    pub id: String,
    pub name: String,
    pub kind: String,
    pub bytes: u64,
    pub sha256: String,
    pub archive_name: String,
    #[serde(default,skip_serializing_if="Vec::is_empty")]
    pub edit: Vec<gallery::EditOperation>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Manifest {
    #[serde(default,skip_serializing_if="std::ops::Not::not")] restricted:bool,
    #[serde(default,skip_serializing_if="Option::is_none")]
    creative: Option<creative::Creative>,
    format: String,
    version: u32,
    name: String,
    request: Option<ImageRequest>,
    model: Option<ModelReference>,
    assets: Vec<Asset>,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    #[serde(default)] pub restricted:bool,
    #[serde(default,skip_deserializing)] pub locked:bool,
    #[serde(default,skip_serializing_if="Option::is_none")]
    pub creative: Option<creative::Creative>,
    pub id: String,
    pub name: String,
    pub path: Option<String>,
    pub version: Option<String>,
    pub request: Option<ImageRequest>,
    pub model: Option<ModelReference>,
    pub assets: Vec<Asset>,
    #[serde(default)]
    pub removed: Vec<Asset>,
    pub directory: String,
    pub dirty: bool,
    pub recovery: bool,
}
struct State {
    db: Connection,
    project: Option<Project>,
}
pub(crate) fn project_restricted(p:&Project)->bool{p.restricted||p.assets.iter().chain(&p.removed).any(|a|a.restricted)||p.request.as_ref().is_some_and(crate::privacy::request)}
pub struct Projects {
    state: Mutex<State>,
}
fn err(error: impl std::fmt::Display) -> String {
    #[cfg(test)]
    eprintln!("project storage: {error}");
    #[cfg(not(test))]
    let _ = error;
    "project_storage".into()
}
fn uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}
fn valid_id(id: &str) -> bool {
    uuid::Uuid::parse_str(id).is_ok_and(|v| v.to_string() == id)
}
fn digest(input: &mut impl Read) -> Result<String> {
    let mut h = Sha256::new();
    let mut b = vec![0; 1024 * 1024];
    loop {
        let n = input.read(&mut b).map_err(err)?;
        if n == 0 {
            break;
        }
        h.update(&b[..n]);
    }
    Ok(format!("{:x}", h.finalize()))
}
fn valid_request(request: &Option<ImageRequest>) -> Result<()> {
    if let Some(r) = request {
        let mut r = r.clone();
        if r.prompt.trim().is_empty() {
            r.prompt = "draft".into();
        }
        crate::image_engine::validate(&r).map_err(|_| "project_manifest")?;
        if r.model_path.len() > 32768 {
            return Err("project_manifest".into());
        }
    }
    Ok(())
}
fn validate(m: &Manifest) -> Result<()> {
    if m.format != "local-studio" || !matches!(m.version,1|2|3|4|5) {
        return Err("project_version".into());
    }
    if m.name.trim().is_empty()
        || m.name.len() > 180
        || m.name.chars().any(char::is_control)
        || m.assets.len() > MAX_MEDIA
    {
        return Err("project_manifest".into());
    }
    if serde_json::to_vec(m).map_err(err)?.len() as u64 > MAX_MANIFEST { return Err("project_limit".into()); }
    if let Some(c)=&m.creative {if m.version<3{return Err("project_version".into());}creative::validate(c,&m.assets)?;}
    if m.version<5&&(m.restricted||m.assets.iter().any(|a|a.restricted)){return Err("project_version".into());}
    valid_request(&m.request)?;
    if m.version<4&&m.request.as_ref().is_some_and(|r|r.vae_on_cpu){return Err("project_version".into());}
    if let Some(reference)=m.request.as_ref().and_then(|r|r.reference.as_ref()) {
        if m.version<4||reference.inputs().iter().any(|(_,r)|!m.assets.iter().any(|a|a.archive_name==r.path&&a.sha256==r.sha256&&a.kind=="image")) {return Err("project_manifest".into());}
    }
    if m.request.as_ref().is_some_and(|r| !r.model_path.is_empty()) {
        return Err("project_manifest".into());
    }
    let hash = |s: &str| s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit());
    if m.model
        .as_ref()
        .is_some_and(|r| !hash(&r.sha256) || r.name.len() > 180 || r.source != "local")
    {
        return Err("project_manifest".into());
    }
    let mut ids = HashSet::new();
    let mut total = 0u64;
    for a in &m.assets {
        gallery::validate_operations(&a.edit)?;
        if !a.edit.is_empty() && (m.version < 2 || a.kind != "image") { return Err("project_manifest".into()); }
        let p = Path::new(&a.name);
        let ext = p
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if !valid_id(&a.id)
            || !ids.insert(&a.id)
            || !gallery::valid_name(&a.name)
            || !hash(&a.sha256)
            || gallery::media(p).map(|(kind, _)| kind) != Some(a.kind.as_str())
            || a.archive_name != format!("media/{}.{}", a.id, ext)
        {
            return Err("project_manifest".into());
        }
        total = total.checked_add(a.bytes).ok_or("project_limit")?;
        if total > MAX_BYTES {
            return Err("project_limit".into());
        }
    }
    Ok(())
}
fn owned(p: &Project, a: &Asset) -> Result<PathBuf> {
    let directory = Path::new(&p.directory);
    if !valid_id(&p.id)
        || directory.file_name().and_then(|n| n.to_str()) != Some(&p.id)
        || directory
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            != Some("project-sessions")
    {
        return Err("project_path".into());
    }
    let name = a
        .archive_name
        .strip_prefix("media/")
        .filter(|n| !n.contains(['/', '\\', ':']))
        .ok_or("project_path")?;
    Ok(directory.join(name))
}
fn persist(state: &mut State, mut project: Option<Project>) -> Result<()> {
    if let Some(p)=project.as_mut(){if project_restricted(p){p.restricted=true;crate::privacy::protect_path(Path::new(&p.directory))?;}}
    history::persist(state, project)
}
fn create(root: &Path, name: String, request: Option<ImageRequest>) -> Result<Project> {
    if name.trim().is_empty() || name.len() > 180 || name.chars().any(char::is_control) {
        return Err("project_name".into());
    }
    valid_request(&request)?;
    let _guards = gallery::directory_guards(root)?;
    let sessions = root.join("project-sessions");
    fs::create_dir_all(&sessions).map_err(err)?;
    let _guards = gallery::directory_guards(&sessions)?;
    let id = uuid();
    let directory = sessions.join(&id);
    fs::create_dir(&directory).map_err(err)?;
    Ok(Project { restricted:request.as_ref().is_some_and(crate::privacy::request),locked:false,
        creative: None,
        id,
        name,
        path: None,
        version: None,
        request,
        model: None,
        assets: vec![],
        removed: vec![],
        directory: directory.to_string_lossy().into(),
        dirty: true,
        recovery: false,
    })
}
// Read only the bounded, conventional ZIP32 directory before the library allocates metadata.
fn archive(input: &mut File) -> Result<ZipArchive<&mut File>> {
    let length = input.metadata().map_err(err)?.len();
    if length > MAX_BYTES + 16 * 1024 * 1024 || length < 22 {
        return Err("project_limit".into());
    }
    let tail = length.min(65557);
    input.seek(SeekFrom::End(-(tail as i64))).map_err(err)?;
    let mut b = vec![0; tail as usize];
    input.read_exact(&mut b).map_err(err)?;
    let i = b
        .windows(4)
        .rposition(|v| v == b"PK\x05\x06")
        .ok_or("project_archive")?;
    if i + 22 > b.len() {
        return Err("project_archive".into());
    }
    let u16at = |at| u16::from_le_bytes(b[at..at + 2].try_into().unwrap());
    let u32at = |at| u32::from_le_bytes(b[at..at + 4].try_into().unwrap());
    let count = u16at(i + 10) as usize;
    let size = u32at(i + 12) as u64;
    let offset = u32at(i + 16) as u64;
    if u16at(i + 4) != 0
        || u16at(i + 6) != 0
        || u16at(i + 8) as usize != count
        || count == 0
        || count > MAX_MEDIA + 1
        || size > 256 * 1024
        || offset + size != length - tail + i as u64
        || i + 22 + u16at(i + 20) as usize != b.len()
    {
        return Err("project_archive".into());
    }
    input.rewind().map_err(err)?;
    let z = ZipArchive::new(input).map_err(|_| "project_archive")?;
    if z.len() != count || z.offset() != 0 {
        return Err("project_archive".into());
    }
    Ok(z)
}
fn read_manifest(z: &mut ZipArchive<&mut File>) -> Result<Manifest> {
    let mut file = z.by_name("project.json").map_err(|_| "project_manifest")?;
    if file.size() > MAX_MANIFEST {
        return Err("project_limit".into());
    }
    let mut b = Vec::new();
    (&mut file)
        .take(MAX_MANIFEST + 1)
        .read_to_end(&mut b)
        .map_err(|_| "project_archive")?;
    if b.len() as u64 > MAX_MANIFEST {
        return Err("project_limit".into());
    }
    let manifest: Manifest = serde_json::from_slice(&b).map_err(|_| "project_manifest")?;
    validate(&manifest)?;
    drop(file);
    if z.len() != manifest.assets.len() + 1 {
        return Err("project_archive".into());
    }
    let names: HashSet<_> = manifest
        .assets
        .iter()
        .map(|a| a.archive_name.as_str())
        .chain(std::iter::once("project.json"))
        .collect();
    for i in 0..z.len() {
        let f = z.by_index(i).map_err(|_| "project_archive")?;
        if !names.contains(f.name())
            || f.is_dir()
            || f.unix_mode()
                .is_some_and(|m| m & 0o170000 != 0 && m & 0o170000 != 0o100000)
            || !matches!(
                f.compression(),
                CompressionMethod::Stored | CompressionMethod::Deflated
            )
        {
            return Err("project_archive".into());
        }
    }
    Ok(manifest)
}
fn transfer(input: &mut impl Read, output: &mut impl Write, limit: u64) -> Result<(u64, String)> {
    let mut h = Sha256::new();
    let mut bytes = 0u64;
    let mut buffer = vec![0; 1024 * 1024];
    loop {
        let n = input.read(&mut buffer).map_err(|_| "project_archive")?;
        if n == 0 {
            break;
        }
        bytes = bytes.checked_add(n as u64).ok_or("project_limit")?;
        if bytes > limit {
            return Err("project_limit".into());
        }
        output.write_all(&buffer[..n]).map_err(err)?;
        h.update(&buffer[..n]);
    }
    Ok((bytes, format!("{:x}", h.finalize())))
}
impl Projects {
    pub fn new(config: &Path) -> Result<Arc<Self>> {
        let db = Connection::open(config.join("projects.sqlite3")).map_err(err)?;
        history::initialize(&db)?;
        recent::initialize(&db)?;
        db.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE IF NOT EXISTS project_session(id INTEGER PRIMARY KEY,json TEXT NOT NULL); CREATE TABLE IF NOT EXISTS project_save_intent(id INTEGER PRIMARY KEY,json TEXT NOT NULL);").map_err(err)?;
        let json: Option<String> = db
            .query_row("SELECT json FROM project_session WHERE id=1", [], |r| {
                r.get(0)
            })
            .optional()
            .map_err(err)?;
        let mut project: Option<Project> = json
            .map(|s| serde_json::from_str(&s).map_err(err))
            .transpose()?
            .flatten();
        let mut state = State {
            db,
            project: project.take(),
        };
        let _ = container::recover_save(&mut state);
        if let Some(p) = &mut state.project {
            p.recovery = true;
        }
        Ok(Arc::new(Self {
            state: Mutex::new(state),
        }))
    }
    pub(crate) fn protect_current(&self,id:&str)->Result<()> {let mut s=self.state.lock().map_err(err)?;let mut p=Self::require(&s)?;if p.id!=id{return Err("project_changed".into());}p.restricted=true;p.dirty=true;persist(&mut s,Some(p))}
    pub fn privacy_restricted(&self)->Result<bool>{Ok(self.state.lock().map_err(err)?.project.as_ref().is_some_and(project_restricted))}
    pub fn snapshot(&self) -> Result<Option<Project>> {
        Ok(self.state.lock().map_err(err)?.project.clone())
    }
    fn require(state: &State) -> Result<Project> {
        container::no_intent(state)?;
        state
            .project
            .clone()
            .filter(|p| !p.recovery)
            .ok_or("project_inactive".into())
    }
    fn new_project(
        &self,
        root: &Path,
        name: String,
        request: Option<ImageRequest>,
        confirmed: bool,
    ) -> Result<Project> {
        let mut s = self.state.lock().map_err(err)?;
        container::no_intent(&s)?;
        if s.project.as_ref().is_some_and(|p| p.dirty) && !confirmed {
            return Err("project_unsaved".into());
        }
        let p = create(root, name, request)?;
        persist(&mut s, Some(p.clone()))?;
        Ok(p)
    }
    fn add(&self, sources: Vec<String>) -> Result<Project> {
        self.add_for(None, sources, None)
    }
    fn add_to(&self, id: &str, sources: Vec<String>) -> Result<Project> {
        self.add_for(Some(id), sources, None)
    }
    fn add_for(&self, id: Option<&str>, sources: Vec<String>, edit: Option<Vec<gallery::EditOperation>>) -> Result<Project> {
        let mut s = self.state.lock().map_err(err)?;
        let mut p = Self::require(&s)?;
        if id.is_some_and(|id| id != p.id) {
            return Err("project_changed".into());
        }
        if sources.is_empty() || p.assets.len() + sources.len() > MAX_MEDIA {
            return Err("project_limit".into());
        }
        let _guards = gallery::directory_guards(Path::new(&p.directory))?;
        let mut total: u64 = p.assets.iter().map(|a| a.bytes).sum();
        let mut created = Vec::new();
        let result = (|| {
            for source in sources {
                let source = Path::new(&source);
                let (kind, _) = gallery::media(source).ok_or("project_media")?;
                let name = source
                    .file_name()
                    .and_then(|s| s.to_str())
                    .filter(|n| gallery::valid_name(n))
                    .ok_or("project_media")?;
                let _pins = gallery::directory_guards(source.parent().ok_or("project_path")?)?;
                let mut input = gallery::lock_file(source)?;
                let bytes = input.metadata().map_err(err)?.len();
                total = total.checked_add(bytes).ok_or("project_limit")?;
                if total > MAX_BYTES {
                    return Err("project_limit".into());
                }
                let id = uuid();
                let ext = source.extension().unwrap().to_string_lossy().to_lowercase();
                let mut a = Asset { restricted:crate::privacy::media(source),
                    id: id.clone(),
                    name: name.into(),
                    kind: kind.into(),
                    bytes,
                    sha256: String::new(),
                    edit: edit.clone().unwrap_or_default(),
                    archive_name: format!("media/{id}.{ext}"),
                };
                let destination = owned(&p, &a)?;
                let mut output = OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .open(&destination)
                    .map_err(err)?;
                created.push(destination);
                let (actual, hash) = transfer(&mut input, &mut output, bytes)?;
                output.sync_all().map_err(err)?;
                if actual != bytes {
                    return Err("project_changed".into());
                }
                a.sha256 = hash;
                drop(output);if a.restricted{crate::privacy::mark(&owned(&p,&a)?)?;p.restricted=true;}
                p.assets.push(a);
            }
            p.dirty = true;
            if serde_json::to_vec(&p).map_err(err)?.len() as u64>MAX_MANIFEST{return Err("project_limit".into());}
            persist(&mut s, Some(p.clone()))?;
            Ok(p.clone())
        })();
        if result.is_err() {
            for path in created {
                let _ = gallery::delete_owned(&path);
            }
        }
        result
    }
    fn open(&self, path: &Path, recovery: &Path, confirmed: bool) -> Result<Project> {
        let mut s = self.state.lock().map_err(err)?;
        container::no_intent(&s)?;
        if s.project.as_ref().is_some_and(|p| p.dirty) && !confirmed {
            return Err("project_unsaved".into());
        }
        if path
            .extension()
            .and_then(|e| e.to_str())
            .is_none_or(|e| !e.eq_ignore_ascii_case("localstudio"))
        {
            return Err("project_path".into());
        }
        let _pins = gallery::directory_guards(path.parent().ok_or("project_path")?)?;
        let mut input = gallery::lock_file(path)?;
        let version = digest(&mut input)?;
        let mut z = archive(&mut input)?;
        let m = read_manifest(&mut z)?;
        if crate::privacy::locked()&&(m.restricted||m.assets.iter().any(|a|a.restricted)){return Err("privacy_locked".into());}
        let mut p = create(recovery, m.name, m.request)?;
        p.restricted=m.restricted;p.creative=m.creative;
        p.model = m.model;
        p.path = Some(path.to_string_lossy().into());
        p.version = Some(version);
        let _guards = gallery::directory_guards(Path::new(&p.directory))?;
        let mut created = Vec::new();
        let result = (|| {
            for a in m.assets {
                let mut entry = z.by_name(&a.archive_name).map_err(|_| "project_archive")?;
                if entry.size() != a.bytes {
                    return Err("project_archive".into());
                }
                let target = owned(&p, &a)?;
                let mut output = OpenOptions::new()
                    .create_new(true)
                    .write(true)
                    .open(&target)
                    .map_err(err)?;
                created.push(target);
                let (bytes, hash) = transfer(&mut entry, &mut output, a.bytes)?;
                output.sync_all().map_err(err)?;
                if bytes != a.bytes || hash != a.sha256 {
                    return Err("project_hash".into());
                }
                p.assets.push(a);
            }
            for a in &p.assets{if a.restricted{crate::privacy::mark(&owned(&p,a)?)?;}}
            image_reference::restore_reference(&mut p)?;
            p.dirty = false;
            persist(&mut s, Some(p.clone()))?;
            Ok(p.clone())
        })();
        if result.is_err() {
            drop(_guards);
            for file in created {
                let _ = gallery::delete_owned(&file);
            }
            let _ = fs::remove_dir(&p.directory);
        }
        result
    }
    fn update(&self, id: &str, request: Option<ImageRequest>) -> Result<Project> {
        valid_request(&request)?;
        let mut s = self.state.lock().map_err(err)?;
        let mut p = Self::require(&s)?;
        if p.id != id {
            return Err("project_changed".into());
        }
        if serde_json::to_value(&p.request).map_err(err)?
            != serde_json::to_value(&request).map_err(err)?
        {
            p.request = request;
            p.dirty = true;
            if serde_json::to_vec(&p).map_err(err)?.len() as u64>MAX_MANIFEST{return Err("project_limit".into());}
            persist(&mut s, Some(p.clone()))?;
        }
        Ok(p)
    }
    #[cfg(test)]
    fn remove(&self, id: &str) -> Result<Project> {
        self.remove_limited(id, 100)
    }
    fn remove_limited(&self, id: &str, limit: usize) -> Result<Project> {
        let mut s = self.state.lock().map_err(err)?;
        let mut p = Self::require(&s)?;
        if !p.assets.iter().any(|a| a.id == id) {
            return Err("project_media".into());
        }
        if p.creative.as_ref().is_some_and(|c|creative::referenced(c,id)){return Err("creative_in_use".into());}
        p.removed
            .push(p.assets.iter().find(|a| a.id == id).unwrap().clone());
        while p.removed.len() > limit.clamp(1, 1000) {
            p.removed.remove(0);
        }
        p.assets.retain(|a| a.id != id);
        p.dirty = true;
        persist(&mut s, Some(p.clone()))?;
        Ok(p)
    }
    fn close(&self, confirmed: bool) -> Result<()> {
        let mut s = self.state.lock().map_err(err)?;
        container::no_intent(&s)?;
        if s.project.as_ref().is_some_and(|p| p.dirty) && !confirmed {
            return Err("project_unsaved".into());
        }
        persist(&mut s, None)
    }
    fn recover(&self) -> Result<Project> {
        let mut s = self.state.lock().map_err(err)?;
        container::recover_save(&mut s)?;
        let mut p = s.project.clone().ok_or("project_inactive")?;
        for a in &p.assets {
            let mut f = gallery::lock_file(&owned(&p, a)?)?;
            if digest(&mut f)? != a.sha256 {
                return Err("project_hash".into());
            }
        }
        p.recovery = false;
        persist(&mut s, Some(p.clone()))?;
        Ok(p)
    }
    fn relink(&self, path: &Path) -> Result<Project> {
        let mut s = self.state.lock().map_err(err)?;
        let mut p = Self::require(&s)?;
        let reference = p.model.as_ref().ok_or("project_model")?;
        let mut input = gallery::lock_file(path)?;
        if digest(&mut input)? != reference.sha256 {
            return Err("project_model_changed".into());
        }
        if let Some(r) = &mut p.request {
            r.model_path = path.to_string_lossy().into();
        }
        persist(&mut s, Some(p.clone()))?;
        Ok(p)
    }
}

mod cleanup;
mod container;
mod history;
mod transfer;
pub use history::{
    project_checkpoint, project_history, project_rename, project_restore_media,
    project_restore_point,
};
pub use transfer::{project_add_gallery, project_add_image};
pub mod preview;
#[tauri::command]
pub async fn project_snapshot(state: tauri::State<'_, Arc<Projects>>) -> Result<Option<Project>> {let privacy_epoch=crate::privacy::epoch();let privacy_result=(async {
    let p = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {let mut result=p.snapshot()?;if crate::privacy::locked(){if let Some(p)=result.as_mut(){if project_restricted(p){p.locked=true;p.restricted=true;p.name="18+".into();p.path=None;p.version=None;p.request=None;p.model=None;p.assets.clear();p.removed.clear();p.creative=None;p.directory.clear();}}}Ok(result)})
        .await
        .map_err(err)?
}).await;crate::privacy::finish(privacy_epoch,privacy_result)}
#[tauri::command]
pub async fn project_new(
    name: String,
    request: Option<ImageRequest>,
    confirmed: bool,
    state: tauri::State<'_, Arc<Projects>>,
    core: tauri::State<'_, Arc<Core>>,
) -> Result<Project> {let privacy_epoch=crate::privacy::epoch();let privacy_result=(async {
    let p = state.inner().clone();
    let root = core.storage_paths()?.recovery;
    tauri::async_runtime::spawn_blocking(move || {
        p.new_project(Path::new(&root), name, request, confirmed)
    })
    .await
    .map_err(err)?
}).await;crate::privacy::finish(privacy_epoch,privacy_result)}
#[tauri::command]
pub async fn project_open(
    path: String,
    confirmed: bool,
    state: tauri::State<'_, Arc<Projects>>,
    core: tauri::State<'_, Arc<Core>>,
) -> Result<Project> {let privacy_epoch=crate::privacy::epoch();let privacy_result=(async {
    let p = state.inner().clone();
    let root = core.storage_paths()?.recovery;
    tauri::async_runtime::spawn_blocking(move || {
        p.open(Path::new(&path), Path::new(&root), confirmed)
    })
    .await
    .map_err(err)?
}).await;crate::privacy::finish(privacy_epoch,privacy_result)}
#[tauri::command]
pub async fn project_save(path: String, state: tauri::State<'_, Arc<Projects>>) -> Result<Project> {let privacy_epoch=crate::privacy::epoch();let privacy_result=(async {
    let p = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || p.save(Path::new(&path)))
        .await
        .map_err(err)?
}).await;crate::privacy::finish(privacy_epoch,privacy_result)}
#[tauri::command]
pub async fn project_update(
    id: String,
    request: Option<ImageRequest>,
    state: tauri::State<'_, Arc<Projects>>,
) -> Result<Project> {let privacy_epoch=crate::privacy::epoch();let privacy_result=(async {
    let p = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || p.update(&id, request))
        .await
        .map_err(err)?
}).await;crate::privacy::finish(privacy_epoch,privacy_result)}
#[tauri::command]
pub async fn project_add(
    sources: Vec<String>,
    state: tauri::State<'_, Arc<Projects>>,
) -> Result<Project> {let privacy_epoch=crate::privacy::epoch();let privacy_result=(async {
    let p = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || p.add(sources))
        .await
        .map_err(err)?
}).await;crate::privacy::finish(privacy_epoch,privacy_result)}
#[tauri::command]
pub async fn project_remove(
    id: String,
    state: tauri::State<'_, Arc<Projects>>,
    core: tauri::State<'_, Arc<Core>>,
) -> Result<Project> {let privacy_epoch=crate::privacy::epoch();let privacy_result=(async {
    let p = state.inner().clone();
    let limit = core.current_settings()?.max_undo as usize;
    tauri::async_runtime::spawn_blocking(move || p.remove_limited(&id, limit))
        .await
        .map_err(err)?
}).await;crate::privacy::finish(privacy_epoch,privacy_result)}
#[tauri::command]
pub async fn project_close(confirmed: bool, state: tauri::State<'_, Arc<Projects>>) -> Result<()> {let privacy_epoch=crate::privacy::epoch();let privacy_result=(async {
    let p = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || p.close(confirmed))
        .await
        .map_err(err)?
}).await;crate::privacy::finish(privacy_epoch,privacy_result)}
#[tauri::command]
pub async fn project_recover(state: tauri::State<'_, Arc<Projects>>) -> Result<Project> {let privacy_epoch=crate::privacy::epoch();let privacy_result=(async {
    let p = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || p.recover())
        .await
        .map_err(err)?
}).await;crate::privacy::finish(privacy_epoch,privacy_result)}
#[tauri::command]
pub async fn project_relink(
    path: String,
    state: tauri::State<'_, Arc<Projects>>,
) -> Result<Project> {let privacy_epoch=crate::privacy::epoch();let privacy_result=(async {
    let p = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || p.relink(Path::new(&path)))
        .await
        .map_err(err)?
}).await;crate::privacy::finish(privacy_epoch,privacy_result)}
#[tauri::command]
pub async fn project_export_gallery(
    state: tauri::State<'_, Arc<Projects>>,
    core: tauri::State<'_, Arc<Core>>,
) -> Result<gallery::ImportResult> {let privacy_epoch=crate::privacy::epoch();let privacy_result=(async {
    let p = state.inner().clone();
    let root = PathBuf::from(core.storage_paths()?.gallery);
    tauri::async_runtime::spawn_blocking(move || {
        let s = p.state.lock().map_err(err)?;
        let project = Projects::require(&s)?;
        if project.assets.is_empty() {
            return Err("project_media".into());
        }
        let _guards = gallery::directory_guards(&root)?;
        let folder = root.join(format!("Project-{}", uuid()));
        fs::create_dir(&folder).map_err(err)?;
        let _pins = gallery::directory_guards(&folder)?;
        let mut report = gallery::ImportResult {
            imported: vec![],
            errors: vec![],
        };
        for a in &project.assets {
            let result = (|| {
                let source = owned(&project, a)?;
                let _pins = gallery::directory_guards(source.parent().ok_or("project_path")?)?;
                let mut file = gallery::lock_file(&source)?;
                if digest(&mut file)? != a.sha256 {
                    return Err("project_hash".into());
                }
                gallery::copy_one_named(&root, &folder, &source, &a.name)
            })();
            match result {
                Ok(path) => report.imported.push(path),
                Err(error) => report.errors.push(format!("{}: {}", a.name, error)),
            }
        }
        Ok(report)
    })
    .await
    .map_err(err)?
}).await;crate::privacy::finish(privacy_epoch,privacy_result)}
#[cfg(test)]
mod tests;
