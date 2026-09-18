use super::*;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelUpdate {
    pub download_id: String,
    pub repo: String,
    pub installed_revision: String,
    pub revision: Option<String>,
    pub status: String,
    pub changed_files: Vec<String>,
    pub missing_files: Vec<String>,
    pub total_bytes: Option<u64>,
    pub error: Option<String>,
}
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    pub checking: bool,
    pub checked_at: Option<String>,
    pub entries: Vec<ModelUpdate>,
    pub error: Option<String>,
}
fn compare(job: &Download, detail: &hub::ModelDetail) -> ModelUpdate {
    let mut entry = ModelUpdate {
        download_id: job.id.clone(),
        repo: job.repo.clone(),
        installed_revision: job.revision.clone(),
        revision: Some(detail.revision.clone()),
        status: "current".into(),
        changed_files: vec![],
        missing_files: vec![],
        total_bytes: Some(0),
        error: None,
    };
    for old in &job.files {
        match detail.files.iter().find(|f| f.path == old.path) {
            None => entry.missing_files.push(old.path.clone()),
            Some(file) => {
                entry.total_bytes = entry
                    .total_bytes
                    .and_then(|n| file.size.and_then(|s| n.checked_add(s)));
                let same = file.size == Some(old.size)
                    && if let Some(sha) = &file.sha256 {
                        old.actual_sha256
                            .as_ref()
                            .or(old.sha256.as_ref())
                            .is_some_and(|s| s.eq_ignore_ascii_case(sha))
                    } else {
                        file.git_sha1
                            .as_ref()
                            .zip(old.git_sha1.as_ref())
                            .is_some_and(|(a, b)| a.eq_ignore_ascii_case(b))
                    };
                if !same {
                    entry.changed_files.push(old.path.clone());
                }
            }
        }
    }
    if !entry.missing_files.is_empty() || entry.total_bytes.is_none() {
        entry.status = "review".into();
    } else if !entry.changed_files.is_empty() {
        entry.status = "available".into();
    }
    entry
}
impl Downloads {
    pub(super) fn load_updates(db: &Connection) -> Result<UpdateStatus> {
        db.execute_batch("CREATE TABLE IF NOT EXISTS model_updates(id INTEGER PRIMARY KEY CHECK(id=1),json TEXT NOT NULL)").map_err(|_|"download_storage")?;
        let mut status: UpdateStatus = db
            .query_row("SELECT json FROM model_updates WHERE id=1", [], |r| {
                r.get::<_, String>(0)
            })
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        status.checking = false;
        Ok(status)
    }
    fn check_updates(self: &Arc<Self>) -> Result<UpdateStatus> {
        let jobs = self.list()?;
        let mut seen = std::collections::HashSet::new();
        let jobs: Vec<_> = jobs
            .into_iter()
            .filter(|j| {
                j.status == "completed"
                    && seen.insert((
                        j.repo.clone(),
                        j.files.iter().map(|f| f.path.clone()).collect::<Vec<_>>(),
                    ))
            })
            .collect();
        {
            let mut state = self.updates.lock().map_err(|_| "download_storage")?;
            if state.checking {
                return Ok(state.clone());
            }
            state.checking = true;
            state.error = None;
            state.entries.clear();
        }
        let manager = self.clone();
        std::thread::spawn(move || {
            let result = (|| {
                let token = manager.auth.token()?;
                let mut details = HashMap::new();
                for job in jobs.iter().take(100) {
                    if manager
                        .state
                        .lock()
                        .map_err(|_| "download_storage")?
                        .stopped
                    {
                        break;
                    }
                    let detail = details.entry(job.repo.clone()).or_insert_with(|| {
                        hub::detail_metadata(
                            &job.repo,
                            "main",
                            token.as_ref().map(|s| s.as_str()),
                            false,
                        )
                    });
                    let entry = match detail {
                        Ok(detail) => compare(job, detail),
                        Err(error) => ModelUpdate {
                            download_id: job.id.clone(),
                            repo: job.repo.clone(),
                            installed_revision: job.revision.clone(),
                            revision: None,
                            status: "error".into(),
                            changed_files: vec![],
                            missing_files: vec![],
                            total_bytes: None,
                            error: Some(error.clone()),
                        },
                    };
                    manager
                        .updates
                        .lock()
                        .map_err(|_| "download_storage")?
                        .entries
                        .push(entry);
                }
                Ok::<(), String>(())
            })();
            if let Ok(mut state) = manager.updates.lock() {
                state.checking = false;
                state.checked_at = Some(crate::database::now());
                state.error = result
                    .err()
                    .or_else(|| (jobs.len() > 100).then(|| "model_update_limit".into()));
                if let Ok(json) = serde_json::to_string(&*state) {
                    if let Ok(s) = manager.state.lock() {
                        let _ = s
                            .db
                            .execute("INSERT OR REPLACE INTO model_updates VALUES(1,?1)", [json]);
                    }
                }
            }
        });
        Ok(self.updates.lock().map_err(|_| "download_storage")?.clone())
    }
    fn prepare_updates(
        &self,
        ids: Vec<String>,
        paths: crate::types::StoragePaths,
    ) -> Result<Vec<Plan>> {
        if ids.is_empty()
            || ids.len() > 16
            || ids.iter().collect::<std::collections::HashSet<_>>().len() != ids.len()
        {
            return Err("model_update_selection".into());
        }
        let status = self.updates.lock().map_err(|_| "download_storage")?.clone();
        if status.checking {
            return Err("model_update_busy".into());
        }
        let jobs = self.list()?;
        let mut plans = vec![];
        let mut required = 0u64;
        let mut available = u64::MAX;
        for id in ids {
            let update = status
                .entries
                .iter()
                .find(|e| e.download_id == id && e.status == "available")
                .ok_or("model_update_selection")?;
            let job = jobs
                .iter()
                .find(|j| j.id == id && j.status == "completed")
                .ok_or("model_update_selection")?;
            let plan = self.plan(
                job.repo.clone(),
                update.revision.clone().ok_or("model_update_selection")?,
                job.files.iter().map(|f| f.path.clone()).collect(),
                paths.clone(),
            )?;
            required = required
                .checked_add(plan.additional_bytes)
                .ok_or("download_size")?;
            available = available.min(plan.available_bytes);
            plans.push(plan);
        }
        if required.saturating_add(64 * 1024 * 1024) > available {
            return Err("download_space".into());
        }
        Ok(plans)
    }
}
#[tauri::command]
pub fn model_updates_status(state: tauri::State<'_, Arc<Downloads>>) -> Result<UpdateStatus> {
    Ok(state
        .updates
        .lock()
        .map_err(|_| "download_storage")?
        .clone())
}
#[tauri::command]
pub fn model_updates_check(state: tauri::State<'_, Arc<Downloads>>) -> Result<UpdateStatus> {
    state.inner().check_updates()
}
#[tauri::command]
pub async fn model_updates_prepare(
    ids: Vec<String>,
    state: tauri::State<'_, Arc<Downloads>>,
    core: tauri::State<'_, Arc<crate::core::Core>>,
) -> Result<Vec<Plan>> {
    let state = state.inner().clone();
    let paths = core.storage_paths()?;
    tauri::async_runtime::spawn_blocking(move || state.prepare_updates(ids, paths))
        .await
        .map_err(|_| "download_storage")?
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn revision_change_alone_is_not_a_model_update_and_missing_files_require_review() {
        let job:Download=serde_json::from_value(serde_json::json!({"id":"a","repo":"a/b","revision":"old","license":null,"task":null,"files":[{"path":"weights.gguf","size":4,"sha256":"abcd","gitSha1":null,"downloaded":4,"actualSha256":"abcd"}],"destination":"D:\\models","partialDirectory":"D:\\partial","status":"completed","priority":0,"totalBytes":4,"downloadedBytes":4,"bytesPerSecond":0,"error":null,"createdAt":"now","verifyOnly":false})).unwrap();
        let summary:hub::ModelSummary=serde_json::from_value(serde_json::json!({"id":"a/b","downloads":0,"likes":0,"tags":[],"task":null,"library":null,"license":null,"gated":false,"private":false,"lastModified":null,"sizeBytes":null})).unwrap();
        let mut detail = hub::ModelDetail {
            model: summary,
            revision: "new".into(),
            files: vec![hub::ModelFile {
                path: "weights.gguf".into(),
                size: Some(4),
                sha256: Some("abcd".into()),
                git_sha1: None,
            }],
            card: None,
            card_error: None,
        };
        assert_eq!(compare(&job, &detail).status, "current");
        detail.files[0].sha256 = Some("efgh".into());
        assert_eq!(compare(&job, &detail).status, "available");
        detail.files.clear();
        let result = compare(&job, &detail);
        assert_eq!(result.status, "review");
        assert_eq!(result.missing_files, vec!["weights.gguf"]);
    }
}
