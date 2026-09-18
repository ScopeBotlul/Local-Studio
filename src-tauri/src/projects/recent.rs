use super::*;
use rusqlite::params;
#[derive(Serialize)]
#[serde(rename_all="camelCase")]
pub struct Recent {path:String,name:String,opened_at:i64,available:bool}
pub(super) fn initialize(db:&Connection)->Result<()>{db.execute_batch("CREATE TABLE IF NOT EXISTS recent_projects(path TEXT PRIMARY KEY,name TEXT NOT NULL,opened_at INTEGER NOT NULL);").map_err(err)}
pub(super) fn remember(db:&Connection,p:&Project)->Result<()>{if let Some(path)=&p.path{db.execute("INSERT INTO recent_projects VALUES(?1,?2,?3) ON CONFLICT(path) DO UPDATE SET name=excluded.name,opened_at=excluded.opened_at",params![path,p.name,chrono::Utc::now().timestamp_millis()]).map_err(err)?;db.execute("DELETE FROM recent_projects WHERE path NOT IN (SELECT path FROM recent_projects ORDER BY opened_at DESC LIMIT 12)",[]).map_err(err)?;}Ok(())}
#[tauri::command]
pub async fn project_recent(state:tauri::State<'_,Arc<Projects>>)->Result<Vec<Recent>>{let p=state.inner().clone();tauri::async_runtime::spawn_blocking(move||{let s=p.state.lock().map_err(err)?;let mut stmt=s.db.prepare("SELECT path,name,opened_at FROM recent_projects ORDER BY opened_at DESC,path LIMIT 12").map_err(err)?;let rows=stmt.query_map([],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,i64>(2)?))).map_err(err)?;let mut result=vec![];for row in rows{let(path,name,opened_at)=row.map_err(err)?;let available=gallery::lock_file(Path::new(&path)).is_ok();result.push(Recent{path,name,opened_at,available});}Ok(result)}).await.map_err(err)?}
#[tauri::command]
pub async fn project_forget_recent(path:String,state:tauri::State<'_,Arc<Projects>>)->Result<()>{let p=state.inner().clone();tauri::async_runtime::spawn_blocking(move||{p.state.lock().map_err(err)?.db.execute("DELETE FROM recent_projects WHERE path=?1",[path]).map_err(err)?;Ok(())}).await.map_err(err)?}
