# Contributing to virsh-tui

Thanks for helping. Bug reports, ideas and pull requests are all welcome.
For anything bigger than a small fix, please open an issue first so we can
agree on the approach before you spend time on it.

## Getting started

```sh
git clone https://github.com/lemon-sh/virsh-tui
cd virsh-tui
cargo run -- --demo        # example data, nothing is executed
```

You need Rust 1.88 or newer. A libvirt daemon is only needed to connect to a real hypervisor.

## Before you open a pull request

Run the same checks as CI:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

- If you change behaviour or key bindings, update the user guide in `book/src/`
  and the documentation in `docs/`.
- If a screen changes visibly, regenerate the screenshots with
  `python3 scripts/screenshots.py` (needs a release build, headless Chrome
  and ImageMagick).

## Code guidelines

- Every change to libvirt goes through a `CommandPlan`, so it can be shown,
  confirmed, dry-run and copied. Run commands as argument lists, never
  through a shell.
- Text from libvirt or guests is untrusted: sanitize it before display and
  escape it before writing XML.
- Parse errors become messages, not panics.
- Code, UI text, docs and commit messages are in English.
- Never use `sudo`.
