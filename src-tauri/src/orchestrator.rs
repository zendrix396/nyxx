// src-tauri/src/orchestrator.rs
use serde::Serialize;
use tauri::{Emitter, Manager};
use thiserror::Error;
use std::{sync::{mpsc::Receiver, Arc}, time::{Duration, Instant}, fs};
use tokio::sync::Mutex;
use rdev::Key;

use crate::modules::macro_engine::{Macro, TimedEvent};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum AppState {
    IDLE,
    LISTENING,
    RECORDING,
    EXECUTING,
}

#[derive(Debug, Serialize, Clone)]
pub struct TaskResult {
    pub success: bool,
    pub message: String,
}

#[derive(Error, Debug)]
pub enum OrchestratorError {
    #[error("Invalid state transition: Cannot go from {from:?} to {to:?}")]
    InvalidStateTransition { from: AppState, to: AppState },

    #[error("Cognition module failed: {0}")]
    CognitionError(String),

    #[error("Tooling module failed: {0}")]
    ToolingError(String),

    #[error("File system error: {0}")]
    FileSystemError(String),
    
    #[error("Tauri event emission failed: {0}")]
    EventError(#[from] tauri::Error),
}

// Placeholder structs for other modules
pub struct Perception;
pub struct Cognition;
pub struct Tooling;
pub struct Knowledge;

#[allow(dead_code)]
pub struct Orchestrator {
    pub state: AppState,
    app_handle: tauri::AppHandle,
    session_context: Option<String>,
    // Module references will be added here
    perception: Arc<Mutex<Perception>>,
    cognition: Arc<Mutex<Cognition>>,
    tooling: Arc<Mutex<Tooling>>,
    knowledge: Arc<Mutex<Knowledge>>,
    recording_buffer: Vec<TimedEvent>,
    last_event_time: Option<Instant>,
    last_recorded_event_type: Option<rdev::EventType>,
}

impl Orchestrator {
    pub fn new(app_handle: tauri::AppHandle) -> Self {
        Self {
            state: AppState::IDLE,
            app_handle,
            session_context: None,
            perception: Arc::new(Mutex::new(Perception)),
            cognition: Arc::new(Mutex::new(Cognition)),
            tooling: Arc::new(Mutex::new(Tooling)),
            knowledge: Arc::new(Mutex::new(Knowledge)),
            recording_buffer: Vec::new(),
            last_event_time: None,
            last_recorded_event_type: None,
        }
    }

    fn set_state(&mut self, new_state: AppState) -> Result<(), OrchestratorError> {
        log::info!("State transition: {:?} -> {:?}", self.state, new_state);
        self.state = new_state.clone();
        self.app_handle.emit("app_state_changed", new_state)?;
        Ok(())
    }

    pub fn start_listening(&mut self) -> Result<(), OrchestratorError> {
        if self.state != AppState::IDLE {
            return Err(OrchestratorError::InvalidStateTransition {
                from: self.state.clone(),
                to: AppState::LISTENING,
            });
        }
        self.set_state(AppState::LISTENING)
    }

    pub fn start_recording(&mut self) -> Result<(), OrchestratorError> {
        if self.state != AppState::IDLE {
            return Err(OrchestratorError::InvalidStateTransition {
                from: self.state.clone(),
                to: AppState::RECORDING,
            });
        }
        log::info!("Starting macro recording...");
        self.recording_buffer.clear();
        self.last_event_time = None; // Reset timer for the new recording
        self.last_recorded_event_type = None; // Reset last recorded event type
        self.set_state(AppState::RECORDING)
    }

