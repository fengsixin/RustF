# GEMINI.md

## Project Overview

This project, named "RustF", is a graphical desktop Markdown editor developed in Rust. It utilizes the `egui` and `eframe` libraries for its user interface.

The core functionality includes:
- A side-by-side view with a text editor for raw Markdown on the left and a real-time rendered preview on the right.
- The ability to open and save Markdown (`.md`) and text (`.txt`) files using native system dialogs.
- A "synchronous scrolling" feature that keeps the editor and preview panes in sync.
- Basic text formatting via keyboard shortcuts (e.g., Ctrl+B for bold).
- Support for Chinese character rendering on Windows by attempting to load common system fonts.

The application is configured for optimized release builds, suggesting a focus on performance.

## Building and Running

This is a standard Rust project managed by Cargo.

- **Build the project:**
  ```sh
  cargo build
  ```

- **Run the application:**
  ```sh
  cargo run
  ```

- **Run tests:**
  ```sh
  cargo test
  ```

- **Create an optimized release build:**
  ```sh
  cargo build --release
  ```

## Development Conventions

- **Formatting:** The project follows standard Rust formatting. Use `cargo fmt` to format the code.
- **Linting:** Use `cargo clippy` to check for common mistakes and style issues.
- **Dependencies:** Dependencies are managed in `Cargo.toml`.
