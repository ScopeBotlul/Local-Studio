use super::*;

fn setup() -> (tempfile::TempDir,PathBuf,PathBuf) {
    let temp=tempfile::tempdir().unwrap(); let root=fs::canonicalize(temp.path()).unwrap(); let cache=root.join("cache"); fs::create_dir(&cache).unwrap(); (temp,root,cache)
}
fn write_image(path: &Path,format: ImageFormat,w: u32,h: u32) {
    let image=DynamicImage::ImageRgba8(image::RgbaImage::from_pixel(w,h,image::Rgba([20,80,170,100])));
    let image=if format==ImageFormat::Jpeg { DynamicImage::ImageRgb8(image.to_rgb8()) } else { image };
    image.save_with_format(path,format).unwrap();
}
fn query(root: &Path,name: &str) -> ThumbnailQuery {
    let path=root.join(name); let meta=fs::metadata(&path).unwrap(); let id=catalog::path_identity(&path).ok();
    ThumbnailQuery { root_id:root_id(root),path:name.into(),version:version(root,&path,&meta,id.as_deref()) }
}
fn bytes(thumb: &Thumbnail) -> Vec<u8> { base64::engine::general_purpose::STANDARD.decode(thumb.data_url.strip_prefix("data:image/png;base64,").unwrap()).unwrap() }

