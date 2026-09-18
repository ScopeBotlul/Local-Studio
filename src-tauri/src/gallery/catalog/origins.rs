use super::*;
use crate::image_engine::ImageJob;

#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub struct Origin { pub job_id:String,pub request:ImageRequest,pub model_name:String,pub model_sha256:Option<String>,pub runtime:String,pub created_at:String,pub association:String }
#[derive(Clone)]
pub struct BoundOrigin { pub stamp:String,pub info:Origin }
pub(in crate::gallery) fn stamp(meta:&fs::Metadata)->String {format!("{}:{}",meta.len(),meta.last_write_time())}
impl GalleryCatalog {
    pub(in crate::gallery) fn sync_origins(&self,root:&Path,jobs:Vec<ImageJob>)->Result<HashMap<String,BoundOrigin>> {
        let mut db=self.db.lock().map_err(|_|"gallery_storage")?;
        let tx=db.transaction().map_err(|_|"gallery_storage")?;
        for job in jobs {
            if job.status!="completed" || job.discarded {continue;}
            let Some(saved)=job.saved_path.as_ref() else {continue;};
            let exists:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM origins WHERE root=?1 AND job=?2)",params![root_id(root),job.id],|r|r.get(0)).map_err(|_|"gallery_storage")?;
            if exists {continue;}
            let Ok(path)=fs::canonicalize(saved) else {continue;};
            if !path.starts_with(root) || media(&path).map(|m|m.0)!=Some("image") {continue;}
            let Ok(_guards)=directory_guards(path.parent().ok_or("gallery_path")?) else {continue;};
            let Ok(file)=lock_file(&path) else {continue;};
            let Ok(id)=identity(&file) else {continue;};let meta=file.metadata().map_err(|_|"gallery_missing")?;
            if job.saved_binding.as_ref().is_some_and(|binding|*binding!=format!("{}:{}",id,stamp(&meta))){continue;}
            let association=if job.saved_binding.is_some(){"fileIdentity"}else{"savedPath"};
            let info=Origin {job_id:job.id.clone(),model_name:Path::new(&job.request.model_path).file_name().unwrap_or_default().to_string_lossy().into(),request:job.request,model_sha256:job.model_sha256,runtime:job.runtime,created_at:job.created_at,association:association.into()};
            tx.execute("INSERT INTO origins(root,job,file_id,stamp,json) VALUES(?1,?2,?3,?4,?5)",params![root_id(root),job.id,id,stamp(&meta),serde_json::to_string(&info).map_err(|_|"gallery_storage")?]).map_err(|_|"gallery_storage")?;
        }
        tx.commit().map_err(|_|"gallery_storage")?;
        let mut stmt=db.prepare("SELECT file_id,stamp,json FROM origins WHERE root=?1").map_err(|_|"gallery_storage")?;
        let rows=stmt.query_map([root_id(root)],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?))).map_err(|_|"gallery_storage")?;
        let mut result=HashMap::new();
        for row in rows {let(id,stamp,json)=row.map_err(|_|"gallery_storage")?;result.insert(id,BoundOrigin{stamp,info:serde_json::from_str(&json).map_err(|_|"gallery_storage")?});}
        Ok(result)
    }
    pub(super) fn origin_job(&self,root:&Path,id:&str)->Result<Option<String>> {
        self.db.lock().map_err(|_|"gallery_storage")?.query_row("SELECT job FROM origins WHERE root=?1 AND file_id=?2",params![root_id(root),id],|r|r.get(0)).optional().map_err(|_|"gallery_storage".into())
    }
}
