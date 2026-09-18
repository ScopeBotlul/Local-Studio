use super::*;
fn query() -> Query { Query { sort:Sort::default(),folder: "".into(), search: "".into(), kind: "all".into(), recursive: true, offset: 0, favorites_only: false, tag: String::new() } }
fn canon(path: &Path) -> PathBuf { fs::canonicalize(path).unwrap() }
#[test]fn sorting_is_global_before_pagination_deterministic_and_filters_still_apply(){
 let t=tempfile::tempdir().unwrap();let r=canon(t.path());
 for i in 0..61 {let p=r.join(format!("image-{i:02}.png"));fs::write(&p,vec![1;i+1]).unwrap();let file=OpenOptions::new().write(true).open(p).unwrap();file.set_times(fs::FileTimes::new().set_modified(UNIX_EPOCH+std::time::Duration::from_secs(1_700_000_000+i as u64))).unwrap();}
 for sort in [Sort::NameAsc,Sort::NameDesc,Sort::SizeAsc,Sort::SizeDesc,Sort::ModifiedAsc,Sort::ModifiedDesc] {
   let mut q=query();q.sort=sort;let first=list(&r,q).unwrap();let mut q=query();q.sort=sort;q.offset=50;let second=list(&r,q).unwrap();assert_eq!(first.entries.len(),50);assert_eq!(second.entries.len(),11);
   let names:Vec<_>=first.entries.into_iter().chain(second.entries).map(|e|e.name).collect();let mut expected:Vec<_>=(0..61).map(|i|format!("image-{i:02}.png")).collect();if matches!(sort,Sort::NameDesc|Sort::SizeDesc|Sort::ModifiedDesc){expected.reverse();}assert_eq!(names,expected);
 }
 let mut q=query();q.search="image-0".into();q.sort=Sort::SizeDesc;let found=list(&r,q).unwrap();assert_eq!(found.total,10);assert_eq!(found.entries[0].name,"image-09.png");
 let invalid=serde_json::json!({"folder":"","search":"","kind":"all","recursive":true,"offset":0,"sort":"execute"});assert!(serde_json::from_value::<Query>(invalid).is_err());
}
#[test]fn model_prompt_search_and_origin_binding_survive_rename_but_not_replacement(){
 let temp=tempfile::tempdir().unwrap();let root=canon(temp.path());let cat=GalleryCatalog::new(&root).unwrap();fs::write(root.join("original.png"),b"fixture output").unwrap();
 let mut job:crate::image_engine::ImageJob=serde_json::from_value(serde_json::json!({"id":"test-job","request":{"modelPath":"D:\\models\\Alpine.safetensors","prompt":"A quiet lake","negativePrompt":"watermark","width":512,"height":512,"steps":20,"guidance":5,"seed":42,"sampler":"euler"},"status":"completed","phase":"completed","step":20,"hashedBytes":0,"modelBytes":0,"modelSha256":null,"runtime":"test fixture","device":"fixture","createdAt":"2026-09-18T00:00:00Z","elapsedMs":0,"error":null,"output":null,"savedPath":root.join("original.png").to_str().unwrap(),"logTail":""})).unwrap();
 job.saved_binding=Some("wrong-file".into());assert!(cat.sync_origins(&root,vec![job.clone()]).unwrap().is_empty());job.saved_binding=Some(saved_binding(&root.join("original.png")).unwrap());let origins=cat.sync_origins(&root,vec![job]).unwrap();assert_eq!(origins.len(),1);
 fs::rename(root.join("original.png"),root.join("renamed.png")).unwrap();for search in ["ALPINE","quiet LAKE","watermark"] {let mut q=query();q.search=search.into();let found=list_full(&root,q,&HashMap::new(),&origins).unwrap();assert_eq!(found.total,1);assert_eq!(found.entries[0].path,"renamed.png");assert_eq!(found.entries[0].origin.as_ref().unwrap().association,"fileIdentity");}
 fs::write(root.join("original.png"),b"new unrelated file").unwrap();let listing=list_full(&root,query(),&HashMap::new(),&origins).unwrap();assert!(listing.entries.iter().find(|e|e.path=="original.png").unwrap().origin.is_none());
 fs::write(root.join("renamed.png"),b"edited different bytes").unwrap();let mut q=query();q.search="quiet lake".into();assert_eq!(list_full(&root,q,&HashMap::new(),&origins).unwrap().total,0);
 fs::create_dir(root.join(TRASH)).unwrap();fs::write(root.join(TRASH).join("hidden.png"),b"trash").unwrap();assert_eq!(list(&root,query()).unwrap().total,2);assert!(list(&root,query()).unwrap().folders.is_empty());
}
#[test]
fn actual_tree_discovers_all_media_filters_pages_and_external_changes() {
    let temp = tempfile::tempdir().unwrap(); let root = canon(temp.path());
    fs::create_dir(root.join("Nested")).unwrap();
    for name in ["one.png", "two.mp4", "three.wav", "four.mp3", "metadata.json", "active.svg", "bad.exe"] { fs::write(root.join("Nested").join(name),b"fixture").unwrap(); }
    let first = list(&root,query()).unwrap(); assert_eq!(first.total,4); assert_eq!(first.folders.len(),1);
    let mut q = query(); q.kind="audio".into(); assert_eq!(list(&root,q).unwrap().total,2);
    let mut q=query(); q.recursive=false; assert_eq!(list(&root,q).unwrap().total,0);
    fs::write(root.join("new.webp"),b"new external file").unwrap(); assert_eq!(list(&root,query()).unwrap().total,5);
    for i in 0..60 { fs::write(root.join(format!("page-{i}.jpg")),b"fixture").unwrap(); }
    let first=list(&root,query()).unwrap(); assert_eq!(first.entries.len(),50);
    let mut q=query(); q.offset=50; let next=list(&root,q).unwrap(); assert_eq!(next.entries.len(),15);
    assert!(next.entries.iter().all(|n| first.entries.iter().all(|a| a.path != n.path)));
    let mut q=query(); q.search="PAGE-59".into(); assert_eq!(list(&root,q).unwrap().total,1);
}
#[test]
fn paths_formats_and_directory_names_cannot_escape_or_execute() {
    let temp=tempfile::tempdir().unwrap(); let root=canon(temp.path());
    for path in ["../outside", "C:/Windows", "a\\b", "a/../x", "name:stream", "/absolute", "foo.", "foo "] { assert!(resolve(&root,path).is_err(),"{path}"); }
    for name in ["..", "CON", "con.txt", "NUL", "bad/name", "bad:stream", "trailing.", "LPT9.mp4", "a\n"] { assert!(!valid_name(name),"{name}"); }
    assert!(media(Path::new("page.html")).is_none()); assert!(media(Path::new("active.svg")).is_none()); assert!(valid_name("Urlaub 2026"));
    assert_eq!(resolve(&root,"").unwrap(),root);
}
#[test]
fn verified_import_never_overwrites_and_preserves_originals() {
    let temp=tempfile::tempdir().unwrap(); let source=temp.path().join("source"); let target=temp.path().join("target"); fs::create_dir(&source).unwrap(); fs::create_dir(&target).unwrap();
    let source=canon(&source); let target=canon(&target); let file=source.join("tone.wav"); fs::write(&file,b"immutable audio fixture").unwrap(); fs::write(target.join("tone.wav"),b"existing").unwrap();
    let _guards=directory_guards(&target).unwrap(); let copied=copy_one(&target,&target,&file).unwrap();
    assert_ne!(copied,"tone.wav"); assert_eq!(fs::read(&file).unwrap(),fs::read(target.join(copied)).unwrap()); assert_eq!(fs::read(target.join("tone.wav")).unwrap(),b"existing");
    assert!(copy_one(&target,&target,&source.join("absent.mp4")).is_err()); assert!(copy_one(&target,&target,Path::new("relative.wav")).is_err());
    assert!(fs::read_dir(target).unwrap().all(|e| !e.unwrap().file_name().to_string_lossy().ends_with(".tmp")));
}
#[test]
fn range_reads_are_bounded_exact_and_reject_invalid_ranges() {
    assert_eq!(range(Some("bytes=4-7"),20,false).unwrap(),(4,7,true));
    assert_eq!(range(Some("bytes=-5"),20,false).unwrap(),(15,19,true));
    assert_eq!(range(Some("bytes=0-"),CHUNK*3,false).unwrap(),(0,CHUNK-1,true));
    for header in ["bytes=20-", "bytes=3-2", "bytes=0-1,3-4", "bytes=-0", "bytes=bad", "bytes=18446744073709551615-"] { assert!(range(Some(header),20,false).is_err()); }
    assert!(range(None,MAX_IMAGE+1,true).is_err()); assert!(range(None,0,false).is_err());
}
#[test]
fn media_protocol_restricts_webview_root_and_path_and_returns_real_ranges() {
    let temp=tempfile::tempdir().unwrap(); let root=canon(temp.path()); fs::write(root.join("media.wav"),b"0123456789").unwrap();
    let uri=format!("http://gallery.localhost/{}/{}",root_id(&root),URL_SAFE_NO_PAD.encode("media.wav"));
    let request=|| tauri::http::Request::builder().uri(&uri).header("Range","bytes=2-5").body(vec![]).unwrap();
    assert_eq!(response(&root,"hf-website",request()).status(),403);
    let result=response(&root,"main",request()); assert_eq!(result.status(),206); assert_eq!(result.body(),b"2345"); assert_eq!(result.headers()["Content-Range"],"bytes 2-5/10");
    let escaped=format!("http://gallery.localhost/{}/{}",root_id(&root),URL_SAFE_NO_PAD.encode("../outside.wav"));
    assert_eq!(response(&root,"main",tauri::http::Request::builder().uri(escaped).body(vec![]).unwrap()).status(),403);
    let other=tempfile::tempdir().unwrap(); assert_eq!(response(&canon(other.path()),"main",request()).status(),403);
}

