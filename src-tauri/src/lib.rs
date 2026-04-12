use std::{sync::{mpsc, Arc}, thread};
use tokio::sync::Mutex;
use crate::orchestrator::{Orchestrator, TaskResult};
use crate::macro_runtime::MacroRuntime;

use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Manager, RunEvent,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

pub mod modules;
pub mod orchestrator;
mod api_server;
mod macro_runtime;
use modules::io_controller;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(move |app, shortcut, event| {
                    if let (ShortcutState::Pressed, Some(window)) =
                        (event.state(), app.get_webview_window("main"))
                    {
                        if shortcut.matches(Modifiers::empty(), Code::F4) {
                            toggle_window_visibility(&window);
                        }
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            // IO Controller Commands
            io_controller::execute_mouse_move,
            io_controller::execute_mouse_click,
            io_controller::execute_key_press,
            io_controller::execute_key_release,
            io_controller::execute_type_string,
            io_controller::test_io,
            // Orchestrator Commands
            commands::execute_task_command,
            commands::get_app_state_command,
            commands::start_recording_command,
            commands::stop_recording_command,
            // New Macro Commands
            commands::play_macro_command,
            commands::list_macros_command,
            commands::list_macro_configs_command,
            commands::set_macro_profile_command,
            commands::delete_macro_command,
            commands::stop_running_macro_command,
            // Gemini API Commands
            commands::set_gemini_api_key,
            commands::test_gemini_api
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Setup the orchestrator and add it to the managed state
            let orchestrator = Orchestrator::new(app.handle().clone());
            let orchestrator_state = Arc::new(Mutex::new(orchestrator));
            app.manage(orchestrator_state.clone());
            let macro_runtime = MacroRuntime::new();
            app.manage(macro_runtime.clone());
            api_server::spawn_macro_api_server(
                app.handle().clone(),
                orchestrator_state.clone(),
                macro_runtime,
            );

            // Create a channel to send events from the listener to the processor
            let (tx, rx) = mpsc::channel::<rdev::Event>();

            // Spawn the event listener thread
            thread::spawn(move || {
                if let Err(error) = rdev::listen(move |event| {
                    if tx.send(event).is_err() {
                        log::error!("Failed to send event, receiver has likely been dropped.");
                    }
                }) {
                    log::error!("Could not listen for events: {:?}", error);
                }
            });

            tauri::async_runtime::spawn(orchestrator::event_processor_task(orchestrator_state.clone(), rx));

            app.global_shortcut()
                .register(Shortcut::new(None, Code::F4))?;

            // Set up window event listeners to prevent hiding on blur
            if let Some(window) = app.get_webview_window("main") {
                // Set always on top
                let _ = window.set_always_on_top(true);
                
                // Listen for window blur events and prevent hiding
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { .. } = event {
                        // Prevent closing, just hide instead
                        let _ = window_clone.hide();
                    }
                });
            }

            // Create menu items
            let toggle_i = MenuItem::with_id(app, "toggle", "Show/Hide Agent", true, None::<&str>)?;
            let settings_i = MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            // Create menu with items
            let menu = Menu::with_items(app, &[&toggle_i, &settings_i, &quit_i])?;

            // Build the tray icon with menu and event handlers
            // Decode the PNG image to get RGBA data and dimensions
            let image_bytes = include_bytes!("../icons/image.png");
            let img = image::load_from_memory(image_bytes)
                .map_err(|e| {
                    tauri::Error::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to load image: {}", e),
                    ))
                })?;
            let rgba_img = img.to_rgba8();
            let (width, height) = rgba_img.dimensions();
            let rgba_data = rgba_img.into_raw();

            let tray_icon = tauri::image::Image::new_owned(rgba_data, width, height);

            let _tray = TrayIconBuilder::new()
                .icon(tray_icon)
                .menu(&menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "toggle" => {
                            if let Some(window) = app.get_webview_window("main") {
                                toggle_window_visibility(&window);
                            }
                        }
                        "settings" => {
                            println!("Settings menu item clicked!");
                            // In the future, you would open a settings window here
                        }
                        "quit" => {
                            app.exit(0);
                        }
                        _ => {
                            println!("menu item {:?} not handled", event.id);
                        }
                    }
                })
                .on_tray_icon_event(|tray, event| match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } => {
                        // Show and focus the main window on left click
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    _ => {}
                })
                .build(app)?;

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while running tauri application")
        .run(|_app_handle, event| {
            if let RunEvent::ExitRequested { api, .. } = event {
                api.prevent_exit();
            }
        });
}

