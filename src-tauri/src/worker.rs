use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::Path;
use std::time::{Duration, Instant};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HashRequest {
    pub protocol: u32,
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum WorkerEvent {
    Ready { protocol: u32 },
    Progress { bytes_read: u64, total_bytes: u64 },
    Completed { sha256: String, bytes_read: u64 },
    Failed { message: String },
}

fn send(event: &WorkerEvent) -> Result<(), String> {
    let stdout = std::io::stdout();
    let mut output = stdout.lock();
    serde_json::to_writer(&mut output, event).map_err(|e| e.to_string())?;
    output
        .write_all(b"\n")
        .and_then(|_| output.flush())
        .map_err(|e| e.to_string())
}

pub fn hash_file(
    path: &Path,
    mut progress: impl FnMut(u64, u64) -> Result<(), String>,
) -> Result<(String, u64), String> {
    let mut file = File::open(path).map_err(|e| format!("Cannot read selected file: {e}"))?;
    let before = file.metadata().map_err(|e| e.to_string())?;
    if !before.is_file() {
        return Err("Select a regular file.".into());
    }
    let total = before.len();
    let mut digest = Sha256::new();
    let mut buffer = vec![0u8; 1024 * 1024];
    let mut bytes = 0;
    let mut last_update = Instant::now();
    progress(0, total)?;
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|e| format!("Reading selected file failed: {e}"))?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
        bytes += count as u64;
        if last_update.elapsed() >= Duration::from_millis(100) {
            progress(bytes, total)?;
            last_update = Instant::now();
        }
    }
    let after = file.metadata().map_err(|e| e.to_string())?;
    if bytes != total
        || before.len() != after.len()
        || before.modified().ok() != after.modified().ok()
    {
        return Err("The file changed while it was being read. Verify it again after the writer has finished.".into());
    }
    progress(bytes, total)?;
    Ok((format!("{:x}", digest.finalize()), bytes))
}

pub fn run_worker() -> i32 {
    let mut input = BufReader::new(std::io::stdin());
    let mut line = String::new();
    if input.read_line(&mut line).is_err() {
        return 2;
    }
    let request: HashRequest = match serde_json::from_str(&line) {
        Ok(request) => request,
        Err(error) => {
            let _ = send(&WorkerEvent::Failed {
                message: format!("Invalid worker request: {error}"),
            });
            return 2;
        }
    };
    if request.protocol != 1 {
        let _ = send(&WorkerEvent::Failed {
            message: "Unsupported worker protocol.".into(),
        });
        return 2;
    }
    // Parent retains its stdin pipe for the entire job. EOF means parent death or shutdown.
    // This watchdog also prevents orphan workers when the desktop is forcibly terminated.
    std::thread::spawn(move || {
        let mut byte = [0u8; 1];
        let _ = input.read(&mut byte);
        std::process::exit(3);
    });
    if send(&WorkerEvent::Ready { protocol: 1 }).is_err() {
        return 2;
    }
    match hash_file(Path::new(&request.path), |bytes_read, total_bytes| {
        send(&WorkerEvent::Progress {
            bytes_read,
            total_bytes,
        })
    }) {
        Ok((sha256, bytes_read)) => {
            if send(&WorkerEvent::Completed { sha256, bytes_read }).is_ok() {
                0
            } else {
                2
            }
        }
        Err(message) => {
            let _ = send(&WorkerEvent::Failed { message });
            1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn hashes_real_bytes_and_reports_real_progress() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("abc.txt");
        std::fs::write(&file, b"abc").unwrap();
        let mut progress = Vec::new();
        let (hash, bytes) = hash_file(&file, |read, total| {
            progress.push((read, total));
            Ok(())
        })
        .unwrap();
        assert_eq!(
            hash,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(bytes, 3);
        assert_eq!(progress, vec![(0, 3), (3, 3)]);
    }
    #[test]
    fn hashes_empty_file_and_rejects_missing_input() {
        let dir = tempfile::tempdir().unwrap();
        let empty = dir.path().join("empty");
        std::fs::write(&empty, []).unwrap();
        assert_eq!(
            hash_file(&empty, |_, _| Ok(())).unwrap().0,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert!(hash_file(&dir.path().join("missing"), |_, _| Ok(())).is_err());
    }
    #[test]
    fn reader_failure_aborts_instead_of_reporting_success() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("input");
        std::fs::write(&file, b"abc").unwrap();
        assert!(hash_file(&file, |_, _| Err("Parent disconnected".into())).is_err());
    }
}
