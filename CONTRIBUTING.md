# Contributing to Vaultura

Thanks for your interest in contributing to Vaultura! This document covers guidelines and instructions for contributing.

## Getting Started

1. Fork the repository and clone your fork
2. Install Rust (stable) via [rustup](https://rustup.rs/)
3. Build and run tests:

```sh
cargo build
cargo test
cargo clippy -- -W clippy::all
```

## Development Setup

### Prerequisites

- Rust stable (latest)
- A terminal emulator that supports 256 colors

### Running

```sh
# Run with default settings
cargo run

# Run with a specific vault file
cargo run -- --vault /tmp/test.vault
```

### Testing

Tests use fast KDF parameters (1 MB / 1 iteration) to keep the suite fast.

```sh
# Run all tests
cargo test

# Run tests for a specific module
cargo test crypto::
cargo test core::vault_service::
cargo test storage::

# Run with output
cargo test -- --nocapture
```

## Project Structure

```
src/
  main.rs                  # CLI entry point, terminal setup
  lib.rs                   # Crate root
  error.rs                 # Error types

  crypto/                  # Cryptographic primitives
    kdf.rs                 # Argon2id key derivation
    aead.rs                # XChaCha20-Poly1305

  core/                    # Business logic
    models.rs              # Data models (Group, Item, VaultPayload)
    vault_service.rs       # CRUD, search, lock/unlock
    password_generator.rs  # Password generation

  storage/                 # Persistence
    format.rs              # Binary format constants
    vault_file.rs          # Read/write vault files

  ui/                      # Terminal UI
    mod.rs                 # Action enum, Component trait
    app.rs                 # Event loop, action dispatch
    theme.rs               # Colors and styles
    screens/               # Full-screen views
    panels/                # 3-pane layout panels
    modals/                # Overlay dialogs

  clipboard/               # Clipboard with auto-clear
  config/                  # TOML configuration
```

### Layer Dependencies

The codebase follows strict layering:

```
ui --> core --> storage --> crypto
```

- **crypto** has no internal dependencies
- **storage** depends on crypto
- **core** depends on storage and crypto
- **ui** depends on core

Panels and modals never access `VaultService` directly. They emit `Action` values which `App` dispatches.

## Code Guidelines

### General

- No `unsafe` code. The crate enforces `#![forbid(unsafe_code)]`
- All code must pass `cargo clippy -- -W clippy::all` with zero warnings
- All tests must pass before submitting a PR
- Keep dependencies minimal. Justify any new dependency

### Style

- Follow standard Rust formatting (`cargo fmt`)
- Use `thiserror` for error types
- Prefer returning `Result` over panicking
- Keep functions focused and short
- Add tests for new functionality

### Security

- Never log, print, or expose passwords in plaintext
- Use `SecretBox` for sensitive data at application boundaries
- All errors from crypto operations must be handled (never unwrap)
- Clipboard must always auto-clear
- Test that wrong passwords are rejected

### UI Components

All UI components implement the `Component` trait:

```rust
pub trait Component {
    fn handle_key(&mut self, key: KeyEvent) -> Action;
    fn render(&self, frame: &mut Frame, area: Rect);
}
```

- `handle_key` returns an `Action` for the app to dispatch
- `render` draws the component into the given area
- Components should not have side effects in `render`
- Keep rendering logic separate from business logic

### Adding a New Modal

1. Create a new file in `src/ui/modals/`
2. Implement the `Component` trait
3. Add a variant to the `Modal` enum in `app.rs`
4. Add corresponding `Action` variants in `ui/mod.rs`
5. Handle the actions in `App::handle_action`
6. Register the modal in `App::render` and `App::handle_input`

### Adding a New Action

1. Add the variant to `Action` in `src/ui/mod.rs`
2. Handle it in `App::handle_action` in `src/ui/app.rs`
3. Emit it from the relevant component's `handle_key`

## Submitting Changes

1. Create a feature branch from `main`
2. Make your changes in focused, logical commits
3. Ensure all checks pass:
   ```sh
   cargo fmt --check
   cargo clippy -- -W clippy::all
   cargo test
   ```
4. Open a pull request with a clear description of the change
5. Link any related issues

### Commit Messages

- Use imperative mood: "Add search filtering" not "Added search filtering"
- Keep the first line under 72 characters
- Reference issues where applicable: "Fix clipboard clear race condition (#42)"

### Pull Requests

- Keep PRs focused on a single change
- Include tests for new functionality
- Update the README if adding user-facing features
- Describe the "what" and "why" in the PR description

## Reporting Issues

- Check existing issues before creating a new one
- Include your OS, terminal emulator, and Rust version
- For bugs, include steps to reproduce
- For crashes, include the panic message and backtrace (`RUST_BACKTRACE=1`)

## Areas for Contribution

Here are some areas where contributions are welcome:

- **TOTP support** — Add time-based one-time password generation
- **Custom fields** — Allow items to have user-defined key-value fields
- **Theming** — User-configurable color schemes via config
- **Vault migration** — Import from other password managers (KeePass, Bitwarden CSV)
- **Accessibility** — Screen reader support, high-contrast themes
- **Performance** — Benchmarks and optimization for large vaults (10k+ items)
- **Packaging** — Homebrew formula, AUR package, Nix flake
