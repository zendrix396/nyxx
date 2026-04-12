use std::{
    borrow::Cow,
    io::Write,
    net::SocketAddr,
    process::{Command, Stdio},
    sync::Arc,
    thread,
    time::Duration,
};

use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

use crate::macro_runtime::MacroRuntime;
use crate::modules::{io_controller, macro_engine};
use crate::orchestrator::{AppState, Orchestrator};
use rdev::{EventType, Key};

#[derive(Clone)]
struct ApiServerState {
    app_handle: tauri::AppHandle,
    orchestrator_state: Arc<Mutex<Orchestrator>>,
    macro_runtime: MacroRuntime,
}

#[derive(Serialize)]
struct ApiMessage {
    success: bool,
    message: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

#[derive(Serialize)]
struct StateResponse {
    state: AppState,
}

#[derive(Deserialize)]
struct StopRecordingRequest {
    name: String,
    method: Option<macro_engine::MacroHttpMethod>,
    input_kind: Option<macro_engine::MacroDataKind>,
    output_kind: Option<macro_engine::MacroDataKind>,
    description: Option<String>,
    use_when: Option<String>,
}

#[derive(Deserialize)]
struct MouseMoveRequest {
    x: f64,
    y: f64,
}

#[derive(Deserialize)]
struct MouseDragRequest {
    delta_x: f64,
    delta_y: f64,
}

#[derive(Deserialize, Clone)]
struct InvokeBody {
    #[serde(alias = "input", alias = "payload", alias = "prompt")]
    text: Option<String>,
    #[serde(alias = "imageBase64", alias = "image_data")]
    image_base64: Option<String>,
    #[serde(alias = "attachment", alias = "filePath", alias = "path")]
    attachment_path: Option<String>,
}

#[derive(Deserialize)]
struct InvokeQuery {
    input: Option<String>,
}

#[derive(Serialize)]
struct InvokeResponse {
    success: bool,
    macro_name: String,
    message: String,
    output_text: Option<String>,
    output_image_base64: Option<String>,
}

#[derive(Serialize)]
struct EndpointHelp {
    method: &'static str,
    path: &'static str,
    description: &'static str,
}

#[derive(Serialize)]
struct MacroAboutResponse {
    name: String,
    description: String,
    use_when: String,
    method: macro_engine::MacroHttpMethod,
    input_kind: macro_engine::MacroDataKind,
    output_kind: macro_engine::MacroDataKind,
    accepts: String,
    returns: String,
    invoke_endpoint: String,
}

#[derive(Serialize)]
struct HelpResponse {
    service: &'static str,
    base_url: &'static str,
    endpoints: Vec<EndpointHelp>,
    macros: Vec<MacroAboutResponse>,
}

pub fn spawn_macro_api_server(
    app_handle: tauri::AppHandle,
    orchestrator_state: Arc<Mutex<Orchestrator>>,
    macro_runtime: MacroRuntime,
) {
    tauri::async_runtime::spawn(async move {
        let port = std::env::var("NYX_API_PORT")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .unwrap_or(4777);

        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let state = ApiServerState {
            app_handle,
            orchestrator_state,
            macro_runtime,
        };

        let app = Router::new()
            .route("/health", get(health))
            .route("/help", get(help))
            .route("/state", get(get_state))
            .route("/macros", get(list_macros))
            .route("/macros/:name", get(get_macro).delete(delete_macro))
            .route("/macros/:name/help", get(macro_help))
            .route("/macros/:name/invoke", get(invoke_macro_get).post(invoke_macro_post))
            .route("/macros/:name/run", post(run_macro_legacy))
            .route("/macros/:name/stop", post(stop_macro_named))
            .route("/macros/stop", post(stop_macro))
            .route("/macros/stop-all", post(stop_macro_all))
            .route("/recording/start", post(start_recording))
            .route("/recording/stop", post(stop_recording))
            .route("/mouse/move", post(mouse_move_endpoint))
            .route("/mouse/drag", post(mouse_drag_endpoint))
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            )
            .with_state(state);

        let listener = match TcpListener::bind(addr).await {
            Ok(listener) => listener,
            Err(err) => {
                log::error!("Failed to bind macro API on {}: {}", addr, err);
                return;
            }
        };

        log::info!("Macro API listening on http://{}", addr);

        if let Err(err) = axum::serve(listener, app).await {
            log::error!("Macro API server stopped: {}", err);
        }
    });
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "nyx-macro-api",
    })
}

