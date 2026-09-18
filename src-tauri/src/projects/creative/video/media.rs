use super::*;
use std::sync::atomic::AtomicU64;
use crate::maintenance::{self,Candidate,Inventory};

pub struct MediaTools {
    frame_gate: Mutex<()>, frame_ticket: AtomicU64,
    task: Mutex<Option<MediaTask>>, busy: AtomicBool, cancel: AtomicBool,
    sources: Mutex<Vec<(String,Asset,PathBuf,File,Vec<File>)>>,
    verified_cache:Mutex<Vec<(String,File,Vec<File>)>>,
}
impl MediaTools {
    pub fn new()->Self {Self {frame_gate:Mutex::new(()),frame_ticket:AtomicU64::new(0),task:Mutex::new(None),busy:AtomicBool::new(false),cancel:AtomicBool::new(false),verified_cache:Mutex::new(vec![]),sources:Mutex::new(vec![])}}
    pub fn stop(&self){self.cancel.store(true,Ordering::SeqCst);self.frame_ticket.fetch_add(1,Ordering::SeqCst);}
}
#[derive(Clone,Serialize)]#[serde(rename_all="camelCase")]
pub struct MediaTask {pub id:String,pub asset_id:String,pub name:String,pub kind:String,pub status:String,pub progress:f64,pub error:Option<String>}
#[derive(Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
struct CacheFile {id:String,kind:String,source:String,path:String,binding:String,sha256:String,bytes:u64,used:i64,payload:serde_json::Value}
#[derive(Clone,Serialize,Deserialize)]#[serde(rename_all="camelCase")]
pub struct Waveform {duration:f64,peaks:Vec<f32>}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct MediaInfo {asset_id:String,proxies:Vec<ProxyInfo>,waveform:Option<Waveform>}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct ProxyInfo {id:String,width:u32,height:u32,bytes:u64}
#[derive(Serialize)]#[serde(rename_all="camelCase")]
pub struct Frame {data_url:String,time:f64,proxy_assets:Vec<String>}

