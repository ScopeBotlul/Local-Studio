use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Settings {
    #[serde(default)] pub live_hardware: bool,
    #[serde(default)] pub system_accent: bool,
    #[serde(default)] pub minimize_to_tray: bool,
    #[serde(default)] pub parallel_generation: bool,
    #[serde(default = "crate::settings::auto_cleanup_default")] pub auto_model_updates: bool,
    #[serde(default = "crate::settings::auto_cleanup_default")]
    pub auto_update_check: bool,
    pub language: String,
    pub theme: String,
    pub accent_color: String,
    pub ui_scale: f64,
    pub data_root: String,
    pub restore_session: bool,
    pub setup_complete: bool,
    pub max_undo: u32,
    pub temp_retention_days: u32,
    #[serde(default = "crate::settings::auto_cleanup_default")]
    pub auto_cleanup: bool,
    #[serde(default)]
    pub storage_overrides: std::collections::BTreeMap<String, String>,
    #[serde(default = "crate::shortcuts::defaults", deserialize_with = "crate::shortcuts::deserialize")]
    pub shortcuts: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoragePaths {
    pub models: String,
    pub assistant_models: String,
    pub vision_models: String,
    pub downloads: String,
    pub gallery: String,
    pub projects: String,
    pub temporary: String,
    pub recovery: String,
    pub cache: String,
    pub proxies: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuInfo {
    pub name: String,
    pub vendor: String,
    pub vram_bytes: Option<u64>,
    pub driver: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HardwareInfo {
    pub cpu: String,
    pub logical_cores: usize,
    pub total_memory_bytes: u64,
    pub available_memory_bytes: u64,
    pub os: String,
    pub gpus: Vec<GpuInfo>,
    pub disks: Vec<DiskInfo>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Job {
    pub id: String,
    pub kind: String,
    pub input_path: String,
    pub status: String,
    pub progress: Option<f64>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub result: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSnapshot {
    pub version: String,
    pub settings: Settings,
    pub paths: StoragePaths,
    pub hardware: HardwareInfo,
    pub jobs: Vec<Job>,
    pub recovery_available: bool,
    pub database_path: String,
    pub portable: bool,
}
