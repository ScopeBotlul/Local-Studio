use super::*;
use catalog::files::FileTarget;

#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct CompareRequest { root_id:String, targets:Vec<FileTarget> }

fn prepare(root:&Path,request:CompareRequest,origins:&HashMap<String,BoundOrigin>)->Result<Vec<Detail>> {
    if request.root_id!=root_id(root){return Err("gallery_changed".into());}
    if request.targets.len()!=2 || request.targets[0].file_id==request.targets[1].file_id {return Err("gallery_compare_selection".into());}
    let mut details=Vec::with_capacity(2);
    for target in request.targets {
        let path=resolve(root,&target.path)?;
        let _guards=directory_guards(path.parent().ok_or("gallery_path")?)?;
        let file=lock_file(&path)?;let meta=file.metadata().map_err(|_|"gallery_missing")?;
        if catalog::identity(&file)?!=target.file_id || thumbnails::version(root,&path,&meta,Some(&target.file_id))!=target.version {return Err("gallery_changed".into());}
        if media(&path).map(|m|m.0)!=Some("image") {return Err("gallery_compare_selection".into());}
        if meta.len()>MAX_IMAGE {return Err("gallery_compare_limit".into());}
        let mut reader=image::ImageReader::new(std::io::BufReader::new(file)).with_guessed_format().map_err(|_|"gallery_compare_format")?;
        let mut limits=image::Limits::default();limits.max_alloc=Some(128*1024*1024);limits.max_image_width=Some(16384);limits.max_image_height=Some(16384);reader.limits(limits);
        let (w,h)=reader.into_dimensions().map_err(|_|"gallery_compare_format")?;
        if w==0||h==0||w>16384||h>16384||u64::from(w)*u64::from(h)>32_000_000 {return Err("gallery_compare_limit".into());}
        let origin=origins.get(&target.file_id).filter(|o|o.stamp==stamp(&meta)).map(|o|o.info.clone());
        details.push(Detail {url:format!("http://gallery.localhost/{}/{}?v={}",root_id(root),URL_SAFE_NO_PAD.encode(target.path.as_bytes()),target.version),kind:"image".into(),dimensions:Some((w,h)),request:origin.as_ref().map(|o|o.request.clone()),job_id:origin.as_ref().map(|o|o.job_id.clone()),origin});
    }
    Ok(details)
}

#[tauri::command]
pub async fn gallery_compare(request:CompareRequest,core:State<'_,Arc<Core>>,catalog:State<'_,Arc<GalleryCatalog>>,images:State<'_,Arc<ImageEngine>>)->Result<Vec<Detail>>{
    let root=root(&core)?;let cat=catalog.inner().clone();let images=images.inner().clone();
    tauri::async_runtime::spawn_blocking(move||{cat.recover(&root,&images)?;let origins=cat.sync_origins(&root,images.list()?)?;prepare(&root,request,&origins)}).await.map_err(|_|"gallery_storage")?
}

#[cfg(test)]
mod tests {
    use super::*;
    fn png(path:&Path){image::RgbaImage::from_pixel(12,8,image::Rgba([12,56,90,128])).save(path).unwrap();}
    fn target(root:&Path,name:&str)->FileTarget{let path=root.join(name);let file_id=catalog::path_identity(&path).unwrap();FileTarget{path:name.into(),version:thumbnails::version(root,&path,&fs::metadata(&path).unwrap(),Some(&file_id)),file_id}}
    #[test]fn comparison_checks_exact_images_and_versioned_protocol_refuses_replacement(){
        let t=tempfile::tempdir().unwrap();let r=fs::canonicalize(t.path()).unwrap();png(&r.join("a.png"));png(&r.join("b.png"));
        let before=fs::read(r.join("a.png")).unwrap();let req=||CompareRequest{root_id:root_id(&r),targets:vec![target(&r,"a.png"),target(&r,"b.png")]};
        let result=prepare(&r,req(),&HashMap::new()).unwrap();assert_eq!(result[0].dimensions,Some((12,8)));assert!(result[0].origin.is_none());
        let get=||tauri::http::Request::builder().uri(&result[0].url).body(Vec::new()).unwrap();assert_eq!(response(&r,"main",get()).status(),200);assert_eq!(response(&r,"hf-website",get()).status(),403);
        let mut stale=req();stale.targets[1].version="old".into();assert!(matches!(prepare(&r,stale,&HashMap::new()),Err(s) if s=="gallery_changed"));
        let mut duplicate=req();duplicate.targets[1]=duplicate.targets[0].clone();assert!(prepare(&r,duplicate,&HashMap::new()).is_err());
        fs::rename(r.join("a.png"),r.join("old.png")).unwrap();png(&r.join("a.png"));assert_eq!(response(&r,"main",get()).status(),409);assert_eq!(fs::read(r.join("old.png")).unwrap(),before);
    }
    #[test]fn comparison_rejects_non_images_damaged_sources_traversal_and_wrong_root(){
        let t=tempfile::tempdir().unwrap();let r=fs::canonicalize(t.path()).unwrap();png(&r.join("a.png"));fs::write(r.join("b.wav"),b"audio fixture").unwrap();
        let req=||CompareRequest{root_id:root_id(&r),targets:vec![target(&r,"a.png"),target(&r,"b.wav")]};assert!(matches!(prepare(&r,req(),&HashMap::new()),Err(s) if s=="gallery_compare_selection"));
        let mut bad=req();bad.targets[1].path="../a.png".into();assert!(prepare(&r,bad,&HashMap::new()).is_err());let mut bad=req();bad.root_id="other".into();assert!(prepare(&r,bad,&HashMap::new()).is_err());
        fs::write(r.join("broken.png"),b"not a picture").unwrap();let mut bad=req();bad.targets[1]=target(&r,"broken.png");assert!(matches!(prepare(&r,bad,&HashMap::new()),Err(s) if s=="gallery_compare_format"));
    }
}
