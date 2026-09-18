use super::*;
use rusqlite::params;
#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Node {pub id:String,pub parent:Option<String>,pub group:String,pub path:String,pub name:String,pub kind:String,pub file_id:String,pub stamp:String,pub created_at:i64,pub operation:String,pub origin:Option<Origin>,#[serde(default)]pub edit:Option<serde_json::Value>}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Version {node:Node,available:bool,version:Option<String>}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Family {current:String,primary:Option<String>,versions:Vec<Version>}
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct LineageQuery {pub(super) root_id:String,pub(super) target:FileTarget}
fn error(_:impl std::fmt::Display)->String{"gallery_storage".into()}
pub(super) fn initialize(db:&Connection)->Result<()>{db.execute_batch("CREATE TABLE IF NOT EXISTS lineage(root TEXT NOT NULL,id TEXT NOT NULL,group_id TEXT NOT NULL,file_id TEXT NOT NULL,stamp TEXT NOT NULL,json TEXT NOT NULL,PRIMARY KEY(root,id),UNIQUE(root,file_id,stamp));CREATE INDEX IF NOT EXISTS lineage_group ON lineage(root,group_id);CREATE TABLE IF NOT EXISTS lineage_primary(root TEXT NOT NULL,group_id TEXT NOT NULL,node TEXT NOT NULL,PRIMARY KEY(root,group_id));").map_err(error)}
pub(super) fn checked(root:&Path,q:&LineageQuery)->Result<(PathBuf,File,Vec<File>)>{if q.root_id!=root_id(root){return Err("gallery_changed".into());}let path=resolve(root,&q.target.path)?;let pins=directory_guards(path.parent().ok_or("gallery_path")?)?;let file=lock_file(&path)?;let meta=file.metadata().map_err(error)?;if identity(&file)?!=q.target.file_id||thumbnails::version(root,&path,&meta,Some(&q.target.file_id))!=q.target.version{return Err("gallery_changed".into());}if media(&path).is_none(){return Err("gallery_format".into());}Ok((path,file,pins))}
pub(super) fn put(db:&Connection,root:&Path,n:&Node)->Result<()>{db.execute("INSERT INTO lineage VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(root,id) DO UPDATE SET json=excluded.json",params![root_id(root),n.id,n.group,n.file_id,n.stamp,serde_json::to_string(n).map_err(error)?]).map_err(error)?;Ok(())}
pub(super) fn source(db:&Connection,root:&Path,q:&LineageQuery,path:&Path,file:&File,origin:Option<BoundOrigin>)->Result<Node>{
 let stamp=stamp(&file.metadata().map_err(error)?);let saved:Option<String>=db.query_row("SELECT json FROM lineage WHERE root=?1 AND file_id=?2 AND stamp=?3",params![q.root_id,q.target.file_id,stamp],|r|r.get(0)).optional().map_err(error)?;
 if let Some(json)=saved{let mut n:Node=serde_json::from_str(&json).map_err(error)?;n.path=q.target.path.clone();n.name=path.file_name().unwrap().to_string_lossy().into();put(db,root,&n)?;return Ok(n);}
 let origin=origin.filter(|o|o.stamp==stamp).map(|o|o.info);
 let id=uuid::Uuid::new_v4().to_string();let n=Node{id:id.clone(),parent:None,group:id,path:q.target.path.clone(),name:path.file_name().unwrap().to_string_lossy().into(),kind:media(path).unwrap().0.into(),file_id:q.target.file_id.clone(),stamp,created_at:chrono::Utc::now().timestamp_millis(),operation:if origin.is_some(){"textToImage"}else{"original"}.into(),origin,edit:None};put(db,root,&n)?;Ok(n)
}
fn present(root:&Path,n:&Node)->bool{resolve(root,&n.path).ok().and_then(|path|lock_file(&path).ok()).is_some_and(|file|identity(&file).ok().as_deref()==Some(&n.file_id)&&file.metadata().is_ok_and(|m|stamp(&m)==n.stamp))}
// Only search for known identities whose old paths disappeared. No inferred visual similarity.
fn relocate_missing(root:&Path,nodes:&mut [Node]){
 let missing:std::collections::HashSet<String>=nodes.iter().filter(|n|!present(root,n)).map(|n|n.file_id.clone()).collect();if missing.is_empty(){return;}
 let mut pending=vec![(root.to_path_buf(),0)];let mut scanned=0;
 while let Some((folder,depth))=pending.pop(){if depth>32||scanned>=MAX_ENTRIES{break;}let Ok(_pins)=directory_guards(&folder)else{continue;};let Ok(entries)=fs::read_dir(folder)else{continue;};for entry in entries.flatten(){scanned+=1;if scanned>MAX_ENTRIES{break;}let path=entry.path();let Ok(meta)=fs::symlink_metadata(&path)else{continue;};if meta.file_attributes()&0x400!=0||entry.file_name().to_string_lossy().eq_ignore_ascii_case(TRASH){continue;}if meta.is_dir(){pending.push((path,depth+1));continue;}if media(&path).is_none(){continue;}let Ok(file)=lock_file(&path)else{continue;};let Ok(id)=identity(&file)else{continue;};if !missing.contains(&id){continue;}for n in nodes.iter_mut().filter(|n|n.file_id==id&&n.stamp==stamp(&meta)){if let Ok(rel)=relative(root,&path){n.path=rel;n.name=entry.file_name().to_string_lossy().into();}}}}
}
impl GalleryCatalog {
 fn family(&self,root:&Path,q:LineageQuery,origin:Option<BoundOrigin>)->Result<Family>{let _gate=self.files_gate.lock().map_err(error)?;let(path,file,_pins)=checked(root,&q)?;let db=self.db.lock().map_err(error)?;let current=source(&db,root,&q,&path,&file,origin)?;let mut stmt=db.prepare("SELECT json FROM lineage WHERE root=?1 AND group_id=?2 ORDER BY rowid LIMIT 100").map_err(error)?;let rows=stmt.query_map(params![q.root_id,current.group],|r|r.get::<_,String>(0)).map_err(error)?;let mut nodes=vec![];for row in rows{nodes.push(serde_json::from_str::<Node>(&row.map_err(error)?).map_err(error)?);}drop(stmt);relocate_missing(root,&mut nodes);let mut versions=vec![];for n in nodes{put(&db,root,&n)?;let available=present(root,&n);let version=if available{resolve(root,&n.path).ok().and_then(|p|fs::metadata(&p).ok().map(|m|thumbnails::version(root,&p,&m,Some(&n.file_id))))}else{None};versions.push(Version{available,version,node:n});}
 let preferred:Option<String>=db.query_row("SELECT node FROM lineage_primary WHERE root=?1 AND group_id=?2",params![q.root_id,current.group],|r|r.get(0)).optional().map_err(error)?;
 let primary=preferred.filter(|id|versions.iter().any(|v|v.available&&v.node.id==*id)).or_else(||versions.iter().rev().find(|v|v.available).map(|v|v.node.id.clone()));Ok(Family{current:current.id,primary,versions})}
 fn variant(&self,root:&Path,q:LineageQuery,origin:Option<BoundOrigin>)->Result<String>{let _gate=self.files_gate.lock().map_err(error)?;let(path,file,_pins)=checked(root,&q)?;let mut db=self.db.lock().map_err(error)?;let parent=source(&db,root,&q,&path,&file,origin)?;let count:i64=db.query_row("SELECT count(*) FROM lineage WHERE root=?1 AND group_id=?2",params![q.root_id,parent.group],|r|r.get(0)).map_err(error)?;if count>=100{return Err("gallery_lineage_limit".into());}
 let name=format!("{} - {}.{}",path.file_stem().unwrap_or_default().to_string_lossy().chars().take(70).collect::<String>(),&uuid::Uuid::new_v4().to_string()[..8],path.extension().unwrap_or_default().to_string_lossy());let copied=copy_one_named(root,path.parent().ok_or("gallery_path")?,&path,&name)?;let copied_path=resolve(root,&copied)?;let copy=lock_file(&copied_path)?;
 let node=Node{id:uuid::Uuid::new_v4().to_string(),parent:Some(parent.id),group:parent.group,path:copied.clone(),name:copied_path.file_name().unwrap().to_string_lossy().into(),kind:parent.kind,file_id:identity(&copy)?,stamp:stamp(&copy.metadata().map_err(error)?),created_at:chrono::Utc::now().timestamp_millis(),operation:"copy".into(),origin:parent.origin,edit:parent.edit};let tx=db.transaction().map_err(error)?;put(&tx,root,&node)?;tx.commit().map_err(error)?;Ok(copied)}
 fn primary(&self,root:&Path,q:LineageQuery)->Result<()>{let _gate=self.files_gate.lock().map_err(error)?;let(path,file,_pins)=checked(root,&q)?;let db=self.db.lock().map_err(error)?;let node=source(&db,root,&q,&path,&file,None)?;db.execute("INSERT INTO lineage_primary VALUES(?1,?2,?3) ON CONFLICT(root,group_id) DO UPDATE SET node=excluded.node",params![q.root_id,node.group,node.id]).map_err(error)?;Ok(())}
}
#[tauri::command]
pub async fn gallery_lineage(query:LineageQuery,core:State<'_,Arc<Core>>,catalog:State<'_,Arc<GalleryCatalog>>,images:State<'_,Arc<ImageEngine>>)->Result<Family>{let root=root(&core)?;let c=catalog.inner().clone();let i=images.inner().clone();tauri::async_runtime::spawn_blocking(move||{let origins=c.sync_origins(&root,i.list()?)?;let origin=origins.get(&query.target.file_id).cloned();c.family(&root,query,origin)}).await.map_err(error)?}
#[tauri::command]
pub async fn gallery_create_variant(query:LineageQuery,core:State<'_,Arc<Core>>,catalog:State<'_,Arc<GalleryCatalog>>,images:State<'_,Arc<ImageEngine>>)->Result<String>{let root=root(&core)?;let c=catalog.inner().clone();let i=images.inner().clone();tauri::async_runtime::spawn_blocking(move||{let origins=c.sync_origins(&root,i.list()?)?;let origin=origins.get(&query.target.file_id).cloned();c.variant(&root,query,origin)}).await.map_err(error)?}
#[tauri::command]
pub async fn gallery_set_primary(query:LineageQuery,core:State<'_,Arc<Core>>,catalog:State<'_,Arc<GalleryCatalog>>)->Result<()>{let root=root(&core)?;let c=catalog.inner().clone();tauri::async_runtime::spawn_blocking(move||c.primary(&root,query)).await.map_err(error)?}
#[cfg(test)]mod tests {
 use super::*;
 fn query(root:&Path,path:&str)->LineageQuery{let target=resolve(root,path).unwrap();let file=lock_file(&target).unwrap();let id=identity(&file).unwrap();let version=thumbnails::version(root,&target,&file.metadata().unwrap(),Some(&id));LineageQuery{root_id:root_id(root),target:FileTarget{path:path.into(),file_id:id,version}}}
 #[test]fn copies_keep_lineage_after_rename_and_source_deletion_and_primary_persists(){
 let t=tempfile::tempdir().unwrap();let root=fs::canonicalize(t.path()).unwrap();fs::write(root.join("original.png"),b"passive original bytes").unwrap();let c=GalleryCatalog::new(&root).unwrap();let first=c.family(&root,query(&root,"original.png"),None).unwrap();assert_eq!(first.versions.len(),1);
 let copied=c.variant(&root,query(&root,"original.png"),None).unwrap();assert_eq!(fs::read(root.join(&copied)).unwrap(),b"passive original bytes");let family=c.family(&root,query(&root,&copied),None).unwrap();assert_eq!(family.versions.len(),2);assert_eq!(family.versions[1].node.parent.as_deref(),Some(first.current.as_str()));assert_eq!(family.primary.as_deref(),Some(family.current.as_str()));
 c.primary(&root,query(&root,"original.png")).unwrap();drop(c);let c=GalleryCatalog::new(&root).unwrap();assert_eq!(c.family(&root,query(&root,&copied),None).unwrap().primary,Some(first.current.clone()));
 fs::rename(root.join("original.png"),root.join("renamed.png")).unwrap();let family=c.family(&root,query(&root,&copied),None).unwrap();assert_eq!(family.versions[0].node.path,"renamed.png");assert!(family.versions[0].available);
 fs::remove_file(root.join("renamed.png")).unwrap();let family=c.family(&root,query(&root,&copied),None).unwrap();assert!(!family.versions[0].available);assert!(family.versions[1].available);assert_eq!(family.primary,Some(family.current));assert_eq!(fs::read(root.join(&copied)).unwrap(),b"passive original bytes");
 }
 #[test]fn generated_origin_is_inherited_only_from_the_bound_source(){
 let t=tempfile::tempdir().unwrap();let root=fs::canonicalize(t.path()).unwrap();fs::write(root.join("generated.png"),b"passive metadata fixture").unwrap();let c=GalleryCatalog::new(&root).unwrap();
 let request=ImageRequest{model_path:"D:/models/test.safetensors".into(),prompt:"Recorded prompt, no inference in this test".into(),negative_prompt:"noise".into(),width:512,height:512,steps:20,guidance:5.0,seed:42,sampler:"euler".into()};
 let origin=Origin{job_id:uuid::Uuid::new_v4().to_string(),request,model_name:"test.safetensors".into(),model_sha256:Some("a".repeat(64)),runtime:"metadata fixture".into(),created_at:"2026-01-01T00:00:00Z".into(),association:"fileIdentity".into()};
 let bound=BoundOrigin{stamp:stamp(&fs::metadata(root.join("generated.png")).unwrap()),info:origin.clone()};
 let copied=c.variant(&root,query(&root,"generated.png"),Some(bound)).unwrap();let family=c.family(&root,query(&root,&copied),None).unwrap();assert_eq!(family.versions[0].node.operation,"textToImage");assert_eq!(family.versions[1].node.origin.as_ref().unwrap().job_id,origin.job_id);assert_eq!(family.versions[1].node.origin.as_ref().unwrap().request.prompt,origin.request.prompt);
 fs::write(root.join("changed.png"),b"unrelated file").unwrap();let wrong=BoundOrigin{stamp:"stale".into(),info:origin};let family=c.family(&root,query(&root,"changed.png"),Some(wrong)).unwrap();assert!(family.versions[0].node.origin.is_none());assert_eq!(family.versions[0].node.operation,"original");
 }
 #[test]fn stale_variants_are_rejected_and_changed_files_start_separate_lineage(){let t=tempfile::tempdir().unwrap();let root=fs::canonicalize(t.path()).unwrap();fs::write(root.join("a.png"),b"old").unwrap();let c=GalleryCatalog::new(&root).unwrap();let before=c.family(&root,query(&root,"a.png"),None).unwrap();let stale=query(&root,"a.png");fs::write(root.join("a.png"),b"new longer bytes").unwrap();assert!(c.variant(&root,stale,None).is_err());let after=c.family(&root,query(&root,"a.png"),None).unwrap();assert_ne!(before.current,after.current);assert_eq!(after.versions.len(),1);let mut wrong=query(&root,"a.png");wrong.root_id="outside".into();assert!(c.variant(&root,wrong,None).is_err());}
}