    pub fn stop_recording(&mut self, name: String) -> Result<(), OrchestratorError> {
        if self.state != AppState::RECORDING {
            return Err(OrchestratorError::InvalidStateTransition {
                from: self.state.clone(),
                to: AppState::IDLE,
            });
        }
    
        log::info!("Stopping recording for macro: {}", name);
        log::info!("Recorded {} events", self.recording_buffer.len());
    
        // --- NEW: Heuristic to fix missed initial Win key press ---
        if let Some(first_event) = self.recording_buffer.first() {
            // Check if the macro starts with typing, not a modifier.
            if let rdev::EventType::KeyPress(key) = first_event.event_type {
                if !matches!(key, Key::ControlLeft | Key::ControlRight | Key::ShiftLeft | Key::ShiftRight | Key::Alt | Key::AltGr | Key::MetaLeft | Key::MetaRight) {
                    
                    // Now, find the first Win key press/release pair that happened *later*.
                    let meta_press_pos = self.recording_buffer.iter().position(|ev| ev.event_type == rdev::EventType::KeyPress(Key::MetaLeft));
                    let meta_release_pos = self.recording_buffer.iter().position(|ev| ev.event_type == rdev::EventType::KeyRelease(Key::MetaLeft));
                    
                    if let (Some(press_idx), Some(release_idx)) = (meta_press_pos, meta_release_pos) {
                        if release_idx > press_idx {
                            log::warn!("[Heuristic] Detected typing before a Win key press. Attempting to auto-correct macro.");

                            // Remove the events from their incorrect position. Must remove from back to front.
                            let release_event = self.recording_buffer.remove(release_idx);
                            let press_event = self.recording_buffer.remove(press_idx);

                            // Insert them correctly at the very beginning.
                            self.recording_buffer.insert(0, release_event);
                            self.recording_buffer.insert(0, press_event);
                            
                            // Adjust timings for reliable playback.
                            self.recording_buffer[0].time_since_previous = Duration::ZERO; // Win press is instant.
                            self.recording_buffer[1].time_since_previous = Duration::from_millis(80); // Hold Win for 80ms.
                            self.recording_buffer[2].time_since_previous = Duration::from_millis(300); // Wait for Start Menu.

                            log::info!("[Heuristic] Macro event order corrected successfully.");
                        }
                    }
                }
            }
        }
        // --- END of Heuristic ---
    
        let macro_data = Macro {
            name: name.clone(),
            events: self.recording_buffer.clone(),
        };
    
        let json_string = serde_json::to_string_pretty(&macro_data)
            .map_err(|e| OrchestratorError::FileSystemError(format!("Serialization failed: {}", e)))?;
    
        let config_dir = self.app_handle.path().app_config_dir()
            .map_err(|e| OrchestratorError::FileSystemError(format!("Failed to get config dir: {}", e)))?;
        let macros_dir = config_dir.join("nyx-agent/macros");
        
        log::info!("Saving macro to: {:?}", macros_dir);
        
        fs::create_dir_all(&macros_dir)
            .map_err(|e| OrchestratorError::FileSystemError(format!("Failed to create macros dir: {}", e)))?;
    
        let file_path = macros_dir.join(format!("{}.json", name));
        fs::write(&file_path, json_string)
            .map_err(|e| OrchestratorError::FileSystemError(format!("Failed to write macro file: {}", e)))?;
    
        log::info!("Macro saved successfully to: {:?}", file_path);
    
        self.stop()
    }
    
    pub fn start_executing(&mut self, task: String) -> Result<(), OrchestratorError> {
        if self.state != AppState::IDLE && self.state != AppState::LISTENING {
            return Err(OrchestratorError::InvalidStateTransition {
                from: self.state.clone(),
                to: AppState::EXECUTING,
            });
        }
        self.session_context = Some(task);
        self.set_state(AppState::EXECUTING)
    }

    pub fn stop(&mut self) -> Result<(), OrchestratorError> {
        self.session_context = None;
        self.set_state(AppState::IDLE)
    }

    pub async fn execute_task(&mut self, task_description: String) -> Result<TaskResult, OrchestratorError> {
        self.start_executing(task_description.clone())?;

        // 1. Call Cognition to get a plan
        self.app_handle.emit("task_progress", "Generating plan...")?;
        log::info!("Cognition: Generating plan for task: '{}'", task_description);
        // let plan = self.cognition.lock().unwrap().generate_plan(&task_description)?; // Real implementation
        tokio::time::sleep(std::time::Duration::from_secs(1)).await; // Placeholder for async work

        // 2. Iterate through plan and execute steps
        log::info!("Tooling: Executing plan steps...");
        for i in 1..=3 {
            self.app_handle.emit("task_progress", format!("Executing step {} of 3...", i))?;
            // self.tooling.lock().unwrap().execute_step(step)?; // Real implementation
            log::info!("Executing step {}", i);
            tokio::time::sleep(std::time::Duration::from_millis(500)).await; // Placeholder
        }

        // 3. Log to Knowledge Base
        log::info!("Knowledge: Logging execution results...");
        // self.knowledge.lock().unwrap().log_task(result)?; // Real implementation

        // 4. Return to IDLE
        self.stop()?;

        Ok(TaskResult {
            success: true,
            message: "Task completed successfully!".to_string(),
        })
    }

