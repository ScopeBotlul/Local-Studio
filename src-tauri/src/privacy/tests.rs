use super::*;
fn setup()->(tempfile::TempDir,Arc<Privacy>){let dir=tempfile::tempdir().unwrap();let p=Privacy::new(dir.path()).unwrap();(dir,p)}
#[test]fn credentials_are_salted_rate_limited_and_restart_locked(){
 let(t,p)=setup();assert!(p.status().unwrap().locked);assert!(!p.status().unwrap().enabled);
 assert_eq!(p.setup("pin".into(),Zeroizing::new("123456".into()),false).unwrap_err(),"privacy_age");
 assert!(p.setup("pin".into(),Zeroizing::new("123".into()),true).is_err());
 p.setup("pin".into(),Zeroizing::new("123456".into()),true).unwrap();assert!(!p.status().unwrap().locked);
 let raw:String=p.0.lock().unwrap().db.query_row("SELECT json FROM privacy_config",[],|r|r.get(0)).unwrap();assert!(!raw.contains("123456"));
 p.lock(false).unwrap();assert_eq!(p.unlock(Zeroizing::new("000000".into())).unwrap_err(),"privacy_wrong");assert_eq!(p.unlock(Zeroizing::new("123456".into())).unwrap_err(),"privacy_retry");
 drop(p);let p=Privacy::new(t.path()).unwrap();assert!(p.status().unwrap().locked);assert_eq!(p.unlock(Zeroizing::new("123456".into())).unwrap_err(),"privacy_retry");
}
#[test]fn explicit_remember_password_change_and_manual_lock_persist(){
 let(t,p)=setup();p.setup("password".into(),Zeroizing::new("a long password".into()),true).unwrap();p.options(Zeroizing::new("a long password".into()),false,Some(Zeroizing::new("654321".into())),Some("pin".into())).unwrap();drop(p);
 let p=Privacy::new(t.path()).unwrap();assert!(!p.status().unwrap().locked);p.lock(false).unwrap();drop(p);let p=Privacy::new(t.path()).unwrap();assert!(p.status().unwrap().locked);p.unlock(Zeroizing::new("654321".into())).unwrap();p.lock(true).unwrap();assert!(!p.status().unwrap().enabled);
}
#[test]fn media_identity_survives_rename_but_does_not_mark_replacement_or_manual_media(){
 let(t,p)=setup();let a=t.path().join("a.png");std::fs::write(&a,b"original").unwrap();p.mark_file(&a).unwrap();let b=t.path().join("renamed.png");std::fs::rename(&a,&b).unwrap();assert!(p.media(&b));std::fs::write(&a,b"unrelated manual import").unwrap();assert!(!p.media(&a));
 let work=t.path().join("job");std::fs::create_dir(&work).unwrap();p.mark_path(&work).unwrap();std::fs::write(work.join("private.png"),b"private").unwrap();std::fs::create_dir(work.join("nested")).unwrap();assert!(p.media(&work.join("nested/../private.png")));assert!(p.media(&work.join("prompt.txt")));assert!(!p.media(&t.path().join("job-other/prompt.txt")));
}
