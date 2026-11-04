# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added
- Dynamic which-key leader popup reflecting real-time keymap and prefixes.
- Focus cycling with `Tab` / `Shift-Tab` (`cycle_focus`) across multi-panel views (Networks, Storage, Settings).
- Instant tab switching with single digits `1`..`5`.
- Comprehensive architecture and developer documentation suite under `docs/`.
- Deferred raw mode and session TTY detachment for clean PolicyKit authentication handling.

### Changed
- Removed counts and dot-repeat in favor of instant view transitions.
- Replaced separate pane focus keys (`h`/`l`) with unified `Tab`/`Shift-Tab` focus cycling.
- Refactored project documentation suite.

## [0.1.0] - 2026-10-07

First public release.

- Domains dashboard with live CPU, memory, disk and network graphs, filters,
  sorting, marks and bulk lifecycle actions
- Domain detail: overview, monitor with 60 s / 5 min / 1 h history, hardware
  editor with XML diff, snapshots, console and XML
- Host, Networks, Storage and Events views
- New domain wizard that builds and runs `virt-install`
- `:` command line with Tab completion and `virsh` passthrough, command
  palette, which-key popup, remappable keys
- Nine built-in themes, custom themes, settings screen, 256-colour fallback
- `--demo`, `--dry-run`, `--probe` and `--dump`

[Unreleased]: https://github.com/lemon-sh/virsh-tui/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/lemon-sh/virsh-tui/releases/tag/v0.1.0
