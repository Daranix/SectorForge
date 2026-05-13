use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    #[serde(rename = "DeviceID")]
    pub device_id: String,
    #[serde(rename = "Caption")]
    pub caption: String,
    #[serde(rename = "Size")]
    pub size: u64,
    #[serde(rename = "AvailableSpace")]
    pub available_space: u64,
    #[serde(rename = "MediaType")]
    pub media_type: Option<String>,
    #[serde(rename = "Partitions")]
    pub partitions: u32,
    #[serde(rename = "DriveLetters")]
    pub drive_letters: Option<String>,
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

// ============================================================================
// Windows Implementation - Raw Win32 APIs via FFI (no PowerShell)
// ============================================================================
#[cfg(target_os = "windows")]
pub fn list_physical_disks() -> Result<Vec<DiskInfo>, String> {
    use std::ffi::c_void;
    use std::os::windows::ffi::OsStringExt;
    use std::os::windows::fs::OpenOptionsExt;
    use std::os::windows::io::AsRawHandle;

    const FILE_SHARE_READ: u32 = 0x00000001;
    const FILE_SHARE_WRITE: u32 = 0x00000002;

    const IOCTL_DISK_GET_LENGTH_INFO: u32 = 0x0007405C;
    const IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS: u32 = 0x00560000;
    const IOCTL_DISK_GET_DRIVE_LAYOUT_EX: u32 = 0x00070050;

    #[repr(C)]
    struct GetLengthInformation {
        length: i64,
    }

    #[repr(C, align(8))]
    struct ExtentBuffer([u8; 256]);

    #[repr(C)]
    struct ULARGE_INTEGER {
        quad_part: u64,
    }

    extern "system" {
        fn DeviceIoControl(
            hDevice: *mut c_void,
            dwIoControlCode: u32,
            lpInBuffer: *const c_void,
            nInBufferSize: u32,
            lpOutBuffer: *mut c_void,
            nOutBufferSize: u32,
            lpBytesReturned: *mut u32,
            lpOverlapped: *mut c_void,
        ) -> i32;

        fn GetLogicalDriveStringsW(nBufferLength: u32, lpBuffer: *mut u16) -> u32;

        fn GetDiskFreeSpaceExW(
            lpDirectoryName: *const u16,
            lpFreeBytesAvailableToCaller: *mut ULARGE_INTEGER,
            lpTotalNumberOfBytes: *mut ULARGE_INTEGER,
            lpTotalNumberOfFreeBytes: *mut ULARGE_INTEGER,
        ) -> i32;

        fn GetFileSizeEx(hFile: *mut c_void, lpFileSize: *mut i64) -> i32;
    }

    unsafe fn ioctl(
        handle: *mut c_void,
        code: u32,
        in_buf: Option<&[u8]>,
        out_buf: &mut [u8],
    ) -> Result<u32, String> {
        let mut bytes_returned = 0u32;
        let result = DeviceIoControl(
            handle,
            code,
            in_buf.map(|b| b.as_ptr() as *const c_void).unwrap_or(std::ptr::null()),
            in_buf.map(|b| b.len() as u32).unwrap_or(0),
            out_buf.as_mut_ptr() as *mut c_void,
            out_buf.len() as u32,
            &mut bytes_returned,
            std::ptr::null_mut(),
        );
        if result == 0 {
            let err = std::io::Error::last_os_error();
            Err(format!("DeviceIoControl failed: {}", err))
        } else {
            Ok(bytes_returned)
        }
    }

    // Helper to get partition count via IOCTL_DISK_GET_DRIVE_LAYOUT_EX
    unsafe fn get_partition_count(handle: *mut c_void) -> u32 {
        let mut buf = [0u8; 32 * 1024];
        if let Ok(_) = ioctl(handle, IOCTL_DISK_GET_DRIVE_LAYOUT_EX, None, &mut buf) {
            // DRIVE_LAYOUT_INFORMATION_EX: PartitionStyle (4 bytes) + PartitionCount (4 bytes)
            u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]])
        } else {
            0
        }
    }

    // Helper to get disk size via DeviceIoControl or GetFileSizeEx fallback
    unsafe fn get_disk_size(handle: *mut c_void) -> Result<u64, String> {
        // Try DeviceIoControl first
        let mut size_buf = ExtentBuffer([0u8; 256]);
        if let Ok(_) = ioctl(handle, IOCTL_DISK_GET_LENGTH_INFO, None, &mut size_buf.0) {
            let info = &*(size_buf.0.as_ptr() as *const GetLengthInformation);
            if info.length > 0 {
                return Ok(info.length as u64);
            }
        }

        // Fallback to GetFileSizeEx
        let mut size: i64 = 0;
        if GetFileSizeEx(handle, &mut size) != 0 && size > 0 {
            return Ok(size as u64);
        }

        Err("Could not determine disk size".to_string())
    }

    // Step 1: Enumerate physical drives with error collection
    let mut physical_disks: Vec<(u32, u64)> = Vec::new();
    let mut open_errors: Vec<String> = Vec::new();

    for disk_number in 0..64u32 {
        let path = format!(r"\\.\PhysicalDrive{}", disk_number);
        let file = match std::fs::OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .open(&path)
        {
            Ok(f) => f,
            Err(e) => {
                // Only report first few errors to avoid spam
                if open_errors.len() < 3 {
                    open_errors.push(format!("{} open error: {}", path, e));
                }
                continue;
            }
        };

        let handle = file.as_raw_handle() as *mut c_void;

        match unsafe { get_disk_size(handle) } {
            Ok(size) if size > 0 => {
                physical_disks.push((disk_number, size));
            }
            Ok(_) => {
                open_errors.push(format!(
                    "PhysicalDrive{}: returned size is zero",
                    disk_number
                ));
            }
            Err(e) => {
                open_errors.push(format!("PhysicalDrive{} size error: {}", disk_number, e));
            }
        }
    }

    // If no physical disks found, check if it's a permission issue
    if physical_disks.is_empty() {
        let has_access_denied = open_errors.iter().any(|e| e.contains("os error 5"));
        if has_access_denied {
            return Err(
                "Access denied when reading physical disks. \
Please run CloneTool as Administrator (right-click → Run as administrator)."
                    .to_string(),
            );
        }
        if !open_errors.is_empty() {
            return Err(format!(
                "No physical disks found. Errors:\n{}",
                open_errors.join("\n")
            ));
        }
        return Err("No physical disks found on this system.".to_string());
    }

    // Step 2: Map drive letters to physical disk numbers + collect free space
    let mut letter_map: std::collections::HashMap<u32, Vec<String>> =
        std::collections::HashMap::new();
    let mut free_space_map: std::collections::HashMap<String, u64> =
        std::collections::HashMap::new();

    let mut drive_buffer = vec![0u16; 512];
    let count = unsafe { GetLogicalDriveStringsW(drive_buffer.len() as u32, drive_buffer.as_mut_ptr()) };

    if count > 0 && count <= drive_buffer.len() as u32 {
        let mut i = 0usize;
        while i < count as usize && drive_buffer[i] != 0 {
            let start = i;
            while i < count as usize && drive_buffer[i] != 0 {
                i += 1;
            }
            let drive_str = std::ffi::OsString::from_wide(&drive_buffer[start..i])
                .to_string_lossy()
                .to_string();

            if drive_str.len() >= 2 {
                let drive_letter = drive_str[..2].to_string(); // "C:"

                // Get free space for this drive
                let mut free_avail = ULARGE_INTEGER { quad_part: 0 };
                let mut total = ULARGE_INTEGER { quad_part: 0 };
                let mut total_free = ULARGE_INTEGER { quad_part: 0 };
                let result = unsafe {
                    GetDiskFreeSpaceExW(
                        drive_buffer[start..].as_ptr(),
                        &mut free_avail,
                        &mut total,
                        &mut total_free,
                    )
                };
                if result != 0 {
                    free_space_map.insert(drive_letter.clone(), free_avail.quad_part);
                }

                // Open volume to get its physical disk number
                let volume_path = format!(r"\\.\{}", drive_letter);
                if let Ok(file) = std::fs::OpenOptions::new()
                    .read(true)
                    .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
                    .open(&volume_path)
                {
                    let handle = file.as_raw_handle() as *mut c_void;
                    let mut ext_buf = ExtentBuffer([0u8; 256]);

                    if unsafe {
                        ioctl(handle, IOCTL_VOLUME_GET_VOLUME_DISK_EXTENTS, None, &mut ext_buf.0)
                    }
                    .is_ok()
                    {
                        let disk_num = u32::from_le_bytes([
                            ext_buf.0[8], ext_buf.0[9], ext_buf.0[10], ext_buf.0[11],
                        ]);
                        letter_map
                            .entry(disk_num)
                            .or_default()
                            .push(drive_letter);
                    }
                }
            }
            i += 1; // skip null terminator
        }
    }

    // Step 3: Build DiskInfo structs
    let mut disks = Vec::new();
    for (disk_number, size) in physical_disks {
        let device_id = format!(r"\\.\PhysicalDrive{}", disk_number);
        let letters = letter_map.get(&disk_number).cloned().unwrap_or_default();
        let drive_letters = if letters.is_empty() {
            None
        } else {
            Some(letters.join(", "))
        };

        // Sum free space from all volumes on this disk
        let available_space: u64 = letters
            .iter()
            .filter_map(|l| free_space_map.get(l))
            .sum();

        let caption = if let Some(ref l) = drive_letters {
            format!("Physical Drive {} ({})", disk_number, l)
        } else {
            format!("Physical Drive {}", disk_number)
        };

        // Query actual partition count (best-effort; reopen handle since original was dropped)
        let partition_count = {
            let path = format!(r"\\.\PhysicalDrive{}", disk_number);
            std::fs::OpenOptions::new()
                .read(true)
                .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
                .open(&path)
                .ok()
                .and_then(|f| {
                    let handle = f.as_raw_handle() as *mut c_void;
                    Some(unsafe { get_partition_count(handle) })
                })
                .unwrap_or(0)
        };

        disks.push(DiskInfo {
            device_id,
            caption,
            size,
            available_space,
            media_type: Some("Fixed hard disk media".to_string()),
            partitions: partition_count,
            drive_letters,
        });
    }

    Ok(disks)
}

