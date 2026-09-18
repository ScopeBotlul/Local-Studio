use super::*;

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Target { path: String, file_id: String, revision: i64, version: String }
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum Action { Favorite { value: bool }, AddTag { tag: String }, RemoveTag { tag: String } }
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Batch { root_id: String, targets: Vec<Target>, action: Action }

impl GalleryCatalog {
    fn batch(&self,root: &Path,batch: Batch) -> Result<usize> {
        if batch.root_id!=root_id(root) { return Err("gallery_changed".into()); }
        if batch.targets.is_empty() || batch.targets.len()>50 { return Err("gallery_batch_limit".into()); }
        let tag=match &batch.action {
            Action::Favorite {..}=>None,
            Action::AddTag {tag}|Action::RemoveTag {tag}=>Some(normalize(vec![tag.clone()])?.into_iter().next().ok_or("gallery_tags")?),
        };
        let mut identities=HashSet::new(); let mut pins=Vec::new();
        for target in &batch.targets {
            if !identities.insert(&target.file_id) { return Err("gallery_batch_duplicate".into()); }
            let path=resolve(root,&target.path)?;
            if media(&path).is_none() { return Err("gallery_format".into()); }
            let guards=directory_guards(path.parent().ok_or("gallery_path")?)?;
            let file=lock_file(&path)?; let id=identity(&file)?;
            let meta=file.metadata().map_err(|_| "gallery_missing")?;
            if id!=target.file_id || thumbnails::version(root,&path,&meta,Some(&id))!=target.version { return Err("gallery_changed".into()); }
            pins.push((guards,file));
        }
        // Pin every selected file until the one transaction commits. A conflict
        // or failure on even the last row rolls back all preceding row changes.
        let mut db=self.db.lock().map_err(|_| "gallery_storage")?;
        let tx=db.transaction().map_err(|_| "gallery_storage")?;
        for target in &batch.targets {
            let previous: Option<(bool,String,i64)>=tx.query_row("SELECT favorite,tags,revision FROM annotations WHERE root=?1 AND file_id=?2",params![batch.root_id,target.file_id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(|_| "gallery_storage")?;
            let mut saved=if let Some((favorite,tags,revision))=previous {
                let tags: Vec<String>=serde_json::from_str(&tags).map_err(|_| "gallery_storage")?;
                if revision<1 || normalize(tags.clone())?!=tags { return Err("gallery_storage".into()); }
                Annotation { favorite,tags,revision }
            } else { Annotation::default() };
            if target.revision!=saved.revision { return Err("gallery_conflict".into()); }
            match &batch.action {
                Action::Favorite {value}=>saved.favorite=*value,
                Action::AddTag {..}=>{saved.tags.push(tag.clone().unwrap());saved.tags=normalize(saved.tags)?;},
                Action::RemoveTag {..}=>{let tag=tag.as_ref().unwrap().to_lowercase();saved.tags.retain(|t|t.to_lowercase()!=tag);},
            }
            saved.revision=saved.revision.checked_add(1).ok_or("gallery_storage")?;
            tx.execute("INSERT INTO annotations VALUES(?1,?2,?3,?4,?5) ON CONFLICT(root,file_id) DO UPDATE SET favorite=excluded.favorite,tags=excluded.tags,revision=excluded.revision",params![batch.root_id,target.file_id,saved.favorite,serde_json::to_string(&saved.tags).map_err(|_| "gallery_storage")?,saved.revision]).map_err(|_| "gallery_storage")?;
        }
        tx.commit().map_err(|_| "gallery_storage")?;
        drop(pins);
        Ok(batch.targets.len())
    }
}
#[tauri::command]
pub async fn gallery_annotate_batch(batch: Batch,core: State<'_,Arc<Core>>,catalog: State<'_,Arc<GalleryCatalog>>) -> Result<usize> {let privacy_epoch=crate::privacy::epoch();let privacy_result=(async {
    let root=root(&core)?;let catalog=catalog.inner().clone();
    tauri::async_runtime::spawn_blocking(move || catalog.batch(&root,batch)).await.map_err(|_| "gallery_storage")?
}).await;crate::privacy::finish(privacy_epoch,privacy_result)}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> (tempfile::TempDir,PathBuf,Arc<GalleryCatalog>,Vec<Target>) {
        let temp=tempfile::tempdir().unwrap();let root=fs::canonicalize(temp.path()).unwrap();let catalog=GalleryCatalog::new(&root).unwrap();let mut targets=Vec::new();
        for name in ["a.png","b.webm","c.wav"] {
            let path=root.join(name);fs::write(&path,name.as_bytes()).unwrap();let file_id=path_identity(&path).unwrap();let version=thumbnails::version(&root,&path,&fs::metadata(&path).unwrap(),Some(&file_id));
            targets.push(Target {path:name.into(),file_id,revision:0,version});
        } (temp,root,catalog,targets)
    }
    fn request(root: &Path,targets: &[Target],action: Action) -> Batch { Batch {root_id:root_id(root),targets:targets.to_vec(),action} }
    fn advance(targets: &mut [Target]) {for target in targets {target.revision+=1;}}
    #[test] fn batch_preserves_individual_tags_and_media_across_reopen() {
        let (_temp,root,catalog,mut targets)=setup();
        catalog.edit(&root,Edit {root_id:root_id(&root),path:targets[0].path.clone(),file_id:targets[0].file_id.clone(),revision:0,favorite:false,tags:vec!["individuell".into()]}).unwrap();targets[0].revision=1;
        assert_eq!(catalog.batch(&root,request(&root,&targets,Action::AddTag {tag:" Urlaub  2026 ".into()})).unwrap(),3);advance(&mut targets);
        catalog.batch(&root,request(&root,&targets,Action::AddTag {tag:"urlaub 2026".into()})).unwrap();advance(&mut targets);
        catalog.batch(&root,request(&root,&targets,Action::Favorite {value:true})).unwrap();advance(&mut targets);
        catalog.batch(&root,request(&root,&targets,Action::RemoveTag {tag:"URLAUB 2026".into()})).unwrap();advance(&mut targets);
        drop(catalog);let catalog=GalleryCatalog::new(&root).unwrap();let stored=catalog.snapshot(&root).unwrap();
        for (i,target) in targets.iter().enumerate() {let value=&stored[&target.file_id];assert!(value.favorite);assert_eq!(value.revision,target.revision);assert_eq!(value.tags,if i==0 {vec!["individuell"]} else {vec![]});assert_eq!(fs::read(root.join(&target.path)).unwrap(),target.path.as_bytes());}
    }
    #[test] fn late_conflicts_and_failed_writes_roll_back_the_entire_selection() {
        let (_temp,root,catalog,mut targets)=setup();targets[2].revision=9;
        assert_eq!(catalog.batch(&root,request(&root,&targets,Action::Favorite {value:true})).unwrap_err(),"gallery_conflict");assert!(catalog.snapshot(&root).unwrap().is_empty());targets[2].revision=0;
        let db=catalog.db.lock().unwrap();db.execute_batch(&format!("CREATE TRIGGER fail_last BEFORE INSERT ON annotations WHEN NEW.file_id='{}' BEGIN SELECT RAISE(ABORT,'storage fixture'); END;",targets[2].file_id)).unwrap();drop(db);
        assert_eq!(catalog.batch(&root,request(&root,&targets,Action::Favorite {value:true})).unwrap_err(),"gallery_storage");assert!(catalog.snapshot(&root).unwrap().is_empty());
    }
    #[test] fn changed_missing_and_replaced_files_never_partially_apply() {
        let (_temp,root,catalog,targets)=setup();let last=root.join(&targets[2].path);fs::write(&last,b"changed content").unwrap();
        assert_eq!(catalog.batch(&root,request(&root,&targets,Action::Favorite {value:true})).unwrap_err(),"gallery_changed");assert!(catalog.snapshot(&root).unwrap().is_empty());
        fs::rename(&last,root.join("old.wav")).unwrap();fs::write(&last,b"c.wav").unwrap();assert!(catalog.batch(&root,request(&root,&targets,Action::Favorite {value:true})).is_err());
        fs::remove_file(last).unwrap();assert!(catalog.batch(&root,request(&root,&targets,Action::Favorite {value:true})).is_err());assert!(catalog.snapshot(&root).unwrap().is_empty());
    }
    #[test] fn tag_limit_on_one_file_rolls_back_others_and_input_is_bounded() {
        let (_temp,root,catalog,mut targets)=setup();let last=targets.last_mut().unwrap();catalog.edit(&root,Edit {root_id:root_id(&root),path:last.path.clone(),file_id:last.file_id.clone(),revision:0,favorite:false,tags:(0..32).map(|n|format!("tag{n}")).collect()}).unwrap();last.revision=1;
        let before=catalog.snapshot(&root).unwrap();assert_eq!(catalog.batch(&root,request(&root,&targets,Action::AddTag {tag:"extra".into()})).unwrap_err(),"gallery_tags");assert_eq!(catalog.snapshot(&root).unwrap(),before);
        assert_eq!(catalog.batch(&root,request(&root,&[],Action::Favorite {value:true})).unwrap_err(),"gallery_batch_limit");
        let many=vec![targets[0].clone();51];assert_eq!(catalog.batch(&root,request(&root,&many,Action::Favorite {value:true})).unwrap_err(),"gallery_batch_limit");
        assert_eq!(catalog.batch(&root,request(&root,&[targets[0].clone(),targets[0].clone()],Action::Favorite {value:true})).unwrap_err(),"gallery_batch_duplicate");
        let mut bad=request(&root,&targets,Action::Favorite {value:true});bad.root_id="wrong".into();assert_eq!(catalog.batch(&root,bad).unwrap_err(),"gallery_changed");
        let mut bad=request(&root,&targets,Action::Favorite {value:true});bad.targets[0].path="../escape.png".into();assert_eq!(catalog.batch(&root,bad).unwrap_err(),"gallery_path");
        assert_eq!(catalog.snapshot(&root).unwrap(),before);
    }
}
