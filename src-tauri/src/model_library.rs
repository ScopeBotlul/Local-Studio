use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{collections::{BTreeSet, HashSet}, fs::{self, File}, io::{Read}, path::{Component, Path, PathBuf, Prefix}, sync::{atomic::{AtomicBool, Ordering}, Arc, Mutex}, time::UNIX_EPOCH};

#[path = "model_discovery.rs"]
mod discovery;
use discovery::{classify_discovery, excluded_location, migrate_discovery};

type Result<T> = std::result::Result<T, String>;
const MAX_HEADER: u64 = 16 * 1024 * 1024;
#[derive(Clone, Copy, Default, Serialize, Deserialize, PartialEq, Debug)]
#[serde(rename_all = "lowercase")]
pub enum ScanMode { #[default] Folder, Quick, Full }
#[derive(Clone, Copy)]
struct ScanLimits { visited: u64, entries: usize, depth: usize, files: usize }
impl ScanMode {
    fn limits(self) -> ScanLimits {
        match self {
            Self::Folder | Self::Quick => ScanLimits { visited: 100_000, entries: 2000, depth: 32, files: 20_000 },
            Self::Full => ScanLimits { visited: u64::MAX, entries: 10_000, depth: 128, files: 20_000 },
        }
    }
}
fn quick_candidates(profile: &Path, drives: &[PathBuf], models: &Path, hf: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut roots = vec![models.to_path_buf(), profile.join(".cache/huggingface/hub"), profile.join(".lmstudio/models"), profile.join(".cache/lm-studio/models"), profile.join("Documents/Models"), profile.join("Documents/Modelle"), profile.join("Downloads/Models")];
    roots.extend(hf);
    for base in drives.iter().chain(std::iter::once(&profile.to_path_buf())) {
        for relative in ["Models", "Modelle", "AI/models", "LocalAI/models", "LocalAI/ComfyUI_windows_portable/ComfyUI/models", "ComfyUI/models", "ComfyUI_windows_portable/ComfyUI/models", "stable-diffusion-webui/models"] { roots.push(base.join(relative)); }
    }
    roots
}
fn unique_roots(candidates: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut roots = Vec::<PathBuf>::new();
    let key = |p: &Path| p.to_string_lossy().to_lowercase().replace('/', "\\").trim_start_matches(r"\\?\").trim_end_matches('\\').to_string();
    for path in candidates {
        if !local_path(&path) || !path.is_dir() { continue; }
        let value = key(&path);
        if roots.iter().any(|p| value == key(p) || value.starts_with(&(key(p) + "\\"))) { continue; }
        roots.retain(|p| !key(p).starts_with(&(value.clone() + "\\")));
        roots.push(path);
    }
    roots
}
fn quick_scan_roots(models: &Path) -> Result<Vec<PathBuf>> {
    let profile = std::env::var_os("USERPROFILE").map(PathBuf::from).ok_or("local_path")?;
    let mut hf = Vec::new();
    for name in ["HF_HUB_CACHE", "HUGGINGFACE_HUB_CACHE"] { if let Some(path) = std::env::var_os(name) { hf.push(path.into()); } }
    if let Some(path) = std::env::var_os("HF_HOME") { hf.push(PathBuf::from(path).join("hub")); }
    if let Some(path) = std::env::var_os("XDG_CACHE_HOME") { hf.push(PathBuf::from(path).join("huggingface/hub")); }
    let roots = unique_roots(quick_candidates(&profile, &full_scan_roots().unwrap_or_default(), models, hf));
    if roots.is_empty() { return Err("local_quick_empty".into()); } Ok(roots)
}
fn local_drive_kind(kind: u32) -> bool { matches!(kind, 2 | 3 | 5 | 6) }
fn full_scan_roots() -> Result<Vec<PathBuf>> {
    use windows_sys::Win32::Storage::FileSystem::{GetDriveTypeW, GetLogicalDrives};
    // Windows returns a bit per assigned drive letter. Never include mapped network drives.
    let mask = unsafe { GetLogicalDrives() };
    if mask == 0 { return Err("local_drives".into()); }
    let mut roots = Vec::new();
    for n in 0..26 {
        if mask & (1 << n) == 0 { continue; }
        let path = format!("{}:\\", (b'A' + n) as char);
        let wide: Vec<u16> = path.encode_utf16().chain(Some(0)).collect();
        if local_drive_kind(unsafe { GetDriveTypeW(wide.as_ptr()) }) { roots.push(PathBuf::from(path)); }
    }
    if roots.is_empty() { return Err("local_drives".into()); }
    Ok(roots)
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalFile { pub path: String, pub size: Option<u64>, pub modified: Option<u64> }
#[derive(Clone, Copy, Default, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Discovery { Model, #[default] Candidate, Excluded }

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalModel {
    pub id: String, pub name: String, pub path: String, pub source_root: String,
    pub format: String, pub kind: String, pub family: Option<String>, pub total_bytes: u64,
    #[serde(default)] pub discovery: Discovery,
    #[serde(default)] pub discovery_reason: String,
    #[serde(default)] pub scan_mode: Option<ScanMode>,
    pub status: String, pub completeness: String, pub files: Vec<LocalFile>, pub checked_at: String,
}
#[derive(Clone, Serialize, Deserialize)]
pub struct ScanNote { pub path: String, pub code: String }
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scan {
    #[serde(default)] pub mode: ScanMode,
    #[serde(default)] pub roots: Vec<String>,
    #[serde(default)] pub roots_finished: usize,
    pub status: String, pub root: String, pub visited: u64, pub found: usize,
    pub imported: usize, pub skipped: u64, pub truncated: bool, pub notes: Vec<ScanNote>,
}
impl Default for Scan {
    fn default() -> Self { Self { mode: ScanMode::Folder, roots: Vec::new(), roots_finished: 0, status: "idle".into(), root: String::new(), visited: 0, found: 0, imported: 0, skipped: 0, truncated: false, notes: Vec::new() } }
}
struct State { db: Connection, scan: Scan }
pub struct ModelLibrary { state: Mutex<State>, cancel: AtomicBool }
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot { entries: Vec<LocalModel>, scan: Scan }

// Import only reads local disk paths. Junctions, symlinks and other reparse points are skipped.
fn local_path(path: &Path) -> bool {
    path.is_absolute() && matches!(path.components().next(), Some(Component::Prefix(p)) if matches!(p.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_)))
        && !path.components().any(|p| matches!(p, Component::ParentDir))
}
pub(crate) fn no_links(path: &Path) -> Result<()> {
    use std::os::windows::fs::MetadataExt;
    if !local_path(path) { return Err("local_path".into()); }
    for ancestor in path.ancestors() {
        let meta = fs::symlink_metadata(ancestor).map_err(|_| "local_unavailable")?;
        if meta.file_attributes() & 0x400 != 0 { return Err("local_link".into()); }
    }
    Ok(())
}
// Only the normal Hugging Face snapshot -> same-repository blob file link is allowed.
// Directory links, links escaping the repository, and arbitrary file links remain rejected.
fn snapshot_repository(path: &Path) -> Result<PathBuf> {
    let snapshots = path.ancestors().find(|p| p.file_name().is_some_and(|n| n == "snapshots")).ok_or("local_link")?;
    let repository = snapshots.parent().filter(|p| p.file_name().is_some_and(|n| n.to_string_lossy().starts_with("models--"))).ok_or("local_link")?;
    let relative = path.strip_prefix(snapshots).map_err(|_| "local_link")?;
    let revision = relative.components().next().ok_or("local_link")?.as_os_str().to_string_lossy();
    if revision.len() != 40 || !revision.bytes().all(|b| b.is_ascii_hexdigit()) { return Err("local_link".into()); }
    Ok(repository.to_path_buf())
}
fn same_repository_blob(target: &Path, blobs: &Path) -> bool {
    let Some(name) = target.file_name() else { return false; }; let name = name.to_string_lossy();
    target.parent() == Some(blobs) && [40,64].contains(&name.len()) && name.bytes().all(|b| b.is_ascii_hexdigit())
}
fn physical_file(path: &Path) -> Result<PathBuf> {
    if no_links(path).is_ok() { return Ok(path.to_path_buf()); }
    if !local_path(path) || !fs::symlink_metadata(path).map_err(|_| "local_unavailable")?.file_type().is_symlink() { return Err("local_link".into()); }
    no_links(path.parent().ok_or("local_link")?)?;
    let repository = snapshot_repository(path)?;
    let target = fs::canonicalize(path).map_err(|_| "local_unavailable")?;
    let blobs = fs::canonicalize(repository.join("blobs")).map_err(|_| "local_link")?;
    no_links(&repository.join("blobs"))?; no_links(&target)?;
    if !same_repository_blob(&target, &blobs) { return Err("local_link".into()); }
    Ok(target)
}
pub(crate) fn safe_file(path: &Path) -> Result<File> {
    use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
    let path = physical_file(path)?;
    // Open the leaf without following a link introduced between inspection and open.
    let file = fs::OpenOptions::new().read(true).custom_flags(0x00200000).open(&path).map_err(|_| "local_unavailable")?;
    let meta = file.metadata().map_err(|_| "local_unavailable")?;
    if !meta.is_file() || meta.file_attributes() & 0x400 != 0 { return Err("local_link".into()); }
    Ok(file)
}
fn stamp(path: &Path) -> LocalFile {
    let meta = physical_file(path).ok().and_then(|p| fs::metadata(p).ok()).filter(|m| m.is_file());
    LocalFile { path: path.to_string_lossy().into(), size: meta.as_ref().map(|m| m.len()), modified: meta.and_then(|m| m.modified().ok()).and_then(|t| t.duration_since(UNIX_EPOCH).ok()).map(|d| d.as_millis() as u64) }
}
fn read_json(path: &Path, cap: u64) -> Result<Value> {
    let file = safe_file(path)?;
    if file.metadata().map_err(|_| "local_unavailable")?.len() > cap { return Err("local_limit".into()); }
    let mut data = Vec::new(); file.take(cap + 1).read_to_end(&mut data).map_err(|_| "local_unavailable")?;
    if data.len() as u64 > cap { return Err("local_limit".into()); }
    serde_json::from_slice(&data).map_err(|_| "local_invalid".into())
}
fn family(path: &Path) -> Option<String> {
    let config = read_json(&path.parent()?.join("config.json"), 1024 * 1024).ok()?;
    config["model_type"].as_str().filter(|s| s.len() <= 100 && !s.chars().any(char::is_control)).map(str::to_owned)
}

// Structural inspection reads the header only, never tensor payloads or model code.
pub(crate) fn safetensors(path: &Path, budget: &mut u64) -> Result<(bool, bool)> {
    let mut file = safe_file(path)?;
    let size = file.metadata().map_err(|_| "local_unavailable")?.len();
    let mut length = [0; 8]; file.read_exact(&mut length).map_err(|_| "local_invalid")?;
    let length = u64::from_le_bytes(length);
    // A length that cannot fit in the file is invalid even when it exceeds our read limit.
    if length < 2 || length.checked_add(8).is_none_or(|n| n > size) { return Err("local_invalid".into()); }
    if length > MAX_HEADER || length > *budget { return Err("local_limit".into()); }
    *budget -= length;
    let mut data = vec![0; length as usize]; file.read_exact(&mut data).map_err(|_| "local_invalid")?;
    if data.first() != Some(&b'{') { return Err("local_invalid".into()); }
    // Reject duplicate tensor keys instead of silently accepting JSON last-key-wins.
    struct UniqueHeader;
    impl<'de> serde::de::Visitor<'de> for UniqueHeader {
        type Value = std::collections::BTreeMap<String, Value>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { f.write_str("unique tensor names") }
        fn visit_map<M: serde::de::MapAccess<'de>>(self, mut map: M) -> std::result::Result<Self::Value, M::Error> {
            let mut values = std::collections::BTreeMap::new();
            while let Some((key, value)) = map.next_entry::<String, Value>()? {
                if values.insert(key, value).is_some() { return Err(serde::de::Error::custom("duplicate tensor name")); }
            }
            Ok(values)
        }
    }
    let mut parser = serde_json::Deserializer::from_slice(&data);
    let object = serde::Deserializer::deserialize_map(&mut parser, UniqueHeader).map_err(|_| "local_invalid")?;
    parser.end().map_err(|_| "local_invalid")?;
    let mut ranges = Vec::new(); let mut understood = true; let mut component = false;
    for (name, tensor) in object {
        if name == "__metadata__" {
            let metadata = tensor.as_object().ok_or("local_invalid")?;
            if metadata.values().any(|v| !v.is_string()) { return Err("local_invalid".into()); }
            component |= metadata.contains_key("ss_network_module"); continue;
        }
        let lowered = name.to_lowercase();
        component |= lowered.contains("lora_") || lowered.contains(".lora_a.") || lowered.contains(".lora_b.");
        let offset = tensor["data_offsets"].as_array().filter(|a| a.len() == 2).ok_or("local_invalid")?;
        let start = offset[0].as_u64().ok_or("local_invalid")?; let end = offset[1].as_u64().ok_or("local_invalid")?;
        if start > end || end > size - length - 8 { return Err("local_invalid".into()); }
        let shape = tensor["shape"].as_array().ok_or("local_invalid")?;
        let count = shape.iter().try_fold(1u64, |n, dim| n.checked_mul(dim.as_u64()?)).ok_or("local_invalid")?;
        let dtype = tensor["dtype"].as_str().ok_or("local_invalid")?;
        let width = match dtype { "BOOL" | "U8" | "I8" | "F8_E4M3" | "F8_E5M2" => Some(1), "I16" | "U16" | "F16" | "BF16" => Some(2), "I32" | "U32" | "F32" => Some(4), "I64" | "U64" | "F64" => Some(8), _ => None };
        if let Some(width) = width { if count.checked_mul(width) != Some(end - start) { return Err("local_invalid".into()); } } else { understood = false; }
        ranges.push((start, end));
    }
    if ranges.is_empty() { return Err("local_invalid".into()); }
    ranges.sort_unstable(); let mut end = 0;
    for (start, next) in ranges { if start != end { return Err("local_invalid".into()); } end = next; }
    if end != size - length - 8 { return Err("local_invalid".into()); }
    Ok((understood, component))
}
fn classify(path: &Path) -> Option<&'static str> {
    let name = path.file_name()?.to_str()?.to_lowercase();
    if name.ends_with(".safetensors.index.json") { return Some("safetensors-index"); }
    if name.ends_with(".bin.index.json") { return Some("pytorch-index"); }
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "safetensors" => Some("safetensors"), "gguf" => Some("gguf"), "onnx" => Some("onnx"),
        "ckpt" | "pt" | "pth" => Some("pytorch"), "bin" if name.starts_with("pytorch_model") => Some("pytorch"), _ => None,
    }
}
fn relative_file(value: &str) -> bool {
    !value.is_empty() && value.len() < 1024 && !value.contains(['\\', ':', '\0'])
        && value.split('/').all(|p| !p.is_empty() && p != "." && p != ".." && !p.ends_with(['.', ' ']))
}
fn inspect(path: &Path, root: &str) -> LocalModel { inspect_limited(path, root, None) }
fn inspect_limited(path: &Path, root: &str, cancel: Option<&AtomicBool>) -> LocalModel {
    let mut budget = 64 * 1024 * 1024u64;
    let source = path.to_string_lossy().to_string();
    let format = classify(path).unwrap_or("unknown").to_string();
    let mut entry = LocalModel { id: format!("{:x}", Sha256::digest(source.to_lowercase().as_bytes())), name: path.file_name().unwrap_or_default().to_string_lossy().into(), path: source, source_root: root.into(), format: format.clone(), discovery: Discovery::Candidate, discovery_reason: String::new(), scan_mode: None, kind: "candidate".into(), family: family(path), total_bytes: 0, status: "unverified".into(), completeness: "unknown".into(), files: vec![stamp(path)], checked_at: crate::database::now() };
    if entry.files[0].size.is_none() { entry.status = "missing".into(); return entry; }
    if format.ends_with("-index") {
        let result = (|| -> Result<()> {
            let index = read_json(path, MAX_HEADER)?;
            let weights = index["weight_map"].as_object().filter(|v| !v.is_empty() && v.len() <= 100_000).ok_or("local_invalid")?;
            let mut names = BTreeSet::new();
            for value in weights.values() {
                let name = value.as_str().filter(|v| relative_file(v)).ok_or("local_invalid")?;
                if (format == "safetensors-index" && !name.ends_with(".safetensors")) || (format == "pytorch-index" && !name.ends_with(".bin")) { return Err("local_invalid".into()); }
                names.insert(name);
            }
            if names.len() > 2048 { return Err("local_limit".into()); }
            let mut missing = false; let mut invalid = false; let mut unknown = false;
            for name in names {
                if cancel.is_some_and(|c| c.load(Ordering::Relaxed)) { return Err("local_cancelled".into()); }
                let sibling = path.parent().ok_or("local_invalid")?.join(name);
                let file = stamp(&sibling); missing |= file.size.is_none();
                if file.size.is_some() && format == "safetensors-index" {
                    match safetensors(&sibling, &mut budget) { Ok((true, component)) => { if component { entry.kind = "component".into(); } }, Ok(_) => unknown = true, Err(e) if e == "local_invalid" => invalid = true, Err(_) => unknown = true }
                }
                entry.files.push(file);
            }
            entry.completeness = "index".into();
            entry.status = if missing { "incomplete" } else if invalid { "invalid" } else if unknown || format == "pytorch-index" { "unverified" } else { "checked" }.into();
            Ok(())
        })();
        if let Err(error) = result { entry.status = if error == "local_invalid" { "invalid" } else { "unverified" }.into(); }
    } else if format == "safetensors" {
        match safetensors(path, &mut budget) {
            Ok((known, component)) => { entry.status = if known { "checked" } else { "recognized" }.into(); entry.completeness = "container".into(); if component { entry.kind = "component".into(); } },
            Err(error) => entry.status = if error == "local_invalid" { "invalid" } else { "unverified" }.into(),
        }
    } else if format == "gguf" {
        let result = (|| -> Result<bool> {
            let mut file = safe_file(path)?; let mut header = [0; 24]; file.read_exact(&mut header).map_err(|_| "local_invalid")?;
            if &header[..4] != b"GGUF" { return Err("local_invalid".into()); }
            let version = u32::from_le_bytes(header[4..8].try_into().unwrap());
            // Magic/version recognition only. Do not claim payload or split completeness.
            Ok(version == 2 || version == 3)
        })();
        entry.status = match result { Ok(true) => "recognized", Err(e) if e == "local_invalid" => "invalid", _ => "unverified" }.into();
    }
    entry.total_bytes = entry.files.iter().filter_map(|f| f.size).fold(0, u64::saturating_add);
    classify_discovery(&mut entry, ScanMode::Folder);
    entry
}

