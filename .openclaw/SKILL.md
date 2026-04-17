---
name: nyxx-macro-control
description: Control and execute automation macros via a local Rust-based orchestrator API (http://127.0.0.1:4777). Use when Aditya (the user) asks to run, list, or record macros.
---

# Nyxx Macro Control

This skill interfaces with the Rust-based macro orchestrator.

## API Specification

- **Base URL**: `http://127.0.0.1:4777`

### Endpoints Reference

| Method | Path | Description |
|--------|------|-------------|
| GET | `/help` | **Crucial**: Fetch this to see macro metadata (method, input/output kinds). |
| GET | `/macros` | List basic macro profiles. |
| POST | `/macros/:name/invoke` | Invoke a macro. Returns `output_text` / `output_image_base64`. |
| GET | `/macros/:name/invoke` | Invoke a GET-only macro. Returns NO output fields (null). |
| POST | `/macros/stop` | Stop any currently running macro (Global Stop). |
| POST | `/macros/stop-all` | Alias for global stop. |
| POST | `/macros/:name/stop` | Stop a specific macro only. |
| POST | `/mouse/move` | Move mouse to absolute `x`, `y`. |
| POST | `/mouse/drag` | Drag mouse by `delta_x`, `delta_y`. |
| POST | `/recording/start` | Start recording. |
| POST | `/recording/stop` | Stop recording and save with profile options. |
| DELETE | `/macros/:name` | Delete a macro. |

## Critical Invocation Rules

1. **Check Metadata First**: Always use `GET /help` to determine a macro's required `method` (GET or POST) and `input_kind`.
2. **Method Enforcement**:
   - If a macro's profile says `POST`, you **must** use `POST .../invoke`.
   - If a macro's profile says `GET`, use `GET .../invoke`.
3. **Handling Inputs (POST)**:
   - **Text Input**: Send JSON `{"text": "your command/text"}`.
   - **Image Input**: Send JSON `{"attachment_path": "path/to/img"}` or `{"image_base64": "..."}`.
4. **Handling Outputs**:
   - Only `POST .../invoke` populates `output_text` and `output_image_base64`.
   - `GET .../invoke` returns `null` for these fields.
5. **PowerShell Escaping**: When using `curl.exe` in PowerShell, use a temporary JSON file for the body to avoid quoting hell:
   ```powershell
   @'
   {"text":"cmd here"}
   '@ | Set-Content -Path body.json -Encoding utf8
   curl.exe -s -X POST "http://127.0.0.1:4777/macros/name/invoke" -H "Content-Type: application/json" -d "@body.json"
   ```

6. **Clipboard Data Retrieval**: When a macro (like `image-gen` or `img-gen`) returns `output_image_base64`, treat this as a high-priority asset.
   - If `output_image_base64` is not null, convert/save it and present it to the user.
   - If `output_text` is present, display it formatted as markdown.

7. **Interruption Handling**: If the user says "stop", "cancel", or "kill it" while a macro is running, immediately call `POST /macros/stop`.

8. **Mouse Control**:
   - To move: `POST /mouse/move` with `{"x": float, "y": float}`.
   - To drag: `POST /mouse/drag` with `{"delta_x": float, "delta_y": float}`.

## Macro-Specific Notes

- **`image-gen` / `img-gen`**: These often put a generated image on the clipboard. Always look for `output_image_base64` in the POST response and deliver it to Aditya immediately.
- **`paste`**: Grab the request, perform `Ctrl+V`, and returns a status image. Use this for interactive typing or terminal commands where direct invocation is less reliable.
- **`terminal`**: One-liner Windows commands. Use `paste` if it needs more complex interaction.

## Reliability & Scheduling

1. **Graceful Retry**: If a macro fails, attempt one retry with the `paste` macro if applicable, or spawn a fresh instance.
2. **Schedules**: For periodic tasks (e.g., "every 5 hours"), update `HEARTBEAT.md` with the task and track the last run time in `memory/heartbeat-state.json`.

2. **Contextual Response**: If a macro returns an `output_image_base64`, present it to the user. If it returns `output_text`, summarize or display it.
3. **Safety**: Use `/macros/stop` if a macro appears stuck or is taking too long.