impl VideoEngine {
    fn source(&self,p:&Project,id:&str)->Result<(Asset,PathBuf,File,Vec<File>)> {
        let asset=p.assets.iter().find(|a|a.id==id).ok_or("creative_source")?;
        let path=owned(p,asset)?;let key=format!("{}:{}:{}:{}",p.id,id,asset.sha256,path.display());
        let mut sources=self.media.sources.lock().map_err(err)?;
        if !sources.iter().any(|s|s.0==key) {
            if sources.len()>=8 {sources.clear();}
            let(a,path,f,pins)=Projects::creative_source(p,id)?;sources.push((key.clone(),a,path,f,pins));
        }
        let(_,a,path,f,pins)=sources.iter().find(|s|s.0==key).unwrap();
        Ok((a.clone(),path.clone(),f.try_clone().map_err(err)?,pins.iter().map(|f|f.try_clone().map_err(err)).collect::<Result<_>>()?))
    }
    fn records(&self)->Result<Vec<CacheFile>> {
        let s=self.state.lock().map_err(err)?;let mut q=s.0.prepare("SELECT json FROM media_cache").map_err(err)?;
        let rows=q.query_map([],|r|r.get::<_,String>(0)).map_err(err)?;
        rows.map(|r|serde_json::from_str(&r.map_err(err)?).map_err(err)).collect()
    }
    fn register(&self,id:String,kind:&str,source:&str,path:&Path,payload:serde_json::Value)->Result<()> {
        let _pins=gallery::directory_guards(path.parent().ok_or("cleanup_path")?)?;let mut f=gallery::lock_file(path)?;
        let row=CacheFile {id,kind:kind.into(),source:source.into(),path:path.to_string_lossy().into(),binding:gallery::file_binding(&f)?,sha256:digest(&mut f)?,bytes:f.metadata().map_err(err)?.len(),used:chrono::Utc::now().timestamp(),payload};
        let s=self.state.lock().map_err(err)?;s.0.execute("INSERT OR REPLACE INTO media_cache VALUES(?1,?2)",rusqlite::params![row.id,serde_json::to_string(&row).map_err(err)?]).map_err(err)?;Ok(())
    }
    fn cached(&self,row:&CacheFile)->Result<(File,Vec<File>)> {
        let p=Path::new(&row.path);let pins=gallery::directory_guards(p.parent().ok_or("cleanup_path")?)?;let mut f=gallery::lock_file(p)?;
        if gallery::file_binding(&f)?!=row.binding||f.metadata().map_err(err)?.len()!=row.bytes||digest(&mut f)?!=row.sha256{return Err("cleanup_changed".into());}f.rewind().map_err(err)?;Ok((f,pins))
    }
    fn pinned_cache(&self,row:&CacheFile)->Result<(File,Vec<File>)>{let mut cache=self.media.verified_cache.lock().map_err(err)?;if !cache.iter().any(|r|r.0==row.id){if cache.len()>=16{cache.clear();}let(f,pins)=self.cached(row)?;cache.push((row.id.clone(),f,pins));}let(_,f,pins)=cache.iter().find(|r|r.0==row.id).unwrap();Ok((f.try_clone().map_err(err)?,pins.iter().map(|f|f.try_clone().map_err(err)).collect::<Result<_>>()?))}
    fn touch(&self,row:&CacheFile)->Result<()> {let mut row=row.clone();row.used=chrono::Utc::now().timestamp();let s=self.state.lock().map_err(err)?;s.0.execute("UPDATE media_cache SET json=?2 WHERE id=?1",rusqlite::params![row.id,serde_json::to_string(&row).map_err(err)?]).map_err(err)?;Ok(())}
    pub(super) fn register_render_files(&self,j:&VideoJob)->Result<()> {
        let work=Path::new(&j.directory);if work.file_name().and_then(|v|v.to_str())!=Some(&j.id){return Err("cleanup_path".into());}
        let names=["render.mp4","render.webm","filter.txt","render.log"].into_iter().map(String::from).chain((0..j.timeline.captions.len()).map(|i|format!("caption-{i}.txt")));
        for name in names {let path=work.join(&name);if path.is_file(){self.register(format!("render:{}:{name}",j.id),"render",&j.id,&path,serde_json::Value::Null)?;}}Ok(())
    }
    pub(crate) fn cleanup_inventory(&self,cutoff:i64,projects:&Projects)->Result<Inventory> {
        self.media.verified_cache.lock().map_err(err)?.clear();self.media.sources.lock().map_err(err)?.clear();let p=projects.snapshot()?;let mut out=Inventory::default();
        for row in self.records()? {let path=Path::new(&row.path);if !path.exists(){continue;}match maintenance::inspect(path,"video",&row.id){Ok(c)=>{
            if c.binding!=row.binding||c.bytes!=row.bytes{out.unavailable+=1;continue;}
            if self.protected(&row,cutoff,p.as_ref())? {out.protected_bytes+=c.bytes;}else{out.files.push(c);}
        },Err(_)=>out.unavailable+=1}}Ok(out)
    }
    fn protected(&self,row:&CacheFile,cutoff:i64,p:Option<&Project>)->Result<bool> {
        if row.used>=cutoff{return Ok(true);}if row.kind!="render"{return Ok(p.is_some_and(|p|p.assets.iter().any(|a|a.sha256==row.source)));}
        let s=self.state.lock().map_err(err)?;
        let text:Option<String>=s.0.query_row("SELECT json FROM video_jobs WHERE id=?1",[&row.source],|r|r.get(0)).optional().map_err(err)?;
        let Some(text)=text else{return Ok(true)};let job:VideoJob=serde_json::from_str(&text).map_err(err)?;
        Ok(job.status=="running"||job.preview&&p.is_some_and(|p|p.id==job.project_id))
    }
    pub(crate) fn cleanup_file(&self,c:&Candidate,cutoff:i64,projects:&Projects)->Result<bool> {
        let Some(row)=self.records()?.into_iter().find(|r|r.id==c.owner&&r.path==c.path)else{return Ok(false)};
        if self.protected(&row,cutoff,projects.snapshot()?.as_ref())?{return Ok(false);}let(read,_pins)=self.cached(&row)?;
        if gallery::file_binding(&read)?!=c.binding{return Err("cleanup_changed".into());}drop(read);
        let(mut file,_guards)=maintenance::checked_file(c)?;if digest(&mut file)?!=row.sha256{return Err("cleanup_changed".into());}gallery::delete_handle(&file)?;
        let s=self.state.lock().map_err(err)?;s.0.execute("DELETE FROM media_cache WHERE id=?1",[&row.id]).map_err(err)?;Ok(true)
    }
}

