use super::*;
use crate::maintenance::{self, Candidate, Inventory};

fn eligible(s: &State, id: &str, cutoff: i64) -> Result<bool> {
    container::no_intent(s)?;
    if s.project.as_ref().is_some_and(|p| p.id == id) {
        return Ok(false);
    }
    let touched: Option<i64> =
        s.db.query_row(
            "SELECT touched FROM project_locations WHERE id=?1",
            [id],
            |r| r.get(0),
        )
        .optional()
        .map_err(err)?;
    if touched.is_none_or(|t| t >= cutoff) {
        return Ok(false);
    }
    let protected:bool=s.db.query_row("SELECT EXISTS(SELECT 1 FROM project_history WHERE project_id=?1 AND (at>=?2 OR id IN (SELECT id FROM project_history ORDER BY id DESC LIMIT 3)))",rusqlite::params![id,cutoff],|r|r.get(0)).map_err(err)?;
    Ok(!protected)
}
impl Projects {
    pub(crate) fn cleanup_inventory(&self, cutoff: i64) -> Result<Inventory> {
        let s = self.state.lock().map_err(err)?;
        let mut result = Inventory::default();
        let mut stmt =
            s.db.prepare("SELECT path,project_id,binding FROM project_files")
                .map_err(err)?;
        let rows = stmt
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                ))
            })
            .map_err(err)?;
        for row in rows {
            let (path, id, binding) = row.map_err(err)?;
            match maintenance::inspect(Path::new(&path), "project", &id) {
                Ok(file) if file.binding == binding => {
                    if eligible(&s, &id, cutoff)? {
                        result.files.push(file);
                    } else {
                        result.protected_bytes += file.bytes;
                    }
                }
                _ => result.unavailable += 1,
            }
        }
        Ok(result)
    }
    pub(crate) fn cleanup_file(&self, c: &Candidate, cutoff: i64) -> Result<bool> {
        let s = self.state.lock().map_err(err)?;
        if !eligible(&s, &c.owner, cutoff)? {
            return Ok(false);
        }
        let registered:Option<(String,String)>=s.db.query_row("SELECT f.binding,l.directory FROM project_files f JOIN project_locations l ON l.id=f.project_id WHERE f.path=?1 AND f.project_id=?2",rusqlite::params![c.path,c.owner],|r|Ok((r.get(0)?,r.get(1)?))).optional().map_err(err)?;
        let Some((binding, directory)) = registered else {
            return Ok(false);
        };
        let root = Path::new(&directory);
        let path = Path::new(&c.path);
        if binding != c.binding
            || !valid_id(&c.owner)
            || root.file_name().and_then(|v| v.to_str()) != Some(&c.owner)
            || root
                .parent()
                .and_then(|v| v.file_name())
                .and_then(|v| v.to_str())
                != Some("project-sessions")
            || path.parent() != Some(root)
        {
            return Err("cleanup_path".into());
        }
        let (file, _guards) = maintenance::checked_file(c)?;
        // Retire affected old snapshots before deletion; an interrupted cleanup cannot offer a broken point.
        s.db.execute(
            "DELETE FROM project_history WHERE project_id=?1",
            [&c.owner],
        )
        .map_err(err)?;
        gallery::delete_handle(&file)?;
        s.db.execute("DELETE FROM project_files WHERE path=?1", [&c.path])
            .map_err(err)?;
        Ok(true)
    }
}
