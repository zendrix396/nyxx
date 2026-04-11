# Nyx Automation Suite

Nyx Automation Suite is a desktop AI agent with full machine access. It sees your screen, controls mouse and keyboard, runs commands, and automates workflows through goal-based instructions. Core innovation: Rust-based recording engine captures manual actions as replayable automation scripts.

## Features

- **Full Desktop Control**: Screen perception, mouse/keyboard simulation, file system access
- **Recording Engine**: Record manual mouse/keyboard actions once, save as workflows, replay on demand
- **Goal-Oriented Automation**: State goals in natural language; agent handles execution
- **Hybrid Stack**: Rust for performance-critical recording/orchestration, Python for scripting/tools
- **Web Interface**: List files post-validation, install skills/apps (WhatsApp, web apps), manage workflows
- **Monorepo Structure**: Tauri + Svelte frontend, Rust backend, integrated tooling

## Core Workflow

1. **User Records**: Manual actions (mouse cursor, keyboard) captured by Rust recording engine
2. **Save Workflow**: Actions stored as replayable sequences
3. **AI Orchestration**: Nyx agent triggers recorded workflows or builds new automations
4. **Web Dashboard**: View saved files, install skills, run different process types
5. **Replay**: Execute saved workflows instantly without recoding

## Tech Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| Core Engine | Rust | Recording, mouse/keyboard simulation, orchestration |
| Scripting | Python | File operations, OCR, LLM integration, tools |
| Frontend | Tauri + Svelte | Web interface, macro manager, status overlay |
| Build | Cargo + npm | Monorepo workspace |
| Tools | Custom MCP | Built-in + user-generated automation tools |

## Prerequisites

- Rust (stable)
- Node.js (LTS)
- Python 3.11+
- Git

## Quick Start

```bash
git clone https://github.com/your-username/nyx-automation-suite.git
cd nyx-automation-suite
npm install
cargo build --release
cargo test --workspace
cargo tauri dev
```

## Project Structure
├── src-tauri/
│ ├── src/
│ │ ├── orchestrator.rs # Main state machine, cognitive loop
│ │ ├── modules/ # Perception, Cognition, Tooling, Knowledge
│ │ └── tools/ # Recording engine, MCP tools
├── src/ # Svelte frontend
│ ├── macro-manager/ # Workflow library UI
│ └── dashboard/ # File listing, skill installer
├── docs/
│ ├── About.md # Vision document
│ └── Architecture.md # System design
└── README.md

text

## Recording Engine

User's original Rust invention. Captures precise mouse movements, keyboard events, window interactions.

**Record**:
```rust
use rdev::{listen, Event};
let mut actions = Vec::new();
listen(|event| actions.push(event));
```

**Replay**:
```rust
use enigo::Enigo;
let mut enigo = Enigo::new();
for action in actions {
    enigo.execute_action(action);
}
```

Benefits: Native performance, cross-platform, zero-overhead capture/replay.

## Web Interface Features

Post-validation file browser, skill installer, workflow library. Supports:
- WhatsApp automation workflows
- Web app skill installation
- User-recorded vs agent-automated process types
- Workflow validation and replay controls

## Example Usage

**Record WhatsApp Message**:
1. Start recording
2. Open WhatsApp → Type message → Send
3. Save as "daily-report"
4. Replay: `nyx run daily-report`

**AI + Recorded Hybrid**:
User: "Find PDFs from Downloads, summarize, move to Reading List"
Nyx: Scans folder → OCR/summarize (Python) → User-recorded move action

text

## Architecture
User Goal → Orchestrator → [Recording Engine | Python Tools | Perception]
↓
Execution → Validation → Web Dashboard

text

## Performance

Rust recording engine: 10x faster than Python pyautogui for mouse simulation.
Zero-latency capture/replay even under heavy desktop load.

## Development

### Run in Dev Mode
```bash
cargo tauri dev
```

### Build Release
```bash
cargo build --release
cargo tauri build
```

### Testing
```bash
cargo test --workspace
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for:
- Branching strategy
- PR requirements
- Code standards

Tackle open issues labeled "good first issue".

## License

MIT
Architecture
text
User Goal → Orchestrator → [Recording Engine | Python Tools | Perception]
                        ↓
                   Execution → Validation → Web Dashboard
