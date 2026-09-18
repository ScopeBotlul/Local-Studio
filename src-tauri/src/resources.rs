//! Shared admission control for expensive local work. Waiting never occupies the UI thread.
use serde::Serialize;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, OnceLock,
};
use std::time::{Duration, Instant};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceTask {
    pub id: String,
    pub kind: String,
    pub state: String,
    pub ram_bytes: u64,
    pub gpu: bool,
}
#[derive(Default)]
struct State {
    tasks: Vec<ResourceTask>,
    parallel: bool,
}
#[derive(Default)]
pub struct Resources {
    state: Mutex<State>,
}
static INSTANCE: OnceLock<Arc<Resources>> = OnceLock::new();
pub fn shared() -> Arc<Resources> {
    INSTANCE
        .get_or_init(|| Arc::new(Resources::default()))
        .clone()
}
pub fn configure(parallel: bool) {
    if let Ok(mut s) = shared().state.lock() {
        s.parallel = parallel;
    }
}
pub struct Lease {
    manager: Arc<Resources>,
    id: String,
}
impl Drop for Lease {
    fn drop(&mut self) {
        if let Ok(mut s) = self.manager.state.lock() {
            s.tasks.retain(|t| t.id != self.id);
        }
    }
}

fn admitted(s: &State, id: &str, available: u64) -> bool {
    let Some(task) = s.tasks.iter().find(|t| t.id == id) else {
        return false;
    };
    if s.tasks
        .iter()
        .find(|t| t.state == "waiting")
        .is_none_or(|t| t.id != id)
    {
        return false;
    }
    let active: Vec<_> = s.tasks.iter().filter(|t| t.state == "running").collect();
    if active.len() >= if s.parallel { 2 } else { 1 } {
        return false;
    }
    if task.gpu && active.iter().any(|t| t.gpu) {
        return false;
    }
    let reserved = active
        .iter()
        .fold(0u64, |v, t| v.saturating_add(t.ram_bytes));
    available.saturating_sub(reserved) >= task.ram_bytes.saturating_add(512 * 1024 * 1024)
}
impl Resources {
    pub fn acquire(
        self: &Arc<Self>,
        id: &str,
        kind: &str,
        ram_bytes: u64,
        gpu: bool,
        cancel: &AtomicBool,
    ) -> Result<Lease, String> {
        let ticket = format!("{kind}:{id}");
        {
            let mut s = self.state.lock().map_err(|_| "resource_state")?;
            if s.tasks.iter().any(|t| t.id == ticket) {
                return Err("resource_busy".into());
            }
            s.tasks.push(ResourceTask {
                id: ticket.clone(),
                kind: kind.into(),
                state: "waiting".into(),
                ram_bytes,
                gpu,
            });
        }
        let guard = Lease {
            manager: self.clone(),
            id: ticket.clone(),
        };
        let begin = Instant::now();
        let mut memory = sysinfo::System::new();
        loop {
            if cancel.load(Ordering::SeqCst) {
                return Err("resource_cancelled".into());
            }
            if begin.elapsed() > Duration::from_secs(1800) {
                return Err("resource_timeout".into());
            }
            memory.refresh_memory();
            let available = memory.available_memory();
            {
                let mut s = self.state.lock().map_err(|_| "resource_state")?;
                if admitted(&s, &ticket, available) {
                    s.tasks.iter_mut().find(|t| t.id == ticket).unwrap().state = "running".into();
                    return Ok(guard);
                }
                if !s.tasks.iter().any(|t| t.state == "running")
                    && available < ram_bytes.saturating_add(512 * 1024 * 1024)
                {
                    return Err("resource_memory".into());
                }
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }
}
#[tauri::command]
pub fn resource_status() -> Result<Vec<ResourceTask>, String> {
    Ok(shared()
        .state
        .lock()
        .map_err(|_| "resource_state")?
        .tasks
        .clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn task(id: &str, state: &str, gpu: bool) -> ResourceTask {
        ResourceTask {
            id: id.into(),
            kind: "test".into(),
            state: state.into(),
            ram_bytes: 1024 * 1024,
            gpu,
        }
    }
    #[test]
    fn admission_is_fifo_serial_by_default_and_memory_aware() {
        let mut s = State {
            parallel: false,
            tasks: vec![
                task("a", "running", true),
                task("b", "waiting", false),
                task("c", "waiting", false),
            ],
        };
        assert!(!admitted(&s, "b", u64::MAX));
        s.parallel = true;
        assert!(admitted(&s, "b", u64::MAX));
        assert!(!admitted(&s, "c", u64::MAX));
        assert!(!admitted(&s, "b", 1));
        s.tasks[1].gpu = true;
        assert!(!admitted(&s, "b", u64::MAX));
        s.tasks[0].gpu = false;
        assert!(admitted(&s, "b", u64::MAX));
    }
    #[test]
    fn cancelled_waiter_is_removed_and_lease_releases_capacity() {
        let r = Arc::new(Resources::default());
        let cancel = AtomicBool::new(true);
        assert!(r.acquire("a", "test", 0, false, &cancel).is_err());
        assert!(r.state.lock().unwrap().tasks.is_empty());
        r.state
            .lock()
            .unwrap()
            .tasks
            .push(task("a", "running", true));
        let lease = Lease {
            manager: r.clone(),
            id: "a".into(),
        };
        drop(lease);
        assert!(r.state.lock().unwrap().tasks.is_empty());
    }
}
