# Contributing

Bug reports, ideas and pull requests are welcome on
[GitHub](https://github.com/lemon-sh/virsh-tui). For anything larger
than a small fix, open an issue first so we can agree on the approach.

## Building and running

```sh
git clone https://github.com/lemon-sh/virsh-tui
cd virsh-tui
cargo run -- --demo
```

Before sending a pull request, run the same checks as CI:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Project layout

| Path | Contents |
|------|----------|
| `src/backend/` | `virsh` execution, output parsers, the event stream, the demo backend |
| `src/command/` | command plans, builders for each action, execution, temporary XML files |
| `src/app/` | the event loop, reactive dirty rendering, key dispatch per view, background polling |
| `src/ui/` | views, overlays (help, palette, wizard, confirmations) and widgets |
| `src/input/` | key engine, key map, focus cycling, the `:` parser, Tab completion |
| `src/theme/` | built-in themes, custom theme loading, the 256-colour fallback |
| `book/` | this documentation (mdBook) |
| `scripts/screenshots.py` | regenerates the screenshots from demo mode |
| `docs/` | system architecture, backend integration, UI, keymap, and developer documentation |

## Screenshots

The images in the README and this book are rendered from demo mode, so they
never show real hosts:

```sh
cargo build --release
python3 scripts/screenshots.py
```

The script needs headless Chrome or Chromium and ImageMagick.

## Documentation

The user guide lives in `book/src/`. Preview it with:

```sh
mdbook serve book --open
```

Internal architecture specifications and developer guides are located in `docs/`.

## Code style

- Rust 2024 edition; the minimum supported Rust version is 1.88.
- `rustfmt.toml` sets a line width of 110.
- No `unwrap()` on data that comes from `virsh` or from the user: parse
  errors become messages, not panics.
- Every mutation goes through a `CommandPlan`, so it can be shown, confirmed,
  dry-run and copied.
- All code, UI, comments, and documentation must be written in English.