// Helper function to avoid duplicating the show/hide logic
fn toggle_window_visibility(window: &tauri::WebviewWindow) {
    if let Ok(true) = window.is_visible() {
        let _ = window.hide();
    } else {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

mod commands {
    use super::*;
    use crate::modules::macro_engine;
    use crate::modules::cognition;
    use crate::macro_runtime::MacroRuntime;
    use tauri::State;

    #[tauri::command]
    pub async fn execute_task_command(
        task: String,
        orchestrator_state: State<'_, Arc<Mutex<Orchestrator>>>,
    ) -> Result<TaskResult, String> {
        let mut orchestrator = orchestrator_state.lock().await;
        orchestrator.execute_task(task).await.map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub async fn get_app_state_command(orchestrator_state: State<'_, Arc<Mutex<Orchestrator>>>) -> Result<String, String> {
        let orchestrator = orchestrator_state.lock().await;
        Ok(serde_json::to_string(&orchestrator.state).unwrap_or_default())
    }

    #[tauri::command]
    pub async fn start_recording_command(
        orchestrator_state: State<'_, Arc<Mutex<Orchestrator>>>,
    ) -> Result<(), String> {
        let mut orchestrator = orchestrator_state.lock().await;
        orchestrator.start_recording().map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub async fn stop_recording_command(
        name: String,
        orchestrator_state: State<'_, Arc<Mutex<Orchestrator>>>,
        app_handle: tauri::AppHandle,
    ) -> Result<(), String> {
        let mut orchestrator = orchestrator_state.lock().await;
        orchestrator.stop_recording(name.clone()).map_err(|e| e.to_string())?;
        let profile = macro_engine::load_macro_profile(&name, &app_handle).map_err(|e| e.to_string())?;
        macro_engine::save_macro_profile(&profile, &app_handle).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub async fn play_macro_command(
        name: String,
        app_handle: tauri::AppHandle,
        orchestrator_state: State<'_, Arc<Mutex<Orchestrator>>>,
        macro_runtime: State<'_, MacroRuntime>,
    ) -> Result<(), String> {
        let cancel_signal = macro_runtime.begin(name.clone()).await?;

        // Set state to EXECUTING
        {
            let mut orchestrator = orchestrator_state.lock().await;
            if orchestrator.state != orchestrator::AppState::IDLE {
                macro_runtime.clear().await;
                return Err("Cannot play macro while the agent is not idle.".to_string());
            }
            orchestrator
                .start_executing(format!("Playing macro: {}", name))
                .map_err(|e| e.to_string())?;
        }

        // Load the macro
        let macro_data =
            macro_engine::load_macro(&name, &app_handle).map_err(|e| e.to_string())?;

        // Play the macro in a blocking thread to not freeze the UI
        let play_result = tokio::task::spawn_blocking(move || macro_engine::play_macro_with_cancel(&macro_data, cancel_signal))
            .await
            .map_err(|e| format!("Task join error: {}", e))?;

        // Set state back to IDLE
        {
            let mut orchestrator = orchestrator_state.lock().await;
            orchestrator.stop().map_err(|e| e.to_string())?;
        }
        macro_runtime.clear().await;

        play_result.map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn list_macros_command(app_handle: tauri::AppHandle) -> Result<Vec<String>, String> {
        macro_engine::list_macros(&app_handle).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn list_macro_configs_command(
        app_handle: tauri::AppHandle,
    ) -> Result<Vec<macro_engine::MacroConfig>, String> {
        macro_engine::list_macro_configs(&app_handle).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn set_macro_profile_command(
        name: String,
        method: macro_engine::MacroHttpMethod,
        input_kind: macro_engine::MacroDataKind,
        output_kind: macro_engine::MacroDataKind,
        description: String,
        use_when: Option<String>,
        app_handle: tauri::AppHandle,
    ) -> Result<(), String> {
        let profile = macro_engine::MacroProfile {
            name,
            method,
            input_kind,
            output_kind,
            description,
            use_when: use_when.unwrap_or_default(),
        };
        macro_engine::save_macro_profile(&profile, &app_handle).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn delete_macro_command(name: String, app_handle: tauri::AppHandle) -> Result<(), String> {
        macro_engine::delete_macro(&name, &app_handle).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub async fn stop_running_macro_command(
        macro_runtime: State<'_, MacroRuntime>,
    ) -> Result<String, String> {
        match macro_runtime.stop_running().await {
            Some(name) => Ok(format!("Stop signal sent to macro '{}'.", name)),
            None => Ok("No macro is currently running.".to_string()),
        }
    }

    #[tauri::command]
    pub fn set_gemini_api_key(api_key: String) -> Result<(), String> {
        cognition::set_api_key(&api_key).map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub async fn test_gemini_api(prompt: Option<String>) -> Result<String, String> {
        let test_prompt = prompt.unwrap_or_else(|| "Say hello in a friendly way!".to_string());
        cognition::ask_gemini(&test_prompt).await.map_err(|e| e.to_string())
    }
}
