# Project Context for Qwen Code

## Project Overview

This is a Rust project that implements a desktop Markdown editor application. It uses the `egui` crate for its graphical user interface and `egui_commonmark` for rendering Markdown. The application features a dual-pane layout with a text editor on the left and a live Markdown preview on the right.

### Key Features

*   **Dual-Pane UI**: Split view with an editor panel and a preview panel.
*   **File Handling**: Load and save Markdown (.md) and Text (.txt) files using system dialogs (`rfd` crate).
*   **Live Preview**: Real-time rendering of Markdown content as it's typed.
*   **Chinese Font Support**: Attempts to load system Chinese fonts (e.g., Microsoft YaHei, SimSun) for better display of Chinese characters.
*   **Synchronized Scrolling**: An option to link the scrolling of the editor and preview panels.

### Technologies

*   **Language**: Rust
*   **Framework**: `egui` (for UI) via `eframe` (for native app integration)
*   **Markdown Rendering**: `egui_commonmark`
*   **File Dialogs**: `rfd` (Rust File Dialog)
*   **Build Tool**: Cargo

## Building and Running

The project uses standard Cargo commands for building and running.

### Prerequisites

*   Rust and Cargo installed.

### Commands

*   **Build**: `cargo build`
*   **Run (Debug)**: `cargo run`
*   **Run (Release)**: `cargo run --release`
*   **Test**: `cargo test`

The release profile in `Cargo.toml` is configured for optimization with LTO (Link Time Optimization) and other settings for a smaller, faster binary.

## Development Conventions

Based on the existing code:

*   **UI Structure**: The application uses `eframe::App` trait. The main UI logic is in the `update` method.
*   **Layout**: `egui`'s panel system (`TopBottomPanel`, `CentralPanel`) and layout helpers (`ui.horizontal`, `ui.columns`) are used for structuring the interface.
*   **State Management**: Application state (like the Markdown text, scroll positions, and UI flags like `scroll_linked`) is stored in the main `MyApp` struct.
*   **File I/O**: Uses standard `std::fs` for reading and writing files, wrapped with `rfd` for native file dialogs.
*   **Comments**: Code includes descriptive comments, often in Chinese, explaining functionality.
