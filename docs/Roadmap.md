# Nyx Agent Roadmap

This document outlines the planned development milestones for Nyx Agent, from foundational setup to a self-improving AI.

---

### **Milestone 0: Foundation & Core Setup**
*Goal: Establish the project structure and get a basic, empty window running.*

-   **Sub-milestone 0.1: Project Initialization**
    -   [ ] Initialize a new Rust project with Cargo.
    -   [ ] Integrate Tauri to create the hybrid Rust/Webview application structure.
    -   [ ] Initialize a Svelte project within the `src` directory.
    -   **Tools:** `cargo`, `npm`, `tauri-cli`.

-   **Sub-milestone 0.2: Basic UI & Hotkey**
    -   [ ] Implement a global hotkey listener in the Rust backend to show/hide the main window.
    -   [ ] Create a basic Svelte UI with a text input box.
    -   [ ] Establish the basic Tauri command bridge between the Svelte frontend and the Rust backend.
    -   **Tools:** `tauri`, `svelte`, Rust crates: `global-hotkey`.

-   **Sub-milestone 0.3: System Tray Integration**
    -   [ ] Add system tray icon with context menu (Show/Hide Agent, Settings, Quit).
    -   [ ] Implement tray menu event handling.
    -   [ ] Ensure proper icon bundling in the application build.
    -   **Tools:** `tauri` tray-icon feature.

---

### **Milestone 1: The Hands - Low-Level Control & Manual Macros**
*Goal: Give Nyx the ability to control the mouse/keyboard and record/play basic user actions.*

-   **Sub-milestone 1.1: System I/O Control**
    -   [ ] Integrate a Rust crate for simulating keyboard and mouse input.
    -   [ ] Create a Tauri command `execute_keypress(key)` that can be called from the frontend.
    -   **Tools:** Rust crates: `enigo` or `rdev`.

-   **Sub-milestone 1.2: Manual Macro Recording Engine**
    -   [ ] Integrate a Rust crate for listening to global mouse/keyboard events without blocking.
    -   [ ] Implement the `RECORDING` state in the Orchestrator.
    -   [ ] When recording, log all events (key presses, mouse clicks with coordinates) to a temporary data structure.
    -   [ ] Implement "Save Macro" functionality, writing the event list to a JSON file in the Knowledge Base.
    -   **Tools:** Rust crates: `rdev`.

-   **Sub-milestone 1.3: Manual Macro Playback Engine**
    -   [ ] Implement the playback logic that reads a macro JSON file and executes the events in sequence using the System I/O Controller.
    -   [ ] Create a basic UI in Svelte to list and run saved macros.

---

### **Milestone 2: The Brain - Initial Cognition**
*Goal: Connect Nyx to the Gemini LLM and execute a simple, hard-coded task.*

-   **Sub-milestone 2.1: LLM API Integration**
    -   [ ] Set up an API client in Rust to communicate with the Google Gemini API.
    -   [ ] Store API key securely using a crate like `keyring`.
    -   [ ] Create a function that can take a text prompt and return the LLM's response.
    -   **Tools:** Rust crates: `reqwest` (for HTTP), `serde` (for JSON), `tokio` (for async).

-   **Sub-milestone 2.2: Simple Command Execution**
    -   [ ] Implement a basic `Command Runner` in the Tooling module.
    -   [ ] Create a flow where a user can type "open notepad", the prompt is sent to the LLM (with instructions to return a shell command), and the Command Runner executes the result (e.g., `notepad.exe` or `open -a TextEdit`). This validates the end-to-end cognitive loop in its simplest form.

---

### **Milestone 3: The Eyes - Perception Engine**
*Goal: Enable Nyx to see and read the screen.*

-   **Sub-milestone 3.1: Screen Capture & OCR**
    -   [ ] Integrate a high-performance screen capture crate.
    -   [ ] Set up a Rust wrapper around a Tesseract or other local OCR engine to extract text from a captured image.
    -   [ ] Create a Tauri command `read_screen` that returns all visible text as a string.
    -   **Tools:** Rust crates: `scrap`, `lep-tess` or custom FFI bindings.

-   **Sub-milestone 3.2: Voice Input**
    -   [ ] Integrate a local Speech-to-Text engine. `whisper.cpp` is a strong candidate.
    -   [ ] Pipe microphone input to the engine and update the UI's input box with the transcribed text in real-time.
    -   **Tools:** `whisper.cpp`, FFI bindings, audio input library like `cpal`.

