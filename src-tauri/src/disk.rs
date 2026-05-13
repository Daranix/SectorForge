use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    #[serde(rename = "DeviceID")]
    pub device_id: String,
    #[serde(rename = "Caption")]
    pub caption: String,
    #[serde(rename = "Size")]
    pub size: u64,
    #[serde(rename = "MediaType")]
    pub media_type: Option<String>,
    #[serde(rename = "Partitions")]
    pub partitions: u32,
}

pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else {
        format!("{} KB", bytes / KB)
    }
}

pub fn list_physical_disks() -> Result<Vec<DiskInfo>, String> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "@(Get-CimInstance Win32_DiskDrive | Select-Object DeviceID, Caption, Size, MediaType, Partitions) | ConvertTo-Json -Compress",
        ])
        .output()
        .map_err(|e| format!("Failed to execute PowerShell: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("PowerShell error: {}", stderr));
    }

    let json = String::from_utf8_lossy(&output.stdout);
    let trimmed = json.trim();

    if trimmed.is_empty() || trimmed == "null" {
        return Ok(vec![]);
    }

    let disks: Vec<DiskInfo> = if trimmed.starts_with('[') {
        serde_json::from_str(trimmed)
            .map_err(|e| format!("Failed to parse disk list: {} in {}", e, trimmed))?
    } else {
        let disk: DiskInfo = serde_json::from_str(trimmed)
            .map_err(|e| format!("Failed to parse single disk: {} in {}", e, trimmed))?;
        vec![disk]
    };

    Ok(disks)
}