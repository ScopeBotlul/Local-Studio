use crate::types::{DiskInfo, GpuInfo, HardwareInfo};
use serde::Deserialize;
use std::io::Read;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use sysinfo::{Disks, System};

pub(crate) fn hide_console(command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    #[cfg(not(windows))]
    let _ = command;
}

fn capture(command: &mut Command) -> Result<String, String> {
    hide_console(command);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    let mut child = command.spawn().map_err(|e| e.to_string())?;
    let stdout = child
        .stdout
        .take()
        .ok_or("Hardware probe did not provide stdout.")?;
    let output = std::thread::spawn(move || {
        let mut data = String::new();
        stdout
            .take(256 * 1024)
            .read_to_string(&mut data)
            .map(|_| data)
    });
    let started = Instant::now();
    let successful = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status.success(),
            Ok(None) if started.elapsed() < Duration::from_secs(12) => {
                std::thread::sleep(Duration::from_millis(50))
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                break false;
            }
        }
    };
    let data = output
        .join()
        .map_err(|_| "Hardware probe output reader failed.".to_string())?
        .map_err(|e| e.to_string())?;
    if successful {
        Ok(data)
    } else {
        Err("Hardware probe failed or timed out.".into())
    }
}

fn nvidia_gpus() -> Result<Vec<GpuInfo>, String> {
    let candidates = [
        std::env::var_os("SystemRoot")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| "C:\\Windows".into())
            .join("System32/nvidia-smi.exe"),
        std::env::var_os("ProgramFiles")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| "C:\\Program Files".into())
            .join("NVIDIA Corporation/NVSMI/nvidia-smi.exe"),
    ];
    let mut last_error = String::new();
    for path in candidates {
        match capture(Command::new(path).args([
            "--query-gpu=name,memory.total,driver_version",
            "--format=csv,noheader,nounits",
        ])) {
            Ok(output) => return Ok(parse_nvidia(&output)),
            Err(error) => last_error = error,
        }
    }
    Err(last_error)
}

fn parse_nvidia(output: &str) -> Vec<GpuInfo> {
    output
        .lines()
        .filter_map(|line| {
            let mut fields = line.rsplitn(3, ',').map(str::trim);
            let driver = fields.next()?;
            let memory = fields.next()?;
            let name = fields.next()?;
            if name.is_empty() {
                return None;
            }
            Some(GpuInfo {
                name: name.to_string(),
                vendor: "NVIDIA".into(),
                vram_bytes: memory
                    .parse::<u64>()
                    .ok()
                    .and_then(|m| m.checked_mul(1024 * 1024)),
                driver: if driver.is_empty() || driver == "[N/A]" {
                    None
                } else {
                    Some(driver.to_string())
                },
            })
        })
        .collect()
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct CimGpu {
    name: Option<String>,
    adapter_compatibility: Option<String>,
    driver_version: Option<String>,
}

#[cfg(windows)]
fn windows_gpus() -> Result<Vec<GpuInfo>, String> {
    let powershell = std::env::var_os("SystemRoot")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| "C:\\Windows".into())
        .join("System32/WindowsPowerShell/v1.0/powershell.exe");
    let output = capture(Command::new(powershell).args([
        "-NoLogo", "-NoProfile", "-NonInteractive", "-Command",
        "[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false); $ErrorActionPreference = 'Stop'; @(Get-CimInstance -ClassName Win32_VideoController | Select-Object Name, AdapterCompatibility, DriverVersion) | ConvertTo-Json -Compress -Depth 3"
    ]))?;
    parse_cim(&output)
}

fn parse_cim(output: &str) -> Result<Vec<GpuInfo>, String> {
    let value: serde_json::Value =
        serde_json::from_str(output.trim_start_matches('\u{feff}')).map_err(|e| e.to_string())?;
    let adapters: Vec<CimGpu> = if value.is_null() {
        Vec::new()
    } else if value.is_array() {
        serde_json::from_value(value).map_err(|e| e.to_string())?
    } else {
        vec![serde_json::from_value(value).map_err(|e| e.to_string())?]
    };
    Ok(adapters
        .into_iter()
        .filter_map(|adapter| {
            let name = adapter.name?;
            let raw_vendor = adapter.adapter_compatibility.unwrap_or_default();
            let combined = format!("{name} {raw_vendor}").to_lowercase();
            let vendor = if combined.contains("nvidia") {
                "NVIDIA"
            } else if combined.contains("amd") || combined.contains("advanced micro") {
                "AMD"
            } else if combined.contains("intel") {
                "Intel"
            } else {
                raw_vendor.as_str()
            };
            // Win32_VideoController.AdapterRAM is a 32-bit field and cannot reliably describe modern VRAM.
            Some(GpuInfo {
                name,
                vendor: vendor.into(),
                vram_bytes: None,
                driver: adapter.driver_version,
            })
        })
        .collect())
}

pub fn discover() -> HardwareInfo {
    let mut system = System::new();
    system.refresh_cpu_all();
    system.refresh_memory();
    let disks = Disks::new_with_refreshed_list();
    let mut warnings = Vec::new();
    let mut gpus = nvidia_gpus().unwrap_or_default();
    #[cfg(windows)]
    match windows_gpus() {
        Ok(adapters) => {
            for adapter in adapters {
                if !gpus
                    .iter()
                    .any(|g| g.name.eq_ignore_ascii_case(&adapter.name))
                {
                    gpus.push(adapter);
                }
            }
        }
        Err(error) => warnings.push(format!("Windows GPU discovery failed: {error}")),
    }
    if gpus.is_empty() {
        warnings.push(
            "No graphics adapter could be identified. No GPU inference backend has been validated."
                .into(),
        );
    }
    if gpus.iter().any(|g| g.vram_bytes.is_none()) {
        warnings.push("Dedicated VRAM is unknown for one or more adapters. Windows CIM memory values are deliberately not used because they can be truncated.".into());
    }
    HardwareInfo {
        cpu: system
            .cpus()
            .first()
            .map(|c| c.brand().to_string())
            .unwrap_or_else(|| "Unknown CPU".into()),
        logical_cores: system.cpus().len(),
        total_memory_bytes: system.total_memory(),
        available_memory_bytes: system.available_memory(),
        os: System::long_os_version().unwrap_or_else(|| std::env::consts::OS.into()),
        gpus,
        disks: disks
            .list()
            .iter()
            .map(|disk| DiskInfo {
                name: disk.name().to_string_lossy().into_owned(),
                mount_point: disk.mount_point().to_string_lossy().into_owned(),
                total_bytes: disk.total_space(),
                available_bytes: disk.available_space(),
            })
            .collect(),
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nvidia_memory_is_mib_and_unknown_is_never_invented() {
        let cards =
            parse_nvidia("NVIDIA GeForce RTX 4080, 16376, 581.80\nNVIDIA Unknown, [N/A], [N/A]\n");
        assert_eq!(cards[0].vram_bytes, Some(16376 * 1024 * 1024));
        assert_eq!(cards[1].vram_bytes, None);
        assert_eq!(cards[1].driver, None);
    }
    #[test]
    fn cim_detects_vendor_but_does_not_claim_vram() {
        let cards = parse_cim(r#"{"Name":"Intel Graphics","AdapterCompatibility":"Intel Corporation","DriverVersion":"32.1"}"#).unwrap();
        assert_eq!(cards[0].vendor, "Intel");
        assert_eq!(cards[0].vram_bytes, None);
    }
}
