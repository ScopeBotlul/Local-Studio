use crate::{comfy::Comfy, core::Core, hub, image_engine::ImageEngine, updater::Updater};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};
use tauri::State;
use url::Url;
use uuid::Uuid;

type Result<T> = std::result::Result<T, String>;
const ISSUE_URL: &str = "https://github.com/ScopeBotlul/Local-Studio/issues/new";
const MAX_REPORT: usize = 64 * 1024;

#[derive(Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BugReportInput {
    title: String,
    description: String,
    steps: String,
    ui_context: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BugReportDraft {
    report: String,
    included: Vec<String>,
    excluded: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BugReportResult {
    path: String,
    report: String,
}

fn checked(value: &str, max: usize, required: bool) -> Result<String> {
    let value = value.trim();
    if (required && value.is_empty())
        || value.len() > max
        || value.chars().any(|c| c == '\0')
    {
        return Err("bug_report_input".into());
    }
    Ok(value.replace('\r', ""))
}

fn replace_insensitive(mut text: String, needle: &str, replacement: &str) -> String {
    if needle.is_empty() {
        return text;
    }
    text = text.replace(needle, replacement);
    if !needle.is_ascii() {
        return text;
    }
    loop {
        let lower = text.to_lowercase();
        let Some(index) = lower.find(&needle.to_lowercase()) else {
            break;
        };
        let end = index + needle.len();
        if !text.is_char_boundary(index) || !text.is_char_boundary(end) {
            break;
        }
        text.replace_range(index..end, replacement);
    }
    text
}

fn sanitize(input: &str, snapshot: &crate::types::AppSnapshot) -> String {
    let mut text = input.replace('\0', "");
    let mut paths = vec![
        (snapshot.settings.data_root.as_str(), "%LOCAL_STUDIO_DATA%"),
        (snapshot.database_path.as_str(), "%LOCAL_STUDIO_CONFIG%\\local-studio.sqlite3"),
    ];
    let environment = [
        ("USERPROFILE", "%USERPROFILE%"),
        ("LOCALAPPDATA", "%LOCALAPPDATA%"),
        ("APPDATA", "%APPDATA%"),
        ("USERNAME", "%USERNAME%"),
        ("COMPUTERNAME", "%COMPUTERNAME%"),
    ];
    let values: Vec<(String, &'static str)> = environment
        .into_iter()
        .filter_map(|(key, replacement)| {
            std::env::var(key).ok().filter(|v| !v.is_empty()).map(|v| (v, replacement))
        })
        .collect();
    for (value, replacement) in &values {
        paths.push((value, replacement));
    }
    paths.sort_by_key(|(path, _)| std::cmp::Reverse(path.len()));
    for (path, replacement) in paths {
        text = replace_insensitive(text, path, replacement);
    }
    text.lines()
        .map(|line| {
            let lower = line.to_ascii_lowercase();
            if ["authorization:", "cookie:", "access_token", "refresh_token", "password=", "hf_"]
                .iter()
                .any(|secret| lower.contains(secret))
            {
                "[REDACTED SECRET LINE]".to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn tail_chars(input: &str, limit: usize) -> String {
    if input.len() <= limit {
        return input.to_string();
    }
    let mut start = input.len() - limit;
    while !input.is_char_boundary(start) {
        start += 1;
    }
    format!("[older output omitted]\n{}", &input[start..])
}

fn head_tail_chars(input: &str, limit: usize) -> String {
    if input.len() <= limit {
        return input.to_string();
    }
    let half = limit / 2;
    let mut end = half;
    while !input.is_char_boundary(end) {
        end -= 1;
    }
    let mut start = input.len() - half;
    while !input.is_char_boundary(start) {
        start += 1;
    }
    format!(
        "{}\n[middle output omitted]\n{}",
        &input[..end],
        &input[start..]
    )
}

fn diagnostic_chars(input: &str, limit: usize) -> String {
    if input.len() <= limit {
        return input.to_string();
    }
    let lower = input.to_ascii_lowercase();
    let signal = [
        "windows fatal",
        "access violation",
        "out of memory",
        "runtimeerror",
        "traceback",
    ]
    .iter()
    .filter_map(|marker| lower.find(marker))
    .min();
    let Some(signal) = signal else {
        return head_tail_chars(input, limit);
    };
    let part = limit / 3;
    let mut head_end = part;
    while !input.is_char_boundary(head_end) {
        head_end -= 1;
    }
    let mut middle_start = signal.saturating_sub(part / 2);
    while !input.is_char_boundary(middle_start) {
        middle_start += 1;
    }
    let mut middle_end = (middle_start + part).min(input.len());
    while !input.is_char_boundary(middle_end) {
        middle_end -= 1;
    }
    let mut tail_start = input.len().saturating_sub(part);
    while !input.is_char_boundary(tail_start) {
        tail_start += 1;
    }
    format!(
        "{}\n[output omitted before crash]\n{}\n[output omitted after crash]\n{}",
        &input[..head_end],
        &input[middle_start..middle_end],
        &input[tail_start..]
    )
}

fn prefix_chars(input: &str, limit: usize) -> String {
    if input.chars().count() <= limit {
        return input.to_string();
    }
    let mut text = input.chars().take(limit).collect::<String>();
    text.push_str("\n[… full text is in the saved report]");
    text
}

fn settings_json(snapshot: &crate::types::AppSnapshot) -> Result<String> {
    let mut settings = snapshot.settings.clone();
    settings.data_root = "%LOCAL_STUDIO_DATA%".into();
    for value in settings.storage_overrides.values_mut() {
        *value = "%CUSTOM_STORAGE_PATH%".into();
    }
    serde_json::to_string_pretty(&settings).map_err(|_| "bug_report_storage".into())
}

fn job_summary(
    snapshot: &crate::types::AppSnapshot,
    image_jobs: &[crate::image_engine::ImageJob],
) -> BTreeMap<String, usize> {
    let mut summary = BTreeMap::new();
    for job in &snapshot.jobs {
        *summary
            .entry(format!("{}:{}", job.kind, job.status))
            .or_default() += 1;
    }
    for job in image_jobs {
        *summary.entry(format!("image:{}", job.status)).or_default() += 1;
    }
    summary
}

fn build(
    input: BugReportInput,
    core: &Core,
    comfy: &Comfy,
    updater: &Updater,
    images: &ImageEngine,
) -> Result<BugReportDraft> {
    let title = checked(&input.title, 120, false)?;
    let description = checked(&input.description, 4_000, false)?;
    let steps = checked(&input.steps, 4_000, false)?;
    let ui_context = checked(&input.ui_context, 200, false)?;
    let snapshot = core.snapshot()?;
    let settings = settings_json(&snapshot)?;
    let hardware = sanitize(
        &serde_json::to_string_pretty(&snapshot.hardware).map_err(|_| "bug_report_storage")?,
        &snapshot,
    );
    let comfy_status = sanitize(
        &serde_json::to_string_pretty(&comfy.status()).map_err(|_| "bug_report_storage")?,
        &snapshot,
    );
    let update_status = sanitize(
        &serde_json::to_string_pretty(&updater.status_for_report()?)
            .map_err(|_| "bug_report_storage")?,
        &snapshot,
    );
    let logs = tail_chars(&sanitize(&core.logs()?, &snapshot), 16 * 1024);
    let report_id = Uuid::new_v4();
    let image_jobs = images.list()?;
    let jobs = serde_json::to_string_pretty(&job_summary(&snapshot, &image_jobs))
        .map_err(|_| "bug_report_storage")?;
    let report = format!(
        "# Local Studio bug report\n\nReport ID: `{report_id}`\nCreated: `{}`\n\n## User report\n\nTitle: {}\n\n### Description\n{}\n\n### Steps to reproduce\n{}\n\nUI context: `{}`\n\n## Application\n\n```json\n{}\n```\n\n## Settings (paths redacted)\n\n```json\n{}\n```\n\n## Hardware\n\n```json\n{}\n```\n\n## Job summary (no input/output paths)\n\n```json\n{}\n```\n\n## ComfyUI status and startup log\n\n```text\n{}\n```\n\n## Update status\n\n```json\n{}\n```\n\n## Recent Local Studio log\n\n```text\n{}\n```\n\n## Privacy\n\nTokens, cookies, passwords, prompts, media, project contents, database contents and personal path segments are not included.\n",
        Utc::now().to_rfc3339(),
        if title.is_empty() { "(not entered)" } else { &title },
        if description.is_empty() { "(not entered)" } else { &description },
        if steps.is_empty() { "(not entered)" } else { &steps },
        if ui_context.is_empty() { "unknown" } else { &ui_context },
        serde_json::to_string_pretty(&json!({
            "version": snapshot.version,
            "portable": snapshot.portable,
            "recoveryAvailable": snapshot.recovery_available
        })).map_err(|_| "bug_report_storage")?,
        settings,
        hardware,
        jobs,
        diagnostic_chars(&comfy_status, 18 * 1024),
        tail_chars(&update_status, 6 * 1024),
        logs,
    );
    if report.len() > MAX_REPORT {
        return Err("bug_report_size".into());
    }
    Ok(BugReportDraft {
        report,
        included: vec![
            "App version and installation mode".into(),
            "Local Studio settings with redacted paths".into(),
            "Windows, CPU, RAM, GPU, drivers and disk capacity".into(),
            "Job counts without file names or contents".into(),
            "ComfyUI and update status plus bounded local logs".into(),
        ],
        excluded: vec![
            "Tokens, cookies and passwords".into(),
            "Prompts, media and project contents".into(),
            "Model weights and database contents".into(),
            "Windows user, computer name and personal path segments".into(),
        ],
    })
}

fn report_directory(snapshot: &crate::types::AppSnapshot) -> Result<PathBuf> {
    let database = PathBuf::from(&snapshot.database_path);
    let config = database.parent().ok_or("bug_report_storage")?;
    let reports = config.join("bug-reports");
    fs::create_dir_all(&reports).map_err(|_| "bug_report_storage")?;
    crate::model_library::no_links(&reports).map_err(|_| "bug_report_storage")?;
    Ok(reports)
}

fn issue_url(input: &BugReportInput, snapshot: &crate::types::AppSnapshot) -> Result<String> {
    let title = checked(&input.title, 120, false)?.replace('\n', " ");
    let description = prefix_chars(&checked(&input.description, 4_000, false)?, 900);
    let steps = prefix_chars(&checked(&input.steps, 4_000, false)?, 700);
    let gpu = snapshot
        .hardware
        .gpus
        .iter()
        .map(|gpu| format!("{} {} ({})", gpu.vendor, gpu.name, gpu.driver.as_deref().unwrap_or("driver unknown")))
        .collect::<Vec<_>>()
        .join(", ");
    let compact = format!(
        "## Problem\n{}\n\n## Steps to reproduce\n{}\n\n## Diagnostic summary\n- Local Studio: {}\n- Windows: {}\n- CPU: {} ({} logical cores)\n- RAM: {:.1} GB\n- GPU: {}\n- Installation: {}\n\nA full redacted diagnostic report was saved locally by Local Studio. Please attach that `.md` file to this issue if GitHub does not add it automatically. No tokens, prompts or media are included.",
        if description.is_empty() { "(please describe)" } else { &description },
        if steps.is_empty() { "(please add steps)" } else { &steps },
        snapshot.version,
        snapshot.hardware.os,
        snapshot.hardware.cpu,
        snapshot.hardware.logical_cores,
        snapshot.hardware.total_memory_bytes as f64 / 1024_f64.powi(3),
        if gpu.is_empty() { "not detected" } else { &gpu },
        if snapshot.portable { "portable" } else { "installed" },
    );
    let mut url = Url::parse(ISSUE_URL).map_err(|_| "bug_report_open")?;
    url.query_pairs_mut()
        .append_pair("title", if title.is_empty() { "Bug report" } else { &title })
        .append_pair("body", &compact)
        .append_pair("labels", "bug");
    if url.as_str().len() > 8_000 {
        return Err("bug_report_size".into());
    }
    Ok(url.into())
}

#[tauri::command]
pub async fn bug_report_preview(
    input: BugReportInput,
    core: State<'_, Arc<Core>>,
    comfy: State<'_, Arc<Comfy>>,
    updater: State<'_, Arc<Updater>>,
    images: State<'_, Arc<ImageEngine>>,
) -> Result<BugReportDraft> {
    let core = core.inner().clone();
    let comfy = comfy.inner().clone();
    let updater = updater.inner().clone();
    let images = images.inner().clone();
    tauri::async_runtime::spawn_blocking(move || build(input, &core, &comfy, &updater, &images))
        .await
        .map_err(|_| "bug_report_storage")?
}

#[tauri::command]
pub async fn bug_report_submit(
    input: BugReportInput,
    core: State<'_, Arc<Core>>,
    comfy: State<'_, Arc<Comfy>>,
    updater: State<'_, Arc<Updater>>,
    images: State<'_, Arc<ImageEngine>>,
) -> Result<BugReportResult> {
    let core = core.inner().clone();
    let comfy = comfy.inner().clone();
    let updater = updater.inner().clone();
    let images = images.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        checked(&input.title, 120, true)?;
        checked(&input.description, 4_000, true)?;
        let draft = build(input.clone(), &core, &comfy, &updater, &images)?;
        let snapshot = core.snapshot()?;
        let directory = report_directory(&snapshot)?;
        let name = format!("bug-report-{}-{}.md", Utc::now().format("%Y%m%d-%H%M%S"), Uuid::new_v4());
        let path = directory.join(name);
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|_| "bug_report_storage")?;
        file.write_all(draft.report.as_bytes())
            .and_then(|_| file.sync_all())
            .map_err(|_| "bug_report_storage")?;
        let url = issue_url(&input, &snapshot)?;
        hub::open_external_https(&url).map_err(|_| "bug_report_open")?;
        Ok(BugReportResult {
            path: path.to_string_lossy().into_owned(),
            report: draft.report,
        })
    })
    .await
    .map_err(|_| "bug_report_storage")?
}

#[tauri::command]
pub fn bug_report_open_folder(core: State<'_, Arc<Core>>) -> Result<()> {
    let snapshot = core.snapshot()?;
    let directory = report_directory(&snapshot)?;
    let explorer = Path::new(&std::env::var_os("SystemRoot").ok_or("bug_report_open")?)
        .join("explorer.exe");
    let mut command = Command::new(explorer);
    command.arg(&directory);
    crate::hardware::hide_console(&mut command);
    command.spawn().map_err(|_| "bug_report_open")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot() -> crate::types::AppSnapshot {
        crate::types::AppSnapshot {
            version: "1.2.3".into(),
            settings: crate::settings::defaults(Path::new("C:\\Users\\Alice\\Studio")),
            paths: crate::settings::paths("C:\\Users\\Alice\\Studio"),
            hardware: crate::types::HardwareInfo {
                cpu: "Test CPU".into(),
                logical_cores: 8,
                total_memory_bytes: 16 * 1024 * 1024 * 1024,
                available_memory_bytes: 8 * 1024 * 1024 * 1024,
                os: "Windows test".into(),
                gpus: Vec::new(),
                disks: Vec::new(),
                warnings: Vec::new(),
            },
            jobs: Vec::new(),
            recovery_available: false,
            database_path: "C:\\Users\\Alice\\Config\\local-studio.sqlite3".into(),
            portable: false,
        }
    }

    #[test]
    fn sanitizing_removes_paths_identity_and_secret_lines() {
        let mut snapshot = snapshot();
        snapshot.settings.data_root = "C:\\Users\\Alice\\Studio".into();
        let cleaned = sanitize(
            "C:\\Users\\Alice\\Studio\\models\nAuthorization: Bearer secret\npassword=hunter2",
            &snapshot,
        );
        assert!(!cleaned.contains("Alice"));
        assert!(!cleaned.contains("secret"));
        assert!(!cleaned.contains("hunter2"));
        assert!(cleaned.contains("%LOCAL_STUDIO_DATA%"));
    }

    #[test]
    fn github_issue_is_bounded_and_contains_only_a_summary() {
        let input = BugReportInput {
            title: "Failure".into(),
            description: "d".repeat(4_000),
            steps: "s".repeat(4_000),
            ui_context: "studio".into(),
        };
        let url = issue_url(&input, &snapshot()).unwrap();
        assert!(url.len() < 8_000);
        assert!(!url.contains(&"d".repeat(2_000)));
        assert!(url.starts_with(ISSUE_URL));
    }

    #[test]
    fn bounded_diagnostics_keep_the_error_lead_and_crash_tail() {
        let input = format!(
            "startup details\n{}\nWindows fatal exception: access violation\n{}\nstack tail",
            "x".repeat(1_000),
            "y".repeat(1_000)
        );
        let bounded = diagnostic_chars(&input, 300);
        assert!(bounded.starts_with("startup details"));
        assert!(bounded.contains("Windows fatal exception: access violation"));
        assert!(bounded.ends_with("stack tail"));
        assert!(bounded.contains("[output omitted before crash]"));
        assert!(bounded.len() < input.len());
    }
}