async fn get_state(State(state): State<ApiServerState>) -> Json<StateResponse> {
    let orchestrator = state.orchestrator_state.lock().await;
    Json(StateResponse {
        state: orchestrator.state.clone(),
    })
}

async fn list_macros(
    State(state): State<ApiServerState>,
) -> Result<Json<Vec<macro_engine::MacroConfig>>, (StatusCode, Json<ApiMessage>)> {
    macro_engine::list_macro_configs(&state.app_handle)
        .map(Json)
        .map_err(|err| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiMessage {
                    success: false,
                    message: err.to_string(),
                }),
            )
        })
}

async fn help(
    State(state): State<ApiServerState>,
) -> Result<Json<HelpResponse>, (StatusCode, Json<ApiMessage>)> {
    let configs = macro_engine::list_macro_configs(&state.app_handle).map_err(map_error_status)?;
    let macros = configs
        .into_iter()
        .map(|cfg| macro_about_from_profile(&cfg.name, cfg.profile))
        .collect::<Vec<_>>();

    Ok(Json(HelpResponse {
        service: "nyx-macro-api",
        base_url: "http://127.0.0.1:4777",
        endpoints: vec![
            EndpointHelp { method: "GET", path: "/help", description: "API help portal with routes and macro about docs." },
            EndpointHelp { method: "GET", path: "/health", description: "API health check." },
            EndpointHelp { method: "GET", path: "/state", description: "Current app orchestrator state." },
            EndpointHelp { method: "GET", path: "/macros", description: "List macros with profiles." },
            EndpointHelp { method: "GET", path: "/macros/:name", description: "Fetch one macro profile." },
            EndpointHelp { method: "GET", path: "/macros/:name/help", description: "Detailed about info for one macro." },
            EndpointHelp { method: "GET|POST", path: "/macros/:name/invoke", description: "Invoke macro with method enforced by macro profile." },
            EndpointHelp { method: "POST", path: "/macros/:name/stop", description: "Stop a specific running macro by name." },
            EndpointHelp { method: "POST", path: "/macros/stop", description: "Gracefully stop currently running macro." },
            EndpointHelp { method: "POST", path: "/macros/stop-all", description: "Alias for stop-any-running-macro." },
            EndpointHelp { method: "POST", path: "/recording/start", description: "Start recording a new macro." },
            EndpointHelp { method: "POST", path: "/recording/stop", description: "Stop recording and save profile options." },
            EndpointHelp { method: "DELETE", path: "/macros/:name", description: "Delete macro and profile." },
            EndpointHelp { method: "POST", path: "/mouse/move", description: "Move mouse to absolute x, y coordinates." },
            EndpointHelp { method: "POST", path: "/mouse/drag", description: "Drag mouse relatively by delta_x, delta_y." },
        ],
        macros,
    }))
}

async fn get_macro(
    Path(name): Path<String>,
    State(state): State<ApiServerState>,
) -> Result<Json<macro_engine::MacroConfig>, (StatusCode, Json<ApiMessage>)> {
    let profile = macro_engine::load_macro_profile(&name, &state.app_handle).map_err(map_error_status)?;
    Ok(Json(macro_engine::MacroConfig { name, profile }))
}

async fn macro_help(
    Path(name): Path<String>,
    State(state): State<ApiServerState>,
) -> Result<Json<MacroAboutResponse>, (StatusCode, Json<ApiMessage>)> {
    let profile = macro_engine::load_macro_profile(&name, &state.app_handle).map_err(map_error_status)?;
    Ok(Json(macro_about_from_profile(&name, profile)))
}

async fn delete_macro(
    Path(name): Path<String>,
    State(state): State<ApiServerState>,
) -> Result<Json<ApiMessage>, (StatusCode, Json<ApiMessage>)> {
    macro_engine::delete_macro(&name, &state.app_handle).map_err(map_error_status)?;
    Ok(Json(ApiMessage {
        success: true,
        message: format!("Macro '{name}' deleted."),
    }))
}