#[test] fn real_image_formats_resize_preserve_alpha_and_never_change_originals() {
    let (_temp,root,cache)=setup();
    for (name,format) in [("a.png",ImageFormat::Png),("a.jpg",ImageFormat::Jpeg),("a.webp",ImageFormat::WebP),("a.gif",ImageFormat::Gif),("a.bmp",ImageFormat::Bmp)] {
        write_image(&root.join(name),format,640,360); let original=fs::read(root.join(name)).unwrap();
        let thumb=thumbnail(&root,&cache,query(&root,name)).unwrap(); assert_eq!((thumb.width,thumb.height),(320,180));
        assert!(!thumb.cached); assert!(thumb.cache_stored);
        let image=image::load_from_memory(&bytes(&thumb)).unwrap(); assert_eq!((image.width(),image.height()),(320,180));
        if format==ImageFormat::Png { assert_eq!(image.to_rgba8().get_pixel(0,0).0,[20,80,170,100]); }
        assert_eq!(fs::read(root.join(name)).unwrap(),original);
    }
}
#[test] fn jpeg_orientation_is_applied_and_small_images_are_not_upscaled() {
    let (_temp,root,cache)=setup(); let path=root.join("rotate.jpg");write_image(&path,ImageFormat::Jpeg,80,40);
    let source=fs::read(&path).unwrap();
    // Little-endian TIFF with one orientation SHORT (6 = rotate 90 clockwise).
    let exif=b"Exif\0\0II\x2a\0\x08\0\0\0\x01\0\x12\x01\x03\0\x01\0\0\0\x06\0\0\0\0\0\0\0";
    let mut encoded=source[..2].to_vec();encoded.extend_from_slice(&[0xff,0xe1]);encoded.extend_from_slice(&((exif.len()+2) as u16).to_be_bytes());encoded.extend_from_slice(exif);encoded.extend_from_slice(&source[2..]);fs::write(&path,&encoded).unwrap();
    let thumb=thumbnail(&root,&cache,query(&root,"rotate.jpg")).unwrap(); assert_eq!((thumb.width,thumb.height),(40,80)); assert_eq!(fs::read(path).unwrap(),encoded);
}
#[test] fn cache_survives_reopen_invalidates_changed_files_and_rebuilds() {
    let (_temp,root,cache)=setup();let path=root.join("a.png");write_image(&path,ImageFormat::Png,640,360);
    let stale=query(&root,"a.png"); let first=thumbnail(&root,&cache,query(&root,"a.png")).unwrap();
    let second=thumbnail(&root,&cache,query(&root,"a.png")).unwrap();assert!(second.cached);assert_eq!(first.data_url,second.data_url);
    write_image(&path,ImageFormat::Png,360,640);
    assert_eq!(thumbnail(&root,&cache,stale).err().unwrap(),"gallery_changed");
    let changed=thumbnail(&root,&cache,query(&root,"a.png")).unwrap();assert!(!changed.cached);assert_eq!((changed.width,changed.height),(180,320));
    let before=fs::read(&path).unwrap();clear(&cache).unwrap();assert!(!thumbnail(&root,&cache,query(&root,"a.png")).unwrap().cached);assert_eq!(fs::read(&path).unwrap(),before);
    let stale=query(&root,"a.png");fs::remove_file(path).unwrap();assert!(thumbnail(&root,&cache,stale).is_err());
}
#[test] fn cache_is_optional_and_corrupted_entries_are_regenerated() {
    let (_temp,root,cache_path)=setup();write_image(&root.join("a.png"),ImageFormat::Png,40,20);
    let uncached=thumbnail(&root,&root.join("missing-cache"),query(&root,"a.png")).unwrap(); assert!(!uncached.cache_stored);
    thumbnail(&root,&cache_path,query(&root,"a.png")).unwrap();
    { let (db,_guards,_file)=cache(&cache_path).unwrap();db.execute("UPDATE thumbnails SET png=x'0001'",[]).unwrap(); }
    let regenerated=thumbnail(&root,&cache_path,query(&root,"a.png")).unwrap();assert!(!regenerated.cached);assert_eq!(regenerated.data_url,uncached.data_url);
}
#[test] fn cache_evicts_by_byte_and_entry_limits_without_touching_media() {
    let (_temp,root,cache_path)=setup();fs::write(root.join("keep.png"),b"untouched media").unwrap();
    let (mut db,_guards,_file)=cache(&cache_path).unwrap();
    put(&mut db,"old",&[0;12],1,1,30,2).unwrap();db.execute("UPDATE thumbnails SET used=0 WHERE key='old'",[]).unwrap();
    put(&mut db,"second",&[0;12],1,1,30,2).unwrap();put(&mut db,"new",&[0;12],1,1,30,2).unwrap();
    assert_eq!(db.query_row("SELECT count(*) FROM thumbnails WHERE key='old'",[],|r|r.get::<_,i64>(0)).unwrap(),0);
    put(&mut db,"larger",&[0;24],1,1,30,2).unwrap();
    assert!(db.query_row("SELECT sum(length(png)) FROM thumbnails",[],|r|r.get::<_,i64>(0)).unwrap()<=30);
    assert_eq!(fs::read(root.join("keep.png")).unwrap(),b"untouched media");
}
#[test] fn invalid_paths_roots_formats_and_oversized_sources_are_rejected() {
    let (_temp,root,cache)=setup();write_image(&root.join("a.png"),ImageFormat::Png,40,20);
    let mut wrong=query(&root,"a.png");wrong.root_id="other".into();assert!(thumbnail(&root,&cache,wrong).is_err());
    let mut wrong=query(&root,"a.png");wrong.path="../outside.png".into();assert!(thumbnail(&root,&cache,wrong).is_err());
    for name in ["bad.png","unsafe.avif","audio.wav"] {fs::write(root.join(name),b"not an image").unwrap();assert!(thumbnail(&root,&cache,query(&root,name)).is_err());}
    let huge=root.join("huge.png");File::create(&huge).unwrap().set_len(MAX_SOURCE+1).unwrap();assert_eq!(thumbnail(&root,&cache,query(&root,"huge.png")).err().unwrap(),"gallery_thumbnail_limit");
    // Valid BMP header advertising an excessive decoded allocation, with no pixels.
    let mut bmp=vec![0u8;54];bmp[..2].copy_from_slice(b"BM");bmp[10..14].copy_from_slice(&54u32.to_le_bytes());bmp[14..18].copy_from_slice(&40u32.to_le_bytes());bmp[18..22].copy_from_slice(&10000u32.to_le_bytes());bmp[22..26].copy_from_slice(&10000u32.to_le_bytes());bmp[26..28].copy_from_slice(&1u16.to_le_bytes());bmp[28..30].copy_from_slice(&24u16.to_le_bytes());fs::write(root.join("bomb.bmp"),bmp).unwrap();assert!(thumbnail(&root,&cache,query(&root,"bomb.bmp")).is_err());
}
