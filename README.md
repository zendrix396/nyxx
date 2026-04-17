# Nyxx
**MCP for Desktop Automation through Macro Recordings**

Nyxx connects conversational AI to local desktop execution. By integrating with **[OpenClaw](https://github.com/openclaw/openclaw)** via the Model Context Protocol (MCP), Nyxx allows you to trigger local OS macros using natural language chat commands (WhatsApp, Telegram). 

Many legacy enterprise systems, walled gardens, and native apps lack REST APIs. Nyxx solves this by automating the physical UI layer and using the system clipboard as middleware to pass data between the LLM and the application. You can record a workflow once, save the macro profile, and share it so others can run complex automations without manual setup.

## Core Use Case

You text: *"Get this week's metrics from the accounting app and summarize them."*

1. **Intent:** OpenClaw parses the request and calls the `nyxx-macro-control` skill via the local API (`127.0.0.1:4777`).
2. **Execution:** Nyxx runs a pre-recorded macro that brings the offline accounting app to the foreground, navigates to the report, and triggers "Copy to Clipboard".
3. **Extraction:** Nyxx intercepts the raw data from the OS clipboard and returns it to OpenClaw.
4. **Delivery:** OpenClaw summarizes the dense financial data and replies directly in your chat.

## Key Features

* **Local REST API:** Exposes your recorded macros to local AI agents securely via `127.0.0.1:4777`.
* **Clipboard Middleware:** Pass dynamic text and images between the LLM and local apps without needing official integrations.
* **Record and Share:** Record mouse movements and keystrokes via the Svelte GUI. Download and run community-created profiles instantly.
* **Background Jobs:** Leverage OpenClaw's heartbeat system to run scheduled tasks like clearing inboxes or scraping data.
* **Autonomous Vision (WIP):** Evolving from fixed X/Y coordinates to local computer vision models to find and interact with UI elements dynamically.

## Architecture

* **Orchestrator:** Rust (Tauri) for state management and low-level I/O (`rdev`, `enigo`).
* **Local Server:** Axum for handling OpenClaw API requests.
* **Frontend:** Svelte for the macro management GUI.
* **LLM Integration:** OpenClaw for converting human intent into JSON-RPC/MCP tool calls.

## API Endpoints

The `nyxx-macro-control` skill allows OpenClaw to interface with these core endpoints:
* `GET /macros` - List available macros and their I/O requirements.
* `POST /macros/:name/invoke` - Inject payload into clipboard and run the macro.
* `POST /mouse/move` & `/mouse/drag` - Autonomous cursor control.
* `POST /macros/stop` - Global kill switch.

## Getting Started

### Prerequisites
* Rust (latest stable)
* Node.js (v18+)
*[OpenClaw](https://openclaw.ai) running locally

### Installation

1. **Clone the repository:**
   ```bash
   git clone https://github.com/your-username/nyxx.git
   cd nyxx
   ```

2. **Install frontend dependencies:**
   ```bash
   cd src
   npm install
   ```

3. **Start the application:**
   ```bash
   cargo tauri dev
   ```

4. **Connect to OpenClaw:**
   Copy the `nyxx-macro-control` folder from this repository into your local OpenClaw `.agents/skills/` directory. OpenClaw will automatically discover the Nyxx API.

## Security

Nyxx operates entirely locally. No screen data or recordings are sent to the cloud. The port `4777` is strictly bound to `127.0.0.1` to prevent unauthorized network access. By converting LLM intents into predefined macro workflows, Nyxx acts as a sandbox that prevents the AI from executing arbitrary or destructive shell commands.