async fn invoke_macro_get(
    Path(name): Path<String>,
    Query(query): Query<InvokeQuery>,
    State(state): State<ApiServerState>,
) -> Result<Json<InvokeResponse>, (StatusCode, Json<ApiMessage>)> {
    invoke_macro_impl(
        name,
        InvokeBody {
            text: query.input,
            image_base64: None,
            attachment_path: None,
        },
        false,
        state,
    )
    .await
}

async fn invoke_macro_post(
    Path(name): Path<String>,
    State(state): State<ApiServerState>,
    body: Bytes,
) -> Result<Json<InvokeResponse>, (StatusCode, Json<ApiMessage>)> {
    let raw_preview = String::from_utf8_lossy(&body);
    let payload = parse_invoke_body(&body);
    log::info!(
        "invoke_macro_post '{}': raw_len={}, raw_preview={:?}, text={:?}, image={}, attach={:?}",
        name,
        body.len(),
        if raw_preview.len() > 140 {
            format!("{}...", &raw_preview[..140])
        } else {
            raw_preview.to_string()
        },
        payload.text.as_deref().map(|s| if s.len() > 80 {
            format!("{}...", &s[..80])
        } else {
            s.to_string()
        }),
        payload.image_base64.is_some(),
        payload.attachment_path,
    );
    invoke_macro_impl(name, payload, true, state).await
}

fn parse_invoke_body(raw: &[u8]) -> InvokeBody {
    if raw.is_empty() {
        return InvokeBody {
            text: None,
            image_base64: None,
            attachment_path: None,
        };
    }

    let raw = sanitize_raw_body(raw);
    let candidates = body_parse_candidates(&raw);

    for candidate in &candidates {
        if let Ok(parsed) = serde_json::from_str::<InvokeBody>(candidate) {
            if parsed.text.is_some() || parsed.image_base64.is_some() || parsed.attachment_path.is_some() {
                return parsed;
            }
        }
    }

    // Try generic JSON (including nested objects/arrays) and extract a usable text field.
    for candidate in &candidates {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(candidate) {
            if let Some(text) = extract_text_from_json(&value) {
                return InvokeBody {
                    text: Some(text),
                    image_base64: None,
                    attachment_path: None,
                };
            }
        }
    }

    // Try x-www-form-urlencoded payload (common in webhook/proxy tools).
    if let Some(text) = parse_form_encoded_text(&raw) {
        return InvokeBody {
            text: Some(text),
            image_base64: None,
            attachment_path: None,
        };
    }

    // If caller sent JSON as plain string, decode the inner object.
    if !raw.is_empty() {
        let trimmed = raw.trim().trim_matches('"');
        if !trimmed.is_empty() {
            return InvokeBody {
                text: Some(trimmed.to_string()),
                image_base64: None,
                attachment_path: None,
            };
        }
    }

    InvokeBody {
        text: None,
        image_base64: None,
        attachment_path: None,
    }
}

fn sanitize_raw_body(raw: &[u8]) -> String {
    let mut text = String::from_utf8_lossy(raw).to_string();
    if let Some(stripped) = text.strip_prefix('\u{feff}') {
        text = stripped.to_string();
    }
    // Some webhook bridges forward UTF-8 BOM as mojibake text.
    for marker in ["ï»¿", "∩╗┐"] {
        if let Some(stripped) = text.strip_prefix(marker) {
            text = stripped.to_string();
        }
    }
    text.trim().to_string()
}

fn body_parse_candidates(raw: &str) -> Vec<String> {
    let mut out = vec![raw.to_string()];

    if let (Some(start), Some(end)) = (raw.find('{'), raw.rfind('}')) {
        if start < end {
            out.push(raw[start..=end].to_string());
        }
    }

    if let Ok(decoded) = serde_json::from_str::<String>(raw) {
        out.push(decoded);
    }

    if raw.contains("\\\"") {
        let unescaped = raw.replace("\\\"", "\"");
        out.push(unescaped.clone());
        if let Ok(decoded) = serde_json::from_str::<String>(&unescaped) {
            out.push(decoded);
        }
    }

    out.sort();
    out.dedup();
    out
}

