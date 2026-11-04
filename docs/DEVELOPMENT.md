# Development Guidelines & Quality Standards

This document establishes the developer workflow, code style guidelines, CI quality gates, and architectural conventions for virsh-tui.

---

## Toolchain & Build Prerequisites

- **Language & Edition**: Rust 1.88 or newer, Edition 2024.
- **Operating System**: Linux is the primary target platform for real libvirt management. Offline demo mode (`--demo`) can run on any platform supported by Rust and Ratatui (including macOS and Windows).
- **External Dependencies**:
  - `virsh`: Required for live hypervisor interaction.
  - `virt-install`: Required for the New Domain Wizard.
  - `virt-viewer`: Optional; used for launching graphical VM viewer consoles.

---

## CI & Quality Gates

Every code change must satisfy all three quality gates before merging:

```sh
# 1. Formatting check
cargo fmt --check

# 2. Linter check (all targets, zero warnings allowed)
cargo clippy --all-targets -- -D warnings

# 3. Test suite
cargo test
```

---

## Coding Conventions & Principles

### 1. English-Only Policy
All code symbols, comments, docstrings, UI labels, documentation files, and git commit messages must be written in English.

### 2. No Shell Spawning
Commands must always be executed as explicit argument lists using `tokio::process::Command::new("virsh").args(...)` or standard `std::process::Command`. Never pass concatenated command strings to `sh -c` or `bash -c`.

### 3. All Mutations via `CommandPlan`
Any action that alters hypervisor state (domain lifecycle, XML updates, snapshot changes, volume allocation) must be represented as a `CommandPlan`. This ensures transparency, clipboard reproducibility, and dry-run safety.

### 4. Input Sanitization & Escaping
- Text originating from guests or external streams (e.g. DHCP hostnames, event descriptions) is untrusted and must pass through `model::sanitize` before being rendered.
- Hand-constructed XML parameters must pass through `xml::escape` to avoid XML injection or malformed definitions.

### 5. Error Handling & UI Resilience
- Unrecoverable panics in the UI thread must be avoided. Subprocess failures or parse issues should be captured and reported gracefully via the status message line or logged with `tracing`.
- When connecting to remote or constrained hypervisors, missing local files (like `/proc/stat`) must trigger clean telemetry fallbacks rather than failing.

### 6. Screenshots & Visual Testing
Visual artifacts and documentation screenshots are generated using the offline demo backend:
```sh
cargo build --release
python3 scripts/screenshots.py
```
This tool photographically captures frames using headless Chrome and updates images in `book/src/images/`.