---

### **Milestone 4: The Intelligent Agent - The Full Loop**
*Goal: Combine all modules to enable complex, AI-driven tasks and smart macros.*

-   **Sub-milestone 4.1: The Tooling Protocol (RPC & MCP)**
    -   [ ] Design a robust JSON-RPC layer for communication between the agent core and its tools.
    -   [ ] Design the standard JSON structure for MCP tool definitions and calls, to be sent over the RPC layer.
    -   [ ] Implement the first built-in tools (`file.search`, `file.read`, `file.write`) as Rust functions exposed via the MCP protocol.
    -   [ ] Refactor the Cognition Module to generate *plans* that are sequences of MCP tool calls.

-   **Sub-milestone 4.2: Tool Genesis Engine**
    -   [ ] Implement the logic where, if a task requires a tool that doesn't exist, the Cognition module writes a Python script for it.
    -   [ ] The Tooling Module will then save this script and wrap it in a lightweight Python MCP server (e.g., using FastAPI) that it can launch as a subprocess.
    -   [ ] The new tool is registered in the Knowledge Base's Tool Library.

-   **Sub-milestone 4.3: AI-Assisted Macro Engine**
    -   [ ] Augment the macro recorder: on click, trigger the Perception module to analyze the clicked-on UI element and save the contextual data (screenshot, OCR text, element type guess) instead of just coordinates.
    -   [ ] Implement the self-correcting playback logic as described in the Architecture doc.

---

### **Milestone 5: Headless & CLI Operation**
*Goal: Enable scripting, remote access, and headless operation through a powerful CLI and tunneling service.*

-   **Sub-milestone 5.1: CLI Framework**
    -   [ ] Implement a CLI framework using a crate like `clap` for parsing commands and arguments.
    -   [ ] Define initial commands: `nyx status`, `nyx version`, `nyx tunnel`.

-   **Sub-milestone 5.2: Singleton Service**
    -   [ ] Implement singleton logic to ensure only one instance of the Nyx agent core service is running.
    -   [ ] The CLI will communicate with the singleton service via an async pipe (Unix socket/named pipe) using JSON-RPC.

-   **Sub-milestone 5.3: Tunneling Service**
    -   [ ] Integrate a development tunnel service to expose the agent's RPC endpoint securely.
    -   [ ] Implement `nyx tunnel create` and `nyx tunnel list` commands.
    -   [ ] Enable local port forwarding through the tunnel.

---

### **Milestone 6: Optimization & Refinement**
*Goal: Make the agent faster, more accurate, and more user-friendly.*

-   **Sub-milestone 6.1: Perception Optimization**
    -   [ ] Implement the Differential Frame Analyzer to reduce redundant screen analysis.
    -   [ ] Train and integrate a lightweight YOLO model to detect UI elements, providing the LLM with a structured `UI Map` instead of just raw text. This is a major accuracy improvement.
    -   **Tools:** `PyTorch` or `ONNX` runtime for Rust.

-   **Sub-milestone 6.2: UI/UX Polish**
    -   [ ] Build the Svelte UI for managing the Tool Library (viewing, deleting, or re-generating tools).
    -   [ ] Implement the minimalist overlay UI that shows the current action during execution.
    -   [ ] Create a settings panel for configuring the hotkey, API keys, etc.

-   **Sub-milestone 6.3: Self-Update Mechanism**
    -   [ ] Implement a self-update mechanism to allow the agent to fetch and apply new versions of itself.
    -   [ ] Add a `nyx update` command to the CLI.

---

### **Milestone 7: The Learning System - Self-Improvement (Long-Term)**
*Goal: Enable Nyx to learn from its interactions and improve over time.*

-   **Sub-milestone 7.1: Feedback & Data Collection**
    -   [ ] Implement the logging of task trajectories to the Knowledge Base.
    -   [ ] Add a simple "thumbs up/down" feedback mechanism in the UI after a task completes.
    -   [ ] Implement implicit feedback detection (e.g., detecting if a user manually reverses an action Nyx just took).

-   **Sub-milestone 7.2: Reinforcement Learning Pipeline**
    -   [ ] (Research) Develop a process for converting the collected trajectory data into a format suitable for fine-tuning.
    -   [ ] (Future) Set up a background service that periodically uses this data to fine-tune a base language model (likely requiring a dedicated service, not done on-device), improving its ability to generate successful plans.
