# About Nyx Agent

## The Problem: Assistants are Dumb Tools

For years, we've been promised the "Jarvis" experience—an AI that acts as a true assistant. Instead, we got cloud-based toys. Alexa, Siri, Google Assistant... they all live in a box. They can't see your screen, they can't move your mouse, and they certainly can't install a piece of software for you.

They force you to be the computer's translator. You figure out the steps, and they *might* be able to perform one of them, like "set a timer." This isn't an assistant; it's a voice-activated remote control with a very limited set of buttons.

## The Vision: A True Agent

Nyx is not a remote control. It's an **agent**. An agent doesn't need you to tell it *how* to do something. It just needs to know *what* you want to achieve.

-   **Goal-Oriented:** You give Nyx a high-level goal, like "Analyze this CSV of user data and show me a chart of signups by country." Nyx figures out the rest. It writes a Python script, executes it, and presents you with the result.
-   **System-Native:** Nyx lives on your machine. It has the same capabilities you do. It can find files, click buttons, type text, and run programs. It's not limited by a sandboxed API.
-   **Self-Improving:** Nyx learns. Every time it successfully completes a task, it gets better. Every time it generates a new script to solve a problem, it adds that script to its permanent toolbox, making it faster and more capable in the future.

## How is this Different? Three Core Concepts

1.  **AI-Driven Automation, Not Just Recorded Clicks:** Nyx has a "macro" feature, but it's not the brittle recorder you're used to. Its AI-Assisted mode understands *what* you're clicking on, not just *where*. If a button moves, Nyx can still find it. But it goes beyond that—most tasks don't require recording at all. You just ask, and it performs the task autonomously.

2.  **An Ever-Expanding Toolbox (MCP):** Nyx uses a "Model Context Protocol" (MCP). Think of it as a universal language for tools. Out of the box, Nyx has tools for file I/O, web browsing, etc. But when you ask it to do something new, like "pull my latest sales data from Stripe," it will *write the code for a new Stripe tool on the fly*, then add that tool to its library forever. Your agent's capabilities grow with you.

3.  **Visual Understanding:** Nyx sees your screen. Using a combination of OCR and computer vision models, it can read text, identify icons, and understand the layout of applications. This allows it to interact with any graphical application, even those without a proper API.

Nyx is the next step. It's an agent that works with you, on your terms, right on your machine.