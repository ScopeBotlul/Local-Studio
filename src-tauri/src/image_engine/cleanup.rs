use super::*;
use crate::{
    gallery,
    maintenance::{self, Candidate, Inventory},
};

fn eligible(job: &ImageJob, cutoff: i64) -> bool {
    if !matches!(
        job.status.as_str(),
        "completed" | "failed" | "cancelled" | "interrupted"
    ) {
        return false;
    }
    if job.status == "completed" && !job.discarded && job.saved_path.is_none() {
        return false;
    }
    chrono::DateTime::parse_from_rfc3339(job.finished_at.as_deref().unwrap_or(&job.created_at))
        .is_ok_and(|t| t.timestamp() < cutoff)
}
fn duplicate(job: &ImageJob, path: &Path) -> Result<(File, Vec<File>)> {
    let saved = Path::new(job.saved_path.as_ref().ok_or("cleanup_protected")?);
    if fs::canonicalize(saved).ok() == fs::canonicalize(path).ok() {
        return Err("cleanup_protected".into());
    }
    let pins = gallery::directory_guards(saved.parent().ok_or("image_path")?)?;
    let file = gallery::lock_file(saved)?;
    if job.saved_binding.as_deref() != Some(&gallery::file_binding(&file)?) {
        return Err("cleanup_protected".into());
    }
    if png_bytes(saved, job.request.width, job.request.height)?
        != png_bytes(path, job.request.width, job.request.height)?
    {
        return Err("cleanup_changed".into());
    }
    Ok((file, pins))
}
fn known(job: &ImageJob, path: &Path) -> bool {
    let Ok(_pins) = gallery::directory_guards(path.parent().unwrap_or(path)) else {
        return false;
    };
    let Ok(mut file) = gallery::lock_file(path) else {
        return false;
    };
    let Ok(meta) = file.metadata() else {
        return false;
    };
    let read = |file: &mut File| {
        let mut bytes = Vec::new();
        file.take(1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map(|_| bytes)
    };
    match path.file_name().and_then(|n| n.to_str()) {
        Some("prompt.txt") if meta.len() <= 4000 => {
            read(&mut file).is_ok_and(|b| b == job.request.prompt.as_bytes())
        }
        Some("negative.txt") if meta.len() <= 4000 => {
            read(&mut file).is_ok_and(|b| b == job.request.negative_prompt.as_bytes())
        }
        Some("metadata.json") if meta.len() <= 1024 * 1024 => read(&mut file)
            .ok()
            .and_then(|b| serde_json::from_slice::<Value>(&b).ok())
            .is_some_and(|v| {
                v["id"] == job.id
                    && serde_json::to_value(&job.request).is_ok_and(|r| v["request"] == r)
            }),
        Some("image.png") => duplicate(job, path).is_ok(),
        _ => false,
    }
}
impl ImageEngine {
    pub(crate) fn cleanup_inventory(&self, cutoff: i64) -> Result<Inventory> {
        let s = self.state.lock().map_err(|_| "image_storage")?;
        let mut result = Inventory::default();
        for job in &s.jobs {
            let Ok(directory) = job_directory(job, &self.config) else {
                continue;
            };
            for name in ["image.png", "prompt.txt", "negative.txt", "metadata.json"] {
                let path = directory.join(name);
                if !path.exists() {
                    continue;
                }
                match maintenance::inspect(&path, "image", &job.id) {
                    Ok(file) => {
                        if eligible(job, cutoff) && known(job, &path) {
                            result.files.push(file);
                        } else {
                            result.protected_bytes += file.bytes;
                        }
                    }
                    Err(_) => result.unavailable += 1,
                }
            }
        }
        Ok(result)
    }
    pub(crate) fn cleanup_file(&self, c: &Candidate, cutoff: i64) -> Result<bool> {
        let mut s = self.state.lock().map_err(|_| "image_storage")?;
        let Some(mut job) = s.jobs.iter().find(|j| j.id == c.owner).cloned() else {
            return Ok(false);
        };
        if !eligible(&job, cutoff) {
            return Ok(false);
        }
        let path = Path::new(&c.path);
        let directory = job_directory(&job, &self.config)?;
        if path.parent() != Some(directory.as_path()) {
            return Err("cleanup_path".into());
        }
        // Pin both the current source and (for duplicate images) the independently saved gallery copy.
        let pins = gallery::directory_guards(&directory)?;
        let read = gallery::lock_file(path)?;
        if gallery::file_binding(&read)? != c.binding || !known(&job, path) {
            return Err("cleanup_changed".into());
        }
        let saved = if path.file_name().is_some_and(|n| n == "image.png") {
            Some(duplicate(&job, path)?)
        } else {
            None
        };
        drop(read);
        let (file, _guards) = maintenance::checked_file(c)?;
        if path.file_name().is_some_and(|n| n == "image.png") {
            job.output = None;
            persist(&s, &job)?;
            *s.jobs.iter_mut().find(|j| j.id == c.owner).unwrap() = job;
        }
        gallery::delete_handle(&file)?;
        drop(saved);
        drop(pins);
        Ok(true)
    }
}
