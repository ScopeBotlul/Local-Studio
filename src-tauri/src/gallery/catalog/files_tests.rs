use super::*;
fn setup()->(tempfile::TempDir,PathBuf,Arc<GalleryCatalog>,Arc<ImageEngine>){let t=tempfile::tempdir().unwrap();let r=fs::canonicalize(t.path()).unwrap();let c=GalleryCatalog::new(&r).unwrap();let e=ImageEngine::new(&r,r.join("no-runtime")).unwrap();fs::write(r.join("a.png"),b"fixture a").unwrap();fs::write(r.join("b.png"),b"fixture b").unwrap();fs::create_dir(r.join("dest")).unwrap();(t,r,c,e)}
fn target(root:&Path,path:&str)->FileTarget{let p=root.join(path);let id=path_identity(&p).unwrap();FileTarget{path:path.into(),version:thumbnails::version(root,&p,&fs::metadata(&p).unwrap(),Some(&id)),file_id:id}}
fn request(root:&Path,paths:&[&str],action:FileAction)->FileRequest{FileRequest{root_id:root_id(root),targets:paths.iter().map(|p|target(root,p)).collect(),action}}
fn trash_request(root:&Path,items:&[TrashItem],action:&str)->TrashRequest{TrashRequest{root_id:root_id(root),ids:items.iter().map(|i|i.id.clone()).collect(),action:action.into(),confirmed:true}}
#[test]fn trash_protocol_requires_registered_identity_and_purge_journal_recovers_without_repeating_delete(){
 let(_t,r,c,e)=setup();c.file_action(&r,request(&r,&["a.png"],FileAction::Trash{confirmed:true}),&e).unwrap();let item=c.trash_all(&r).unwrap().remove(0);
 let uri=format!("http://gallery.localhost/{}/{}",root_id(&r),URL_SAFE_NO_PAD.encode(item.stored_path.as_bytes()));let req=||tauri::http::Request::builder().uri(&uri).body(Vec::new()).unwrap();
 assert_eq!(response_catalog(&r,"main",req(),Some(&c)).status(),200);assert_eq!(response_catalog(&r,"hf-website",req(),Some(&c)).status(),403);assert_eq!(response(&r,"main",req()).status(),403);
 let intent=Intent{id:uuid::Uuid::new_v4().to_string(),source:item.stored_path.clone(),destination:None,file_id:item.file_id.clone(),action:"purge".into(),trash:Some(item.clone()),job:None};c.journal(&r,&intent).unwrap();let f=destructive_file(&r.join(&item.stored_path)).unwrap();delete_handle(&f).unwrap();drop(f);c.recover(&r,&e).unwrap();assert!(c.trash_all(&r).unwrap().is_empty());assert_eq!(response_catalog(&r,"main",req(),Some(&c)).status(),403);
 fs::write(r.join(&item.stored_path),b"unregistered replacement").unwrap();assert_eq!(response_catalog(&r,"main",req(),Some(&c)).status(),403);assert!(r.join("b.png").exists());
}
#[test]fn rename_move_trash_restore_keep_bytes_identity_and_annotations(){
 let(_t,r,c,e)=setup();let id=path_identity(&r.join("a.png")).unwrap();c.edit(&r,Edit{root_id:root_id(&r),path:"a.png".into(),file_id:id.clone(),revision:0,favorite:true,tags:vec!["Grün".into()]}).unwrap();
 let report=c.file_action(&r,request(&r,&["a.png"],FileAction::Rename{name:"neu.png".into()}),&e).unwrap();assert!(report.errors.is_empty(),"{:?}",report.errors);assert_eq!(report.completed.len(),1);
 assert!(!r.join("a.png").exists());assert_eq!(path_identity(&r.join("neu.png")).unwrap(),id);
 assert!(c.file_action(&r,request(&r,&["neu.png","b.png"],FileAction::Move{folder:"dest".into()}),&e).unwrap().errors.is_empty());
 assert_eq!(c.file_action(&r,request(&r,&["dest/neu.png","dest/b.png"],FileAction::Trash{confirmed:true}),&e).unwrap().completed.len(),2);
 let items=c.trash_all(&r).unwrap();assert_eq!(items.len(),2);assert!(!r.join("dest/neu.png").exists());for i in &items {assert!(allows_trash(&c,&r,&i.stored_path));assert!(resolve(&r,&i.stored_path).is_err());}
 assert_eq!(c.trash_action(&r,trash_request(&r,&items,"restore"),&e).unwrap().completed.len(),2);assert!(c.trash_all(&r).unwrap().is_empty());
 assert_eq!(fs::read(r.join("dest/neu.png")).unwrap(),b"fixture a");assert_eq!(path_identity(&r.join("dest/neu.png")).unwrap(),id);assert!(c.snapshot(&r).unwrap()[&id].favorite);
}
#[test]fn collisions_stale_versions_and_unconfirmed_deletion_change_nothing(){
 let(_t,r,c,e)=setup();fs::write(r.join("dest/b.png"),b"existing").unwrap();
 assert!(matches!(c.file_action(&r,request(&r,&["a.png","b.png"],FileAction::Move{folder:"dest".into()}),&e),Err(s) if s=="gallery_collision"));assert!(r.join("a.png").exists());
 let mut req=request(&r,&["a.png","b.png"],FileAction::Trash{confirmed:true});req.targets[1].version="old".into();assert!(c.file_action(&r,req,&e).is_err());assert!(r.join("a.png").exists());
 assert!(c.file_action(&r,request(&r,&["a.png"],FileAction::Trash{confirmed:false}),&e).is_err());
 for name in ["b.png","../x.png","a.jpg","CON.png"]{assert!(c.file_action(&r,request(&r,&["a.png"],FileAction::Rename{name:name.into()}),&e).is_err());}
 let held=lock_file(&r.join("b.png")).unwrap();assert!(c.file_action(&r,request(&r,&["a.png","b.png"],FileAction::Trash{confirmed:true}),&e).is_err());drop(held);assert!(r.join("a.png").exists());assert!(c.trash_all(&r).unwrap().is_empty());
}
#[test]fn restore_refuses_replacement_and_purge_only_deletes_selected_registered_file(){
 let(_t,r,c,e)=setup();c.file_action(&r,request(&r,&["a.png","b.png"],FileAction::Trash{confirmed:true}),&e).unwrap();let items=c.trash_all(&r).unwrap();fs::write(r.join("a.png"),b"replacement").unwrap();
 assert!(c.trash_action(&r,trash_request(&r,&items,"restore"),&e).is_err());assert!(!r.join("b.png").exists());
 let mut req=trash_request(&r,&items[..1],"purge");req.confirmed=false;assert!(c.trash_action(&r,req,&e).is_err());
 assert_eq!(c.trash_action(&r,trash_request(&r,&items[..1],"purge"),&e).unwrap().completed.len(),1);assert_eq!(c.trash_all(&r).unwrap().len(),1);assert!(!r.join(&items[0].stored_path).exists());assert!(r.join(&items[1].stored_path).exists());assert_eq!(fs::read(r.join("a.png")).unwrap(),b"replacement");
}
#[test]fn journal_recovers_before_and_after_move_and_after_database_failure(){
 let(_t,r,c,e)=setup();let t=target(&r,"a.png");let intent=Intent{id:uuid::Uuid::new_v4().to_string(),source:t.path,destination:Some("dest/a.png".into()),file_id:t.file_id,action:"move".into(),trash:None,job:None};
 c.journal(&r,&intent).unwrap();c.recover(&r,&e).unwrap();assert!(r.join("a.png").exists());
 c.journal(&r,&intent).unwrap();let f=destructive_file(&r.join("a.png")).unwrap();rename_handle(&f,&r.join("dest/a.png")).unwrap();drop(f);c.recover(&r,&e).unwrap();assert!(r.join("dest/a.png").exists());
 c.db.lock().unwrap().execute_batch("CREATE TRIGGER fail_trash BEFORE INSERT ON trash BEGIN SELECT RAISE(ABORT,'test failure'); END;").unwrap();
 let report=c.file_action(&r,request(&r,&["b.png"],FileAction::Trash{confirmed:true}),&e).unwrap();assert_eq!(report.errors.len(),1);assert!(!r.join("b.png").exists());
 c.db.lock().unwrap().execute_batch("DROP TRIGGER fail_trash;").unwrap();c.recover(&r,&e).unwrap();assert_eq!(c.trash_all(&r).unwrap().len(),1);assert_eq!(c.db.lock().unwrap().query_row("SELECT count(*) FROM file_journal",[],|r|r.get::<_,i64>(0)).unwrap(),0);
}