fn extract_text_from_json(value: &serde_json::Value) -> Option<String> {
    const PREFERRED_KEYS: &[&str] = &[
        "text", "input", "payload", "prompt", "body", "message", "content", "command", "query",
    ];

    match value {
        serde_json::Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        serde_json::Value::Object(map) => {
            for key in PREFERRED_KEYS {
                if let Some(v) = map.get(*key) {
                    if let Some(found) = extract_text_from_json(v) {
                        return Some(found);
                    }
                }
            }
            for v in map.values() {
                if let Some(found) = extract_text_from_json(v) {
                    return Some(found);
                }
            }
            None
        }
        serde_json::Value::Array(arr) => {
            for v in arr {
                if let Some(found) = extract_text_from_json(v) {
                    return Some(found);
                }
            }
            None
        }
        _ => None,
    }
}

fn parse_form_encoded_text(input: &str) -> Option<String> {
    const PREFERRED_KEYS: &[&str] = &[
        "text", "input", "payload", "prompt", "body", "message", "content", "command", "query",
    ];
    let pairs = url::form_urlencoded::parse(input.as_bytes());
    for (k, v) in pairs {
        if PREFERRED_KEYS.iter().any(|allowed| k.eq_ignore_ascii_case(allowed)) {
            let trimmed = v.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

async fn run_macro_legacy(
    Path(name): Path<String>,
    State(state): State<ApiServerState>,
    body: Bytes,
) -> Result<Json<ApiMessage>, (StatusCode, Json<ApiMessage>)> {
    let payload = parse_invoke_body(&body);
    log::info!(
        "run_macro_legacy '{}': raw_len={}, text={:?}",
        name,
        body.len(),
        payload.text.as_deref().map(|s| if s.len() > 80 {
            format!("{}...", &s[..80])
        } else {
            s.to_string()
        }),
    );
    let _ = invoke_macro_impl(
        name.clone(),
        payload,
        true,
        state,
    )
    .await?;
    Ok(Json(ApiMessage {
        success: true,
        message: format!("Macro '{name}' executed successfully."),
    }))
}

async fn stop_macro(
    State(state): State<ApiServerState>,
) -> Result<Json<ApiMessage>, (StatusCode, Json<ApiMessage>)> {
    let message = match state.macro_runtime.stop_running().await {
        Some(name) => format!("Stop signal sent to macro '{name}'."),
        None => "No macro is currently running.".to_string(),
    };
    Ok(Json(ApiMessage {
        success: true,
        message,
    }))
}

async fn stop_macro_all(
    State(state): State<ApiServerState>,
) -> Result<Json<ApiMessage>, (StatusCode, Json<ApiMessage>)> {
    let message = match state.macro_runtime.stop_running().await {
        Some(name) => format!("Stop signal sent to macro '{name}'."),
        None => "No macro is currently running.".to_string(),
    };
    Ok(Json(ApiMessage {
        success: true,
        message,
    }))
}

async fn stop_macro_named(
    Path(name): Path<String>,
    State(state): State<ApiServerState>,
) -> Result<Json<ApiMessage>, (StatusCode, Json<ApiMessage>)> {
    match state.macro_runtime.stop_named(&name).await {
        Ok(Some(stopped_name)) => Ok(Json(ApiMessage {
            success: true,
            message: format!("Stop signal sent to macro '{stopped_name}'."),
        })),
        Ok(None) => Ok(Json(ApiMessage {
            success: true,
            message: "No macro is currently running.".to_string(),
        })),
        Err(message) => Err((
            StatusCode::CONFLICT,
            Json(ApiMessage {
                success: false,
                message,
            }),
        )),
    }
}

async fn mouse_move_endpoint(
    Json(payload): Json<MouseMoveRequest>,
) -> Result<Json<ApiMessage>, (StatusCode, Json<ApiMessage>)> {
    io_controller::move_mouse(payload.x, payload.y).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiMessage {
                success: false,
                message: format!("Failed to move mouse: {}", err),
            }),
        )
    })?;
    Ok(Json(ApiMessage {
        success: true,
        message: format!("Mouse moved to ({}, {})", payload.x, payload.y),
    }))
}

