# NeuroNano

> **Minimalist Terminal Editor + Native AI Intelligence.**
>
> *Inspired by GNU Nano. Powered by Rust & Gemini.*

**NeuroNano** is a modern, lightweight text editor built for the terminal. It retains the approachable, muscle-memory-friendly interface of `nano`, but introduces a native AI layer that acts as an intelligent pair programmer, copy-editor, and script generator—all without leaving the TTY context.

## Vision

The goal is simple: **Bring the power of LLMs to the bare-metal terminal.**
Whether you are debugging inside a Docker container, editing configs via SSH, or just want a distraction-free writing environment, NeuroNano provides context-aware AI assistance with a simple keystroke.

## Features (MVP & Roadmap)

- **Blazing Fast:** Built in **Rust** (2021 edition).
- **Nano-like UI:** Familiar layout (Header, Body, Shortcut Footer). No learning curve.
- **AI "Magic" (Ctrl+K):** Context-aware text generation and modification via **Google Gemini**.
- **Async Core:** Non-blocking UI using `tokio`. The interface never freezes while the AI "thinks".
- **Universal:** Works everywhere `crossterm` does (Linux, macOS, Windows, SSH).

## Tech Stack & Architecture

We prioritize modularity and type safety.

* **Language:** [Rust](https://www.rust-lang.org/)
* **TUI Rendering:** [ratatui](https://github.com/ratatui-org/ratatui) (The successor to tui-rs).
* **Event Handling:** [crossterm](https://github.com/crossterm-rs/crossterm) (Raw terminal manipulation).
* **Async Runtime:** [tokio](https://tokio.rs/) (For handling HTTP requests to Gemini concurrently).
* **Editor Logic:** [tui-textarea](https://github.com/rhysd/tui-textarea) (Robust buffer management).
* **AI Client:** [reqwest](https://github.com/seanmonstar/reqwest) (HTTP Client).

### Project Structure

```text
src/
├── main.rs      // Entry point, Event Loop, Terminal setup/teardown.
├── app.rs       // Application State (Model) & Business Logic.
├── ui.rs        // Rendering logic & Widget composition (View).
└── ai.rs        // (Upcoming) Gemini API Client & Prompt Engineering.
