# Architecture

virsh-tui is a keyboard-driven terminal user interface for managing libvirt virtual machines, storage, networks, and hypervisor resources. It is implemented in Rust using [ratatui](https://ratatui.rs) and [crossterm](https://github.com/crossterm-rs/crossterm).

This document outlines the high-level system architecture, threading and async model, message passing, terminal lifecycle management, and reactive rendering pipeline.

---

## High-Level Overview

```
                      +---------------------------------------+
                      |               Terminal                |
                      |   (crossterm / alternate screen)     |
                      +-------------------+-------------------+
                                          |
                                    Key Events
                                          v
+-------------------+             +---------------+             +--------------------+
|  libvirt Daemon   |             |   KeyEngine   |             |  Config & Keymap   |
| (local or remote) |             +-------+-------+             |   (config.toml)    |
+---------+---------+                     |                     +---------+----------+
          |                               |                               |
     virsh CLI                            v                               v
          |                     +-------------------+          +---------------------+
          +<--------------------+    App State      |<---------+   Theme Engine      |
          |  (CommandPlan)      |  (Dashboard, Host,|          |   (9 built-in /     |
          |                     |   Networks, etc.) |          |    256-color map)   |
          v                     +---------+---------+          +---------------------+
+-------------------+                     |
| Background Tasks: |                     | Dirty Flag
|  - app::poller    |                     v
|  - events::stream |---------->+-------------------+
+-------------------+  LiveMsg  |   ratatui Draw    |
                                +-------------------+
```

The application strictly separates **UI rendering / event loop** from **libvirt command execution and background telemetry**. The main thread drives the Ratatui UI and input responsiveness, while asynchronous Tokio tasks handle periodic telemetry polling and streaming events without blocking interface interaction.

---

## Main Modules and Directory Structure

- **`src/main.rs`**: CLI argument parsing with `clap`, logging setup (`tracing-subscriber`), terminal preparation, and invocation of `app::run`.
- **`src/app/`**:
  - `mod.rs`: Main application struct (`App`), central event loop, action dispatching, and view routing.
  - `poller.rs`: Periodic background polling task querying `virsh` for domain summaries, raw statistics, and host metrics.
  - `render.rs`: Top-level draw coordinator orchestrating active views and modal overlays.
  - `domains.rs`, `detail.rs`, `networks.rs`, `storage.rs`, `wizard.rs`, `palette.rs`, `settings.rs`, `ex.rs`: Area-specific state and logic controllers.
- **`src/backend/`**:
  - `virsh/`: Wrapper around the `virsh` CLI tool. Includes subprocess spawning (`exec.rs`), streaming event monitoring (`events.rs`), host local `/proc` inspection (`host_local.rs`), and output parsers (`parse_list.rs`, `parse_domstats.rs`, `parse_nodestats.rs`, `parse_xml.rs`, `parse_leases.rs`, `parse_help.rs`).
  - `demo/`: Offline mock provider with simulated hypervisor metrics, demo domains, and pre-recorded XML definitions for non-destructive local evaluation.
- **`src/command/`**:
  - `plan.rs`: Definition of `CommandPlan`, encapsulating multi-step commands, display descriptions, and confirmation requirements.
  - `exec.rs`: Command execution runner supporting dry-run execution, logging, and history tracking.
  - `tempxml.rs`: Safe temporary XML file management with strict `0600` permissions.
  - `builders/`: Command builders generating precise `virsh` and `virt-install` command lines for domain lifecycle, hardware modification, network actions, snapshots, and storage management.
- **`src/input/`**:
  - `keymap.rs`: Central keymap definitions (`DEFAULT_BINDINGS`) and override loader (`keymap.toml`).
  - `engine.rs`: Vim-style keystroke sequence engine with leader key timeout handling.
  - `ex.rs`: `:` command-line parser and command execution handler.
  - `complete.rs`: Tab-autocompletion engine for internal commands, domain names, network names, themes, and `virsh` flags.
- **`src/metrics/`**:
  - `store.rs`: In-memory historical ring buffer storing time-series telemetry for CPU, memory, balloon, disk IO, and network rates across 60-second, 5-minute, and 1-hour windows.
  - `sampler.rs` / `ring.rs`: Numerical sampling and rate computation algorithms.
- **`src/ui/`**:
  - `chrome.rs`: Top-bar header (host, URI, status) and bottom message / status line.
  - `views/`: View renderers for Dashboard, Detail (Overview, Monitor, Hardware, Snapshots, Console, XML), Host, Networks, Storage, Events, and Settings.
  - `overlays/`: Modal dialogs for Help, Command Palette, New Domain Wizard, and Confirm dialogs.
  - `widgets/`: Reusable Ratatui widgets including Braille charts, sparklines, gauges, horizontal bars, tables, and dynamic which-key popups.
- **`src/theme/`**:
  - Theme definitions, built-in themes (Tokyo Night, Catppuccin, Nord, Dracula, Gruvbox, etc.), custom theme TOML loading, truecolor detection, and Euclidean-distance 256-color fallback palette mapping.
- **`src/xml/`**:
  - XML editing, namespace-preserving formatting, diff generation (`similar`), and syntax highlighting.

---

## Terminal Lifecycle & Safety

Terminal management requires strict guarantees to avoid leaving the user's terminal in an unreadable state or interfering with external processes.

### RAII Terminal Guard (`TerminalGuard`)

virsh-tui encapsulates alternate screen and raw mode initialization within `TerminalGuard`:

1. **Deferred Raw Mode**: Terminal raw mode is not enabled immediately at binary start. If `virsh` triggers a PolicyKit authentication agent (e.g., `pkttyagent`), the authentication prompt requires standard terminal canonical input and echo. Enabling raw mode too early produces broken line breaks and prevents password entry. `TerminalGuard` initializes only after initial preflight checks complete.
2. **Deterministic Cleanup**: `TerminalGuard` implements `Drop`, guaranteeing that `DisableMouseCapture`, `LeaveAlternateScreen`, `disable_raw_mode()`, and cursor visibility restoration are executed on normal exit, error exits, and uncaught panics.

### Detached TTY & Session Isolation (`setsid`)

Once the TUI loop is running, spawned background commands (`virsh domstats`, `virsh event`, etc.) must never attempt to hijack `/dev/tty` or read keyboard input. `backend::virsh::exec` isolates child subprocesses by setting `setsid()` on Unix platforms. Unauthorized calls fail immediately with clear authentication error messages rather than hanging the user interface.

---

## Telemetry Pipeline & Message Passing

```
+--------------------------------------------------------+
|                     Tokio Runtime                      |
|                                                        |
|  +-------------------+        +---------------------+  |
|  |   app::poller     |        |   events::stream    |  |
|  | (periodic virsh)  |        | (virsh event --loop)|  |
|  +---------+---------+        +----------+----------+  |
|            |                             |             |
|            | LiveMsg::DomainsWithStats   |             |
|            | LiveMsg::HostStats          |             |
|            | LiveMsg::Tick               | LiveMsg     |
|            +--------------+--------------+             |
|                           |                            |
|                           v                            |
|             mpsc::Receiver<LiveMsg>                    |
+---------------------------+----------------------------+
                            |
                            v
                +------------------------+
                |    App::on_live(...)   |
                |  - Updates MetricsStore|
                |  - Sets DirtyFlags     |
                +------------------------+
```

1. **Poller Loop**: Runs on a background Tokio task with configurable tick rates (default: 1 second). Executes read-only commands:
   - `virsh list --all`
   - `virsh domstats --raw`
   - `virsh nodecpustats --percent` and `virsh nodememstats` (on remote hypervisors) or `/proc/stat` and `/proc/meminfo` (on local hypervisors)
2. **Event Stream Task**: Runs `virsh event --loop` to receive asynchronous domain lifecycle, network, and storage pool events directly from libvirt.
3. **Channel Delivery**: Background tasks send typed `LiveMsg` enums over an asynchronous `tokio::sync::mpsc` channel to the main loop. The main loop drains pending messages with non-blocking `try_recv()`, updates the local state, feeds new telemetry data into `MetricsStore`, and marks the UI as dirty.

---

## Reactive Dirty-Flag Rendering

To prevent excessive CPU and power consumption from redrawing static frames at terminal refresh rates, virsh-tui implements a reactive dirty-flag system:

- **`DirtyFlags`**: Tracks fine-grained invalidation:
  - `METRICS`: New telemetry samples arrived from poller.
  - `VIEW`: User switched views or tabs.
  - `CURSOR`: List or table cursor selection moved.
  - `OVERLAY`: Help, command palette, wizard, or confirmation modal opened or closed.
  - `MESSAGE`: Status bar notification updated.
  - `TICK`: General tick invalidation.
- **Draw Elision**: In `app::run`, Ratatui's `terminal.draw()` is only invoked if `dirty_flags.is_dirty()` returns true or a terminal resize event was received. When idle with no incoming events or user input, zero redraw cycles are performed.