// ============================================================================
// Linux Implementation - sysfs + /proc/mounts (no shell commands)
// ============================================================================
#[cfg(target_os = "linux")]
pub fn list_physical_disks() -> Result<Vec<DiskInfo>, String> {
    use std::collections::HashMap;

    fn read_sysfs(path: &str) -> Result<String, String> {
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path, e))
    }

    fn is_physical_disk(name: &str) -> bool {
        if name.starts_with("loop")
            || name.starts_with("ram")
            || name.starts_with("zram")
            || name.starts_with("dm-")
            || name.starts_with("sr")
        {
            return false;
        }
        std::path::Path::new(&format!("/sys/block/{}/device", name)).exists()
    }

    fn find_base_device(dev_name: &str) -> Option<String> {
        if dev_name.starts_with("nvme") {
            if let Some(pos) = dev_name.rfind("p") {
                let prefix = &dev_name[..pos];
                if prefix.starts_with("nvme") && prefix.contains("n") {
                    return Some(prefix.to_string());
                }
            }
        }
        if dev_name.starts_with("mmcblk") {
            if let Some(pos) = dev_name.rfind("p") {
                let prefix = &dev_name[..pos];
                if prefix.starts_with("mmcblk") {
                    return Some(prefix.to_string());
                }
            }
        }
        let mut end = dev_name.len();
        while end > 0 && dev_name.as_bytes()[end - 1].is_ascii_digit() {
            end -= 1;
        }
        if end > 0 && end < dev_name.len() {
            return Some(dev_name[..end].to_string());
        }
        None
    }

    // Read mount points and free space
    let mut mount_map: HashMap<String, Vec<(String, u64)>> = HashMap::new();
    if let Ok(mounts) = std::fs::read_to_string("/proc/mounts") {
        for line in mounts.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let dev = parts[0];
                let mount = parts[1];
                if dev.starts_with("/dev/") {
                    if let Some(base) = find_base_device(&dev[5..]) {
                        // Get free space using statvfs
                        let free_bytes = unsafe {
                            let mut stat: libc::statvfs = std::mem::zeroed();
                            let mount_c = std::ffi::CString::new(mount).unwrap();
                            if libc::statvfs(mount_c.as_ptr(), &mut stat) == 0 {
                                stat.f_bavail * stat.f_bsize as u64
                            } else {
                                0
                            }
                        };
                        mount_map
                            .entry(base)
                            .or_default()
                            .push((mount.to_string(), free_bytes));
                    }
                }
            }
        }
    }

    let mut disks = Vec::new();

    let block_dir =
        std::fs::read_dir("/sys/block").map_err(|e| format!("Failed to read /sys/block: {}", e))?;

    for entry in block_dir {
        let entry = entry.map_err(|e| format!("Dir entry error: {}", e))?;
        let name = entry.file_name().to_string_lossy().to_string();

        if !is_physical_disk(&name) {
            continue;
        }

        let size = read_sysfs(&format!("/sys/block/{}/size", name))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0)
            * 512;

        if size == 0 {
            continue;
        }

        let model = read_sysfs(&format!("/sys/block/{}/device/model", name))
            .ok()
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| name.clone());

        let vendor = read_sysfs(&format!("/sys/block/{}/device/vendor", name))
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && s != "ATA");

        let caption = match vendor {
            Some(v) => format!("{} {}", v, model),
            None => model,
        };

        let removable = read_sysfs(&format!("/sys/block/{}/removable", name))
            .ok()
            .map(|s| s.trim() == "1")
            .unwrap_or(false);

        let rotational = read_sysfs(&format!("/sys/block/{}/queue/rotational", name))
            .ok()
            .map(|s| s.trim() == "1")
            .unwrap_or(true);

        let media_type = if removable {
            Some("Removable".to_string())
        } else if !rotational {
            Some("SSD".to_string())
        } else {
            Some("HDD".to_string())
        };

        let partitions = std::fs::read_dir(format!("/sys/block/{}", name))
            .map(|dir| {
                dir.filter_map(|e| e.ok())
                    .filter(|e| {
                        let fname = e.file_name().to_string_lossy().to_string();
                        if fname.starts_with(&name) && fname.len() > name.len() {
                            let rest = &fname[name.len()..];
                            rest.chars().all(|c| c.is_ascii_digit() || c == 'p')
                        } else {
                            false
                        }
                    })
                    .count() as u32
            })
            .unwrap_or(0);

        let mount_data = mount_map.get(&name).cloned().unwrap_or_default();
        let mount_points: Vec<String> = mount_data.iter().map(|(m, _)| m.clone()).collect();
        let available_space: u64 = mount_data.iter().map(|(_, f)| f).sum();

        let drive_letters = if mount_points.is_empty() {
            None
        } else {
            Some(mount_points.join(", "))
        };

        let device_id = format!("/dev/{}", name);

        disks.push(DiskInfo {
            device_id,
            caption,
            size,
            available_space,
            media_type,
            partitions,
            drive_letters,
        });
    }

    Ok(disks)
}