// Every task has an isolated UUID directory. Cleanup lists exact generated names,
// never recursively deletes the directory or follows an imported path.
struct Work {path:PathBuf,pins:Vec<File>,files:Vec<String>}
impl Work {
    fn new(base:&str,kind:&str)->Result<Self>{let base=Path::new(base).join(kind);fs::create_dir_all(&base).map_err(err)?;let mut pins=gallery::directory_guards(&base)?;let path=base.join(uuid());fs::create_dir(&path).map_err(err)?;pins.extend(gallery::directory_guards(&path)?);Ok(Self{path,pins,files:vec![]})}
}
impl Drop for Work {fn drop(&mut self){for name in &self.files{let _=gallery::delete_owned(&self.path.join(name));}self.pins.clear();let _=fs::remove_dir(&self.path);}}
fn run(mut c:Command,log:&Path,timeout:Duration,cancel:impl Fn()->bool,progress:impl Fn(f64))->Result<()> {
    let file=OpenOptions::new().create_new(true).write(true).open(log).map_err(err)?;
    let mut child=c.stdout(Stdio::piped()).stderr(file).spawn().map_err(|_|"video_runtime")?;
    let guard=match crate::image_engine::ProcessGroup::attach(&child){Ok(g)=>g,Err(e)=>{let _=child.kill();let _=child.wait();return Err(e)}};
    let pipe=child.stdout.take().ok_or("video_runtime")?;let(tx,rx)=std::sync::mpsc::channel();
    let reader=thread::spawn(move||{for line in BufReader::new(pipe).lines().map_while(std::result::Result::ok){if let Some(v)=line.strip_prefix("out_time_us=").and_then(|v|v.parse::<f64>().ok()){let _=tx.send(v/1e6);}}});
    let start=Instant::now();let result=loop {
        if cancel()||start.elapsed()>timeout {let _=child.kill();let _=child.wait();break Err(if cancel(){"video_cancelled"}else{"video_timeout"}.into());}
        for time in rx.try_iter(){progress(time);}
        if let Some(status)=child.try_wait().map_err(err)?{break if status.success(){Ok(())}else{Err("video_encode".into())};}
        thread::sleep(Duration::from_millis(30));
    };drop(guard);let _=reader.join();result
}
fn pose(clip:&Clip,time:f64)->Keyframe {
    let mut a=&clip.keyframes[0];for b in clip.keyframes.iter().skip(1){if time<=b.time{let f=((time-a.time)/(b.time-a.time)).clamp(0.,1.);let mix=|a:f64,b:f64|a+(b-a)*f;return Keyframe{time:0.,x:mix(a.x,b.x),y:mix(a.y,b.y),scale:mix(a.scale,b.scale),rotation:mix(a.rotation,b.rotation),opacity:mix(a.opacity,b.opacity),volume:mix(a.volume,b.volume)};}a=b;}Keyframe{time:0.,..a.clone()}
}
fn freeze(t:&Timeline,time:f64)->Timeline {
    let mut next=t.clone();next.clips=t.clips.iter().filter(|c|c.start<=time&&time<c.start+(c.source_out-c.source_in)/c.speed&&t.tracks.iter().any(|tr|tr.id==c.track_id&&!tr.audio&&!tr.muted)).map(|c|{
        let mut n=c.clone();let local=time-c.start;let len=(c.source_out-c.source_in)/c.speed;let mut key=pose(c,local);
        if c.fade_in>0.{key.opacity*= (local/c.fade_in).clamp(0.,1.);}if c.fade_out>0.{key.opacity*=((len-local)/c.fade_out).clamp(0.,1.);}
        n.source_in=c.source_in+local*c.speed;n.source_out=c.source_out;n.start=0.;n.speed=1.;n.fade_in=0.;n.fade_out=0.;n.keyframes=vec![key];n
    }).collect();next.captions=t.captions.iter().filter(|c|c.start<=time&&time<c.end).map(|c|Caption{start:0.,end:0.1,..c.clone()}).collect();next
}

