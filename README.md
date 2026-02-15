# Vaultura

A secure, fully local, terminal-based (TUI) password manager built in Rust.

Vaultura keeps all your credentials encrypted on disk using modern cryptography and provides a fast, keyboard-driven 3-pane interface inspired by 1Password.

## Features

- **Strong encryption** — XChaCha20-Poly1305 authenticated encryption with Argon2id key derivation
- **Fully local** — No network access, no cloud sync, no telemetry. Your data never leaves your machine
- **3-pane TUI** — Groups, items, and details side by side for quick navigation
- **Fast search** — Case-insensitive multi-token search across all item fields
- **Password generator** — Configurable length, character sets, and ambiguous character exclusion
- **Clipboard integration** — Copy passwords/usernames with automatic clipboard clearing
- **Auto-lock** — Configurable idle timeout locks the vault automatically
- **Atomic saves** — Writes use temp file + fsync + rename to prevent corruption
- **Import/Export** — Move vaults between machines with re-encryption
- **Zero unsafe code** — `#![forbid(unsafe_code)]` enforced crate-wide

## Installation

### From source

```sh
git clone https://github.com/zombocoder/vaultura.git
cd vaultura
cargo install --path .
```

### Build and run directly

```sh
cargo run
```

## Usage

```
vaultura [OPTIONS]

Options:
  -v, --vault <PATH>    Path to the vault file
  -c, --config <PATH>   Path to the config file
  -h, --help            Print help
  -V, --version         Print version
```

On first launch, you'll be prompted to create a master password. This creates an encrypted vault file at the default platform data directory.

### Keyboard Shortcuts

#### Lock Screen

| Key | Action |
|-----|--------|
| `Enter` | Unlock / create vault |
| `Esc` | Quit |
| `Ctrl+C` | Quit |

#### Main Screen

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Switch between panels |
| `j` / `k` or arrows | Navigate lists |
| `/` | Activate search |
| `n` | New item |
| `e` | Edit selected item |
| `d` | Delete selected item |
| `g` | New group |
| `G` | Edit selected group |
| `D` | Delete selected group |
| `p` | Copy password to clipboard |
| `u` | Copy username to clipboard |
| `r` | Reveal / hide password |
| `Ctrl+S` | Save vault |
| `Ctrl+L` | Lock vault |
| `q` | Quit |

#### Item / Group Forms

| Key | Action |
|-----|--------|
| `Tab` / `Shift+Tab` | Next / previous field |
| `Ctrl+S` | Save |
| `Ctrl+P` | Open password generator |
| `Esc` | Cancel |

#### Password Generator

| Key | Action |
|-----|--------|
| `j` / `k` or arrows | Navigate options |
| `Space` / `Enter` | Toggle option |
| `Left` / `Right` | Adjust length |
| `r` | Regenerate |
| `Ctrl+S` | Use password |
| `Esc` | Cancel |

## Configuration

Vaultura stores its config as TOML at the platform config directory (`~/.config/vaultura/config.toml` on Linux/macOS).

```toml
vault_path = "/home/user/.local/share/vaultura/vault.vltr"
auto_lock_secs = 300
clipboard_clear_secs = 30
kdf_memory_cost_kib = 65536
kdf_time_cost = 3
kdf_parallelism = 4
```

| Setting | Default | Description |
|---------|---------|-------------|
| `vault_path` | Platform data dir | Path to the encrypted vault file |
| `auto_lock_secs` | `300` | Seconds of inactivity before auto-lock (0 to disable) |
| `clipboard_clear_secs` | `30` | Seconds before clipboard is automatically cleared |
| `kdf_memory_cost_kib` | `65536` | Argon2id memory parameter in KiB (64 MB) |
| `kdf_time_cost` | `3` | Argon2id iteration count |
| `kdf_parallelism` | `4` | Argon2id parallelism degree |

## Vault File Format

The vault file uses a custom binary format:

```
[VLTR magic 4B][version u32 LE][salt 32B][kdf_params 12B][nonce 24B][encrypted payload...]
```

The payload is serialized with bincode, then encrypted with XChaCha20-Poly1305. The key is derived from the master password and salt using Argon2id.

## Architecture

```
ui  -->  core  -->  storage  -->  crypto
```

- **crypto** — Argon2id key derivation, XChaCha20-Poly1305 AEAD
- **storage** — Binary vault file format, atomic writes
- **core** — Data models, CRUD operations, search, password generation
- **ui** — Ratatui/crossterm TUI with component pattern and action dispatch

## Security

- Encryption: XChaCha20-Poly1305 (256-bit key, 192-bit nonce)
- KDF: Argon2id with configurable memory/time/parallelism
- Atomic writes prevent vault corruption on crash
- Clipboard auto-clears after configurable timeout
- Auto-lock on idle
- No unsafe code
- No network access

## License

MIT
