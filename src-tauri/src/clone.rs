use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::os::windows::fs::OpenOptionsExt;
use std::sync::atomic::Ordering;
use std::time::Instant;
use tauri::{AppHandle, Emitter};

const CHUNK_SIZE: usize = 1024 * 1024;
const PROGRESS_INTERVAL_BYTES: u64 = 64 * 1024 * 1024;

const FILE_SHARE_READ: u32 = 0x00000001;
const FILE_SHARE_WRITE: u32 = 0x00000002;

#[derive(serde::Serialize, Clone)]
pub struct ProgressEvent {
    pub bytes_processed: u64,
    pub total_bytes: u64,
    pub speed_bytes_per_sec: f64,
    pub eta_seconds: f64,
    pub percent: f64,
}

fn emit_progress(app: &AppHandle, event_name: &str, bytes_processed: u64, total_bytes: u64, start_time: &Instant) {
    let elapsed = start_time.elapsed().as_secs_f64();
    let speed = if elapsed > 0.0 {
        bytes_processed as f64 / elapsed
    } else {
        0.0
    };
    let remaining = total_bytes.saturating_sub(bytes_processed);
    let eta = if speed > 0.0 {
        remaining as f64 / speed
    } else {
        0.0
    };
    let percent = if total_bytes > 0 {
        (bytes_processed as f64 / total_bytes as f64) * 100.0
    } else {
        0.0
    };

    let payload = ProgressEvent {
        bytes_processed,
        total_bytes,
        speed_bytes_per_sec: speed,
        eta_seconds: eta,
        percent,
    };

    let _ = app.emit(event_name, payload);
}

pub fn clone_disk_to_image(
    app: &AppHandle,
    source_path: &str,
    output_path: &str,
    total_size: u64,
    cancel_flag: &std::sync::atomic::AtomicBool,
    verify: bool,
) -> Result<(), String> {
    let mut source = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .open(source_path)
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                format!("Permission denied opening '{}'. Run as Administrator.", source_path)
            } else {
                format!("Failed to open source disk '{}': {}", source_path, e)
            }
        })?;

    let mut output = File::create(output_path)
        .map_err(|e| format!("Failed to create image file '{}': {}", output_path, e))?;

    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut bytes_read: u64 = 0;
    let mut last_progress_bytes: u64 = 0;
    let start_time = Instant::now();
    let mut hasher = if verify { Some(Sha256::new()) } else { None };

    loop {
        if cancel_flag.load(Ordering::Relaxed) {
            let _ = fs::remove_file(output_path);
            return Err("Operation cancelled by user".to_string());
        }

        let to_read = if total_size > 0 {
            std::cmp::min(CHUNK_SIZE as u64, total_size - bytes_read) as usize
        } else {
            CHUNK_SIZE
        };

        if total_size > 0 && bytes_read >= total_size {
            break;
        }

        let n = source.read(&mut buffer[..to_read]).map_err(|e| {
            format!("Read error at offset {}: {}", bytes_read, e)
        })?;

        if n == 0 {
            break;
        }

        if let Some(ref mut h) = hasher {
            h.update(&buffer[..n]);
        }

        output.write_all(&buffer[..n]).map_err(|e| {
            format!("Write error at offset {}: {}", bytes_read, e)
        })?;

        bytes_read += n as u64;

        if bytes_read - last_progress_bytes >= PROGRESS_INTERVAL_BYTES {
            emit_progress(app, "clone-progress", bytes_read, total_size, &start_time);
            last_progress_bytes = bytes_read;
        }
    }

    output.flush().map_err(|e| format!("Flush error: {}", e))?;
    drop(output);
    drop(source);

    emit_progress(app, "clone-progress", bytes_read, total_size, &start_time);

    if let Some(hasher) = hasher {
        let source_hash = hex::encode(hasher.finalize());
        let _ = app.emit("clone-hash", serde_json::json!({
            "hash": source_hash,
            "source": source_path,
        }));
        verify_image(app, output_path, &source_hash, cancel_flag)?;
    }

    let _ = app.emit("clone-complete", serde_json::json!({
        "success": true,
        "bytesProcessed": bytes_read,
        "totalBytes": total_size,
    }));

    Ok(())
}

