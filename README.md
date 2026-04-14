# Climart

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](#)
[![Ratatui](https://img.shields.io/badge/Ratatui-Terminal%20UI-blue?style=for-the-badge)](#)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](#)
[![Platform Supports](https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey?style=for-the-badge)](#)

Climart is a unified, cross-platform terminal user interface (TUI) acting as an app store for command-line interface (CLI) tools. Instead of memorizing disparate search, install, and run commands across various package ecosystems, Climart abstracts the underlying registry operations into a single, cohesive interface.

By aggregating package metadata concurrently from major package managers, Climart allows developers to discover, install, and seamlessly execute CLI utilities directly from their terminal without manual context switching.

## Key Features

* **Unified Abstracted Interface:** Search for a tool once. Climart queries multiple package registries concurrently and presents deduplicated results natively within the TUI.
* **Interactive Terminal Drop-Out:** When executing an installed tool, Climart intelligently drops out of its alternate screen buffer, allowing the executed CLI tool to inherit the real terminal's standard input/output. Upon completion, the TUI is instantaneously restored.
* **Intelligent Resolution Engine:** Climart automatically intercepts execution intents and applies OS-specific fallbacks (e.g., executing `.cmd` shims on Windows environments) ensuring commands run reliably regardless of the host operating system.
* **Highly Responsive UI:** Built on top of `ratatui` and `crossterm`, featuring asynchronous state management and real-time query updates over Tokio channels.
* **Extensible Architecture:** Designed with a modular provider system, making it trivial to integrate additional package registries.

## Supported Providers

Climart currently aggregates packages from the following package managers:

* **Node Package Manager (npm):** Direct querying against the npms.io registry.
* **Pipx (Python):** Sourced directly against the PyPI JSON API, filtered for executable utility environments.
* **Homebrew (brew):** Searches against the Homebrew Formulae indices (macOS/Linux).
* **Pkgx:** Fetches metadata directly from the decentralized pkgx pantry.

## Installation

Ensure that you have the Rust toolchain installed.

```bash
git clone https://github.com/your-username/climart.git
cd climart
cargo build --release
```

The compiled binary will be located at `target/release/climart`. You can move this to a directory within your system's PATH.

## Usage Guide

Initiate the application by running the executable:

```bash
climart
```

### Keybindings

The interface operates entirely via keyboard commands.

| Keybinding | Action |
| :--- | :--- |
| `/` | Enter Search Mode. |
| `Enter` | Submit query from Search Mode. |
| `Esc` | Leave Search Mode / Cancel input. |
| `Up`, `k` | Move cursor up in the result list. |
| `Down`, `j` | Move cursor down in the result list. |
| `u` | Scroll the details panel text up. |
| `d` | Scroll the details panel text down. |
| `i` | Install the currently selected tool. |
| `r`, `Enter` | Run the currently selected tool interactively. |
| `Ctrl + C` | Force exit the application. |
| `q`, `Q` | Quit the application gracefully (Normal Mode). |

## Configuration

Climart generates a local configuration directory automatically upon initial execution, establishing a default configuration file at `~/.climart/config.toml` (or the equivalent local application data path on your respective operating system).

You may customize this file to toggle specific providers or alter presentation logic:

```toml
[providers]
enable_npm = true
enable_pipx = true
enable_brew = true
enable_pkgx = true

[ui]
show_descriptions = true
nerd_fonts = false
```

## Internal Architecture

The application is structured into discrete layers, strictly separating the execution context from the rendering engine:

* **`app.rs` / `main.rs`:** Handles the synchronous event loop, terminal state transitions, and background task asynchronous message passing via Tokio `mpsc` channels.
* **`ui/`:** Contains the Ratatui layout logic, drawing strictly based on the explicit state provided by the `App` component.
* **`core/`:** Contains the execution logic. Notably, `core::search` executes futures concurrently via `tokio::join!`, while `core::run` manages the `std::process::Command` environment inheritance for the TUI drop-out mechanism.
* **`providers/`:** Implementations of registry-specific API requests and data normalization models.

## Development

Climart utilizes standard Cargo tools for compilation and verification.

```bash
# Run application in debug mode
cargo run

# Execute unit and integration tests
cargo test
```

## Contributing

Contributions regarding architectural improvements or new package provider modules are highly encouraged. Please strictly adhere to standard Rust formatting (`cargo fmt`) and assure all tests pass before submitting a Pull Request.

## License

This project is licensed under the MIT License. See the LICENSE file for details.
