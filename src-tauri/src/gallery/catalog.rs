pub(super) mod editor;
pub use editor::{editor_preview,editor_export};
pub(super) mod lineage;
pub use lineage::{gallery_lineage,gallery_create_variant,gallery_set_primary};
pub(super) mod origins;
pub use origins::Origin;
pub(super) mod files;
pub use files::{gallery_file_action,gallery_trash_list,gallery_trash_action,gallery_trash_detail};
mod batch;
pub use batch::gallery_annotate_batch;
use super::*;
use rusqlite::{params, Connection, OptionalExtension};
use std::{collections::{HashMap, HashSet}, sync::Mutex};

#[derive(Clone, Default, Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Annotation { pub favorite: bool, pub tags: Vec<String>, pub revision: i64 }
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Edit { pub root_id: String, pub path: String, pub file_id: String, pub revision: i64, pub favorite: bool, pub tags: Vec<String> }
pub struct GalleryCatalog { db: Mutex<Connection>, files_gate: Mutex<()> }

// FileIdInfo uses the full 128-bit identifier, including on ReFS. The creation
// time additionally separates identities if a filesystem later reuses an ID.
pub(super) fn identity(file: &File) -> Result<String> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::Storage::FileSystem::{GetFileInformationByHandleEx, FILE_ID_INFO, FileIdInfo};
    let mut info: FILE_ID_INFO = unsafe { std::mem::zeroed() };
    let ok = unsafe { GetFileInformationByHandleEx(file.as_raw_handle() as _, FileIdInfo, &mut info as *mut _ as _, std::mem::size_of_val(&info) as u32) };
    if ok == 0 || info.FileId.Identifier.iter().all(|b| *b == 0) { return Err("gallery_identity".into()); }
    let meta = file.metadata().map_err(|_| "gallery_missing")?;
    if !meta.is_file() || meta.file_attributes() & 0x400 != 0 { return Err("gallery_path".into()); }
    Ok(format!("{:016x}-{}-{:016x}", info.VolumeSerialNumber, info.FileId.Identifier.iter().map(|b| format!("{b:02x}")).collect::<String>(), meta.creation_time()))
}
pub(super) fn path_identity(path: &Path) -> Result<String> {
    let file = OpenOptions::new().access_mode(0x80).share_mode(7).custom_flags(0x00200000).open(path).map_err(|_| "gallery_missing")?;
    identity(&file)
}
fn normalize(tags: Vec<String>) -> Result<Vec<String>> {
    if tags.len() > 64 { return Err("gallery_tags".into()); }
    let mut unique = HashSet::new(); let mut result = Vec::new();
    for tag in tags {
        if tag.chars().any(char::is_control) || tag.contains(',') || tag.len() > 256 { return Err("gallery_tags".into()); }
        let tag = tag.split_whitespace().collect::<Vec<_>>().join(" ");
        if tag.chars().count() > 64 { return Err("gallery_tags".into()); }
        if !tag.is_empty() && unique.insert(tag.to_lowercase()) { result.push(tag); }
    }
    if result.len() > 32 { return Err("gallery_tags".into()); } Ok(result)
}
impl GalleryCatalog {
    pub fn new(config: &Path) -> Result<Arc<Self>> {
        let db = Connection::open(config.join("gallery.sqlite3")).map_err(|_| "gallery_storage")?;
        db.busy_timeout(std::time::Duration::from_secs(5)).map_err(|_| "gallery_storage")?;
        db.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE IF NOT EXISTS annotations(root TEXT NOT NULL, file_id TEXT NOT NULL, favorite INTEGER NOT NULL, tags TEXT NOT NULL, revision INTEGER NOT NULL, PRIMARY KEY(root,file_id));").map_err(|_| "gallery_storage")?;
        db.execute_batch("CREATE TABLE IF NOT EXISTS origins(root TEXT NOT NULL,job TEXT NOT NULL,file_id TEXT NOT NULL,stamp TEXT NOT NULL,json TEXT NOT NULL,PRIMARY KEY(root,job)); CREATE TABLE IF NOT EXISTS file_journal(root TEXT NOT NULL,id TEXT NOT NULL,json TEXT NOT NULL,PRIMARY KEY(root,id)); CREATE TABLE IF NOT EXISTS trash(root TEXT NOT NULL,id TEXT NOT NULL,json TEXT NOT NULL,PRIMARY KEY(root,id)); CREATE INDEX IF NOT EXISTS trash_path ON trash(root,json_extract(json,'$.storedPath')); CREATE INDEX IF NOT EXISTS trash_date ON trash(root,json_extract(json,'$.deletedAt') DESC,id);").map_err(|_| "gallery_storage")?;
        lineage::initialize(&db)?;
        Ok(Arc::new(Self { db: Mutex::new(db),files_gate:Mutex::new(()) }))
    }
    pub(super) fn snapshot(&self, root: &Path) -> Result<HashMap<String,Annotation>> {
        let db = self.db.lock().map_err(|_| "gallery_storage")?;
        let mut stmt = db.prepare("SELECT file_id,favorite,tags,revision FROM annotations WHERE root=?1").map_err(|_| "gallery_storage")?;
        let rows = stmt.query_map([root_id(root)], |row| Ok((row.get::<_,String>(0)?,row.get::<_,bool>(1)?,row.get::<_,String>(2)?,row.get::<_,i64>(3)?))).map_err(|_| "gallery_storage")?;
        let mut result = HashMap::new();
        for row in rows {
            let (id,favorite,tags,revision) = row.map_err(|_| "gallery_storage")?;
            let tags: Vec<String> = serde_json::from_str(&tags).map_err(|_| "gallery_storage")?;
            if revision < 1 || normalize(tags.clone())? != tags { return Err("gallery_storage".into()); }
            result.insert(id, Annotation { favorite,tags,revision });
        } Ok(result)
    }
    pub(super) fn edit(&self, root: &Path, edit: Edit) -> Result<Annotation> {
        if edit.root_id != root_id(root) { return Err("gallery_changed".into()); }
        let tags = normalize(edit.tags)?;
        let path = resolve(root,&edit.path)?; if media(&path).is_none() { return Err("gallery_format".into()); }
        let _guards = directory_guards(path.parent().ok_or("gallery_path")?)?;
        let file = lock_file(&path)?;
        if identity(&file)? != edit.file_id { return Err("gallery_changed".into()); }
        let mut db = self.db.lock().map_err(|_| "gallery_storage")?;
        let tx = db.transaction().map_err(|_| "gallery_storage")?;
        let previous: Option<i64> = tx.query_row("SELECT revision FROM annotations WHERE root=?1 AND file_id=?2",params![edit.root_id,edit.file_id],|r|r.get(0)).optional().map_err(|_| "gallery_storage")?;
        if edit.revision != previous.unwrap_or(0) { return Err("gallery_conflict".into()); }
        let next = Annotation { favorite:edit.favorite, tags, revision:edit.revision.checked_add(1).ok_or("gallery_storage")? };
        tx.execute("INSERT INTO annotations VALUES(?1,?2,?3,?4,?5) ON CONFLICT(root,file_id) DO UPDATE SET favorite=excluded.favorite,tags=excluded.tags,revision=excluded.revision",params![edit.root_id,edit.file_id,next.favorite,serde_json::to_string(&next.tags).map_err(|_| "gallery_storage")?,next.revision]).map_err(|_| "gallery_storage")?;
        tx.commit().map_err(|_| "gallery_storage")?; Ok(next)
    }
}
#[tauri::command]
pub async fn gallery_annotate(edit: Edit, core: State<'_,Arc<Core>>, catalog: State<'_,Arc<GalleryCatalog>>) -> Result<Annotation> {
    let root = root(&core)?; let catalog = catalog.inner().clone();
    tauri::async_runtime::spawn_blocking(move || catalog.edit(&root,edit)).await.map_err(|_| "gallery_storage")?
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> (tempfile::TempDir,PathBuf,Arc<GalleryCatalog>,String) {
        let temp = tempfile::tempdir().unwrap(); let root = fs::canonicalize(temp.path()).unwrap(); let catalog = GalleryCatalog::new(&root).unwrap();
        fs::write(root.join("a.png"),b"media fixture").unwrap(); let id = path_identity(&root.join("a.png")).unwrap(); (temp,root,catalog,id)
    }
    fn edit(root: &Path,id: &str,revision: i64) -> Edit { Edit { root_id:root_id(root),path:"a.png".into(),file_id:id.into(),revision,favorite:true,tags:vec![" Urlaub  2026 ".into(),"urlaub 2026".into(),"Grün".into()] } }
    #[test] fn tags_normalize_without_losing_unicode_and_reject_unbounded_input() {
        assert_eq!(normalize(vec![" Natur ".into(),"natur".into(),"Grün".into()]).unwrap(),vec!["Natur","Grün"]);
        for tags in [vec!["x".repeat(65)],vec!["a\nb".into()],vec!["a,b".into()],(0..33).map(|i|i.to_string()).collect()] { assert_eq!(normalize(tags).unwrap_err(),"gallery_tags"); }
    }
    #[test] fn metadata_persists_without_media_writes_and_survives_rename() {
        let (_temp,root,catalog,id) = setup(); let before=fs::read(root.join("a.png")).unwrap();
        let saved=catalog.edit(&root,edit(&root,&id,0)).unwrap(); assert_eq!(saved.tags,vec!["Urlaub 2026","Grün"]);
        fs::rename(root.join("a.png"),root.join("renamed.png")).unwrap(); assert_eq!(path_identity(&root.join("renamed.png")).unwrap(),id);
        drop(catalog); let catalog=GalleryCatalog::new(&root).unwrap(); assert_eq!(catalog.snapshot(&root).unwrap()[&id],saved); assert_eq!(fs::read(root.join("renamed.png")).unwrap(),before);
    }
    #[test] fn stale_edits_replacement_and_root_change_cannot_modify_wrong_annotations() {
        let (_temp,root,catalog,id) = setup(); let saved=catalog.edit(&root,edit(&root,&id,0)).unwrap();
        assert_eq!(catalog.edit(&root,edit(&root,&id,0)).unwrap_err(),"gallery_conflict");
        fs::rename(root.join("a.png"),root.join("old.png")).unwrap(); fs::write(root.join("a.png"),b"new file").unwrap();
        assert_ne!(path_identity(&root.join("a.png")).unwrap(),id); assert_eq!(catalog.edit(&root,edit(&root,&id,1)).unwrap_err(),"gallery_changed");
        let mut request=edit(&root,&id,1);request.root_id="different".into(); assert_eq!(catalog.edit(&root,request).unwrap_err(),"gallery_changed");
        assert_eq!(catalog.snapshot(&root).unwrap()[&id],saved);
    }
    #[test] fn failed_commit_keeps_previous_favorite_and_tags() {
        let (_temp,root,catalog,id)=setup(); let saved=catalog.edit(&root,edit(&root,&id,0)).unwrap();
        catalog.db.lock().unwrap().execute_batch("CREATE TRIGGER fail_write BEFORE INSERT ON annotations BEGIN SELECT RAISE(ABORT,'test storage failure'); END;").unwrap();
        let mut request=edit(&root,&id,1);request.favorite=false;request.tags.clear();assert_eq!(catalog.edit(&root,request).unwrap_err(),"gallery_storage");assert_eq!(catalog.snapshot(&root).unwrap()[&id],saved);
    }
}