pub fn restore_image_to_disk(
    app: &AppHandle,
    image_path: &str,
    target_path: &str,
    cancel_flag: &std::sync::atomic::AtomicBool,
) -> Result<(), String> {
    let image_metadata = fs::metadata(image_path)
        .map_err(|e| format!("Failed to read image file metadata: {}", e))?;
    let total_size = image_metadata.len();

    let mut image = File::open(image_path)
        .map_err(|e| format!("Failed to open image file '{}': {}", image_path, e))?;

    let mut target = std::fs::OpenOptions::new()
        .write(true)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .open(target_path)
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                format!("Permission denied opening '{}'. Run as Administrator.", target_path)
            } else {
                format!("Failed to open target disk '{}': {}", target_path, e)
            }
        })?;

    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut bytes_written: u64 = 0;
    let mut last_progress_bytes: u64 = 0;
    let start_time = Instant::now();

    loop {
        if cancel_flag.load(Ordering::Relaxed) {
            return Err("Operation cancelled by user".to_string());
        }

        let n = image.read(&mut buffer).map_err(|e| {
            format!("Read error from image at offset {}: {}", bytes_written, e)
        })?;

        if n == 0 {
            break;
        }

        target.write_all(&buffer[..n]).map_err(|e| {
            format!("Write error to disk at offset {}: {}", bytes_written, e)
        })?;

        bytes_written += n as u64;

        if bytes_written - last_progress_bytes >= PROGRESS_INTERVAL_BYTES {
            emit_progress(app, "restore-progress", bytes_written, total_size, &start_time);
            last_progress_bytes = bytes_written;
        }
    }

    target.flush().map_err(|e| format!("Flush error: {}", e))?;

    emit_progress(app, "restore-progress", bytes_written, total_size, &start_time);

    let _ = app.emit("restore-complete", serde_json::json!({
        "success": true,
        "bytesProcessed": bytes_written,
        "totalBytes": total_size,
    }));

    Ok(())
}

pub fn verify_image(
    app: &AppHandle,
    image_path: &str,
    expected_hash: &str,
    cancel_flag: &std::sync::atomic::AtomicBool,
) -> Result<(), String> {
    let metadata = fs::metadata(image_path)
        .map_err(|e| format!("Failed to read image metadata: {}", e))?;
    let total_size = metadata.len();

    let mut file = File::open(image_path)
        .map_err(|e| format!("Failed to open image for verification: {}", e))?;

    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut bytes_read: u64 = 0;
    let mut last_progress_bytes: u64 = 0;
    let start_time = Instant::now();

    loop {
        if cancel_flag.load(Ordering::Relaxed) {
            return Err("Verification cancelled".to_string());
        }

        let n = file.read(&mut buffer).map_err(|e| {
            format!("Read error during verification: {}", e)
        })?;

        if n == 0 {
            break;
        }

        hasher.update(&buffer[..n]);
        bytes_read += n as u64;

        if bytes_read - last_progress_bytes >= PROGRESS_INTERVAL_BYTES {
            emit_progress(app, "verify-progress", bytes_read, total_size, &start_time);
            last_progress_bytes = bytes_read;
        }
    }

    let computed_hash = hex::encode(hasher.finalize());

    if computed_hash.eq_ignore_ascii_case(expected_hash) {
        let _ = app.emit("verify-complete", serde_json::json!({
            "success": true,
            "message": "Image verification passed: SHA-256 hash matches",
        }));
        Ok(())
    } else {
        let _ = app.emit("verify-complete", serde_json::json!({
            "success": false,
            "message": "Image verification FAILED: SHA-256 hash mismatch",
        }));
        Err(format!(
            "SHA-256 hash mismatch!\nExpected: {}\nComputed: {}",
            expected_hash, computed_hash
        ))
    }
}

pub fn hash_image_file(
    app: &AppHandle,
    image_path: &str,
    cancel_flag: &std::sync::atomic::AtomicBool,
) -> Result<String, String> {
    let metadata = fs::metadata(image_path)
        .map_err(|e| format!("Failed to read image metadata: {}", e))?;
    let total_size = metadata.len();

    let mut file = File::open(image_path)
        .map_err(|e| format!("Failed to open image for hashing: {}", e))?;

    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut bytes_read: u64 = 0;
    let mut last_progress_bytes: u64 = 0;
    let start_time = Instant::now();

    loop {
        if cancel_flag.load(Ordering::Relaxed) {
            return Err("Hash operation cancelled".to_string());
        }

        let n = file.read(&mut buffer).map_err(|e| {
            format!("Read error during hashing: {}", e)
        })?;

        if n == 0 {
            break;
        }

        hasher.update(&buffer[..n]);
        bytes_read += n as u64;

        if bytes_read - last_progress_bytes >= PROGRESS_INTERVAL_BYTES {
            emit_progress(app, "hash-progress", bytes_read, total_size, &start_time);
            last_progress_bytes = bytes_read;
        }
    }

    let hash = hex::encode(hasher.finalize());

    emit_progress(app, "hash-progress", total_size, total_size, &start_time);

    Ok(hash)
}