impl ModelLibrary {
    pub fn new(config: &Path) -> Result<Arc<Self>> {
        let mut db = Connection::open(config.join("model-library.sqlite3")).map_err(|_| "local_storage")?;
        db.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE IF NOT EXISTS local_models(id TEXT PRIMARY KEY, json TEXT NOT NULL); CREATE TABLE IF NOT EXISTS model_scan(id INTEGER PRIMARY KEY CHECK(id=1), json TEXT NOT NULL);").map_err(|_| "local_storage")?;
        migrate_discovery(&mut db)?;
        let mut scan = db.query_row("SELECT json FROM model_scan WHERE id=1", [], |row| row.get::<_, String>(0)).ok().and_then(|s| serde_json::from_str::<Scan>(&s).ok()).unwrap_or_default();
        if scan.status == "running" { scan.status = "interrupted".into(); }
        Ok(Arc::new(Self { state: Mutex::new(State { db, scan }), cancel: AtomicBool::new(false) }))
    }
    fn note(scan: &mut Scan, path: &Path, code: &str) {
        scan.skipped += 1;
        if scan.notes.len() < 50 { scan.notes.push(ScanNote { path: path.to_string_lossy().into(), code: code.into() }); }
    }
    fn save_scan(state: &State) -> Result<()> {
        state.db.execute("INSERT OR REPLACE INTO model_scan VALUES(1,?1)", [serde_json::to_string(&state.scan).map_err(|_| "local_storage")?]).map_err(|_| "local_storage")?; Ok(())
    }
    #[cfg(test)]
    fn scan_folder(&self, root: PathBuf) { self.scan_roots(vec![root], ScanMode::Folder, ScanMode::Folder.limits()); }
    fn scan_roots(&self, roots: Vec<PathBuf>, mode: ScanMode, limits: ScanLimits) {
        use std::os::windows::fs::MetadataExt;
        let mut progress = Scan { mode, roots: roots.iter().map(|p| p.to_string_lossy().into()).collect(), status: "running".into(), ..Scan::default() };
        let mut found = Vec::new(); let mut file_count = 0;
        let mut published = std::time::Instant::now();
        'roots: for requested in roots {
            if self.cancel.load(Ordering::Relaxed) { break; }
            progress.root = requested.to_string_lossy().into();
            if let Ok(mut state) = self.state.lock() { state.scan = progress.clone(); }
            let prepared = (|| -> Result<PathBuf> {
                no_links(&requested)?;
                fs::canonicalize(&requested).map_err(|_| "local_unavailable".into())
            })();
            let root = match prepared { Ok(root) => root, Err(code) => { Self::note(&mut progress, &requested, &code); progress.roots_finished += 1; continue; } };
            let source = root.to_string_lossy().to_string();
            let children = match fs::read_dir(&root) { Ok(v) => v, Err(_) => { Self::note(&mut progress, &root, "local_unavailable"); progress.roots_finished += 1; continue; } };
            // Keep iterators, not every sibling path, so memory stays bounded on wide trees.
            let mut stack = vec![(root, children)];
            while !stack.is_empty() {
                if self.cancel.load(Ordering::Relaxed) { break 'roots; }
                if published.elapsed() >= std::time::Duration::from_millis(250) {
                    progress.root = stack.last().unwrap().0.to_string_lossy().into();
                    if let Ok(mut state) = self.state.lock() { state.scan = progress.clone(); }
                    published = std::time::Instant::now();
                }
                let (directory, children) = stack.last_mut().unwrap();
                let Some(child) = children.next() else { stack.pop(); continue; };
                if progress.visited >= limits.visited || found.len() >= limits.entries || file_count >= limits.files {
                    progress.truncated = true; Self::note(&mut progress, directory, "local_limit"); break 'roots;
                }
                progress.visited += 1;
                let child = match child { Ok(v) => v, Err(_) => { Self::note(&mut progress, directory, "local_unavailable"); continue; } };
                let path = child.path();
                let meta = match fs::symlink_metadata(&path) { Ok(v) => v, Err(_) => { Self::note(&mut progress, &path, "local_unavailable"); continue; } };
                let cache_link = meta.file_attributes() & 0x400 != 0 && classify(&path).is_some() && physical_file(&path).is_ok();
                if meta.file_attributes() & 0x400 != 0 && !cache_link { Self::note(&mut progress, &path, "local_link"); continue; }
                if mode != ScanMode::Folder {
                    if let Some(reason) = excluded_location(&path) { Self::note(&mut progress, &path, reason); continue; }
                }
                if meta.is_dir() {
                    if stack.len() > limits.depth { Self::note(&mut progress, &path, "local_limit"); progress.truncated = true; continue; }
                    if let Err(code) = no_links(&path) { Self::note(&mut progress, &path, &code); continue; }
                    match fs::read_dir(&path) {
                        Ok(children) => { progress.root = path.to_string_lossy().into(); stack.push((path, children)); },
                        Err(_) => Self::note(&mut progress, &path, "local_unavailable"),
                    }
                } else if (meta.is_file() || cache_link) && classify(&path).is_some() {
                    let mut entry = inspect_limited(&path, &source, Some(&self.cancel));
                    classify_discovery(&mut entry, mode);
                    if entry.discovery == Discovery::Excluded { Self::note(&mut progress, &path, &entry.discovery_reason); continue; }
                    if file_count + entry.files.len() > limits.files { progress.truncated = true; Self::note(&mut progress, &path, "local_limit"); break 'roots; }
                    file_count += entry.files.len(); found.push(entry); progress.found = found.len();
                }
            }
            progress.roots_finished += 1;
        }
        // Shards referenced by an accepted index appear once under that index, not as separate models.
        let grouped: HashSet<_> = found.iter().filter(|e| e.completeness == "index").flat_map(|e| e.files.iter().skip(1).map(|f| f.path.to_lowercase())).collect();
        found.retain(|e| !grouped.contains(&e.path.to_lowercase()));
        if let Ok(mut state) = self.state.lock() {
            if self.cancel.load(Ordering::Relaxed) { progress.status = "cancelled".into(); }
            else {
                let saved = (|| -> Result<()> {
                    let tx = state.db.transaction().map_err(|_| "local_storage")?;
                    for entry in &found { tx.execute("INSERT OR REPLACE INTO local_models VALUES(?1,?2)", params![entry.id, serde_json::to_string(entry).map_err(|_| "local_storage")?]).map_err(|_| "local_storage")?; }
                    // Remove old individual entries now represented by an index.
                    for file in &grouped { let id = format!("{:x}", Sha256::digest(file.as_bytes())); tx.execute("DELETE FROM local_models WHERE id=?1", [id]).map_err(|_| "local_storage")?; }
                    tx.commit().map_err(|_| "local_storage")?; Ok(())
                })();
                if saved.is_ok() { progress.imported = found.len(); progress.status = "completed".into(); }
                else { progress.status = "failed".into(); }
            }
            state.scan = progress; if Self::save_scan(&state).is_err() { state.scan.status = "failed".into(); }
        }
    }
    fn start(self: &Arc<Self>, path: &str) -> Result<Scan> {
        let path = Path::new(path.trim()); no_links(path)?;
        if !path.is_dir() { return Err("local_folder".into()); }
        let root = fs::canonicalize(path).map_err(|_| "local_unavailable")?;
        self.start_roots(vec![root], ScanMode::Folder)
    }
    fn start_roots(self: &Arc<Self>, roots: Vec<PathBuf>, mode: ScanMode) -> Result<Scan> {
        let scan = Scan { mode, roots: roots.iter().map(|p| p.to_string_lossy().into()).collect(), status: "running".into(), root: roots.first().map(|p| p.to_string_lossy().into()).unwrap_or_default(), ..Scan::default() };
        { let mut state = self.state.lock().map_err(|_| "local_storage")?;
            if state.scan.status == "running" { return Err("local_busy".into()); }
            self.cancel.store(false, Ordering::Relaxed); state.scan = scan.clone();
            if let Err(error) = Self::save_scan(&state) { state.scan.status = "failed".into(); return Err(error); }
        }
        let manager = Arc::clone(self);
        tauri::async_runtime::spawn_blocking(move || manager.scan_roots(roots, mode, mode.limits())); Ok(scan)
    }
    pub fn stop(&self) { self.cancel.store(true, Ordering::Relaxed); }
    pub async fn shutdown(&self) {
        self.stop();
        loop {
            if self.state.lock().map(|s| s.scan.status != "running").unwrap_or(true) { break; }
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }
    }
    fn snapshot(&self) -> Result<Snapshot> {
        let (mut entries, scan) = {
            let state = self.state.lock().map_err(|_| "local_storage")?;
            let mut query = state.db.prepare("SELECT json FROM local_models ORDER BY rowid DESC").map_err(|_| "local_storage")?;
            let rows = query.query_map([], |r| r.get::<_, String>(0)).map_err(|_| "local_storage")?;
            let entries = rows.map(|r| serde_json::from_str::<LocalModel>(&r.map_err(|_| "local_storage")?).map_err(|_| "local_storage".to_string())).collect::<Result<Vec<_>>>()?;
            (entries, state.scan.clone())
        };
        for entry in &mut entries {
            if entry.discovery == Discovery::Excluded { continue; }
            let current_files: Vec<_> = entry.files.iter().map(|f| stamp(Path::new(&f.path))).collect();
            for (old, current) in entry.files.iter().zip(&current_files) {
                if current.size.is_none() && old.size.is_some() { entry.status = if Path::new(&old.path).ancestors().last().is_some_and(|p| p.exists()) { "missing" } else { "unavailable" }.into(); break; }
                if current.size != old.size || current.modified != old.modified { entry.status = "changed".into(); }
            }
            entry.total_bytes = current_files.iter().filter_map(|f| f.size).fold(0, u64::saturating_add);
            entry.files = current_files;
        }
        Ok(Snapshot { entries, scan })
    }
    fn recheck(&self, id: &str) -> Result<()> {
        let state = self.state.lock().map_err(|_| "local_storage")?;
        if state.scan.status == "running" { return Err("local_busy".into()); }
        let json: String = state.db.query_row("SELECT json FROM local_models WHERE id=?1", [id], |r| r.get(0)).map_err(|_| "local_missing")?;
        let old: LocalModel = serde_json::from_str(&json).map_err(|_| "local_storage")?;
        let mut next = inspect(Path::new(&old.path), &old.source_root);
        next.discovery = old.discovery;
        classify_discovery(&mut next, old.scan_mode.unwrap_or(ScanMode::Folder));
        state.db.execute("UPDATE local_models SET json=?1 WHERE id=?2", params![serde_json::to_string(&next).map_err(|_| "local_storage")?, id]).map_err(|_| "local_storage")?; Ok(())
    }
    fn forget(&self, id: &str) -> Result<()> {
        let state = self.state.lock().map_err(|_| "local_storage")?;
        if state.scan.status == "running" { return Err("local_busy".into()); }
        state.db.execute("DELETE FROM local_models WHERE id=?1", [id]).map_err(|_| "local_storage")?; Ok(())
    }
}

#[tauri::command]
pub async fn model_library_list(state: tauri::State<'_, Arc<ModelLibrary>>) -> Result<Snapshot> {
    let state = state.inner().clone(); tauri::async_runtime::spawn_blocking(move || state.snapshot()).await.map_err(|_| "local_storage")?
}
#[tauri::command]
pub async fn model_scan_start(path: String, state: tauri::State<'_, Arc<ModelLibrary>>) -> Result<Scan> {
    let state = state.inner().clone(); tauri::async_runtime::spawn_blocking(move || state.start(&path)).await.map_err(|_| "local_storage")?
}
#[tauri::command]
pub async fn model_scan_full(state: tauri::State<'_, Arc<ModelLibrary>>) -> Result<Scan> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || state.start_roots(full_scan_roots()?, ScanMode::Full)).await.map_err(|_| "local_storage")?
}
#[tauri::command]
pub async fn model_scan_quick(core: tauri::State<'_, Arc<crate::core::Core>>, state: tauri::State<'_, Arc<ModelLibrary>>) -> Result<Scan> {
    let paths = core.storage_paths()?;
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut roots = quick_scan_roots(Path::new(&paths.models))?;
        roots.extend([PathBuf::from(paths.assistant_models), PathBuf::from(paths.vision_models)]);
        state.start_roots(unique_roots(roots), ScanMode::Quick)
    }).await.map_err(|_| "local_storage")?
}
#[tauri::command]
pub fn model_scan_cancel(state: tauri::State<'_, Arc<ModelLibrary>>) { state.stop(); }
#[tauri::command]
pub async fn model_library_recheck(id: String, state: tauri::State<'_, Arc<ModelLibrary>>) -> Result<()> {
    let state = state.inner().clone(); tauri::async_runtime::spawn_blocking(move || state.recheck(&id)).await.map_err(|_| "local_storage")?
}
#[tauri::command]
pub fn model_library_forget(id: String, state: tauri::State<'_, Arc<ModelLibrary>>) -> Result<()> { state.forget(&id) }

#[cfg(test)]
#[path = "model_library_tests.rs"]
mod tests;
