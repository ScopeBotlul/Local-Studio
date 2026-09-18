use super::*;
#[test]
fn creative_roundtrip_original_protection_and_revisions() {
 let tmp=tempfile::tempdir().unwrap();let projects=Projects::new(tmp.path()).unwrap();
 projects.new_project(tmp.path(),"creative".into(),None,false).unwrap();
 let source=tmp.path().join("source.png");image::RgbaImage::from_pixel(4,4,image::Rgba([200,100,50,255])).save(&source).unwrap();
 let p=projects.add(vec![source.to_string_lossy().into()]).unwrap();let a=&p.assets[0];
 let l=Layer{id:uuid(),name:"layer".into(),asset_id:a.id.clone(),x:0.,y:0.,width:4.,height:4.,rotation:0.,opacity:0.5,blend:Blend::Normal,visible:true,locked:false,group:None,operations:vec![],mask:None};
 let mut doc=Creative{revision:0,image:Some(Canvas{width:4,height:4,background:None,layers:vec![l.clone()]}),video:None};
 let rendered=canvas::render(&p,doc.image.as_ref().unwrap(),4,4).unwrap();assert_eq!(rendered.get_pixel(2,2).0,[200,100,50,128]);
 let saved=projects.store_creative(&p.id,0,doc.clone()).unwrap();assert_eq!(saved.creative.as_ref().unwrap().revision,1);
 assert!(projects.store_creative(&p.id,0,doc.clone()).is_err());assert!(projects.remove(&a.id).is_err());
 let target=tmp.path().join("both.localstudio");projects.save(&target).unwrap();projects.close(false).unwrap();fs::remove_file(&source).unwrap();
 let opened=projects.open(&target,tmp.path(),false).unwrap();assert_eq!(serde_json::to_value(&opened.creative).unwrap(),serde_json::to_value(&saved.creative).unwrap());
 assert_eq!(canvas::render(&opened,doc.image.as_ref().unwrap(),4,4).unwrap(),rendered);
 doc.image.as_mut().unwrap().layers.push(l);assert!(validate(&doc,&p.assets).is_err());
 doc.image.as_mut().unwrap().layers.pop();doc.image.as_mut().unwrap().layers[0].opacity=f64::NAN;assert!(validate(&doc,&p.assets).is_err());
 let mut bad=serde_json::to_value(&saved.creative).unwrap();bad["image"]["layers"][0]["command"]=serde_json::json!("execute");assert!(serde_json::from_value::<Creative>(bad).is_err());
}
#[test]
fn timeline_rejects_invalid_tracks_times_and_script_fields() {
 let mut t=Timeline{width:1280,height:720,fps:30,background:"#000000".into(),tracks:vec![Track{id:uuid(),name:"Video".into(),audio:false,muted:false,locked:false}],clips:vec![],captions:vec![Caption{id:uuid(),start:0.,end:2.,text:"A: ' % [x] ü".into(),size:30,color:"#ffffff".into(),y:0.9}]};
 let check=|t:Timeline|validate(&Creative{revision:0,image:None,video:Some(t)},&[]);
 assert!(check(t.clone()).is_ok());t.width=1281;assert!(check(t.clone()).is_err());t.width=1280;
 t.captions[0].end=f64::INFINITY;assert!(check(t.clone()).is_err());t.captions[0].end=2.;
 t.tracks.push(t.tracks[0].clone());assert!(check(t.clone()).is_err());t.tracks.pop();
 let mut value=serde_json::to_value(t).unwrap();value["filterGraph"]=serde_json::json!("movie=http://example.invalid");assert!(serde_json::from_value::<Timeline>(value).is_err());
}