async fn mouse_drag_endpoint(
    Json(payload): Json<MouseDragRequest>,
) -> Result<Json<ApiMessage>, (StatusCode, Json<ApiMessage>)> {
    io_controller::drag_mouse_relative(payload.delta_x, payload.delta_y).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiMessage {
                success: false,
                message: format!("Failed to drag mouse: {}", err),
            }),
        )
    })?;
    Ok(Json(ApiMessage {
        success: true,
        message: format!("Mouse dragged by ({}, {})", payload.delta_x, payload.delta_y),
    }))
}

async fn start_recording(
    State(state): State<ApiServerState>,
) -> Result<Json<ApiMessage>, (StatusCode, Json<ApiMessage>)> {
    let mut orchestrator = state.orchestrator_state.lock().await;
    orchestrator.start_recording().map_err(map_error_status)?;
    Ok(Json(ApiMessage {
        success: true,
        message: "Recording started.".to_string(),
    }))
}

async fn stop_recording(
    State(state): State<ApiServerState>,
    Json(payload): Json<StopRecordingRequest>,
) -> Result<Json<ApiMessage>, (StatusCode, Json<ApiMessage>)> {
    {
        let mut orchestrator = state.orchestrator_state.lock().await;
        orchestrator
            .stop_recording(payload.name.clone())
            .map_err(map_error_status)?;
    }

    let mut profile =
        macro_engine::load_macro_profile(&payload.name, &state.app_handle).map_err(map_error_status)?;
    if let Some(method) = payload.method {
        profile.method = method;
    }
    if let Some(input_kind) = payload.input_kind {
        profile.input_kind = input_kind;
    }
    if let Some(output_kind) = payload.output_kind {
        profile.output_kind = output_kind;
    }
    if let Some(description) = payload.description {
        profile.description = description;
    }
    if let Some(use_when) = payload.use_when {
        profile.use_when = use_when;
    }
    macro_engine::save_macro_profile(&profile, &state.app_handle).map_err(map_error_status)?;

    Ok(Json(ApiMessage {
        success: true,
        message: format!("Recording '{}' saved with profile.", payload.name),
    }))
}

async fn invoke_macro_impl(
    name: String,
    input: InvokeBody,
    is_post: bool,
    state: ApiServerState,
) -> Result<Json<InvokeResponse>, (StatusCode, Json<ApiMessage>)> {
    let profile = macro_engine::load_macro_profile(&name, &state.app_handle).map_err(map_error_status)?;

    if is_post && !matches!(profile.method, macro_engine::MacroHttpMethod::POST) {
        return Err((
            StatusCode::METHOD_NOT_ALLOWED,
            Json(ApiMessage {
                success: false,
                message: format!("Macro '{name}' only supports GET."),
            }),
        ));
    }

    if !is_post && !matches!(profile.method, macro_engine::MacroHttpMethod::GET) {
        return Err((
            StatusCode::METHOD_NOT_ALLOWED,
            Json(ApiMessage {
                success: false,
                message: format!("Macro '{name}' only supports POST."),
            }),
        ));
    }

    execute_macro(name.clone(), &state, &profile, &input)
        .await
        .map_err(map_error_status)?;

    let (output_text, output_image_base64) = if is_post {
        extract_clipboard_output(&profile).map_err(map_error_status)?
    } else {
        (None, None)
    };

    Ok(Json(InvokeResponse {
        success: true,
        macro_name: name,
        message: "Macro executed successfully.".to_string(),
        output_text,
        output_image_base64,
    }))
}

