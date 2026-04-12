use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use tokio::sync::Mutex;

#[derive(Clone)]
pub struct MacroRuntime {
    inner: Arc<Mutex<Option<RunningMacro>>>,
}

struct RunningMacro {
    name: String,
    cancel_signal: Arc<AtomicBool>,
}

impl MacroRuntime {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn begin(&self, name: String) -> Result<Arc<AtomicBool>, String> {
        let mut guard = self.inner.lock().await;
        if let Some(current) = guard.as_ref() {
            return Err(format!("Macro '{}' is already running.", current.name));
        }

        let cancel_signal = Arc::new(AtomicBool::new(false));
        *guard = Some(RunningMacro {
            name,
            cancel_signal: cancel_signal.clone(),
        });

        Ok(cancel_signal)
    }

    pub async fn stop_running(&self) -> Option<String> {
        let guard = self.inner.lock().await;
        if let Some(current) = guard.as_ref() {
            current.cancel_signal.store(true, Ordering::SeqCst);
            return Some(current.name.clone());
        }
        None
    }

    pub async fn stop_named(&self, requested_name: &str) -> Result<Option<String>, String> {
        let guard = self.inner.lock().await;
        let Some(current) = guard.as_ref() else {
            return Ok(None);
        };

        if current.name.eq_ignore_ascii_case(requested_name) {
            current.cancel_signal.store(true, Ordering::SeqCst);
            return Ok(Some(current.name.clone()));
        }

        Err(format!(
            "Macro '{}' is running; requested stop for '{}'.",
            current.name, requested_name
        ))
    }

    pub async fn clear(&self) {
        let mut guard = self.inner.lock().await;
        *guard = None;
    }
}
