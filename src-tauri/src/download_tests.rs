use super::*;
use std::{net::TcpListener, sync::atomic::{AtomicBool, Ordering}, thread};

struct Server { url:String, ranges:Arc<Mutex<Vec<u64>>>, stop:Arc<AtomicBool>, thread:Option<thread::JoinHandle<()>> }
impl Server {
    fn new(data:Vec<u8>, support_range:bool) -> Self {
        let listener=TcpListener::bind("127.0.0.1:0").unwrap();listener.set_nonblocking(true).unwrap();
        let url=format!("http://{}/file",listener.local_addr().unwrap());
        let stop=Arc::new(AtomicBool::new(false));let worker_stop=stop.clone();let ranges=Arc::new(Mutex::new(Vec::new()));let worker_ranges=ranges.clone();
        let handle=thread::spawn(move|| {
            while !worker_stop.load(Ordering::SeqCst) {
                let Ok((mut stream,_))=listener.accept() else {thread::sleep(Duration::from_millis(5));continue};
                // Accepted sockets inherit nonblocking mode on Windows. Read complete headers.
                stream.set_nonblocking(false).unwrap();
                stream.set_read_timeout(Some(Duration::from_secs(2))).unwrap();stream.set_write_timeout(Some(Duration::from_millis(250))).unwrap();
                let mut request=Vec::new();let mut buffer=[0;1024];
                while !request.windows(4).any(|v|v==b"\r\n\r\n") {match stream.read(&mut buffer){Ok(0)|Err(_)=>break,Ok(n)=>request.extend_from_slice(&buffer[..n])}}
                let request=String::from_utf8_lossy(&request).to_ascii_lowercase();
                let requested=request.lines().find_map(|l|l.strip_prefix("range: bytes=").and_then(|v|v.trim().strip_suffix('-')).and_then(|v|v.parse::<u64>().ok())).unwrap_or(0);
                worker_ranges.lock().unwrap().push(requested);
                let offset=if support_range {requested as usize}else{0};
                let range=if offset>0 {format!("Content-Range: bytes {}-{}/{}\r\n",offset,data.len()-1,data.len())}else{String::new()};
                let status=if offset>0 {"206 Partial Content"}else{"200 OK"};
                if write!(stream,"HTTP/1.1 {status}\r\nContent-Length: {}\r\n{range}Connection: close\r\n\r\n",data.len()-offset).is_err(){continue;}
                for chunk in data[offset..].chunks(8192) {
                    if worker_stop.load(Ordering::SeqCst)||stream.write_all(chunk).is_err(){break;}thread::sleep(Duration::from_millis(12));
                }
            }
        });Self{url,ranges,stop,thread:Some(handle)}
    }
}
impl Drop for Server {fn drop(&mut self){self.stop.store(true,Ordering::SeqCst);self.thread.take().unwrap().join().unwrap();}}
fn fixture(dir:&Path,server:&Server,data:&[u8])->(Arc<Downloads>,Download) {
    let config=dir.join("config");directory(&config).unwrap();let manager=Downloads::new(&config,HfAuth::new(&config)).unwrap();
    let id=uuid::Uuid::new_v4().to_string();let job=Download {
        id:id.clone(),repo:"test/fixture".into(),revision:"a".repeat(40),license:Some("test-fixture".into()),task:None,
        files:vec![DownloadFile{path:"weights/data.bin".into(),size:data.len() as u64,sha256:Some(format!("{:x}",Sha256::digest(data))),git_sha1:None,downloaded:0,actual_sha256:None}],
        destination:dir.join("models").join(format!("hf-{id}")).to_string_lossy().into(),partial_directory:dir.join("downloads").join(&id).to_string_lossy().into(),status:"queued".into(),priority:0,
        total_bytes:data.len() as u64,downloaded_bytes:0,bytes_per_second:0,error:None,created_at:crate::database::now(),verify_only:false,test_url:Some(server.url.clone()),
    };
    manager.state.lock().unwrap().plans.insert(id.clone(),(Instant::now(),Plan{id,download:job.clone(),additional_bytes:job.total_bytes*2,available_bytes:u64::MAX}));(manager,job)
}
async fn wait(manager:&Downloads,id:&str,condition:impl Fn(&Download)->bool)->Download {
    for _ in 0..800 {let job=manager.list().unwrap().into_iter().find(|j|j.id==id).unwrap();if condition(&job){return job;}tokio::time::sleep(Duration::from_millis(10)).await;}
    panic!("Download did not reach expected state: {:?}",manager.list().unwrap().iter().map(|j|(&j.status,&j.error)).collect::<Vec<_>>());
}
#[tokio::test]
async fn pause_stops_disk_writes_and_resume_uses_actual_range_after_restart() {
    let dir=tempfile::tempdir().unwrap();let data:Vec<_>=(0..512*1024).map(|i|(i%251) as u8).collect();let server=Server::new(data.clone(),true);
    let (manager,job)=fixture(dir.path(),&server,&data);manager.start(&job.id).unwrap();
    wait(&manager,&job.id,|j|j.downloaded_bytes>0 && j.status=="downloading").await;
    manager.action(&job.id,"pause",None).unwrap();let paused=wait(&manager,&job.id,|j|j.status=="paused").await;
    let part=Path::new(&job.partial_directory).join("0.part");let length=fs::metadata(&part).unwrap().len();assert!(length>0 && length<job.total_bytes);
    tokio::time::sleep(Duration::from_millis(150)).await;assert_eq!(fs::metadata(&part).unwrap().len(),length);assert_eq!(paused.downloaded_bytes,length);
    manager.shutdown().await;drop(manager);
    let config=dir.path().join("config");let reopened=Downloads::new(&config,HfAuth::new(&config)).unwrap();
    reopened.state.lock().unwrap().jobs[0].test_url=Some(server.url.clone());
    assert_eq!(reopened.list().unwrap()[0].status,"paused");reopened.action(&job.id,"resume",None).unwrap();
    let complete=wait(&reopened,&job.id,|j|j.status=="completed"||j.status=="failed").await;assert_eq!(complete.status,"completed", "{:?}",complete.error);
    assert_eq!(fs::read(Path::new(&job.destination).join("weights/data.bin")).unwrap(),data);assert!(!part.exists());
    assert!(server.ranges.lock().unwrap().contains(&length));
    // Completed local bytes are checked again, without another HTTP request.
    let requests=server.ranges.lock().unwrap().len();reopened.action(&job.id,"verify",None).unwrap();wait(&reopened,&job.id,|j|j.status=="completed").await;assert_eq!(server.ranges.lock().unwrap().len(),requests);
    fs::write(Path::new(&job.destination).join("weights/data.bin"),vec![0;data.len()]).unwrap();reopened.action(&job.id,"verify",None).unwrap();let invalid=wait(&reopened,&job.id,|j|j.status=="invalid").await;assert_eq!(invalid.error.as_deref(),Some("download_hash"));reopened.shutdown().await;
}
#[tokio::test]
async fn unsupported_resume_preserves_partial_until_explicit_restart() {
    let dir=tempfile::tempdir().unwrap();let data=vec![42;128*1024];let server=Server::new(data.clone(),false);let(manager,job)=fixture(dir.path(),&server,&data);
    directory(Path::new(&job.partial_directory)).unwrap();let part=Path::new(&job.partial_directory).join("0.part");fs::write(&part,&data[..32768]).unwrap();
    manager.start(&job.id).unwrap();let failed=wait(&manager,&job.id,|j|j.status=="failed").await;assert_eq!(failed.error.as_deref(),Some("download_resume_unsupported"));assert_eq!(fs::metadata(&part).unwrap().len(),32768);
    manager.action(&job.id,"restart",None).unwrap();wait(&manager,&job.id,|j|j.status=="completed").await;assert_eq!(fs::read(Path::new(&job.destination).join("weights/data.bin")).unwrap(),data);manager.shutdown().await;
}
#[tokio::test]
async fn cancellation_keeps_partial_and_bad_hash_never_publishes() {
    let dir=tempfile::tempdir().unwrap();let data=vec![9;512*1024];let server=Server::new(data.clone(),true);let(manager,job)=fixture(dir.path(),&server,&data);
    manager.state.lock().unwrap().plans.get_mut(&job.id).unwrap().1.download.files[0].sha256=Some("0".repeat(64));
    manager.start(&job.id).unwrap();wait(&manager,&job.id,|j|j.downloaded_bytes>0).await;manager.action(&job.id,"cancel",None).unwrap();wait(&manager,&job.id,|j|j.status=="cancelled").await;
    assert!(Path::new(&job.partial_directory).join("0.part").is_file());assert!(!Path::new(&job.destination).exists());
    manager.action(&job.id,"resume",None).unwrap();let failed=wait(&manager,&job.id,|j|j.status=="failed").await;assert_eq!(failed.error.as_deref(),Some("download_hash"));assert!(!Path::new(&job.destination).exists());manager.shutdown().await;
}
#[test]
fn interrupted_jobs_recover_paused_and_old_destination_survives() {
    let dir=tempfile::tempdir().unwrap();let server=Server::new(vec![1;100],true);let(manager,mut job)=fixture(dir.path(),&server,&vec![1;100]);
    job.status="installing".into();directory(Path::new(&job.destination)).unwrap();fs::write(Path::new(&job.destination).join("original"),b"keep").unwrap();
    save(&manager.state.lock().unwrap().db,&job).unwrap();drop(manager);
    let config=dir.path().join("config");let reopened=Downloads::new(&config,HfAuth::new(&config)).unwrap();assert_eq!(reopened.list().unwrap()[0].status,"paused");assert_eq!(fs::read(Path::new(&job.destination).join("original")).unwrap(),b"keep");assert!(server.ranges.lock().unwrap().is_empty());
}

