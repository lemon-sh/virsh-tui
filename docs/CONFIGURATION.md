# Configuration & Persistence

virsh-tui uses TOML configuration files to persist application settings, key bindings, and custom themes across sessions.

---

## Configuration Paths

All configuration files are stored according to XDG base directory specifications:

| File | Purpose | Default Location |
|------|---------|------------------|
| **`config.toml`** | Application settings & options | `~/.config/virsh-tui/config.toml` |
| **`keymap.toml`** | Custom key binding overrides | `~/.config/virsh-tui/keymap.toml` |
| **`themes/*.toml`** | Custom user-created color themes | `~/.config/virsh-tui/themes/` |

The configuration directory can be overridden at startup using the `--config <path>` command-line option.

---

## `config.toml` Schema

```toml
[general]
# Default libvirt connection URI (e.g. "qemu:///system", "qemu:///session", "qemu+ssh://host/system")
default_uri = "qemu:///system"

# Telemetry poller refresh interval in seconds (default: 1)
refresh_interval_secs = 1

# Active theme name
theme = "tokyo-night"

[ui]
# Panel border style: "rounded", "plain", "double", or "thick"
border_style = "rounded"

# Enable transparent terminal background
transparent = false

# Use Unicode nerd-font/unicode icons (set false for plain ASCII glyphs)
icons = true

[monitoring]
# Opt-in to periodic guest memory balloon collection via virsh dommemstat
guest_memory_stats = false

# Balloon polling interval in seconds
period_secs = 5

[confirm]
# Prompt confirmation modal before running destructive commands (destroy, undefine, delete)
destructive = true

# Prompt confirmation modal before running standard lifecycle commands (start, shutdown, reboot)
lifecycle = false
```

---

## Interactive Configuration

Settings can be inspected and updated in two ways without editing files manually:

### 1. Settings View (`,`)
Pressing `,` opens the interactive settings view:
- Categorized into Appearance, Monitoring, Confirmations, and Keybindings.
- Press `Tab` / `Shift+Tab` to switch focus between the categories list and the options panel.
- Press `Enter` or `Space` to toggle booleans or cycle options.
- Changes are written to `config.toml` immediately.

### 2. Command Line (`:set key=value`)
Options can be modified directly from the command line:
```text
:set theme=dracula
:set refresh_interval_secs=2
:set confirm.destructive=false
```

---

## Keymap Overrides (`keymap.toml`)

To rebind keys, define binding objects in `keymap.toml`:

```toml
[[bindings]]
key = "Tab"
action = "cycle_focus"

[[bindings]]
key = "Ctrl-w"
action = "new_domain_wizard"

[[bindings]]
key = "Space k"
action = "shutdown"
```

Bindings in `keymap.toml` take precedence over default bindings. Remapped keys automatically update in the Help overlay (`?`), the Command Palette (`Ctrl-p`), and the Which-Key popup.