#[tauri::command]
pub async fn video_frame(id:String,timeline:Timeline,time:f64,proxies:bool,state:tauri::State<'_,Arc<Projects>>,engine:tauri::State<'_,Arc<VideoEngine>>,core:tauri::State<'_,Arc<Core>>)->Result<Frame> {
    let p=state.creative_snapshot(&id)?;validate(&Creative{revision:0,image:None,video:Some(timeline.clone())},&p.assets)?;bounded(time,0.,3600.)?;
    let e=engine.inner().clone();let ticket=e.media.frame_ticket.fetch_add(1,Ordering::SeqCst)+1;let temporary=core.storage_paths()?.temporary;
    tauri::async_runtime::spawn_blocking(move||{
        let _gate=e.media.frame_gate.lock().map_err(err)?;let cancelled=||e.media.frame_ticket.load(Ordering::SeqCst)!=ticket;if cancelled(){return Err("video_cancelled".into());}
        let t=freeze(&timeline,time);let(runtime,_rp)=e.runtime()?;let mut work=Work::new(&temporary,"video-frames")?;
        work.files=vec!["render.png".into(),"render.log".into(),"filter.txt".into()];work.files.extend((0..t.captions.len()).map(|i|format!("caption-{i}.txt")));
        let mut sources=vec![];let mut probes=vec![];let mut proxy_assets=vec![];let records=if proxies{e.records()?}else{vec![]};
        for clip in &t.clips {if cancelled(){return Err("video_cancelled".into());}let mut source=e.source(&p,&clip.asset_id)?;
            if let Some(row)=records.iter().filter(|r|r.kind=="proxy"&&r.source==source.0.sha256).max_by_key(|r|r.used){if let Ok((file,pins))=e.pinned_cache(row){source.1=PathBuf::from(&row.path);source.2=file;source.3.extend(pins);e.touch(row)?;proxy_assets.push(clip.asset_id.clone());}}
            probes.push(probe(&runtime,&source.1)?);sources.push(source);
        }
        let c=build(&t,&sources,&probes,&work.path,"png",1000,true,&runtime,true)?;
        run(c,&work.path.join("render.log"),Duration::from_secs(30),cancelled,|_|{})?;
        if cancelled(){return Err("video_cancelled".into());}let mut f=gallery::lock_file(&work.path.join("render.png"))?;if f.metadata().map_err(err)?.len()>4*1024*1024{return Err("creative_limit".into());}let mut bytes=vec![];f.read_to_end(&mut bytes).map_err(err)?;drop(f);
        Ok(Frame{data_url:format!("data:image/png;base64,{}",base64::engine::general_purpose::STANDARD.encode(bytes)),time,proxy_assets})
    }).await.map_err(err)?
}

#[tauri::command]
pub async fn media_info(id:String,state:tauri::State<'_,Arc<Projects>>,engine:tauri::State<'_,Arc<VideoEngine>>)->Result<Vec<MediaInfo>> {
    let p=state.creative_snapshot(&id)?;let e=engine.inner().clone();tauri::async_runtime::spawn_blocking(move||{
        let rows=e.records()?;let mut out=vec![];for asset in &p.assets {let mut info=MediaInfo{asset_id:asset.id.clone(),proxies:vec![],waveform:None};for row in rows.iter().filter(|r|r.source==asset.sha256) {
            let path=Path::new(&row.path);if !path.is_file(){continue;}let Ok(c)=maintenance::inspect(path,"video",&row.id)else{continue};if c.binding!=row.binding||c.bytes!=row.bytes{continue;}
            if row.kind=="proxy"{info.proxies.push(ProxyInfo{id:row.id.clone(),width:row.payload["width"].as_u64().unwrap_or(0) as u32,height:row.payload["height"].as_u64().unwrap_or(0) as u32,bytes:row.bytes});}
            if row.kind=="wave"{info.waveform=serde_json::from_value(row.payload.clone()).ok();}
        }out.push(info);}Ok(out)
    }).await.map_err(err)?
}

