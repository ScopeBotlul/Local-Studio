use super::*;
#[cfg(test)]mod tests;
pub mod canvas;
pub mod video;
pub use canvas::{canvas_preview,canvas_export};
pub use video::{video_frame,media_prepare,media_status,media_cancel,media_info,caption_read,caption_write,video_probe,video_start,video_jobs,video_cancel,VideoEngine};

#[derive(Clone,Serialize,Deserialize,Default)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Creative {pub revision:u64,pub image:Option<Canvas>,pub video:Option<Timeline>}
#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Canvas {pub width:u32,pub height:u32,pub background:Option<String>,pub layers:Vec<Layer>}
#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Layer {pub id:String,pub name:String,pub asset_id:String,pub x:f64,pub y:f64,pub width:f64,pub height:f64,pub rotation:f64,pub opacity:f64,pub blend:Blend,pub visible:bool,pub locked:bool,pub group:Option<String>,pub operations:Vec<gallery::EditOperation>,pub mask:Option<Mask>}
#[derive(Clone,Copy,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub enum Blend {Normal,Multiply,Screen,Overlay}
#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Mask {pub base:bool,pub inverted:bool,pub strokes:Vec<Stroke>}
#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Stroke {pub shape:Shape,pub restore:bool,pub size:f64,pub hardness:f64,pub opacity:f64,pub points:Vec<[f64;2]>}
#[derive(Clone,Copy,Serialize,Deserialize)]
#[serde(rename_all="camelCase")]
pub enum Shape {Brush,Rectangle,Ellipse}
#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Timeline {pub width:u32,pub height:u32,pub fps:u32,pub background:String,pub tracks:Vec<Track>,pub clips:Vec<Clip>,pub captions:Vec<Caption>}
#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Track {pub id:String,pub name:String,pub audio:bool,pub muted:bool,pub locked:bool}
#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Clip {pub id:String,pub asset_id:String,pub track_id:String,pub name:String,pub start:f64,pub source_in:f64,pub source_out:f64,pub speed:f64,pub fade_in:f64,pub fade_out:f64,pub blur:f64,pub brightness:f64,pub contrast:f64,pub saturation:f64,pub keyframes:Vec<Keyframe>}
#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Keyframe {pub time:f64,pub x:f64,pub y:f64,pub scale:f64,pub rotation:f64,pub opacity:f64,pub volume:f64}
#[derive(Clone,Serialize,Deserialize)]
#[serde(rename_all="camelCase",deny_unknown_fields)]
pub struct Caption {pub id:String,pub start:f64,pub end:f64,pub text:String,pub size:u32,pub color:String,pub y:f64}
fn bounded(x:f64,lo:f64,hi:f64)->Result<()> {if !x.is_finite()||x<lo||x>hi{Err("creative_parameters".into())}else{Ok(())}}
fn color(s:&str)->bool{s.len()==7&&s.starts_with('#')&&s.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)}
fn size(w:u32,h:u32)->Result<()>{if w==0||h==0||w>8192||h>8192||u64::from(w)*u64::from(h)>16_000_000{Err("creative_limit".into())}else{Ok(())}}
fn name(s:&str)->bool{!s.trim().is_empty()&&s.len()<=180&&!s.chars().any(char::is_control)}
pub fn referenced(c:&Creative,id:&str)->bool{c.image.as_ref().is_some_and(|d|d.layers.iter().any(|l|l.asset_id==id))||c.video.as_ref().is_some_and(|t|t.clips.iter().any(|l|l.asset_id==id))}
pub fn validate(c:&Creative,assets:&[Asset])->Result<()> {
 let asset=|id:&str|assets.iter().find(|a|a.id==id).ok_or("creative_source".to_string());
 if let Some(d)=&c.image {size(d.width,d.height)?;if d.layers.len()>32||d.background.as_ref().is_some_and(|s|!color(s)){return Err("creative_limit".into());}let mut ids=HashSet::new();let mut points=0;
 for l in &d.layers {if !valid_id(&l.id)||!ids.insert(&l.id)||!name(&l.name)||asset(&l.asset_id)?.kind!="image"||l.group.as_ref().is_some_and(|g|!valid_id(g)){return Err("creative_source".into());}
 bounded(l.x,-32768.,32768.)?;bounded(l.y,-32768.,32768.)?;bounded(l.width,1.,32768.)?;bounded(l.height,1.,32768.)?;bounded(l.rotation,-360.,360.)?;bounded(l.opacity,0.,1.)?;gallery::validate_operations(&l.operations)?;
 if let Some(m)=&l.mask {if m.strokes.len()>1000{return Err("creative_limit".into());}for s in &m.strokes {bounded(s.size,0.001,2.)?;bounded(s.hardness,0.,1.)?;bounded(s.opacity,0.,1.)?;if s.points.is_empty()||(!matches!(s.shape,Shape::Brush)&&s.points.len()!=2){return Err("creative_parameters".into());}points+=s.points.len();if points>20000{return Err("creative_limit".into());}for p in &s.points{bounded(p[0],-1.,2.)?;bounded(p[1],-1.,2.)?;}}}
 }}
 if let Some(t)=&c.video {size(t.width,t.height)?;if t.width%2!=0||t.height%2!=0||t.width>3840||t.height>2160||![24,25,30,50,60].contains(&t.fps)||!color(&t.background)||t.tracks.len()>8||t.clips.len()>64||t.captions.len()>500{return Err("creative_limit".into());}
 let mut tracks=HashSet::new();for tr in &t.tracks{if !valid_id(&tr.id)||!tracks.insert(&tr.id)||!name(&tr.name){return Err("creative_parameters".into());}}
 let mut ids=HashSet::new();for clip in &t.clips {let a=asset(&clip.asset_id)?;let tr=t.tracks.iter().find(|tr|tr.id==clip.track_id).ok_or("creative_source")?;
 if !valid_id(&clip.id)||!ids.insert(&clip.id)||!name(&clip.name)||(!tr.audio&&a.kind=="audio")||(tr.audio&&a.kind=="image"){return Err("creative_source".into());}
 bounded(clip.start,0.,3600.)?;bounded(clip.source_in,0.,86400.)?;bounded(clip.source_out,clip.source_in+0.04,86400.)?;bounded(clip.speed,0.25,4.)?;let duration=(clip.source_out-clip.source_in)/clip.speed;bounded(clip.start+duration,0.04,3600.)?;bounded(clip.fade_in,0.,duration/2.)?;bounded(clip.fade_out,0.,duration/2.)?;bounded(clip.blur,0.,20.)?;bounded(clip.brightness,-1.,1.)?;bounded(clip.contrast,0.,2.)?;bounded(clip.saturation,0.,2.)?;
 if clip.keyframes.is_empty()||clip.keyframes.len()>100||clip.keyframes[0].time!=0.{return Err("creative_parameters".into());}let mut last=-1.;for k in &clip.keyframes{bounded(k.time,0.,duration)?;if k.time<=last{return Err("creative_parameters".into());}last=k.time;bounded(k.x,-8192.,8192.)?;bounded(k.y,-8192.,8192.)?;bounded(k.scale,0.05,4.)?;bounded(k.rotation,-360.,360.)?;bounded(k.opacity,0.,1.)?;bounded(k.volume,0.,2.)?;}
 }
 for c in &t.captions{if !valid_id(&c.id)||!ids.insert(&c.id)||c.text.len()>4000||c.text.contains('\0')||!color(&c.color)||!(8..=200).contains(&c.size){return Err("creative_parameters".into());}bounded(c.start,0.,3600.)?;bounded(c.end,c.start+0.04,3600.)?;bounded(c.y,0.,1.)?;}
 }
 if serde_json::to_vec(c).map_err(err)?.len()>750_000{return Err("creative_limit".into());}Ok(())
}
impl Projects {
 pub(crate) fn creative_snapshot(&self,id:&str)->Result<Project>{let s=self.state.lock().map_err(err)?;let p=Self::require(&s)?;if p.id!=id{return Err("project_changed".into());}Ok(p)}
 pub(crate) fn creative_source(p:&Project,id:&str)->Result<(Asset,PathBuf,File,Vec<File>)>{let a=p.assets.iter().find(|a|a.id==id).ok_or("creative_source")?.clone();let path=owned(p,&a)?;let pins=gallery::directory_guards(path.parent().ok_or("project_path")?)?;let mut f=gallery::lock_file(&path)?;if f.metadata().map_err(err)?.len()!=a.bytes||digest(&mut f)?!=a.sha256{return Err("project_hash".into());}f.rewind().map_err(err)?;Ok((a,path,f,pins))}
 fn store_creative(&self,id:&str,expected:u64,mut creative:Creative)->Result<Project>{let mut s=self.state.lock().map_err(err)?;let mut p=Self::require(&s)?;if p.id!=id||p.creative.as_ref().map_or(0,|c|c.revision)!=expected{return Err("project_changed".into());}validate(&creative,&p.assets)?;creative.revision=expected.checked_add(1).ok_or("creative_limit")?;p.creative=Some(creative);p.dirty=true;persist(&mut s,Some(p.clone()))?;Ok(p)}
}
#[tauri::command]
pub async fn project_creative_save(id:String,expected:u64,creative:Creative,state:tauri::State<'_,Arc<Projects>>)->Result<Project>{let p=state.inner().clone();tauri::async_runtime::spawn_blocking(move||p.store_creative(&id,expected,creative)).await.map_err(err)?}