// ============================================================================
// Admin / Elevation Check
// ============================================================================

#[cfg(target_os = "windows")]
pub fn is_elevated() -> bool {
    use std::ffi::c_void;

    const TOKEN_QUERY: u32 = 0x0008;
    const TOKEN_ELEVATION: u32 = 20;

    #[repr(C)]
    struct TokenElevation {
        token_is_elevated: u32,
    }

    extern "system" {
        fn GetCurrentProcess() -> *mut c_void;
        fn OpenProcessToken(
            process: *mut c_void,
            desired_access: u32,
            token_handle: *mut *mut c_void,
        ) -> i32;
        fn GetTokenInformation(
            token_handle: *mut c_void,
            token_information_class: u32,
            token_information: *mut c_void,
            token_information_length: u32,
            return_length: *mut u32,
        ) -> i32;
        fn CloseHandle(h_object: *mut c_void) -> i32;
    }

    unsafe {
        let mut token: *mut c_void = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
            return false;
        }

        let mut elevation = TokenElevation {
            token_is_elevated: 0,
        };
        let size = std::mem::size_of::<TokenElevation>() as u32;
        let mut ret_len: u32 = 0;

        let result = GetTokenInformation(
            token,
            TOKEN_ELEVATION,
            &mut elevation as *mut _ as *mut c_void,
            size,
            &mut ret_len,
        );

        CloseHandle(token);

        result != 0 && elevation.token_is_elevated != 0
    }
}

#[cfg(target_os = "linux")]
pub fn is_elevated() -> bool {
    unsafe { libc::geteuid() == 0 }
}

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn is_elevated() -> bool {
    false
}

// ============================================================================
// Unsupported Platform
// ============================================================================
#[cfg(not(any(target_os = "windows", target_os = "linux")))]
pub fn list_physical_disks() -> Result<Vec<DiskInfo>, String> {
    Err("Unsupported platform. Only Windows and Linux are supported.".to_string())
}