pub fn disk_to_disk(
    app: &AppHandle,
    source_path: &str,
    target_path: &str,
    total_size: u64,
    cancel_flag: &std::sync::atomic::AtomicBool,
    verify: bool,
) -> Result<(), String> {
    if source_path.eq_ignore_ascii_case(target_path) {
        return Err("Source and target disks must be different".to_string());
    }

    let mut source = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .open(source_path)
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                format!("Permission denied opening '{}'. Run as Administrator.", source_path)
            } else {
                format!("Failed to open source disk '{}': {}", source_path, e)
            }
        })?;

    let mut target = std::fs::OpenOptions::new()
        .write(true)
        .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
        .open(target_path)
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                format!("Permission denied opening '{}'. Run as Administrator.", target_path)
            } else {
                format!("Failed to open target disk '{}': {}", target_path, e)
            }
        })?;

    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut bytes_copied: u64 = 0;
    let mut last_progress_bytes: u64 = 0;
    let start_time = Instant::now();
    let mut hasher = if verify { Some(Sha256::new()) } else { None };

    loop {
        if cancel_flag.load(Ordering::Relaxed) {
            return Err("Operation cancelled by user".to_string());
        }

        let to_read = if total_size > 0 {
            std::cmp::min(CHUNK_SIZE as u64, total_size - bytes_copied) as usize
        } else {
            CHUNK_SIZE
        };

        if total_size > 0 && bytes_copied >= total_size {
            break;
        }

        let n = source.read(&mut buffer[..to_read]).map_err(|e| {
            format!("Read error at offset {}: {}", bytes_copied, e)
        })?;

        if n == 0 {
            break;
        }

        if let Some(ref mut h) = hasher {
            h.update(&buffer[..n]);
        }

        target.write_all(&buffer[..n]).map_err(|e| {
            format!("Write error at offset {}: {}", bytes_copied, e)
        })?;

        bytes_copied += n as u64;

        if bytes_copied - last_progress_bytes >= PROGRESS_INTERVAL_BYTES {
            emit_progress(app, "disk2disk-progress", bytes_copied, total_size, &start_time);
            last_progress_bytes = bytes_copied;
        }
    }

    target.flush().map_err(|e| format!("Flush error: {}", e))?;
    drop(target);
    drop(source);

    emit_progress(app, "disk2disk-progress", bytes_copied, total_size, &start_time);

    if let Some(hasher) = hasher {
        let source_hash = hex::encode(hasher.finalize());
        let _ = app.emit("disk2disk-hash", serde_json::json!({
            "hash": source_hash,
            "source": source_path,
        }));

        let mut verify_file = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE)
            .open(target_path)
            .map_err(|e| format!("Failed to open target disk for verification: {}", e))?;

        let mut verify_hasher = Sha256::new();
        let mut verify_buffer = vec![0u8; CHUNK_SIZE];
        let mut verify_bytes: u64 = 0;
        let mut verify_last_progress: u64 = 0;
        let verify_start = Instant::now();

        loop {
            if cancel_flag.load(Ordering::Relaxed) {
                return Err("Verification cancelled by user".to_string());
            }

            let to_read = if total_size > 0 {
                std::cmp::min(CHUNK_SIZE as u64, total_size - verify_bytes) as usize
            } else {
                CHUNK_SIZE
            };

            if total_size > 0 && verify_bytes >= total_size {
                break;
            }

            let n = verify_file.read(&mut verify_buffer[..to_read]).map_err(|e| {
                format!("Read error during verification at offset {}: {}", verify_bytes, e)
            })?;

            if n == 0 {
                break;
            }

            verify_hasher.update(&verify_buffer[..n]);
            verify_bytes += n as u64;

            if verify_bytes - verify_last_progress >= PROGRESS_INTERVAL_BYTES {
                emit_progress(app, "disk2disk-verify-progress", verify_bytes, total_size, &verify_start);
                verify_last_progress = verify_bytes;
            }
        }

        let target_hash = hex::encode(verify_hasher.finalize());

        if source_hash.eq_ignore_ascii_case(&target_hash) {
            let _ = app.emit("disk2disk-verify-complete", serde_json::json!({
                "success": true,
                "message": "Disk-to-disk copy verified: SHA-256 hash matches",
            }));
        } else {
            let _ = app.emit("disk2disk-verify-complete", serde_json::json!({
                "success": false,
                "message": "Disk-to-disk verification FAILED: SHA-256 hash mismatch",
            }));
            return Err(format!(
                "SHA-256 verification failed!\nSource: {}\nTarget: {}",
                source_hash, target_hash
            ));
        }
    }

    let _ = app.emit("disk2disk-complete", serde_json::json!({
        "success": true,
        "bytesProcessed": bytes_copied,
        "totalBytes": total_size,
    }));

    Ok(())
}