use std::sync::atomic::AtomicBool;
use std::sync::Arc;

pub struct AppState {
    pub cancel_flag: Arc<AtomicBool>,
    pub is_running: Arc<AtomicBool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            cancel_flag: Arc::new(AtomicBool::new(false)),
            is_running: Arc::new(AtomicBool::new(false)),
        }
    }
}