    // This function will be called by the event processor task
    pub fn handle_event(&mut self, event: rdev::Event) {
        if self.state != AppState::RECORDING {
            return;
        }

        // --- ONLY keep the key-repeat filter ---
        if let Some(last_event) = &self.last_recorded_event_type {
            if last_event == &event.event_type {
                if matches!(event.event_type, rdev::EventType::KeyPress { .. }) {
                    return;
                }
            }
        }

        let now = Instant::now();
        let time_since_previous = self
            .last_event_time
            .map_or(Duration::ZERO, |last_time| now.duration_since(last_time));

        self.recording_buffer.push(TimedEvent {
            event_type: event.event_type.clone(),
            time_since_previous,
        });

        self.last_recorded_event_type = Some(event.event_type.clone());
        self.last_event_time = Some(now);

        if self.recording_buffer.len() % 100 == 0 {
            log::info!("Recorded {} events so far...", self.recording_buffer.len());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tauri;
    use std::sync::{Arc, Mutex};

    // Mock AppHandle setup for testing is complex.
    // These tests will focus on the state logic, assuming event emission works.
    
    fn create_orchestrator_for_test() -> Orchestrator {
        // We can't easily mock AppHandle, so we'll need to build a minimal app instance for testing.
        // For pure unit tests, we'd abstract the event emitter behind a trait.
        // Given the scope, we will test the logic, and trust the tauri::Error handling.
        let app = tauri::Builder::default().build(tauri::generate_context!()).unwrap();
        Orchestrator::new(app.handle().clone())
    }

    #[test]
    fn test_initial_state_is_idle() {
        let orchestrator = create_orchestrator_for_test();
        assert_eq!(orchestrator.state, AppState::IDLE);
    }

    #[test]
    fn test_start_listening_from_idle() {
        let mut orchestrator = create_orchestrator_for_test();
        assert!(orchestrator.start_listening().is_ok());
        assert_eq!(orchestrator.state, AppState::LISTENING);
    }

    #[test]
    fn test_fail_start_listening_from_executing() {
        let mut orchestrator = create_orchestrator_for_test();
        orchestrator.state = AppState::EXECUTING;
        assert!(orchestrator.start_listening().is_err());
    }

    #[test]
    fn test_start_recording_from_idle() {
        let mut orchestrator = create_orchestrator_for_test();
        assert!(orchestrator.start_recording().is_ok());
        assert_eq!(orchestrator.state, AppState::RECORDING);
    }

    #[test]
    fn test_start_executing_from_idle() {
        let mut orchestrator = create_orchestrator_for_test();
        let task = "test task".to_string();
        assert!(orchestrator.start_executing(task.clone()).is_ok());
        assert_eq!(orchestrator.state, AppState::EXECUTING);
        assert_eq!(orchestrator.session_context, Some(task));
    }

    #[test]
    fn test_stop_returns_to_idle() {
        let mut orchestrator = create_orchestrator_for_test();
        orchestrator.state = AppState::EXECUTING;
        orchestrator.session_context = Some("a task".to_string());
        
        assert!(orchestrator.stop().is_ok());
        assert_eq!(orchestrator.state, AppState::IDLE);
        assert!(orchestrator.session_context.is_none());
    }

    #[tokio::test]
    async fn test_cognitive_loop_flow() {
        let mut orchestrator = create_orchestrator_for_test();
        let task = "a trivial plan".to_string();
        
        let result = orchestrator.execute_task(task).await;

        assert!(result.is_ok());
        let task_result = result.unwrap();
        assert!(task_result.success);
        assert_eq!(orchestrator.state, AppState::IDLE);
    }
}

pub async fn event_processor_task(
    orchestrator_state: Arc<Mutex<Orchestrator>>,
    receiver: Receiver<rdev::Event>,
) {
    for event in receiver {
        let mut orchestrator = orchestrator_state.lock().await;
        orchestrator.handle_event(event);
    }
}

