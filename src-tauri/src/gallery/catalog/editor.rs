use super::*;
use super::lineage::{LineageQuery,Node,checked,source,put};
use image::{DynamicImage,ImageReader,ImageDecoder,ImageEncoder,ImageFormat,Limits};
use std::io::BufReader;

#[derive(Clone,Serialize,Deserialize,Debug,PartialEq)]
#[serde(tag="type",rename_all="camelCase",deny_unknown_fields)]
pub enum Operation { Rotate { clockwise:bool }, Flip { horizontal:bool }, Crop {x:u32,y:u32,width:u32,height:u32}, Resize {width:u32,height:u32}, Adjust {brightness:i16,contrast:i16,saturation:i16,temperature:i16} }
#[derive(Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Request {query:LineageQuery,operations:Vec<Operation>}
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Preview {data_url:String,width:u32,height:u32,profile:bool}
fn err(_:impl std::fmt::Display)->String{"editor_image".into()}
fn dimensions(w:u32,h:u32)->Result<()> {if w==0||h==0||w>16384||h>16384||u64::from(w)*u64::from(h)>32_000_000{Err("editor_limit".into())}else{Ok(())}}
pub(crate) fn validate_operations(ops:&[Operation])->Result<()>{
 if ops.len()>1000{return Err("editor_limit".into());}
 for op in ops {match *op {
 Operation::Adjust{brightness,contrast,saturation,temperature}=>if [brightness,contrast,saturation,temperature].iter().any(|v|!(-100..=100).contains(v)){return Err("editor_adjust".into());},
 Operation::Crop{x,y,width,height}=>{dimensions(width,height)?;if x.checked_add(width).is_none()||y.checked_add(height).is_none(){return Err("editor_crop".into());}},
 Operation::Resize{width,height}=>dimensions(width,height)?,_=>{}
 }}Ok(())
}
fn decode(mut file:File)->Result<(DynamicImage,Option<Vec<u8>>)> {
 if file.metadata().map_err(err)?.len()>64*1024*1024{return Err("editor_limit".into());}
 let mut signature=[0u8;8];file.read_exact(&mut signature).map_err(err)?;
 if &signature==b"\x89PNG\r\n\x1a\n"{let length=file.metadata().map_err(err)?.len();loop{let pos=file.stream_position().map_err(err)?;if pos+12>length{break;}let mut chunk=[0;8];file.read_exact(&mut chunk).map_err(err)?;let len=u32::from_be_bytes(chunk[..4].try_into().unwrap()) as u64;if &chunk[4..]==b"acTL"{return Err("editor_format".into());}if pos+12+len>length{return Err("editor_image".into());}file.seek(SeekFrom::Start(pos+12+len)).map_err(err)?;if &chunk[4..]==b"IEND"{break;}}}file.seek(SeekFrom::Start(0)).map_err(err)?;
 let mut reader=ImageReader::new(BufReader::new(file)).with_guessed_format().map_err(err)?;
 if !matches!(reader.format(),Some(ImageFormat::Png|ImageFormat::Jpeg|ImageFormat::Bmp)){return Err("editor_format".into());}
 let mut limits=Limits::default();limits.max_image_width=Some(16384);limits.max_image_height=Some(16384);limits.max_alloc=Some(128*1024*1024);reader.limits(limits);
 let mut decoder=reader.into_decoder().map_err(err)?;let(w,h)=decoder.dimensions();dimensions(w,h)?;
 if !matches!(decoder.color_type(),image::ColorType::L8|image::ColorType::La8|image::ColorType::Rgb8|image::ColorType::Rgba8){return Err("editor_depth".into());}
 let profile=decoder.icc_profile().map_err(err)?;
 // RGB output must never inherit a grayscale/CMYK profile.
 if profile.as_ref().is_some_and(|p|p.len()>1024*1024||p.get(16..20)!=Some(b"RGB ")){return Err("editor_profile".into());}
 let orientation=decoder.orientation().map_err(err)?;
 let mut image=DynamicImage::from_decoder(decoder).map_err(err)?;image.apply_orientation(orientation);
 Ok((DynamicImage::ImageRgba8(image.to_rgba8()),profile))
}
fn transform(mut image:DynamicImage,ops:&[Operation])->Result<DynamicImage>{
 validate_operations(ops)?;
 // Validate every intermediate size before allocating; cap total work as well.
 let(mut w,mut h)=(image.width(),image.height());let mut work=0u64;
 for op in ops {match *op {
 Operation::Rotate{..}=>{std::mem::swap(&mut w,&mut h);},Operation::Flip{..}=>{},
 Operation::Crop{x,y,width,height}=>{if x.checked_add(width).is_none_or(|v|v>w)||y.checked_add(height).is_none_or(|v|v>h){return Err("editor_crop".into());}w=width;h=height;},
 Operation::Resize{width,height}=>{w=width;h=height;},Operation::Adjust{..}=>{}
 }dimensions(w,h)?;work+=u64::from(w)*u64::from(h);if work>512_000_000{return Err("editor_limit".into());}}
 for op in ops {image=match *op {Operation::Adjust{brightness,contrast,saturation,temperature}=>{
 let mut rgba=image.to_rgba8();let b=f64::from(brightness)*2.55;let c=(1.+f64::from(contrast)/100.).powi(2);let s=1.+f64::from(saturation)/100.;let warm=f64::from(temperature)*0.4;
 for p in rgba.pixels_mut(){let rgb=[f64::from(p[0]),f64::from(p[1]),f64::from(p[2])];let l=rgb[0]*0.2126+rgb[1]*0.7152+rgb[2]*0.0722;for k in 0..3{let value=((l+(rgb[k]-l)*s)-127.5)*c+127.5+b+if k==0{warm}else if k==2{-warm}else{0.};p[k]=value.round().clamp(0.,255.) as u8;}}
 DynamicImage::ImageRgba8(rgba)
 },Operation::Rotate{clockwise:true}=>image.rotate90(),Operation::Rotate{clockwise:false}=>image.rotate270(),Operation::Flip{horizontal:true}=>image.fliph(),Operation::Flip{horizontal:false}=>image.flipv(),Operation::Crop{x,y,width,height}=>image.crop_imm(x,y,width,height),Operation::Resize{width,height}=>{
 // Resize premultiplied alpha to avoid dark/colored fringes at transparent edges.
 let mut rgba=image.to_rgba8();for p in rgba.pixels_mut(){for c in 0..3{p[c]=((u16::from(p[c])*u16::from(p[3])+127)/255) as u8;}}
 let mut scaled=image::imageops::resize(&rgba,width,height,image::imageops::FilterType::Lanczos3);for p in scaled.pixels_mut(){if p[3]>0{for c in 0..3{p[c]=((u32::from(p[c])*255+u32::from(p[3])/2)/u32::from(p[3])).min(255) as u8;}}}
 DynamicImage::ImageRgba8(scaled)
 } };}Ok(image)
}
fn encode(image:&DynamicImage,profile:Option<Vec<u8>>,format:&str,quality:u8)->Result<Vec<u8>>{
 let mut bytes=Vec::new();match format {
 "png"=>{let mut e=image::codecs::png::PngEncoder::new(&mut bytes);if let Some(p)=profile{e.set_icc_profile(p).map_err(err)?;}let rgba=image.to_rgba8();e.write_image(&rgba,image.width(),image.height(),image::ExtendedColorType::Rgba8).map_err(err)?;},
 "jpeg"=>{if !(1..=100).contains(&quality){return Err("editor_format".into());}let mut rgb=image::RgbImage::new(image.width(),image.height());let rgba=image.to_rgba8();for (src,dst) in rgba.pixels().zip(rgb.pixels_mut()){for c in 0..3{dst[c]=((u32::from(src[c])*u32::from(src[3])+255*(255-u32::from(src[3]))+127)/255) as u8;}}let mut e=image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes,quality);if let Some(p)=profile{e.set_icc_profile(p).map_err(err)?;}e.encode(&rgb,image.width(),image.height(),image::ExtendedColorType::Rgb8).map_err(err)?;},
 _=>return Err("editor_format".into())};Ok(bytes)
}
pub(crate) fn render_preview(file:File,operations:&[Operation])->Result<Preview>{let(image,profile)=decode(file)?;let image=transform(image,operations)?;let(width,height)=(image.width(),image.height());let small=image.thumbnail(1600,1600);let has_profile=profile.is_some();let bytes=encode(&small,profile,"png",90)?;Ok(Preview{width,height,profile:has_profile,data_url:format!("data:image/png;base64,{}",base64::engine::general_purpose::STANDARD.encode(bytes))})}
fn preview(root:&Path,request:Request)->Result<Preview>{let(_path,file,_pins)=checked(root,&request.query)?;render_preview(file,&request.operations)}
pub(crate) fn render_bytes(file:File,operations:&[Operation],format:&str,quality:u8)->Result<Vec<u8>>{let(image,profile)=decode(file)?;encode(&transform(image,operations)?,profile,format,quality)}
impl GalleryCatalog {
 pub(crate) fn export_project_edit(&self,root:&Path,name:&str,bytes:&[u8],recipe:serde_json::Value,format:&str)->Result<String>{
 let _gate=self.files_gate.lock().map_err(err)?;let _pins=directory_guards(root)?;let id=uuid::Uuid::new_v4().to_string();let name=format!("{} - edit-{}.{}",Path::new(name).file_stem().unwrap_or_default().to_string_lossy().chars().take(60).collect::<String>(),&id[..8],if format=="jpeg"{"jpg"}else{"png"});let destination=root.join(&name);let temp=root.join(format!(".localstudio-edit-{id}.tmp"));let mut output=OpenOptions::new().read(true).write(true).access_mode(0xc0010000).share_mode(1).create_new(true).open(&temp).map_err(err)?;
 let result=(||{output.write_all(bytes).map_err(err)?;output.sync_all().map_err(err)?;let node=Node{id:id.clone(),parent:None,group:id,path:relative(root,&destination)?,name,kind:"image".into(),file_id:identity(&output)?,stamp:stamp(&output.metadata().map_err(err)?),created_at:chrono::Utc::now().timestamp_millis(),operation:"edit".into(),origin:None,edit:Some(recipe)};let db=self.db.lock().map_err(err)?;put(&db,root,&node)?;rename_handle(&output,&destination)?;Ok(node.path)})();if result.is_err(){let _=delete_handle(&output);}result
 }
 fn export_edit(&self,root:&Path,request:Request,format:String,quality:u8,origin:Option<BoundOrigin>)->Result<String>{
 let _gate=self.files_gate.lock().map_err(err)?;let(path,file,_pins)=checked(root,&request.query)?;let(image,profile)=decode(file.try_clone().map_err(err)?)?;let image=transform(image,&request.operations)?;let bytes=encode(&image,profile,&format,quality)?;
 let db=self.db.lock().map_err(err)?;let parent=source(&db,root,&request.query,&path,&file,origin)?;
 let count:i64=db.query_row("SELECT count(*) FROM lineage WHERE root=?1 AND group_id=?2",params![request.query.root_id,parent.group],|r|r.get(0)).map_err(err)?;if count>=100{return Err("gallery_lineage_limit".into());}
 let folder=path.parent().ok_or("gallery_path")?;let id=uuid::Uuid::new_v4().to_string();let temp=folder.join(format!(".localstudio-edit-{id}.tmp"));let name=format!("{} - edit-{}.{}",path.file_stem().unwrap_or_default().to_string_lossy().chars().take(60).collect::<String>(),&id[..8],if format=="jpeg"{"jpg"}else{"png"});let destination=folder.join(&name);
 // Keep a deletion-capable exclusive handle from creation through atomic publication.
 let mut output=OpenOptions::new().read(true).write(true).access_mode(0xc0010000).share_mode(1).create_new(true).open(&temp).map_err(err)?;
 let result=(||{output.write_all(&bytes).map_err(err)?;output.sync_all().map_err(err)?;let node=Node{id,parent:Some(parent.id),group:parent.group,path:relative(root,&destination)?,name,kind:"image".into(),file_id:identity(&output)?,stamp:stamp(&output.metadata().map_err(err)?),created_at:chrono::Utc::now().timestamp_millis(),operation:"edit".into(),origin:parent.origin,edit:Some(serde_json::to_value(&request.operations).map_err(err)?)};
 // Persist provenance before publishing. If interrupted, metadata may refer to a missing file,
 // but a published file can never lose its recorded parent/recipe.
 put(&db,root,&node)?;rename_handle(&output,&destination)?;Ok(node.path)})();
 if result.is_err(){let _=delete_handle(&output);}result
 }
}
#[tauri::command]
pub async fn editor_preview(request:Request,core:State<'_,Arc<Core>>)->Result<Preview>{let permit=thumbnails::exclusive().await?;let root=root(&core)?;tauri::async_runtime::spawn_blocking(move||{let _permit=permit;preview(&root,request)}).await.map_err(err)?}
#[tauri::command]
pub async fn editor_export(request:Request,format:String,quality:u8,core:State<'_,Arc<Core>>,catalog:State<'_,Arc<GalleryCatalog>>,images:State<'_,Arc<ImageEngine>>)->Result<String>{let permit=thumbnails::exclusive().await?;let root=root(&core)?;let c=catalog.inner().clone();let i=images.inner().clone();tauri::async_runtime::spawn_blocking(move||{let _permit=permit;let origins=c.sync_origins(&root,i.list()?)?;let origin=origins.get(&request.query.target.file_id).cloned();c.export_edit(&root,request,format,quality,origin)}).await.map_err(err)?}

#[cfg(test)]
#[path="editor_tests.rs"]mod tests;
