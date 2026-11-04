# Input Handling & Keymap System

This document explains the input architecture, keystroke processing engine, unified keymap definition, dynamic UI synchronization, and command-line execution.

---

## Single Source of Truth (`input::keymap`)

All default key bindings in virsh-tui are defined in a single, authoritative data structure: `input::keymap::DEFAULT_BINDINGS`.

```rust
// Defined in src/input/keymap.rs
pub static DEFAULT_BINDINGS: LazyLock<Vec<BindingDef>> = ...;
```

### Architectural Benefits
1. **No Out-of-Sync Help or Hints**: The Key Engine, Which-Key Popup, Help Overlay, and Command Palette derive their shortcuts directly from `DEFAULT_BINDINGS`.
2. **Dynamic Adaptation to User Remapping**: When a user overrides a binding in `~/.config/virsh-tui/keymap.toml`, every hint in the UI automatically updates to reflect the new key.
3. **Multi-Key Grouping**: Related keys (e.g. `s`, `S`, `D` for lifecycle or `␣s c`, `␣s r`, `␣s d` for snapshots) are automatically grouped in the help overlay and which-key popup.

---

## Key Processing Engine (`input::engine::KeyEngine`)

Keystrokes captured by crossterm are processed through `KeyEngine`:

```
crossterm KeyEvent
       |
       v
+-----------------------------------+
|       KeyEngine::process(...)     |
+-----------------+-----------------+
                  |
         +--------+--------+
         |                 |
  Prefix / Pending    Complete Sequence
  (e.g. "Space", "g")      |
         |                 v
         v         Resolve KeyAction
   Show Which-Key          |
       Popup               v
                   Dispatch to Active View
```

1. **Sequence Buffering**: Supports multi-key sequences (e.g. `g g` to jump to top, `Space n` for new domain).
2. **Instant View Switching**: View numbers `1` through `5` switch immediately to the corresponding view (Domains, Host, Networks, Storage, Events) without delay.
3. **Leader Key & Which-Key Popup**: Pressing the leader key (`Space`) or `g` enters a pending state. If no continuation key is pressed within 300 ms, the dynamic which-key popup renders at the bottom of the screen displaying all valid continuations.
4. **Context Precedence**: Active modal overlays (Help, Command Palette, Wizard, Confirm) intercept keystrokes before they reach view-specific handlers.

---

## Panel Focus Cycling via `Tab` / `Shift+Tab`

virsh-tui uses `Tab` and `Shift+Tab` (`BackTab`) mapped to `KeyAction::CycleFocus` to navigate between independent panels within a view:

- **Networks View (Tab 3)**: Cycles focus between the virtual networks table and the active DHCP leases table.
- **Storage View (Tab 4)**: Cycles focus between the storage pools table and the volumes table.
- **Settings View**: Cycles focus between the settings category list and the configuration options panel.

This consistent focus model avoids conflicting with vim cursor navigation (`h`/`l`/`j`/`k`).

---

## Standard Navigation & Action Keys

| Key | Action | Scope |
|-----|--------|-------|
| `j` / `k` or `Down` / `Up` | Move cursor down / up | Global lists & tables |
| `h` / `l` or `[` / `]` | Previous / next inner tab or filter | Dashboard, Detail, Events |
| `g g` / `G` | Jump to first / last item | Global lists & tables |
| `Tab` / `Shift+Tab` | Cycle focus between panels | Multi-panel views |
| `Enter` | Open selected item / detail view | Selection-dependent |
| `Backspace` / `Esc` | Return to previous view / close modal | Overlays & detail views |
| `1` … `5` | Switch to view (Domains, Host, Networks, Storage, Events) | Global |
| `/` | Open live filter prompt | Global |
| `:` | Open command-line prompt | Global |
| `Ctrl-p` | Open Command Palette | Global |
| `?` | Toggle Help overlay | Global |
| `,` | Open Settings view | Global |
| `q` | Back (in sub-views) or Quit application | Global |

### Domain Lifecycle Keys
- `s`: Start shut-off domain
- `S`: Clean ACPI shutdown (tracked until the guest powers off)
- `D`: Force destroy (immediate power cut)
- `r`: Reboot domain
- `R`: Hard reset
- `p`: Pause / resume domain execution
- `Z`: Managed save domain state
- `a`: Toggle autostart on boot
- `u`: Undefine domain configuration

### Leader Key Combinations (`Space ...`)
- `Space n`: Launch New Domain Wizard (`virt-install`)
- `Space c`: Clone domain
- `Space s c`: Create snapshot
- `Space s r`: Revert to snapshot
- `Space s d`: Delete snapshot
- `Space m i`: Insert CD-ROM / ISO image
- `Space m e`: Eject CD-ROM media
- `Space l`: Pin DHCP lease to static reservation (Networks view)

---

## Command Line (`:`) & Tab Completion

Pressing `:` enters command-line mode powered by `input::ex` and `input::complete`:

- **Built-in Commands**:
  - `:quit` or `:q`: Exit virsh-tui.
  - `:connect <uri>`: Switch hypervisor connection.
  - `:theme <name>`: Switch active color theme.
  - `:set <option>=<value>`: Change and persist configuration settings.
  - `:start <vm>`, `:shutdown <vm>`, `:destroy <vm>`, `:reboot <vm>`: Manage domains.
  - `:rename <vm> <new_name>`: Rename virtual machine.
  - `:desc <vm> <description>`: Update domain description metadata.
- **Tab Autocompletion**:
  - Pressing `Tab` completes command names, domain names, network names, theme names, file paths, and `virsh` flags.
- **Virsh Passthrough**:
  - Any command unrecognized as a virsh-tui internal command is passed directly to the `virsh` CLI tool. Output is captured and reported in the status bar.

---

## Key Remapping (`keymap.toml`)

Users can customize key bindings by creating or editing `~/.config/virsh-tui/keymap.toml`:

```toml
# Remap focus cycling to Ctrl-n / Ctrl-p
[[bindings]]
key = "Ctrl-n"
action = "cycle_focus"

# Remap wizard launch
[[bindings]]
key = "Ctrl-w"
action = "new_domain_wizard"
```

Remapped keys are validated on load and automatically propagate through the KeyEngine, the Help overlay, and the Command Palette.

