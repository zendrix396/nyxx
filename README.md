# Nyx Agent

An AI agent that lives on your machine and gets things done. Less instructing, more achieving.

## What's This All About?

Standard voice assistants are sandboxed. They can't interact with your desktop, manage your files, or use your local tools. They're glorified search engines with a voice.

Nyx is different. It's an agent that has full access to your machine's environment (just like you do). It sees your screen, controls your mouse and keyboard, and runs commands. You don't tell it *how* to do something; you just state your goal.

> **User:** "Hey Nyx, find all the PDFs I downloaded this week, summarize each one, and move them into my 'Reading List' folder."
>
> **Nyx:** *Scans the default Downloads folder using a high-speed file search tool. Filters the results by creation date. For each PDF, it uses an OCR tool to extract the text, sends it to an LLM for summarization, and saves the summary. Finally, it uses a file system tool to create the 'Reading List' folder and move the original PDFs into it.*

Want the full breakdown of the vision, the core problems it solves, and how it works?
**Check out the vision doc: [`docs/About.md`](docs/About.md)**

---

## Getting Started

You'll need the latest stable versions of Rust, Node.js, and Python (for scripting tools).

1.  **Clone the repo:**
    ```bash
    git clone https://github.com/your-username/nyx-agent.git
    cd nyx-agent
    ```

2.  **Install frontend dependencies:**
    ```bash
    npm install
    ```

3.  **Build the workspace:**
    This compiles the core Rust backend.
    ```bash
    cargo build --release
    ```

4.  **Run the tests:**
    Make sure all core components are functioning correctly.
    ```bash
    cargo test --workspace
    ```

5.  **Run Nyx Agent in development mode:**
    This starts the Tauri app with hot-reloading for the frontend.
    ```bash
    cargo tauri dev
    ```

---

## Repo Layout

The project is a monorepo containing the core application and its documentation.

-   `src-tauri/`: The heart of the agent. A Rust application containing:
    -   `src/orchestrator`: Manages the main state machine and the cognitive loop.
    -   `src/modules`: Contains the Perception, Cognition, Tooling, and Knowledge modules.
    -   `src/tools`: The library of built-in and user-generated MCP tools.
-   `src/`: The face. A Svelte frontend for the settings UI, macro manager, and status overlay.
-   `docs/`: All project documentation.

For a deeper dive into the "why" behind this structure and how all the pieces interact, see [`docs/Architecture.md`](docs/Architecture.md).

---

## Want to Contribute?

Awesome. We'd love the help. The best way to start is by tackling an open issue.

All the specific details on our branching strategy, how to format your PRs, and our code of conduct are in the contribution guide.

**Please read it here: [`CONTRIBUTING.md`](CONTRIBUTING.md)**