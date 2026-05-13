mod clone;
mod disk;
mod state;

use state::AppState;
use std::sync::atomic::Ordering;

#[tauri::command]
fn list_disks() -> Result<Vec<disk::DiskInfo>, String> {
    disk::list_physical_disks()
}

#[tauri::command]
fn format_size(bytes: u64) -> String {
    disk::format_size(bytes)
}

#[tauri::command]
async fn start_clone(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    source_path: String,
    output_path: String,
    total_size: u64,
    verify: bool,
) -> Result<(), String> {
    if state.is_running.load(Ordering::SeqCst) {
        return Err("An operation is already running".to_string());
    }
    state.is_running.store(true, Ordering::SeqCst);
    state.cancel_flag.store(false, Ordering::SeqCst);

    let cancel_flag = state.cancel_flag.clone();
    let is_running = state.is_running.clone();
    let app_handle = app.clone();

    let result = tauri::async_runtime::spawn_blocking(move || {
        let result = clone::clone_disk_to_image(
            &app_handle,
            &source_path,
            &output_path,
            total_size,
            &cancel_flag,
            verify,
        );
        is_running.store(false, Ordering::SeqCst);
        cancel_flag.store(false, Ordering::SeqCst);
        result
    })
    .await;

    match result {
        Ok(inner) => inner,
        Err(e) => {
            state.is_running.store(false, Ordering::SeqCst);
            state.cancel_flag.store(false, Ordering::SeqCst);
            Err(format!("Task error: {}", e))
        }
    }
}

#[tauri::command]
async fn start_restore(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    image_path: String,
    target_path: String,
) -> Result<(), String> {
    if state.is_running.load(Ordering::SeqCst) {
        return Err("An operation is already running".to_string());
    }
    state.is_running.store(true, Ordering::SeqCst);
    state.cancel_flag.store(false, Ordering::SeqCst);

    let cancel_flag = state.cancel_flag.clone();
    let is_running = state.is_running.clone();
    let app_handle = app.clone();

    let result = tauri::async_runtime::spawn_blocking(move || {
        let result = clone::restore_image_to_disk(
            &app_handle,
            &image_path,
            &target_path,
            &cancel_flag,
        );
        is_running.store(false, Ordering::SeqCst);
        cancel_flag.store(false, Ordering::SeqCst);
        result
    })
    .await;

    match result {
        Ok(inner) => inner,
        Err(e) => {
            state.is_running.store(false, Ordering::SeqCst);
            state.cancel_flag.store(false, Ordering::SeqCst);
            Err(format!("Task error: {}", e))
        }
    }
}

#[tauri::command]
async fn start_verify(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    image_path: String,
) -> Result<String, String> {
    if state.is_running.load(Ordering::SeqCst) {
        return Err("An operation is already running".to_string());
    }
    state.is_running.store(true, Ordering::SeqCst);
    state.cancel_flag.store(false, Ordering::SeqCst);

    let cancel_flag = state.cancel_flag.clone();
    let is_running = state.is_running.clone();
    let app_handle = app.clone();

    let result = tauri::async_runtime::spawn_blocking(move || {
        let result = clone::hash_image_file(&app_handle, &image_path, &cancel_flag);
        is_running.store(false, Ordering::SeqCst);
        cancel_flag.store(false, Ordering::SeqCst);
        result
    })
    .await;

    match result {
        Ok(inner) => inner,
        Err(e) => {
            state.is_running.store(false, Ordering::SeqCst);
            state.cancel_flag.store(false, Ordering::SeqCst);
            Err(format!("Task error: {}", e))
        }
    }
}

#[tauri::command]
async fn start_disk_to_disk(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    source_path: String,
    target_path: String,
    total_size: u64,
    verify: bool,
) -> Result<(), String> {
    if state.is_running.load(Ordering::SeqCst) {
        return Err("An operation is already running".to_string());
    }
    state.is_running.store(true, Ordering::SeqCst);
    state.cancel_flag.store(false, Ordering::SeqCst);

    let cancel_flag = state.cancel_flag.clone();
    let is_running = state.is_running.clone();
    let app_handle = app.clone();

    let result = tauri::async_runtime::spawn_blocking(move || {
        let result = clone::disk_to_disk(
            &app_handle,
            &source_path,
            &target_path,
            total_size,
            &cancel_flag,
            verify,
        );
        is_running.store(false, Ordering::SeqCst);
        cancel_flag.store(false, Ordering::SeqCst);
        result
    })
    .await;

    match result {
        Ok(inner) => inner,
        Err(e) => {
            state.is_running.store(false, Ordering::SeqCst);
            state.cancel_flag.store(false, Ordering::SeqCst);
            Err(format!("Task error: {}", e))
        }
    }
}

#[tauri::command]
fn cancel_operation(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.cancel_flag.store(true, Ordering::SeqCst);
    Ok(())
}

#[tauri::command]
fn is_operation_running(state: tauri::State<'_, AppState>) -> bool {
    state.is_running.load(Ordering::SeqCst)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            list_disks,
            format_size,
            start_clone,
            start_restore,
            start_verify,
            start_disk_to_disk,
            cancel_operation,
            is_operation_running,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}