#[test]
fn favorite_and_tag_filters_apply_before_pagination_and_never_follow_a_replaced_path() {
    let temp=tempfile::tempdir().unwrap(); let root=canon(temp.path());
    for i in 0..61 { fs::write(root.join(format!("media-{i}.png")),b"fixture").unwrap(); }
    let path=root.join("media-0.png");let id=catalog::path_identity(&path).unwrap();
    let annotations=HashMap::from([(id,Annotation { favorite:true,tags:vec!["Grüner Wald".into()],revision:1 })]);
    let mut q=query();q.favorites_only=true;let found=list_with(&root,q,&annotations).unwrap();assert_eq!(found.total,1);assert_eq!(found.entries[0].path,"media-0.png");
    let mut q=query();q.search="GRÜNER".into();assert_eq!(list_with(&root,q,&annotations).unwrap().total,1);
    let mut q=query();q.tag="grüner wald".into();assert_eq!(list_with(&root,q,&annotations).unwrap().total,1);
    fs::rename(&path,root.join("old.png")).unwrap();fs::write(&path,b"replacement").unwrap();
    let mut q=query();q.favorites_only=true;let found=list_with(&root,q,&annotations).unwrap();assert_eq!(found.entries[0].path,"old.png");assert_eq!(found.total,1);
    fs::remove_file(root.join("old.png")).unwrap();assert!(list_with(&root,query(),&annotations).unwrap().tags.is_empty());
}
