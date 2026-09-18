use super::*;
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Preference {
    pub model_sha256: String,
    pub model_name: String,
    pub width: u32,
    pub height: u32,
    pub steps: u32,
    pub guidance: f32,
    pub sampler: String,
    pub uses: u64,
    pub custom: bool,
    pub updated_at: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PreferenceEdit {
    pub model_sha256: String,
    pub width: u32,
    pub height: u32,
    pub steps: u32,
    pub guidance: f32,
    pub sampler: String,
}
pub(super) fn learn(tx: &rusqlite::Transaction<'_>, record: &Benchmark) -> Result<()> {
    let mut preference: Preference = tx
        .query_row(
            "SELECT json FROM preferences WHERE id=?1",
            [&record.model_sha256],
            |r| r.get::<_, String>(0),
        )
        .ok()
        .map(|s| serde_json::from_str(&s).map_err(|_| "benchmark_storage"))
        .transpose()?
        .unwrap_or(Preference {
            model_sha256: record.model_sha256.clone(),
            model_name: record.model_name.clone(),
            width: 512,
            height: 512,
            steps: 25,
            guidance: 5.,
            sampler: "euler".into(),
            uses: 0,
            custom: false,
            updated_at: String::new(),
        });
    if !preference.custom {
        preference.width = record.settings["width"]
            .as_u64()
            .ok_or("benchmark_storage")? as u32;
        preference.height = record.settings["height"]
            .as_u64()
            .ok_or("benchmark_storage")? as u32;
        preference.steps = record.settings["steps"]
            .as_u64()
            .ok_or("benchmark_storage")? as u32;
        preference.guidance = record.settings["guidance"]
            .as_f64()
            .ok_or("benchmark_storage")? as f32;
        preference.sampler = record.settings["sampler"]
            .as_str()
            .ok_or("benchmark_storage")?
            .into();
    }
    preference.uses = preference.uses.saturating_add(1);
    preference.updated_at = record.created_at.clone();
    tx.execute(
        "INSERT OR REPLACE INTO preferences VALUES(?1,?2)",
        params![
            preference.model_sha256,
            serde_json::to_string(&preference).map_err(|_| "benchmark_storage")?
        ],
    )
    .map_err(|_| "benchmark_storage")?;
    Ok(())
}
impl Benchmarks {
    pub(crate) fn preferences(&self) -> Result<Vec<Preference>> {
        let db = self.db.lock().map_err(|_| "benchmark_storage")?;
        let mut q = db
            .prepare(
                "SELECT json FROM preferences ORDER BY json_extract(json,'$.uses') DESC LIMIT 500",
            )
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
    fn edit_preference(&self, edit: PreferenceEdit) -> Result<Preference> {
        crate::image_engine::validate(&crate::image_engine::ImageRequest {
            vae_on_cpu: false,
            reference: None,
            model_path: String::new(),
            prompt: "validation".into(),
            negative_prompt: String::new(),
            width: edit.width,
            height: edit.height,
            steps: edit.steps,
            guidance: edit.guidance,
            seed: 0,
            sampler: edit.sampler.clone(),
        })?;
        let db = self.db.lock().map_err(|_| "benchmark_storage")?;
        let json: String = db
            .query_row(
                "SELECT json FROM preferences WHERE id=?1",
                [&edit.model_sha256],
                |r| r.get(0),
            )
            .map_err(|_| "preference_missing")?;
        let mut p: Preference = serde_json::from_str(&json).map_err(|_| "benchmark_storage")?;
        p.width = edit.width;
        p.height = edit.height;
        p.steps = edit.steps;
        p.guidance = edit.guidance;
        p.sampler = edit.sampler;
        p.custom = true;
        p.updated_at = crate::database::now();
        db.execute(
            "UPDATE preferences SET json=?1 WHERE id=?2",
            params![
                serde_json::to_string(&p).map_err(|_| "benchmark_storage")?,
                p.model_sha256
            ],
        )
        .map_err(|_| "benchmark_storage")?;
        Ok(p)
    }
    fn forget_preference(&self, sha: Option<String>, confirmed: bool) -> Result<()> {
        if !confirmed {
            return Err("preference_confirmation".into());
        }
        let db = self.db.lock().map_err(|_| "benchmark_storage")?;
        if let Some(sha) = sha {
            db.execute("DELETE FROM preferences WHERE id=?1", [sha])
        } else {
            db.execute("DELETE FROM preferences", [])
        }
        .map_err(|_| "benchmark_storage")?;
        Ok(())
    }
}
#[tauri::command]
pub fn preferences_list(state: tauri::State<'_, Arc<Benchmarks>>) -> Result<Vec<Preference>> {let privacy_epoch=crate::privacy::epoch();let privacy_result=(||{
    let protected:std::collections::HashSet<_>=state.list()?.into_iter().filter(|b|crate::privacy::model(Path::new(&b.model_path))).map(|b|b.model_sha256).collect();
    Ok(state.preferences()?.into_iter().filter(|p|!crate::privacy::locked()||!protected.contains(&p.model_sha256)).collect())
})();crate::privacy::finish(privacy_epoch,privacy_result)}
#[tauri::command]
pub fn preference_save(
    edit: PreferenceEdit,
    state: tauri::State<'_, Arc<Benchmarks>>,
) -> Result<Preference> {
    state.edit_preference(edit)
}
#[tauri::command]
pub fn preference_forget(
    model_sha256: Option<String>,
    confirmed: bool,
    state: tauri::State<'_, Arc<Benchmarks>>,
) -> Result<()> {
    state.forget_preference(model_sha256, confirmed)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn edited_preferences_survive_learning_and_reset_does_not_touch_benchmarks() {
        let t = tempfile::tempdir().unwrap();
        let b = Benchmarks::new(t.path()).unwrap();
        let mut r = super::super::tests::record();
        b.insert(&r).unwrap();
        let p = b.preferences().unwrap()[0].clone();
        assert_eq!(p.uses, 1);
        b.edit_preference(PreferenceEdit {
            model_sha256: p.model_sha256.clone(),
            width: 768,
            height: 512,
            steps: 8,
            guidance: 3.,
            sampler: "dpm++2m".into(),
        })
        .unwrap();
        r.id = "next".into();
        b.insert(&r).unwrap();
        let p = b.preferences().unwrap()[0].clone();
        assert_eq!(p.uses, 2);
        assert_eq!(p.width, 768);
        assert_eq!(p.steps, 8);
        assert!(p.custom);
        assert!(b.forget_preference(None, false).is_err());
        b.forget_preference(Some(p.model_sha256), true).unwrap();
        assert!(b.preferences().unwrap().is_empty());
        assert_eq!(b.list().unwrap().len(), 2);
    }
}
