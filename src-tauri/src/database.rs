use crate::types::{Job, Settings};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;

pub struct Database {
    connection: Connection,
}

pub fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

impl Database {
    pub fn open(path: &Path, defaults: &Settings) -> Result<Self, String> {
        let connection = Connection::open(path).map_err(|e| e.to_string())?;
        connection
            .busy_timeout(std::time::Duration::from_secs(5))
            .map_err(|e| e.to_string())?;
        connection.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL; PRAGMA foreign_keys=ON;
            CREATE TABLE IF NOT EXISTS metadata (key TEXT PRIMARY KEY, value TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS settings (id INTEGER PRIMARY KEY CHECK(id=1), json TEXT NOT NULL);
            CREATE TABLE IF NOT EXISTS jobs (
                id TEXT PRIMARY KEY, kind TEXT NOT NULL, input_path TEXT NOT NULL, status TEXT NOT NULL,
                progress REAL, created_at TEXT NOT NULL, started_at TEXT, finished_at TEXT, result TEXT, error TEXT
            );
            CREATE INDEX IF NOT EXISTS jobs_queue ON jobs(status, created_at);
            CREATE INDEX IF NOT EXISTS jobs_history ON jobs(created_at) WHERE status NOT IN ('queued','running');
            PRAGMA user_version=1;").map_err(|e| e.to_string())?;
        let mut db = Self { connection };
        db.connection
            .execute(
                "INSERT OR IGNORE INTO settings(id,json) VALUES(1,?1)",
                [serde_json::to_string(defaults).map_err(|e| e.to_string())?],
            )
            .map_err(|e| e.to_string())?;
        let unclean = db.meta("clean_exit")?.as_deref() == Some("false");
        let tx = db.connection.transaction().map_err(|e| e.to_string())?;
        let interrupted = tx.execute("UPDATE jobs SET status='interrupted', finished_at=?1, error='The application stopped before this job finished. Start a new job to verify the file again.' WHERE status IN ('running','queued')", [now()]).map_err(|e| e.to_string())?;
        if unclean || interrupted > 0 {
            tx.execute("INSERT INTO metadata(key,value) VALUES('recovery_available','true') ON CONFLICT(key) DO UPDATE SET value='true'", []).map_err(|e| e.to_string())?;
        }
        tx.execute("INSERT INTO metadata(key,value) VALUES('clean_exit','false') ON CONFLICT(key) DO UPDATE SET value='false'", []).map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(db)
    }

    pub fn settings(&self) -> Result<Settings, String> {
        let json: String = self
            .connection
            .query_row("SELECT json FROM settings WHERE id=1", [], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        serde_json::from_str(&json).map_err(|e| format!("Saved settings are invalid: {e}"))
    }

    pub fn save_settings(&self, settings: &Settings) -> Result<(), String> {
        let json = serde_json::to_string(settings).map_err(|e| e.to_string())?;
        self.connection
            .execute("UPDATE settings SET json=?1 WHERE id=1", [json])
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn meta(&self, key: &str) -> Result<Option<String>, String> {
        self.connection
            .query_row("SELECT value FROM metadata WHERE key=?1", [key], |r| {
                r.get(0)
            })
            .optional()
            .map_err(|e| e.to_string())
    }

    pub fn set_meta(&self, key: &str, value: &str) -> Result<(), String> {
        self.connection.execute("INSERT INTO metadata(key,value) VALUES(?1,?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value", params![key, value]).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn insert_job(&self, path: &Path) -> Result<Job, String> {
        let job = Job {
            id: uuid::Uuid::new_v4().to_string(),
            kind: "sha256".into(),
            input_path: path.to_string_lossy().into_owned(),
            status: "queued".into(),
            progress: None,
            created_at: now(),
            started_at: None,
            finished_at: None,
            result: None,
            error: None,
        };
        self.connection
            .execute(
                "INSERT INTO jobs(id,kind,input_path,status,created_at) VALUES(?1,?2,?3,?4,?5)",
                params![job.id, job.kind, job.input_path, job.status, job.created_at],
            )
            .map_err(|e| e.to_string())?;
        Ok(job)
    }

    fn row_job(row: &rusqlite::Row<'_>) -> rusqlite::Result<Job> {
        Ok(Job {
            id: row.get(0)?,
            kind: row.get(1)?,
            input_path: row.get(2)?,
            status: row.get(3)?,
            progress: row.get(4)?,
            created_at: row.get(5)?,
            started_at: row.get(6)?,
            finished_at: row.get(7)?,
            result: row.get(8)?,
            error: row.get(9)?,
        })
    }

    pub fn jobs(&self) -> Result<Vec<Job>, String> {
        // Only cap finished history: every active job must stay visible and cancellable.
        let mut statement = self.connection.prepare(
            "SELECT id,kind,input_path,status,progress,created_at,started_at,finished_at,result,error
             FROM jobs WHERE rowid IN (
                 SELECT rowid FROM jobs WHERE status IN ('queued','running')
                 UNION ALL
                 SELECT rowid FROM (
                     SELECT rowid FROM jobs WHERE status NOT IN ('queued','running')
                     ORDER BY created_at DESC, rowid DESC LIMIT 250
                 )
             ) ORDER BY created_at DESC, rowid DESC",
        ).map_err(|e| e.to_string())?;
        let rows = statement
            .query_map([], Self::row_job)
            .map_err(|e| e.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    }

    pub fn job(&self, id: &str) -> Result<Option<Job>, String> {
        self.connection.query_row("SELECT id,kind,input_path,status,progress,created_at,started_at,finished_at,result,error FROM jobs WHERE id=?1", [id], Self::row_job).optional().map_err(|e| e.to_string())
    }

    pub fn next_job(&self) -> Result<Option<Job>, String> {
        self.connection.query_row("SELECT id,kind,input_path,status,progress,created_at,started_at,finished_at,result,error FROM jobs WHERE status='queued' ORDER BY created_at,rowid LIMIT 1", [], Self::row_job).optional().map_err(|e| e.to_string())
    }

    pub fn start_job(&self, id: &str) -> Result<(), String> {
        self.connection.execute("UPDATE jobs SET status='running',started_at=?2,progress=0 WHERE id=?1 AND status='queued'", params![id, now()]).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn progress(&self, id: &str, progress: f64) -> Result<(), String> {
        self.connection
            .execute(
                "UPDATE jobs SET progress=?2 WHERE id=?1 AND status='running'",
                params![id, progress.clamp(0.0, 1.0)],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn finish(
        &self,
        id: &str,
        status: &str,
        result: Option<&str>,
        error: Option<&str>,
    ) -> Result<(), String> {
        self.connection.execute("UPDATE jobs SET status=?2,finished_at=?3,result=?4,error=?5,progress=CASE WHEN ?2='completed' THEN 1 ELSE progress END WHERE id=?1 AND status IN ('queued','running')", params![id, status, now(), result, error]).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn clean_shutdown(&mut self) -> Result<(), String> {
        let tx = self.connection.transaction().map_err(|e| e.to_string())?;
        tx.execute("UPDATE jobs SET status='interrupted',finished_at=?1,error='Application closed before completion. Start a new job to verify the file again.' WHERE status IN ('queued','running')", [now()]).map_err(|e| e.to_string())?;
        tx.execute("INSERT INTO metadata(key,value) VALUES('clean_exit','true') ON CONFLICT(key) DO UPDATE SET value='true'", []).map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn settings_persist_when_data_root_changes_and_jobs_recover() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.sqlite");
        let mut settings = crate::settings::defaults(&dir.path().join("first"));
        let id;
        {
            let db = Database::open(&path, &settings).unwrap();
            settings.data_root = dir.path().join("second").to_string_lossy().into_owned();
            settings.language = "de".into();
            db.save_settings(&settings).unwrap();
            id = db.insert_job(&dir.path().join("input.bin")).unwrap().id;
            db.start_job(&id).unwrap();
        }
        let mut db = Database::open(&path, &settings).unwrap();
        assert_eq!(db.settings().unwrap(), settings);
        assert_eq!(db.job(&id).unwrap().unwrap().status, "interrupted");
        assert_eq!(
            db.meta("recovery_available").unwrap().as_deref(),
            Some("true")
        );
        db.set_meta("recovery_available", "false").unwrap();
        db.clean_shutdown().unwrap();
        drop(db);
        let db = Database::open(&path, &settings).unwrap();
        assert_eq!(
            db.meta("recovery_available").unwrap().as_deref(),
            Some("false")
        );
    }

    #[test]
    fn cancelled_jobs_cannot_be_completed_by_late_worker_output() {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::open(
            &dir.path().join("db.sqlite"),
            &crate::settings::defaults(dir.path()),
        )
        .unwrap();
        let job = db.insert_job(&dir.path().join("file")).unwrap();
        db.start_job(&job.id).unwrap();
        db.finish(&job.id, "cancelled", None, None).unwrap();
        db.finish(&job.id, "completed", Some("incorrect late result"), None)
            .unwrap();
        let current = db.job(&job.id).unwrap().unwrap();
        assert_eq!(current.status, "cancelled");
        assert_eq!(current.result, None);
    }

    #[test]
    fn jobs_keep_all_active_jobs_and_only_the_latest_250_finished_jobs() {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::open(
            Path::new(":memory:"),
            &crate::settings::defaults(dir.path()),
        )
        .unwrap();
        let mut active_ids = Vec::new();
        for _ in 0..301 {
            let job = db.insert_job(&dir.path().join("input.bin")).unwrap();
            db.connection
                .execute(
                    "UPDATE jobs SET created_at='2026-01-01T00:00:00.000Z' WHERE id=?1",
                    [&job.id],
                )
                .unwrap();
            active_ids.push(job.id);
        }
        db.start_job(&active_ids[0]).unwrap();

        let mut finished_ids = Vec::new();
        for index in 0..300 {
            let job = db.insert_job(&dir.path().join("input.bin")).unwrap();
            // Insert the newest timestamps first to exercise both date and rowid ordering.
            let created_at = if index < 150 {
                "2026-01-03T00:00:00.000Z"
            } else {
                "2026-01-02T00:00:00.000Z"
            };
            db.connection
                .execute(
                    "UPDATE jobs SET created_at=?2 WHERE id=?1",
                    params![job.id, created_at],
                )
                .unwrap();
            db.finish(
                &job.id,
                ["completed", "failed", "cancelled", "interrupted"][index % 4],
                None,
                None,
            )
            .unwrap();
            finished_ids.push(job.id);
        }

        let jobs = db.jobs().unwrap();
        assert_eq!(jobs.len(), 551);
        let ids: Vec<_> = jobs.iter().map(|job| &job.id).collect();
        assert_eq!(
            ids.iter()
                .copied()
                .collect::<std::collections::HashSet<_>>()
                .len(),
            ids.len(),
            "a job must appear only once"
        );
        let expected_ids: Vec<_> = finished_ids[..150]
            .iter()
            .rev()
            .chain(finished_ids[200..].iter().rev())
            .chain(active_ids.iter().rev())
            .collect();
        assert_eq!(ids, expected_ids);
        assert_eq!(jobs.last().unwrap().status, "running");
        assert_eq!(db.next_job().unwrap().unwrap().id, active_ids[1]);

        // The UI can cancel every active job using an ID from the displayed list.
        for job in jobs
            .iter()
            .filter(|job| matches!(job.status.as_str(), "queued" | "running"))
        {
            db.finish(&job.id, "cancelled", None, None).unwrap();
            assert_eq!(db.job(&job.id).unwrap().unwrap().status, "cancelled");
        }
        assert!(db.next_job().unwrap().is_none());
        assert_eq!(db.jobs().unwrap().len(), 250);
    }

    #[test]
    fn jobs_use_one_newest_first_order_across_active_and_finished_states() {
        let dir = tempfile::tempdir().unwrap();
        let db = Database::open(
            Path::new(":memory:"),
            &crate::settings::defaults(dir.path()),
        )
        .unwrap();
        let mut expected_ids = Vec::new();
        for status in ["running", "completed", "queued", "failed", "queued"] {
            let job = db.insert_job(&dir.path().join("input.bin")).unwrap();
            db.connection
                .execute(
                    "UPDATE jobs SET created_at='2026-01-01T00:00:00.000Z' WHERE id=?1",
                    [&job.id],
                )
                .unwrap();
            match status {
                "running" => db.start_job(&job.id).unwrap(),
                "queued" => {}
                _ => db.finish(&job.id, status, None, None).unwrap(),
            }
            expected_ids.push(job.id);
        }
        expected_ids.reverse();
        let ids: Vec<_> = db.jobs().unwrap().into_iter().map(|job| job.id).collect();
        assert_eq!(ids, expected_ids);
    }
}
