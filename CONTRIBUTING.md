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
  - [Testing Locally](#testing-locally)
  - [Publishing](#publishing)
  - [Submitting to the Channel](#submitting-to-the-channel)
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

## Creating a Plugin

Browseraptor plugins are WebAssembly (WASM) modules that can intercept and route URLs, cancel navigation, or specify a browser profile. Plugins run in a sandboxed WASM runtime and communicate with the host via exported functions and shared memory.

### Plugin API

Every plugin must export the following functions from its WASM module:

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

The module must also export a `memory` section (standard WASM linear memory).

The `evaluate` function must return a JSON-serialized `PluginResult`:

```rust
pub struct PluginResult {
    pub browser: Option<String>,  // Browser name (e.g., "firefox", "chrome")
    pub profile: Option<String>,  // Browser profile (e.g., "work", "personal")
    pub cancel: bool,             // If true, cancel navigation entirely
}
```

If `cancel` is `true`, navigation is blocked. If `browser` is `Some`, the URL is opened in that browser (optionally with a specific `profile`). If both are `None`, the plugin passes — the next plugin or the default router handles the URL.

#### Optional Exports

```rust
// Return current configuration as a JSON string.
#[no_mangle]
pub extern "C" fn get_config() -> i32;

// Receive new configuration (JSON string via pointer/length).
#[no_mangle]
pub extern "C" fn set_config(ptr: i32, len: i32) -> i32;
```

### Plugin Manifest

Every plugin must include a `manifest.json` alongside its WASM file:

```json
{
  "id": "my-plugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "author": "Your Name",
  "description": "Routes work-related URLs to Firefox with the work profile"
}
```

### Development Setup

We recommend writing plugins in Rust and compiling to `wasm32-unknown-unknown`.

**Prerequisites:**

```sh
# Install the wasm32 target
rustup target add wasm32-unknown-unknown

# Install wasm-pack (optional, but recommended for building)
cargo install wasm-pack
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

Or with `wasm-pack`:

```sh
wasm-pack build --target web --out-name plugin.wasm
# Output: pkg/plugin.wasm
```

### Testing Locally

After building your plugin, you can load it into Browseraptor locally for testing:

1. Create a `manifest.json` in the same directory as your `.wasm` file.
2. Copy the WASM file and manifest to a location Browseraptor can find.
3. Run Browseraptor and verify the plugin is loaded via the UI or logs.

Example minimal test:

```sh
# Build
cargo build --release --target wasm32-unknown-unknown

# Create manifest
cat > manifest.json <<EOF
{
  "id": "my-plugin",
  "name": "My Plugin",
  "version": "1.0.0",
  "author": "You",
  "description": "Test plugin"
}
EOF

# Copy WASM to a test location
cp target/wasm32-unknown-unknown/release/my_plugin.wasm ./plugin.wasm
```

### Publishing

To publish a plugin for other users, use the [Browseraptor Publish Action](https://github.com/browseraptor/browseraptor_publish):

1. Generate an Ed25519 key pair:

   ```sh
   git clone https://github.com/browseraptor/browseraptor_publish.git
   cd browseraptor_publish/tools/bap-sig
   cargo build --release
   ./target/release/bap-sig generate
   ```

2. Add the keys to your repository's [GitHub Secrets](https://docs.github.com/en/actions/security-for-github-actions/security-guides/using-secrets-in-github-actions):

   - `BAP_PRIVATE_KEY` — your Ed25519 private key (hex-encoded)
   - `BAP_PUBLIC_KEY` — your Ed25519 public key (hex-encoded)

3. Create `.github/workflows/publish.yml`:

   ```yaml
   name: Publish Plugin
   on:
     push:
       tags:
         - 'v*.*.*'

   jobs:
     publish:
       runs-on: ubuntu-latest
       steps:
         - uses: actions/checkout@v4

         - name: Setup Rust
           uses: dtolnay/rust-toolchain@stable

         - name: Publish plugin
           uses: browseraptor/browseraptor_publish@v1
           with:
             plugin-name: my-plugin
             source-dir: .
             build-command: wasm-pack build --target web --out-name plugin.wasm
             private-key: ${{ secrets.BAP_PRIVATE_KEY }}
             public-key: ${{ secrets.BAP_PUBLIC_KEY }}
             channel-repo: browseraptor/browseraptor_channel
             github-token: ${{ secrets.GITHUB_TOKEN }}
   ```

4. Tag and push:

   ```sh
   git tag v1.0.0
   git push origin v1.0.0
   ```

The action will build your WASM, create a signed `.tar.gz` bundle, create a GitHub Release, and open a PR to the [channel registry](https://github.com/browseraptor/browseraptor_channel).

### Submitting to the Channel

Once your plugin is published, submit it to the [default channel](https://github.com/browseraptor/browseraptor_channel) so users can discover and install it:

1. **Fork** the [browseraptor/browseraptor_channel](https://github.com/browseraptor/browseraptor_channel) repository.
2. **Add an index entry** to `repository/index.json` in alphabetical order by plugin ID.
3. **Create a metadata file** at `repository/{first-letter}/{plugin-id}.json` (e.g., `repository/m/my-plugin.json`).

   ```json
   {
     "id": "my-plugin",
     "name": "My Plugin",
     "description": "Routes work URLs to Firefox with work profile",
     "author": {
       "name": "Your Name",
       "url": "https://github.com/yourusername"
     },
     "version": "1.0.0",
     "license": "MIT",
     "categories": ["url-routing", "productivity"],
     "tags": ["work", "routing", "productivity"],
     "min_browseraptor_version": "0.2.0",
     "permissions": ["navigation"],
     "wasm": {
       "url": "https://github.com/yourusername/my-plugin/releases/download/v1.0.0/plugin.wasm",
       "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
       "size_bytes": 24576,
       "entry_point": "evaluate",
       "api_version": "1.0.0"
     },
     "repository": {
       "url": "https://github.com/yourusername/my-plugin",
       "type": "git"
     },
     "changelog": "https://github.com/yourusername/my-plugin/releases",
     "docs": "https://github.com/yourusername/my-plugin#readme",
     "updated_at": "2026-06-06T00:00:00Z",
     "created_at": "2026-06-06T00:00:00Z"
   }
   ```

4. **Submit a pull request** with your changes. Use the PR template and check all boxes.

**Guidelines:**

- Submit one plugin per PR.
- The `wasm.url` must point to a release asset in your GitHub repository.
- The `wasm.sha256` must be the SHA256 hash of the WASM file.
- The `id` must match the filename (without `.json`) and be placed in the directory matching the first letter of the ID.
- A valid semver tag (e.g., `v1.0.0`) must exist on your plugin repository.
- Only submit plugins you maintain.

For a full reference, see the [channel CONTRIBUTING.md](https://github.com/livrasand/browseraptor_channel/blob/master/CONTRIBUTING.md).

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
