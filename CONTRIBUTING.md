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

- Rust **nightly** toolchain (enforced by `rust-toolchain.toml`)

```sh
rustup toolchain install nightly
```

- **macOS:** Xcode Command Line Tools (`xcode-select --install`)
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

Start the daemon:

```sh
cargo run -- daemon
```

Open a URL with the selector:

```sh
cargo run -- open https://example.com
```

Re-scan installed browsers:

```sh
cargo run -- detect
```

## Project Structure

```
src/
  main.rs               — Entry point, CLI argument parsing, daemon loop
  app.rs                — AppCommand enum, shared types
  tray.rs               — System tray / menu bar
  hotkey.rs             — Global hotkey registration (macOS)
  single_instance.rs    — Single-instance enforcement via IPC
  default_browser.rs    — Default browser detection

  browser/
    mod.rs              — Browser enum and shared types
    detector.rs         — Scan the system for installed browsers
    launcher.rs         — Launch URLs in a specific browser

  config/
    mod.rs              — Config loading, saving, hotkeys, repositories
    rules.rs            — URL routing rules

  plugin/
    mod.rs              — Plugin system: channel fetching, installing, WASM

  ui/
    mod.rs              — Module exports
    selector.rs         — Browser selector window (standalone & daemon modes)
    default_prompt.rs   — "Always use this browser?" prompt

assets/
  icons/                — SVG icons for the UI
assets/
  browseraptor-logo.png
  screenshot.png
```

## Coding Guidelines

### Rust Style

- Format with `rustfmt` (the nightly toolchain's version).
- Run `cargo clippy` before committing and address all warnings.
- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).
- Use `anyhow::Result` for fallible functions unless a specific error type is needed.
- Prefer `gpui::SharedString` over `String` in UI-facing code.
- Use `tracing` for logging (not `println!` or `eprintln!`).
- Add doc comments (`///`) to all public items.

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

This project uses GitHub Actions for CI. The workflow is defined in `.github/workflows/`:

- **`build-binaries.yml`** — reusable workflow that compiles for Linux, macOS, and Windows.
- **`ci.yml`** — invoked on tag pushes (`v*`) or manually via `workflow_dispatch`.

When you open a pull request, ensure that:

- `cargo build --release --locked` succeeds on your target platform.
- You have run `cargo clippy` and fixed any warnings.
- Any new dependencies are justified in the commit message or PR description.
