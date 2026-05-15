# Nyxx

Desktop automation bridge for local AI agents. Nyxx runs as a **Tauri** app (Rust backend, Svelte UI) and exposes a **localhost-only HTTP API** so tools such as [OpenClaw](https://github.com/openclaw/openclaw) can list, record, and run macro workflows: keyboard, mouse, and clipboard hand-off for apps that have no API.

## What it is for

Many desktop and legacy applications only expose a graphical UI. Nyxx records or drives those interactions, uses the **system clipboard** (and optional image payloads) to move data between the agent and the app, and returns captured output to the caller. Execution is constrained to **predefined macros** instead of arbitrary shell access from the model.

## How it fits together

1. An agent (for example OpenClaw) receives a user goal and calls the **nyxx-macro-control** skill, which talks to Nyxx over `http://127.0.0.1:4777`.
2. Nyxx brings the right window forward, runs the saved macro (clicks, keys, navigation).
3. Text or image data is read back via clipboard or API response fields (`output_text`, `output_image_base64` on POST invocations).
4. The agent summarizes or forwards the result to the user.

## Design
```mermaid
flowchart TB
    %% ==========================================
    %% TOP ROW: Interfaces & Clients
    %% ==========================================
    subgraph Top ["Interfaces & External Clients"]
        direction LR
        Agent["External Agent<br/>(OpenClaw / MCP Client)"]
        UI["Svelte Frontend<br/>(Tauri Webview UI)"]
        SysUI["System Tray &<br/>Global Hotkey (F4)"]
    end

    %% ==========================================
    %% MIDDLE ROW: Rust Core (Two Pillars)
    %% ==========================================
    subgraph Middle ["Nyxx Rust Core (Tauri Backend)"]
        direction LR
        
        %% Pillar 1: API, Events, and State
        subgraph Pillar1 ["API & State Management"]
            direction TB
            Axum["Axum Local API<br/>(Port 4777, JSON/B64)"]
            Runtime["Macro Runtime<br/>(Atomic Cancel Signals)"]
            Orch{"Orchestrator<br/>(States: IDLE, REC, EXEC)"}
            Listener["Event Listener<br/>(rdev global hooks)"]
        end

        %% Pillar 2: The Cognitive/Execution Loop
        subgraph Pillar2 ["Cognitive & Execution Pipeline"]
            direction TB
            Cog["Cognition & Perception<br/>(Intent, Gemini API)"]
            Engine["Macro Engine<br/>(JSON Deserialization)"]
            IO["I/O Controller<br/>(Mutex-locked rdev)"]
            ClipHandler["Clipboard Manager<br/>(Arboard, PowerShell, clip.exe)"]
        end
        
        %% Internal Pillar Connections
        Axum -->|"REST /invoke"| Runtime
        Runtime -->|"Thread Mgmt"| Orch
        Listener -.->|"Buffers Inputs"| Orch
        
        Orch ==>|"1. Plans Task"| Cog
        Orch ==>|"2. Routes Macro"| Engine
        Engine ==>|"3. Timed Playback"| IO
        Axum -->|"4. Injects Data"| ClipHandler
    end

    %% ==========================================
    %% BOTTOM ROW: Environment & OS Layer
    %% ==========================================
    subgraph Bottom ["Environment, OS & Cloud"]
        direction LR
        Cloud["Google Gemini API<br/>(LLM Inference)"]
        FS[(Local File System<br/>macro-profiles/*.json)]
        OSHost["Host Operating System<br/>(Input & Display)"]
        Secrets["System Keyring<br/>(API Keys)"]
    end

    %% ==========================================
    %% CROSS-ROW CONNECTIONS
    %% ==========================================
    
    %% Top to Middle
    Agent -->|"HTTP POST/GET"| Axum
    UI <-->|"Tauri IPC (invoke/listen)"| Orch
    UI -->|"Configures"| Engine
    SysUI -->|"Toggle Window"| Orch

    %% Middle to Bottom
    Cog <-->|"Prompts/Responses"| Cloud
    Cog -->|"Reads Key"| Secrets
    Engine <-->|"Read/Write"| FS
    
    %% OS Level Connections
    IO -.->|"Simulates Clicks/Keys"| OSHost
    ClipHandler <-->|"Reads/Writes Data"| OSHost
    Listener -.->|"Captures Activity"| OSHost

    %% ==========================================
    %% THEMING
    %% ==========================================
    classDef ext fill:#1e293b,stroke:#cbd5e1,stroke-width:2px,color:#f8fafc;
    classDef core fill:#0ea5e9,stroke:#0284c7,stroke-width:2px,color:#ffffff;
    classDef util fill:#3b82f6,stroke:#2563eb,stroke-width:1px,color:#ffffff;
    classDef db fill:#10b981,stroke:#047857,stroke-width:2px,color:#ffffff;
    classDef ui fill:#8b5cf6,stroke:#6d28d9,stroke-width:2px,color:#ffffff;
    classDef state fill:#eab308,stroke:#a16207,stroke-width:3px,color:#ffffff;

    class Agent,Cloud,OSHost ext;
    class Engine,IO,Cog core;
    class Axum,Runtime,Listener,ClipHandler util;
    class Orch state;
    class FS,Secrets db;
    class UI,SysUI ui;
```

## Repository layout

| Path | Role |
| --- | --- |
| `src/` | Svelte + Vite frontend (macro UI, dev server on port 5173 in dev). |
| `src-tauri/` | Rust crate: orchestrator, Axum API, input simulation, Tauri shell. |
| `.openclaw/` | OpenClaw-oriented skill definition (`SKILL.md`, name `nyxx-macro-control`). |
| `nyxx-web/` | Optional Next.js showcase site (separate from the desktop app). |
| `docs/` | Architecture, roadmap, and product notes. |

## Prerequisites

- **Rust** (stable; see `src-tauri/Cargo.toml` for `rust-version`).
- **Node.js** 18+ and **npm** (the Tauri config runs `npm run dev` / `npm run build` in `src/`). If you use **pnpm**, install dependencies with `pnpm install` in `src/` and adjust or mirror the `beforeDevCommand` / `beforeBuildCommand` in `src-tauri/tauri.conf.json` if needed.
- **OpenClaw** (or another MCP-capable client) if you want chat-driven control; the app still runs standalone for local API and GUI use.

## Build and run (desktop app)

```bash
git clone https://github.com/zendrix396/nyxx.git
cd nyxx
cd src
npm install
cd ../src-tauri
cargo tauri dev
```

For a release build:

```bash
cd src && npm run build && cd ../src-tauri && cargo tauri build
```

The local API listens on **`127.0.0.1:4777`** by default (see `api_server.rs` if you change the port).

## HTTP API (summary)

Agents should treat **`GET /help`** as the source of truth for macro metadata (HTTP method, input/output kinds). Common routes:

| Method | Path | Purpose |
| --- | --- | --- |
| GET | `/help` | Macro metadata and invocation rules. |
| GET | `/macros` | List macros. |
| GET | `/macros/:name/invoke` | Run a **GET**-configured macro (no output body fields). |
| POST | `/macros/:name/invoke` | Run a macro; JSON body may include `text`, `image_base64`, etc. |
| POST | `/macros/stop`, `/macros/stop-all` | Stop running macro(s). |
| POST | `/recording/start`, `/recording/stop` | Recording lifecycle. |
| POST | `/mouse/move`, `POST /mouse/drag` | Cursor control. |
| DELETE | `/macros/:name` | Remove a macro profile. |

Exact field names and behavior are documented in `.openclaw/SKILL.md` and implemented in `src-tauri/src/api_server.rs`.

## Connecting OpenClaw

Install the skill from this repo: use **`.openclaw/SKILL.md`** (frontmatter name: `nyxx-macro-control`) and place it where your OpenClaw install expects skills (for example under an `agents` / `skills` tree). Start Nyxx first so `http://127.0.0.1:4777` is reachable.

## Optional: showcase site

The `nyxx-web/` directory is a small Next.js demo. See `nyxx-web/README.md` for `npm install` / `npm run dev` (default port 3000).

## Security notes

- Traffic is intended to stay on **loopback**; keep the server bound to `127.0.0.1` in untrusted networks.
- Macros are explicit, user-defined automation paths. Review recordings before exposing them to an agent.
- Nyxx does not need to send your screen to a cloud service for the core macro API; still treat clipboard contents and macro outputs as sensitive.

## More documentation

See **`docs/`** for architecture deep-dives, flow, and roadmap.