async fn execute_macro(
    name: String,
    state: &ApiServerState,
    profile: &macro_engine::MacroProfile,
    input: &InvokeBody,
) -> Result<(), String> {
    let cancel_signal = state.macro_runtime.begin(name.clone()).await?;

    {
        let mut orchestrator = state.orchestrator_state.lock().await;
        if orchestrator.state != AppState::IDLE {
            state.macro_runtime.clear().await;
            return Err("Cannot play macro while the agent is not idle.".to_string());
        }
        orchestrator
            .start_executing(format!("Playing macro: {}", name))
            .map_err(|err| err.to_string())?;
    }

    // Required default pre-step: start each macro on a fresh virtual desktop.
    switch_to_new_desktop()?;

    // Set clipboard BEFORE spawning the blocking playback thread.
    // Using clip.exe ensures the content persists in the OS clipboard
    // regardless of thread/process ownership semantics.
    set_clipboard_input(profile, input)?;

    let execution_result = async {
        let macro_data =
            macro_engine::load_macro(&name, &state.app_handle).map_err(|err| err.to_string())?;

        tokio::task::spawn_blocking(move || {
            macro_engine::play_macro_with_cancel(&macro_data, cancel_signal)
        })
        .await
        .map_err(|err| format!("Task join error: {}", err))?
        .map_err(|err| err.to_string())
    }
    .await;

    let stop_result = {
        let mut orchestrator = state.orchestrator_state.lock().await;
        orchestrator.stop().map_err(|err| err.to_string())
    };
    state.macro_runtime.clear().await;

    match (execution_result, stop_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(exec_err), Ok(())) => Err(exec_err),
        (Ok(()), Err(stop_err)) => Err(stop_err),
        (Err(exec_err), Err(stop_err)) => {
            log::error!("Macro execution failed and reset failed: {}", stop_err);
            Err(exec_err)
        }
    }
}

fn set_clipboard_text_powershell(text: &str) -> Result<(), String> {
    let mut child = Command::new("powershell")
        .args(["-NoProfile", "-Command", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| format!("Failed to start powershell: {}", err))?;

    if let Some(stdin) = child.stdin.as_mut() {
        // Escape single quotes for PowerShell and use Set-Clipboard
        let escaped = text.replace('\'', "''");
        let cmd = format!("Set-Clipboard -Value '{}'", escaped);
        stdin
            .write_all(cmd.as_bytes())
            .map_err(|err| format!("Failed writing to powershell stdin: {}", err))?;
    }
    drop(child.stdin.take());

    let output = child
        .wait_with_output()
        .map_err(|err| format!("Failed waiting for powershell: {}", err))?;
    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("PowerShell Set-Clipboard failed: {}", stderr))
    }
}

fn set_clipboard_text_arboard_persistent(text: &str) -> Result<(), String> {
    let mut clipboard =
        arboard::Clipboard::new().map_err(|err| format!("Clipboard init: {}", err))?;
    clipboard
        .set_text(text.to_string())
        .map_err(|err| format!("Clipboard set_text: {}", err))?;
    // Keep clipboard alive briefly so Windows commits the content.
    thread::sleep(Duration::from_millis(30));
    Ok(())
}

fn extract_clipboard_output(
    profile: &macro_engine::MacroProfile,
) -> Result<(Option<String>, Option<String>), String> {
    match profile.output_kind {
        macro_engine::MacroDataKind::None => Ok((None, None)),
        macro_engine::MacroDataKind::Text => {
            let mut clipboard =
                arboard::Clipboard::new().map_err(|err| format!("Clipboard init failed: {}", err))?;
            let text = clipboard
                .get_text()
                .map_err(|err| format!("Failed to read clipboard text: {}", err))?;
            Ok((Some(text), None))
        }
        macro_engine::MacroDataKind::Image => {
            // Clipboard owners can publish image data slightly after the copy shortcut.
            // Retry briefly before failing, then attempt text/data-url fallback.
            for _ in 0..20 {
                if let Ok(mut clipboard) = arboard::Clipboard::new() {
                    if let Ok(image) = clipboard.get_image() {
                        let rgba = image.bytes.into_owned();
                        let buffer =
                            image::RgbaImage::from_raw(image.width as u32, image.height as u32, rgba)
                                .ok_or_else(|| {
                                    "Failed to construct RGBA image from clipboard.".to_string()
                                })?;
                        let mut bytes = std::io::Cursor::new(Vec::new());
                        image::DynamicImage::ImageRgba8(buffer)
                            .write_to(&mut bytes, image::ImageFormat::Png)
                            .map_err(|err| {
                                format!("Failed to encode clipboard image as PNG: {}", err)
                            })?;
                        let encoded = BASE64_STANDARD.encode(bytes.into_inner());
                        return Ok((None, Some(encoded)));
                    }
                }
                thread::sleep(Duration::from_millis(50));
            }

            // Fallback: sometimes tools copy a data URL / raw base64 text instead of bitmap payload.
            if let Ok(mut clipboard) = arboard::Clipboard::new() {
                if let Ok(text) = clipboard.get_text() {
                    if let Some((_, b64)) = text.split_once("base64,") {
                        if BASE64_STANDARD.decode(b64.trim()).is_ok() {
                            return Ok((None, Some(b64.trim().to_string())));
                        }
                    }
                    if BASE64_STANDARD.decode(text.trim()).is_ok() {
                        return Ok((None, Some(text.trim().to_string())));
                    }
                }
            }

            Err("Failed to read clipboard image payload after retries.".to_string())
        }
    }
}

