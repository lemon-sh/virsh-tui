# virsh-tui

Rust + Ratatui TUI for libvirt/virsh. Everything (UI, code, docs, commits) is in English.

## Documentation Index

- [Architecture & Execution Model](docs/ARCHITECTURE.md): System design, async runtime, message passing via `LiveMsg`, reactive dirty-flag rendering, and terminal lifecycle (`TerminalGuard`, raw mode deferral, detached TTY `setsid`).
- [Backend & Libvirt Integration](docs/BACKEND.md): Subprocess execution layer, output parsers (list, domstats, nodestats, leases, XML), remote hypervisor support, and offline demo simulator.
- [UI & View Architecture](docs/UI.md): Layout hierarchy, views (Dashboard, Domain Detail, Host, Networks, Storage, Events), overlays (Help, Command Palette, Wizard, Settings, Confirm), widgets, and the theme engine.
- [Input Handling & Keymap System](docs/INPUT_AND_KEYMAP.md): Unified keymap definitions (`DEFAULT_BINDINGS`), `KeyEngine`, focus cycling (`Tab`/`Shift+Tab`), instant tab switching (`1`-`5`), dynamic which-key popup, and `:` command line.
- [Command Planning & Safety](docs/COMMAND_PLANNING.md): `CommandPlan` architecture, atomic operations (safe snapshot revert, in-memory XML editing & validation), secure temporary files (`command::tempxml`), and dry-run execution.
- [Safety & Security Model](docs/SAFETY_AND_SECURITY.md): Least privilege (no `sudo`), untrusted input sanitization (`model::sanitize`), XML escaping, Polkit diagnostics, and demo mode isolation.
- [Configuration & Persistence](docs/CONFIGURATION.md): TOML configuration schema (`config.toml`), key overrides (`keymap.toml`), custom themes, and interactive `:set` command.
- [Development Guidelines & Quality Standards](docs/DEVELOPMENT.md): Coding standards, quality gates, error handling, and visual screenshot regeneration.
- [User Guide](book/src/SUMMARY.md): End-user manual and reference book.

## Core Rules

- **Gates**: `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test` must pass cleanly at all times.
- **Safety**: Never execute `sudo`. Every hypervisor mutation must pass through a `CommandPlan`.
- **Language**: All code, comments, documentation, and commit messages must be in English.
- **User Docs**: Keep `book/` synchronized with any changes to application behavior, key bindings, or options.
