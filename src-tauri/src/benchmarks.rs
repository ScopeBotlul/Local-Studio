mod preferences;
pub use preferences::*;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};
type Result<T> = std::result::Result<T, String>;
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Benchmark {
    pub id: String,
    pub task: String,
    pub model_name: String,
    pub model_path: String,
    pub model_sha256: String,
    pub model_bytes: u64,
    pub runtime: String,
    pub device: String,
    pub cpu: String,
    pub logical_cores: usize,
    pub system_ram_bytes: u64,
    pub os: String,
    pub elapsed_ms: u64,
    pub peak_worker_ram_bytes: Option<u64>,
    pub peak_vram_bytes: Option<u64>,
    pub settings: serde_json::Value,
    pub created_at: String,
}
pub struct Benchmarks {
    db: Mutex<Connection>,
}
impl Benchmarks {
    pub fn new(config: &Path) -> Result<Arc<Self>> {
        let db =
            Connection::open(config.join("benchmarks.sqlite3")).map_err(|_| "benchmark_storage")?;
        db.busy_timeout(std::time::Duration::from_secs(5))
            .map_err(|_| "benchmark_storage")?;
        db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; CREATE TABLE IF NOT EXISTS benchmarks(id TEXT PRIMARY KEY,json TEXT NOT NULL); CREATE TABLE IF NOT EXISTS preferences(id TEXT PRIMARY KEY,json TEXT NOT NULL)").map_err(|_|"benchmark_storage")?;
        Ok(Arc::new(Self { db: Mutex::new(db) }))
    }
    pub fn image(
        &self,
        job: &crate::image_engine::ImageJob,
        peak_worker_ram_bytes: Option<u64>,
    ) -> Result<()> {
        if job.status != "completed" || job.elapsed_ms == 0 {
            return Err("benchmark_incomplete".into());
        }
        let mut system = sysinfo::System::new();
        system.refresh_cpu_all();
        system.refresh_memory();
        let request = &job.request;
        let record = Benchmark {
            id: job.id.clone(),
            task: if request.reference.as_ref().is_some_and(|r| r.mask.is_some()) {
                "inpainting"
            } else if request.reference.is_some() {
                "image-to-image"
            } else {
                "text-to-image"
            }
            .into(),
            model_name: Path::new(&request.model_path)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into(),
            model_path: request.model_path.clone(),
            model_sha256: job.model_sha256.clone().ok_or("benchmark_incomplete")?,
            model_bytes: job.model_bytes,
            runtime: job.runtime.clone(),
            device: job.device.clone(),
            cpu: system
                .cpus()
                .first()
                .map(|c| c.brand().to_owned())
                .unwrap_or_default(),
            logical_cores: system.cpus().len(),
            system_ram_bytes: system.total_memory(),
            os: sysinfo::System::long_os_version().unwrap_or_default(),
            elapsed_ms: job.elapsed_ms,
            peak_worker_ram_bytes,
            peak_vram_bytes: None,
            settings: serde_json::json!({"width":request.width,"height":request.height,"steps":request.steps,"actualSamplingSteps":job.sampling_steps,"guidance":request.guidance,"sampler":request.sampler,"scheduler":"karras","seed":request.seed,"vaeOnCpu":request.vae_on_cpu,"referenceSha256":request.reference.as_ref().map(|r|&r.sha256),"maskSha256":request.reference.as_ref().and_then(|r|r.mask.as_ref()).map(|m|&m.sha256),"strength":request.reference.as_ref().map(|r|r.strength)}),
            created_at: crate::database::now(),
        };
        self.insert(&record)
    }
    fn insert(&self, record: &Benchmark) -> Result<()> {
        let mut db = self.db.lock().map_err(|_| "benchmark_storage")?;
        let tx = db.transaction().map_err(|_| "benchmark_storage")?;
        let existed: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM benchmarks WHERE id=?1)",
                [&record.id],
                |r| r.get(0),
            )
            .map_err(|_| "benchmark_storage")?;
        if !existed {
            preferences::learn(&tx, record)?;
        }
        tx.execute(
            "INSERT OR REPLACE INTO benchmarks VALUES(?1,?2)",
            params![
                record.id,
                serde_json::to_string(record).map_err(|_| "benchmark_storage")?
            ],
        )
        .map_err(|_| "benchmark_storage")?;
        tx.execute("DELETE FROM benchmarks WHERE rowid NOT IN (SELECT rowid FROM benchmarks ORDER BY rowid DESC LIMIT 1000)",[]).map_err(|_|"benchmark_storage")?;
        tx.commit().map_err(|_| "benchmark_storage")?;
        Ok(())
    }
    pub fn list(&self) -> Result<Vec<Benchmark>> {
        let db = self.db.lock().map_err(|_| "benchmark_storage")?;
        let mut q = db
            .prepare("SELECT json FROM benchmarks ORDER BY rowid DESC")
            .map_err(|_| "benchmark_storage")?;
        let rows = q
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|_| "benchmark_storage")?;
        rows.map(|r| {
            serde_json::from_str(&r.map_err(|_| "benchmark_storage")?)
                .map_err(|_| "benchmark_storage".into())
        })
        .collect()
    }
}
#[tauri::command]
pub async fn benchmark_list(state: tauri::State<'_, Arc<Benchmarks>>) -> Result<Vec<Benchmark>> {
    let state = state.inner().clone();
    tauri::async_runtime::spawn_blocking(move || state.list())
        .await
        .map_err(|_| "benchmark_storage")?
}
#[tauri::command]
pub fn benchmark_clear(confirmed: bool, state: tauri::State<'_, Arc<Benchmarks>>) -> Result<()> {
    if !confirmed {
        return Err("benchmark_confirmation".into());
    }
    state
        .db
        .lock()
        .map_err(|_| "benchmark_storage")?
        .execute("DELETE FROM benchmarks", [])
        .map_err(|_| "benchmark_storage")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    pub(super) fn record() -> Benchmark {
        Benchmark {
            id: "job".into(),
            task: "text-to-image".into(),
            model_name: "model".into(),
            model_path: "D:\\model".into(),
            model_sha256: "a".repeat(64),
            model_bytes: 42,
            runtime: "test".into(),
            device: "test".into(),
            cpu: "test".into(),
            logical_cores: 1,
            system_ram_bytes: 10,
            os: "test".into(),
            elapsed_ms: 123,
            peak_worker_ram_bytes: Some(12),
            peak_vram_bytes: None,
            settings: serde_json::json!({"width":512,"height":512,"steps":3,"guidance":2,"sampler":"euler"}),
            created_at: "now".into(),
        }
    }
    #[test]
    fn measured_record_persists_without_prompts_and_deduplicates_job() {
        let t = tempfile::tempdir().unwrap();
        let b = Benchmarks::new(t.path()).unwrap();
        let r = record();
        b.insert(&r).unwrap();
        b.insert(&r).unwrap();
        assert_eq!(b.preferences().unwrap()[0].uses, 1);
        drop(b);
        let rows = Benchmarks::new(t.path()).unwrap().list().unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].elapsed_ms, 123);
        assert!(rows[0].peak_vram_bytes.is_none());
        assert!(!serde_json::to_string(&rows).unwrap().contains("prompt"));
    }
}
