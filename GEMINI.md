# GEMINI Project Context: 文档风格转换器 (RustF)

## Project Overview

This is a desktop GUI application named "文档风格转换器" (Document Style Converter), developed in Rust using the `eframe` / `egui` framework. 

The application serves as a real-time Markdown editor with a side-by-side preview panel. Its primary feature is the ability to import from and export to DOCX files, leveraging the Pandoc command-line tool as a backend. It is designed to be cross-platform, with specific logic for locating system fonts on Windows, macOS, and Linux.

Key features include:
- Live Markdown editing and preview.
- Exporting Markdown content to a DOCX file.
- Importing a DOCX file and converting it to Markdown.
- Setting a custom DOCX file as a style template (`--reference-doc` in Pandoc).
- Merging multiple Markdown files.
- A tool for finding and replacing `{{placeholder}}` style template markers.

## Architecture

The application logic is structured into several modules within the `src` directory:
- `main.rs`: The main entry point that initializes and runs the `eframe` application.
- `app.rs`: Contains the core application logic, including the main `MyApp` struct, UI layout, state management, and event handling.
- `font_utils.rs`: Provides utility functions for detecting and loading system-native CJK fonts in a cross-platform manner.

Long-running tasks, such as Pandoc conversions, are executed on separate threads using `crossbeam-channel` to avoid blocking the UI.

## Building and Running

This project uses the standard Rust/Cargo toolchain. Pandoc is a required runtime dependency.

*   **Runtime Dependency:** Before running, ensure **Pandoc** is installed and accessible via the system's `PATH`, or place the `pandoc.exe` executable in the same directory as the final `RustF` executable.

*   **Run (Development):**
    ```sh
    cargo run
    ```

*   **Build (Release):**
    ```sh
    cargo build --release
    ```
    The optimized executable will be located at `target/release/RustF.exe`.

*   **Testing:**
    The project does not currently contain a test suite, but if tests are added, they can be run with:
    ```sh
    cargo test
    ```

## Development Conventions

- **Code Style:** The code follows standard Rust conventions and is formatted with `rustfmt`.
- **Modularity:** Logic is separated into modules (`app`, `font_utils`) for clarity.
- **Cross-Platform:** Platform-specific code, especially for font handling, is managed via conditional compilation (`#[cfg(target_os = "...")]`).
- **Error Handling:** Background tasks communicate results (both `Ok` and `Err`) back to the main UI thread via channels, with errors displayed to the user in native dialog boxes.
- **Dependencies:** Cargo manages all Rust dependencies. Runtime dependencies like Pandoc are expected to be present on the system.
