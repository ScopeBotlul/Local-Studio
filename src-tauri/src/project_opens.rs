use std::{collections::VecDeque,path::{Path,PathBuf},sync::{Arc,Mutex}};
use tauri::{Emitter,Manager};
pub struct ProjectOpens(Mutex<VecDeque<String>>);
impl ProjectOpens {
 pub fn new()->Arc<Self>{Arc::new(Self(Mutex::new(VecDeque::new())))}
 pub fn receive(&self,args:Vec<String>,cwd:&Path){
  // Association and CLI both accept exactly one project, never shell commands or URLs.
  if args.len()!=2{return;}let input=&args[1];if input.len()>32768||input.contains('\0'){return;}let path=PathBuf::from(input);
  if path.extension().and_then(|e|e.to_str()).is_none_or(|e|!e.eq_ignore_ascii_case("localstudio")){return;}
  let path=if path.is_absolute(){path}else{cwd.join(path)};if !path.is_absolute(){return;}
  if let Ok(mut q)=self.0.lock(){let value=path.to_string_lossy().into_owned();if q.len()<8&&!q.contains(&value){q.push_back(value);}}
 }
}
pub fn second(app:&tauri::AppHandle,args:Vec<String>,cwd:String){if let Some(q)=app.try_state::<Arc<ProjectOpens>>(){q.receive(args,Path::new(&cwd));let _=app.emit_to("main","project-open-requested",());}crate::desktop_features::restore(app);}
#[tauri::command]
pub fn project_take_open(state:tauri::State<'_,Arc<ProjectOpens>>)->Result<Option<String>,String>{Ok(state.0.lock().map_err(|_|"project_storage")?.front().cloned())}
#[cfg(test)]mod tests{use super::*;#[test]fn accepts_only_one_local_project_and_bounds_queue(){let q=ProjectOpens::new();for args in [vec!["app","--worker"],vec!["app","x.png"],vec!["app","x.localstudio","y.localstudio"]]{q.receive(args.into_iter().map(String::from).collect(),Path::new("D:/local"));}assert!(q.0.lock().unwrap().is_empty());q.receive(vec!["app".into(),"test.localstudio".into()],Path::new("D:/local"));assert_eq!(q.0.lock().unwrap().len(),1);}}

#[tauri::command]
pub fn project_ack_open(path:String,state:tauri::State<'_,Arc<ProjectOpens>>)->Result<(),String>{let mut q=state.0.lock().map_err(|_|"project_storage")?;if q.front()==Some(&path){q.pop_front();}Ok(())}