#[tauri::command]
pub fn media_status(engine:tauri::State<'_,Arc<VideoEngine>>)->Result<Option<MediaTask>>{Ok(engine.media.task.lock().map_err(err)?.clone())}
#[tauri::command]
pub fn media_cancel(id:String,engine:tauri::State<'_,Arc<VideoEngine>>)->Result<()>{if engine.media.task.lock().map_err(err)?.as_ref().is_some_and(|t|t.id==id&&t.status=="running"){engine.media.cancel.store(true,Ordering::SeqCst);}Ok(())}
#[tauri::command]
pub async fn media_prepare(id:String,asset_id:String,kind:String,resolution:String,width:u32,state:tauri::State<'_,Arc<Projects>>,engine:tauri::State<'_,Arc<VideoEngine>>,core:tauri::State<'_,Arc<Core>>)->Result<MediaTask> {
    let p=state.creative_snapshot(&id)?;let asset=p.assets.iter().find(|a|a.id==asset_id).ok_or("creative_source")?;
    if !["proxy","wave"].contains(&kind.as_str())||asset.kind=="image"||kind=="proxy"&&asset.kind!="video"||!["half","quarter","auto","custom"].contains(&resolution.as_str())||!(64..=1920).contains(&width){return Err("creative_parameters".into());}
    let e=engine.inner().clone();let paths=core.storage_paths()?;
    if e.media.busy.compare_exchange(false,true,Ordering::SeqCst,Ordering::SeqCst).is_err(){return Err("video_busy".into());}e.media.cancel.store(false,Ordering::SeqCst);
    let task=MediaTask{id:uuid(),asset_id:asset_id.clone(),name:asset.name.clone(),kind:kind.clone(),status:"running".into(),progress:0.,error:None};*e.media.task.lock().map_err(err)?=Some(task.clone());
    thread::spawn(move||{
        let result=(||{
            let _admission=crate::resources::shared().acquire(&asset_id,"media",512*1024*1024,false,&e.media.cancel).map_err(|v|if v=="resource_cancelled"{"video_cancelled".into()}else{v})?;
            let source=e.source(&p,&asset_id)?;let(runtime,_runtime)=e.runtime()?;let metadata=probe(&runtime,&source.1)?;if metadata.duration<=0.||metadata.duration>3600.{return Err("video_range".into());}
            if kind=="wave"&&!metadata.audio{return Err("video_no_audio".into());}
            let base=if kind=="proxy"{&paths.proxies}else{&paths.cache};let mut work=Work::new(base,"video-media")?;work.files=vec!["media.log".into(),"wave.pcm".into(),"proxy.mp4".into(),"wave.json".into()];
            let mut c=command(&runtime.join("ffmpeg.exe"));c.args(["-hide_banner","-nostdin","-v","error","-n","-max_alloc","268435456","-protocol_whitelist","file,pipe","-format_whitelist","mov,matroska,webm,mp3,wav,flac,ogg","-threads","2","-i"]).arg(&source.1).args(["-map_metadata","-1","-threads","2","-progress","pipe:1","-nostats"]);
            let (target,cache_id)=if kind=="proxy"{
                let wanted=match resolution.as_str(){"half"=>metadata.width/2,"quarter"=>metadata.width/4,"custom"=>width,_=>metadata.width.min(960)}.clamp(2,1920);let wanted=(wanted/2)*2;
                c.args(["-map","0:v:0","-an","-vf",&format!("scale={wanted}:-2:reset_sar=1"),"-c:v","libopenh264","-b:v","3000k","-g","15","-pix_fmt","yuv420p","-movflags","+faststart"]).arg(work.path.join("proxy.mp4"));
                ("proxy.mp4",format!("proxy:{}:{wanted}",source.0.sha256))
            }else{c.args(["-map","0:a:0","-vn","-ac","1","-ar","48000","-f","f32le"]).arg(work.path.join("wave.pcm"));("wave.json",format!("wave:{}",source.0.sha256))};
            run(c,&work.path.join("media.log"),Duration::from_secs(7200),||e.media.cancel.load(Ordering::SeqCst),|time|{if let Ok(mut task)=e.media.task.lock(){if let Some(t)=task.as_mut(){t.progress=(time/metadata.duration).clamp(0.,0.98);}}})?;
            let payload=if kind=="proxy"{let check=probe(&runtime,&work.path.join(target))?;if !check.video||check.duration<metadata.duration-0.15{return Err("video_encode".into());}serde_json::json!({"width":check.width,"height":check.height,"duration":check.duration})}else{
                let f=File::open(work.path.join("wave.pcm")).map_err(err)?;let count=f.metadata().map_err(err)?.len()/4;let mut f=BufReader::new(f);if count>172_800_000{return Err("creative_limit".into());}let bin=(count.div_ceil(2400)).max(1);let mut peaks=vec![];let mut max=0f32;let mut n=0u64;let mut bytes=[0;4];while f.read_exact(&mut bytes).is_ok(){let value=f32::from_le_bytes(bytes);if value.is_finite(){max=max.max(value.abs().min(1.));}n+=1;if n%bin==0{peaks.push(max);max=0.;}}if n%bin!=0{peaks.push(max);}let wave=Waveform{duration:metadata.duration,peaks};let payload=serde_json::to_value(wave).map_err(err)?;fs::write(work.path.join(target),serde_json::to_vec(&payload).map_err(err)?).map_err(err)?;payload
            };
            if e.media.cancel.load(Ordering::SeqCst){return Err("video_cancelled".into());}
            e.register(format!("{cache_id}:{}",uuid()),&kind,&source.0.sha256,&work.path.join(target),payload)?;work.files.retain(|f|f!=target);Ok::<(),String>(())
        })();
        if let Ok(mut task)=e.media.task.lock(){if let Some(t)=task.as_mut(){match result{Ok(())=>{t.status="completed".into();t.progress=1.;},Err(error)=>{t.status=if error=="video_cancelled"{"cancelled"}else{"failed"}.into();t.error=Some(error);}}}}
        e.media.busy.store(false,Ordering::SeqCst);
    });Ok(task)
}

#[cfg(test)]mod tests {
    use super::*;
    #[test]fn cleanup_protects_active_sources_changed_files_and_unknown_files(){
        let tmp=tempfile::tempdir().unwrap();let projects=Projects::new(tmp.path()).unwrap();projects.new_project(tmp.path(),"cache".into(),None,false).unwrap();
        let original=tmp.path().join("source.wav");fs::write(&original,b"original audio").unwrap();let p=projects.add(vec![original.to_string_lossy().into()]).unwrap();
        let e=VideoEngine::new(tmp.path(),tmp.path().join("runtime")).unwrap();let cached=tmp.path().join("cache.bin");let unknown=tmp.path().join("keep.bin");fs::write(&cached,b"cache bytes").unwrap();fs::write(&unknown,b"user bytes").unwrap();
        e.register("known".into(),"proxy",&p.assets[0].sha256,&cached,serde_json::Value::Null).unwrap();let cutoff=chrono::Utc::now().timestamp()+1;
        assert!(e.cleanup_inventory(cutoff,&projects).unwrap().files.is_empty());projects.close(true).unwrap();
        let inventory=e.cleanup_inventory(cutoff,&projects).unwrap();assert_eq!(inventory.files.len(),1);let candidate=&inventory.files[0];
        fs::write(&cached,b"changed!!!!").unwrap();assert!(e.cleanup_file(candidate,cutoff,&projects).is_err());assert!(cached.exists());assert_eq!(fs::read(&unknown).unwrap(),b"user bytes");assert_eq!(fs::read(&original).unwrap(),b"original audio");
        e.register("known".into(),"proxy",&p.assets[0].sha256,&cached,serde_json::Value::Null).unwrap();let next=e.cleanup_inventory(cutoff,&projects).unwrap();assert!(e.cleanup_file(&next.files[0],cutoff,&projects).unwrap());assert!(!cached.exists());assert!(unknown.exists());
    }
    #[test]fn frozen_pose_interpolates_fades_and_excludes_inactive_audio(){
        let id=uuid();let t=Timeline{width:640,height:360,fps:30,background:"#000000".into(),tracks:vec![Track{id:id.clone(),name:"Video".into(),audio:false,muted:false,locked:false}],clips:vec![Clip{id:uuid(),asset_id:uuid(),track_id:id,name:"c".into(),start:2.,source_in:10.,source_out:14.,speed:2.,fade_in:1.,fade_out:0.,blur:0.,brightness:0.,contrast:1.,saturation:1.,keyframes:vec![Keyframe{time:0.,x:0.,y:0.,scale:1.,rotation:0.,opacity:1.,volume:1.},Keyframe{time:2.,x:100.,y:0.,scale:2.,rotation:90.,opacity:0.5,volume:1.}]}],captions:vec![]};
        assert!(freeze(&t,1.).clips.is_empty());let f=freeze(&t,2.5);let c=&f.clips[0];assert_eq!(c.source_in,11.);assert_eq!(c.keyframes[0].x,25.);assert_eq!(c.keyframes[0].opacity,0.4375);assert!(freeze(&t,4.).clips.is_empty());
    }
}