fn set_clipboard_input(profile: &macro_engine::MacroProfile, input: &InvokeBody) -> Result<(), String> {
    match profile.input_kind {
        macro_engine::MacroDataKind::None => Ok(()),
        macro_engine::MacroDataKind::Text => {
            let payload = input.text.clone().unwrap_or_default();
            if payload.is_empty() {
                log::warn!("set_clipboard_input: text payload is EMPTY — clipboard will not be useful");
            } else {
                log::info!("set_clipboard_input: setting clipboard text ({} chars): {:?}",
                    payload.len(),
                    if payload.len() > 120 { format!("{}...", &payload[..120]) } else { payload.clone() }
                );
            }

            // Use PowerShell Set-Clipboard as primary — it persists in the OS
            // clipboard independent of any Rust object lifetime.
            set_clipboard_text_powershell(&payload)?;
            thread::sleep(Duration::from_millis(40));

            if !verify_clipboard_text(&payload) {
                log::warn!("set_clipboard_input: PowerShell set failed verification, trying clip.exe fallback");
                set_clipboard_text_with_clip_exe(&payload)?;
                thread::sleep(Duration::from_millis(40));
                if !verify_clipboard_text(&payload) {
                    log::warn!("set_clipboard_input: clip.exe also failed, trying arboard");
                    set_clipboard_text_arboard_persistent(&payload)?;
                    thread::sleep(Duration::from_millis(40));
                    if !verify_clipboard_text(&payload) {
                        log::error!("set_clipboard_input: ALL clipboard methods FAILED");
                        return Err("Clipboard text did not match after all methods.".to_string());
                    }
                }
            }
            log::info!("set_clipboard_input: clipboard verified OK");
            Ok(())
        }
        macro_engine::MacroDataKind::Image => {
            if let Some(base64_data) = input.image_base64.as_ref() {
                set_clipboard_image_from_base64(base64_data)?;
                thread::sleep(Duration::from_millis(40));
                return Ok(());
            }

            if let Some(path) = input.attachment_path.as_ref() {
                let file_bytes = std::fs::read(path)
                    .map_err(|err| format!("Failed to read attachment path '{}': {}", path, err))?;
                let encoded = BASE64_STANDARD.encode(file_bytes);
                set_clipboard_image_from_base64(&encoded)?;
                thread::sleep(Duration::from_millis(40));
                return Ok(());
            }
            Err("Image input macro requires `image_base64` or `attachment_path`.".to_string())
        }
    }
}

fn set_clipboard_image_from_base64(payload: &str) -> Result<(), String> {
    let raw = if let Some((_, encoded)) = payload.split_once("base64,") {
        encoded
    } else {
        payload
    };
    let bytes = BASE64_STANDARD
        .decode(raw)
        .map_err(|err| format!("Invalid base64 image payload: {}", err))?;

    let dyn_img = image::load_from_memory(&bytes)
        .map_err(|err| format!("Failed to decode input image payload: {}", err))?;
    let rgba = dyn_img.to_rgba8();
    let (width, height) = rgba.dimensions();

    let mut clipboard = arboard::Clipboard::new().map_err(|err| format!("Clipboard init failed: {}", err))?;
    clipboard
        .set_image(arboard::ImageData {
            width: width as usize,
            height: height as usize,
            bytes: Cow::Owned(rgba.into_raw()),
        })
        .map_err(|err| format!("Failed to set clipboard image: {}", err))
}