#[tokio::test]
async fn queued_priority_selects_next_transfer_without_preempting_current() {
    let dir=tempfile::tempdir().unwrap();let data=vec![7;512*1024];let server=Server::new(data.clone(),true);let(manager,first)=fixture(dir.path(),&server,&data);
    manager.start(&first.id).unwrap();
    let mut queued=Vec::new();
    for _ in 0..2 {
        let mut next=first.clone();next.id=uuid::Uuid::new_v4().to_string();next.repo=format!("fixture/{}",next.id);next.destination=dir.path().join("models").join(format!("hf-{}",next.id)).to_string_lossy().into();next.partial_directory=dir.path().join("downloads").join(&next.id).to_string_lossy().into();
        manager.state.lock().unwrap().plans.insert(next.id.clone(),(Instant::now(),Plan{id:next.id.clone(),download:next.clone(),additional_bytes:next.total_bytes*2,available_bytes:u64::MAX}));manager.start(&next.id).unwrap();queued.push(next.id);
    }
    manager.action(&queued[1],"priority",Some(2)).unwrap();
    assert_eq!(manager.state.lock().unwrap().active.as_ref().unwrap().0,first.id);
    manager.action(&first.id,"cancel",None).unwrap();
    wait(&manager,&queued[1],|j|j.status=="completed").await;
    assert_ne!(manager.list().unwrap().iter().find(|j|j.id==queued[0]).unwrap().status,"completed");
    wait(&manager,&queued[0],|j|j.status=="completed").await;manager.shutdown().await;
}
