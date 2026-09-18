use super::*;
use gallery::{EditOperation as Operation,EditPreview,GalleryCatalog};

#[derive(Clone,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Query {id:String,asset_id:String,sha256:String}
impl Projects {
 fn editor_source(&self,q:&Query)->Result<(Asset,File,Vec<File>)>{
  let s=self.state.lock().map_err(err)?;let p=Self::require(&s)?;
  if p.id!=q.id{return Err("project_changed".into());}
  let a=p.assets.iter().find(|a|a.id==q.asset_id&&a.kind=="image"&&a.sha256==q.sha256).ok_or("project_changed")?.clone();
  if a.bytes>64*1024*1024{return Err("editor_limit".into());}
  let path=owned(&p,&a)?;let pins=gallery::directory_guards(path.parent().ok_or("project_path")?)?;let mut file=gallery::lock_file(&path)?;
  if file.metadata().map_err(err)?.len()!=a.bytes||digest(&mut file)?!=a.sha256{return Err("project_hash".into());}file.rewind().map_err(err)?;
  Ok((a,file,pins))
 }
 fn save_edit(&self,q:&Query,expected:&[Operation],operations:Vec<Operation>)->Result<Project>{
  let(_,file,_pins)=self.editor_source(q)?;gallery::render_preview(file.try_clone().map_err(err)?,&operations)?;
  let mut s=self.state.lock().map_err(err)?;let mut p=Self::require(&s)?;if p.id!=q.id{return Err("project_changed".into());}
  let a=p.assets.iter_mut().find(|a|a.id==q.asset_id&&a.sha256==q.sha256).ok_or("project_changed")?;
  if a.edit!=expected{return Err("project_changed".into());}a.edit=operations;p.dirty=true;
  if serde_json::to_vec(&p).map_err(err)?.len() as u64>MAX_MANIFEST{return Err("project_limit".into());}
  persist(&mut s,Some(p.clone()))?;Ok(p)
 }
}
#[tauri::command]
pub async fn project_editor_preview(query:Query,operations:Vec<Operation>,state:tauri::State<'_,Arc<Projects>>)->Result<EditPreview>{let privacy_epoch=crate::privacy::epoch();let privacy_result=(async {let permit=gallery::editor_permit().await?;let p=state.inner().clone();tauri::async_runtime::spawn_blocking(move||{let _permit=permit;let(_,file,_pins)=p.editor_source(&query)?;gallery::render_preview(file,&operations)}).await.map_err(err)?}).await;crate::privacy::finish(privacy_epoch,privacy_result)}
#[tauri::command]
pub async fn project_editor_save(query:Query,expected:Vec<Operation>,operations:Vec<Operation>,state:tauri::State<'_,Arc<Projects>>)->Result<Project>{let privacy_epoch=crate::privacy::epoch();let privacy_result=(async {let permit=gallery::editor_permit().await?;let p=state.inner().clone();tauri::async_runtime::spawn_blocking(move||{let _permit=permit;p.save_edit(&query,&expected,operations)}).await.map_err(err)?}).await;crate::privacy::finish(privacy_epoch,privacy_result)}
#[tauri::command]
pub async fn project_editor_export(query:Query,operations:Vec<Operation>,format:String,quality:u8,state:tauri::State<'_,Arc<Projects>>,core:tauri::State<'_,Arc<Core>>,catalog:tauri::State<'_,Arc<GalleryCatalog>>)->Result<String>{let privacy_epoch=crate::privacy::epoch();let privacy_result=(async {
 let permit=gallery::editor_permit().await?;let p=state.inner().clone();let c=catalog.inner().clone();let root=gallery::root(&core)?;
 tauri::async_runtime::spawn_blocking(move||{let _permit=permit;let(a,file,_pins)=p.editor_source(&query)?;let bytes=gallery::render_bytes(file,&operations,&format,quality)?;c.export_project_edit(&root,&a.name,&bytes,serde_json::json!({"restricted":p.privacy_restricted()?,"projectAsset":a.id,"sourceSha256":a.sha256,"operations":operations}),&format)}).await.map_err(err)?
}).await;crate::privacy::finish(privacy_epoch,privacy_result)}
#[tauri::command]
pub async fn project_add_edit(id:String,root_id:String,targets:Vec<gallery::FileTarget>,operations:Vec<Operation>,state:tauri::State<'_,Arc<Projects>>,core:tauri::State<'_,Arc<Core>>)->Result<Project>{let privacy_epoch=crate::privacy::epoch();let privacy_result=(async {
 if targets.len()!=1{return Err("project_media".into());}let permit=gallery::editor_permit().await?;let p=state.inner().clone();let core=core.inner().clone();
 tauri::async_runtime::spawn_blocking(move||{let _permit=permit;let sources=gallery::project_sources(&core,&root_id,&targets)?;let path=&sources[0].0;gallery::render_preview(gallery::lock_file(path)?,&operations)?;p.add_for(Some(&id),vec![path.to_string_lossy().into()],Some(operations))}).await.map_err(err)?
}).await;crate::privacy::finish(privacy_epoch,privacy_result)}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn edited_project_roundtrips_without_gallery_and_protects_original_and_conflicts(){
  let t=tempfile::tempdir().unwrap();let root=fs::canonicalize(t.path()).unwrap();let source=root.join("source.png");image::RgbaImage::from_pixel(4,3,image::Rgba([40,80,120,77])).save(&source).unwrap();let original=fs::read(&source).unwrap();let p=Projects::new(&root).unwrap();p.new_project(&root,"edits".into(),None,false).unwrap();let project=p.add(vec![source.to_string_lossy().into()]).unwrap();let a=&project.assets[0];let q=Query{id:project.id.clone(),asset_id:a.id.clone(),sha256:a.sha256.clone()};
  let ops=vec![Operation::Rotate{clockwise:true},Operation::Adjust{brightness:10,contrast:0,saturation:0,temperature:0}];p.save_edit(&q,&[],ops.clone()).unwrap();assert!(p.save_edit(&q,&[],vec![]).is_err());let file=root.join("edits.localstudio");p.save(&file).unwrap();assert_eq!(fs::read(&source).unwrap(),original);fs::remove_file(&source).unwrap();
  p.close(true).unwrap();let reopened=p.open(&file,&root,true).unwrap();assert_eq!(reopened.assets[0].edit,ops);let q=Query{id:reopened.id,asset_id:reopened.assets[0].id.clone(),sha256:reopened.assets[0].sha256.clone()};let(_,file,_pins)=p.editor_source(&q).unwrap();let out=gallery::render_bytes(file,&ops,"png",90).unwrap();let i=image::load_from_memory(&out).unwrap().to_rgba8();assert_eq!(i.dimensions(),(3,4));assert_eq!(i.get_pixel(0,0)[3],77);assert_eq!(i.get_pixel(0,0)[0],66);
  assert!(p.save_edit(&q,&ops,vec![Operation::Adjust{brightness:101,contrast:0,saturation:0,temperature:0}]).is_err());assert_eq!(p.snapshot().unwrap().unwrap().assets[0].edit,ops);
 }
}