fn verify_clipboard_text(expected: &str) -> bool {
    let normalized_expected = normalize_line_endings(expected);

    // Try arboard first (fast).
    for _ in 0..6 {
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            if let Ok(current) = clipboard.get_text() {
                if normalize_line_endings(&current) == normalized_expected {
                    return true;
                }
            }
        }
        thread::sleep(Duration::from_millis(15));
    }

    // Fallback: read via PowerShell Get-Clipboard.
    if let Ok(output) = Command::new("powershell")
        .args(["-NoProfile", "-Command", "Get-Clipboard"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .output()
    {
        if output.status.success() {
            let got = String::from_utf8_lossy(&output.stdout);
            if normalize_line_endings(got.trim()) == normalized_expected {
                return true;
            }
            log::warn!(
                "verify_clipboard: mismatch — expected {:?}, got {:?}",
                if normalized_expected.len() > 80 { format!("{}...", &normalized_expected[..80]) } else { normalized_expected.clone() },
                if got.len() > 80 { format!("{}...", &got[..80]) } else { got.to_string() },
            );
        }
    }
    false
}


fn set_clipboard_text_with_clip_exe(text: &str) -> Result<(), String> {
    let mut child = Command::new("clip.exe")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|err| format!("Failed to start clip.exe: {}", err))?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(text.as_bytes())
            .map_err(|err| format!("Failed writing to clip.exe stdin: {}", err))?;
    }

    let status = child
        .wait()
        .map_err(|err| format!("Failed waiting for clip.exe: {}", err))?;
    if status.success() {
        Ok(())
    } else {
        Err("clip.exe exited with non-zero status.".to_string())
    }
}

fn normalize_line_endings(value: &str) -> String {
    value.replace("\r\n", "\n")
}

fn switch_to_new_desktop() -> Result<(), String> {
    io_controller::send_event(&EventType::KeyPress(Key::MetaLeft))?;
    thread::sleep(Duration::from_millis(8));
    io_controller::send_event(&EventType::KeyPress(Key::ControlLeft))?;
    thread::sleep(Duration::from_millis(8));
    io_controller::send_event(&EventType::KeyPress(Key::KeyD))?;
    thread::sleep(Duration::from_millis(8));
    io_controller::send_event(&EventType::KeyRelease(Key::KeyD))?;
    thread::sleep(Duration::from_millis(8));
    io_controller::send_event(&EventType::KeyRelease(Key::ControlLeft))?;
    thread::sleep(Duration::from_millis(8));
    io_controller::send_event(&EventType::KeyRelease(Key::MetaLeft))?;
    thread::sleep(Duration::from_millis(100));

    Ok(())
}


fn macro_about_from_profile(name: &str, profile: macro_engine::MacroProfile) -> MacroAboutResponse {
    let accepts = match profile.input_kind {
        macro_engine::MacroDataKind::None => "No input body is required.".to_string(),
        macro_engine::MacroDataKind::Text => "Accepts text payload and places it on clipboard before playback.".to_string(),
        macro_engine::MacroDataKind::Image => "Accepts image payload (future extension), then places it on clipboard.".to_string(),
    };

    let returns = match profile.output_kind {
        macro_engine::MacroDataKind::None => "No response payload expected from clipboard.".to_string(),
        macro_engine::MacroDataKind::Text => "Returns text copied to clipboard after macro completion.".to_string(),
        macro_engine::MacroDataKind::Image => "Returns base64-encoded clipboard image after macro completion.".to_string(),
    };

    MacroAboutResponse {
        name: name.to_string(),
        description: profile.description,
        use_when: profile.use_when,
        method: profile.method,
        input_kind: profile.input_kind,
        output_kind: profile.output_kind,
        accepts,
        returns,
        invoke_endpoint: format!("/macros/{name}/invoke"),
    }
}

fn map_error_status(err: impl ToString) -> (StatusCode, Json<ApiMessage>) {
    let message = err.to_string();
    let status = if message.contains("Invalid state transition")
        || message.contains("not idle")
        || message.contains("already")
    {
        StatusCode::CONFLICT
    } else if message.contains("Failed to read macro file") {
        StatusCode::NOT_FOUND
    } else {
        StatusCode::INTERNAL_SERVER_ERROR
    };

    (
        status,
        Json(ApiMessage {
            success: false,
            message,
        }),
    )
}
