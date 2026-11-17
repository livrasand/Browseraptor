# Contributing to Browseraptor

First off, thanks for taking the time to contribute!

The following is a set of guidelines for contributing to Browseraptor. These are mostly guidelines, not rules. Use your best judgment, and feel free to propose changes to this document in a pull request.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [How to Contribute](#how-to-contribute)
  - [Reporting Bugs](#reporting-bugs)
  - [Suggesting Features](#suggesting-features)
  - [Pull Requests](#pull-requests)
- [Development Setup](#development-setup)
  - [Prerequisites](#prerequisites)
  - [Building](#building)
  - [Running](#running)
- [Project Structure](#project-structure)
- [Creating a Plugin](#creating-a-plugin)
  - [Plugin API](#plugin-api)
  - [Plugin Manifest](#plugin-manifest)
  - [Development Setup](#development-setup-1)
  - [Installing a Local Plugin](#installing-a-local-plugin)
  - [Translation Plugins](#translation-plugins)
- [Coding Guidelines](#coding-guidelines)
  - [Rust Style](#rust-style)
  - [Commit Conventions](#commit-conventions)
- [Workflows](#workflows)

## Code of Conduct

This project adheres to the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). By participating, you are expected to uphold it.

## How to Contribute

### Reporting Bugs

Before opening a bug report:

- Check the [existing issues](https://github.com/livrasand/Browseraptor/issues) to see if the problem has already been reported.
- Collect information about your environment: OS version, Rust version (`rustc --version`), and any relevant logs.

When opening an issue, include:

- A clear, descriptive title.
- Steps to reproduce the behavior.
- Expected vs. actual behavior.
- Screenshots / screen recordings if applicable.
- Your OS, architecture, and Browseraptor version.

### Suggesting Features

Feature requests are welcome. When suggesting a feature:

- Explain why it would be useful and how it fits into the project's scope.
- If possible, sketch the API or configuration changes it would involve.
- Be open to discussion and alternative approaches.

### Pull Requests

1. **Fork** the repository and create your branch from `master`.
2. **Test** your changes: `cargo build --locked` and `cargo clippy` should pass without warnings.
3. **Keep changes minimal** — focus on the specific issue or feature. Avoid unrelated refactors or formatting changes.
4. **Update documentation** (doc comments, README) if your change affects the public interface or behavior.
5. Open a pull request with a clear title and description linking to the related issue (if any).

## Development Setup

### Prerequisites

- Rust **stable** toolchain (configured by `rust-toolchain.toml`)

```sh
rustup toolchain install stable
```

- **macOS:** Xcode (with Metal Toolchain) for local builds that compile `gpui`
- **Linux:** System libraries (see [README](./README.md#linux))
- **Windows:** Visual Studio Build Tools with the "Desktop development with C++" workload

### Building

```sh
git clone https://github.com/livrasand/Browseraptor.git
cd Browseraptor
cargo build --release
```

To build the macOS `.app` bundle:

```sh
./bundle_macos.sh
# Output: dist/Browseraptor.app
```

### Running

Run the app:

```sh
cargo run
```

Open a URL with the selector:

```sh
cargo run -- open https://example.com
```

List installed browsers:

```sh
cargo run -- list
```

## Project Structure

```
src/
  main.rs               — Entry point, CLI argument parsing, daemon loop
  app.rs                — AppCommand enum, shared types
  tray.rs               — System tray / menu bar
  hotkey.rs             — Global hotkey registration
  single_instance.rs    — Single-instance enforcement via IPC
  default_browser.rs    — Default browser detection

  browser/
    mod.rs              — Browser enum and shared types
    detector.rs         — Scan the system for installed browsers
    launcher.rs         — Launch URLs in a specific browser

  config/
    mod.rs              — Config loading, saving, hotkeys, installed plugins
    rules.rs            — URL routing rules

  plugin/
    mod.rs              — Plugin system: local/dev loading, WASM host

  ui/
    mod.rs              — Module exports
    selector.rs         — Browser selector window and plugin UI
    default_prompt.rs   — "Set default browser" prompt

assets/
  icons/                — SVG icons for the UI
  os/                   — OS icons used in release notes
  browseraptor-logo.png
  screenshot.png
```

## Creating a Plugin

Browseraptor plugins are WebAssembly (WASM) modules. Plugins run in a sandboxed WASM runtime and communicate with the host via exported functions and shared memory.

Plugins are installed locally (for example, development plugins). Each installed plugin must have a `.wasm` file and a `manifest.json` in the same directory.

### Plugin API

Every plugin must export a `memory` section and the following functions:

```rust
// Allocate `size` bytes in WASM linear memory, return a pointer.
#[no_mangle]
pub extern "C" fn alloc(size: i32) -> i32;

// Free `size` bytes at `ptr` in WASM linear memory.
#[no_mangle]
pub extern "C" fn dealloc(ptr: i32, size: i32);

// Evaluate a URL. Receives a pointer/length pair pointing to a UTF-8 URL string.
// Returns a pointer to a result block: [4-byte LE length][JSON data].
#[no_mangle]
pub extern "C" fn evaluate(ptr: i32, len: i32) -> i32;
```

The `evaluate` function must return a JSON-serialized `PluginResult`:

```rust
pub struct PluginResult {
    pub browser: Option<String>,  // Browser name
    pub profile: Option<String>,  // Browser profile
    pub cancel: bool,             // If true, cancel navigation
}
```

If `cancel` is `true`, navigation is blocked. If `browser` is `Some`, the URL is opened in that browser (optionally with a specific `profile`). If both `browser` and `cancel` are not used, the plugin passes and the next plugin or router handles the URL.

#### Optional Exports

```rust
// Return current configuration as a JSON string.
#[no_mangle]
pub extern "C" fn get_config() -> i32;

// Receive new configuration (JSON string via pointer/length) and return a JSON response.
#[no_mangle]
pub extern "C" fn set_config(ptr: i32, len: i32) -> i32;

// Return UI translations as a JSON object: { "key": "translated text" }.
#[no_mangle]
pub extern "C" fn get_ui_strings() -> i32;
```

`get_config` and `set_config` are used by the plugin configuration UI for local plugins. `get_ui_strings` is used for translation plugins.

### Plugin Manifest

Every plugin must include a `manifest.json` in the same directory as its `.wasm` file:

```json
{
  "id": "my-plugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "author": "Your Name",
  "description": "Routes work-related URLs to a specific browser"
}
```

Browseraptor reads `id`, `name`, `version`, `author`, and `description` from the manifest.

### Development Setup

We recommend writing plugins in Rust and compiling to `wasm32-unknown-unknown`.

**Prerequisites:**

```sh
rustup target add wasm32-unknown-unknown
```

**Scaffold a new plugin:**

```sh
cargo init --lib my-plugin
cd my-plugin
```

**`Cargo.toml`:**

```toml
[package]
name = "my-plugin"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

**Build:**

```sh
cargo build --release --target wasm32-unknown-unknown
# Output: target/wasm32-unknown-unknown/release/my_plugin.wasm
```

### Installing a Local Plugin

1. Build your plugin `.wasm` file and create a `manifest.json` in the same directory.
2. Run Browseraptor and open the plugins panel.
3. Click **+ Dev** and select the plugin `.wasm` file.
4. Browseraptor validates the module and saves the plugin in the local configuration.
5. If the plugin exports `get_config` and `set_config`, you can configure it from the UI.
6. You can uninstall the plugin from the same panel.

### Translation Plugins

A translation plugin changes UI text by exporting `get_ui_strings`.

`get_ui_strings` must return a JSON object that maps translation keys to strings:

```json
{
  "plugins.title": "PLUGINS",
  "plugins.configure": "Configure",
  "plugins.save": "Save",
  "plugins.cancel": "Cancel"
}
```

Notes:

- The function must follow the same result format as other plugin functions: return a pointer to `[4-byte LE length][JSON data]`.
- Browseraptor loads UI translations on startup and after installing or uninstalling a plugin.
- If a key is missing, Browseraptor uses the default English text.
- The first installed local plugin that provides valid `get_ui_strings` data is used for UI translations.

Common keys:

- `plugins.title`, `plugins.empty`, `plugins.path`, `plugins.error`
- `plugins.open_folder`, `plugins.configure`, `plugins.uninstall`
- `plugins.configure_title`, `plugins.default_browser`, `plugins.default_profile`
- `plugins.rules`, `plugins.domain`, `plugins.profile`, `plugins.select`, `plugins.select_browser`, `plugins.default`
- `plugins.add_rule`, `plugins.copy`, `plugins.save`, `plugins.cancel`
- `plugins.invalid_json`, `plugins.invalid_response`, `plugins.read_config_error`, `plugins.load_error`, `plugins.save_error`

## Coding Guidelines

### Rust Style

- Format with `rustfmt`.
- Run `cargo clippy` before committing and address all warnings.
- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).
- Use `anyhow::Result` for fallible functions unless a specific error type is needed.
- Prefer `gpui::SharedString` over `String` in UI-facing code.
- Use `tracing` for logging (not `println!` or `eprintln!`).
- Add doc comments (`///`) to public items when useful.

### What to Avoid

- Unnecessary dependencies. If the project already has what you need, reuse it.
- Dead code. Remove unused functions, imports, and fields.
- Hardcoded paths or values that should be configurable.

### Commit Conventions

Use clear, imperative commit messages:

```
feat: add browser icon cache for macOS
fix: prevent crash when no browsers are detected
refactor: extract hotkey parsing into its own module
docs: update README with Windows build instructions
```

Prefixes: `feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`, `style:`, `ci:`.

## Workflows

This project uses GitHub Actions for CI. The workflows are defined in `.github/workflows/`:

- **`build-binaries.yml`** — reusable workflow that builds binaries for Linux, macOS, and Windows and can publish a GitHub release.
- **`ci.yml`** — runs on tag pushes (`v*`) or manually via `workflow_dispatch`.

Manual runs can set `publish-release`, `tag`, and `prerelease`.

When you open a pull request, ensure that:

- `cargo build --release --locked` succeeds on your target platform.
- You have run `cargo clippy` and fixed any warnings.
- Any new dependencies are justified in the commit message or PR description.
