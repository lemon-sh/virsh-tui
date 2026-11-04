# UI Architecture & Ratatui Layout

virsh-tui renders an interactive terminal user interface built on [Ratatui](https://ratatui.rs) and styled with configurable color themes.

---

## Screen Layout Hierarchy

The application frame is divided vertically into three regions:

```
+-----------------------------------------------------------------------------+
| Top Bar Chrome (Host · URI · Refresh Interval · Active View Tabs · Clock)   |
+-----------------------------------------------------------------------------+
|                                                                             |
|                                                                             |
|                             Active View / Panel                             |
|       (Dashboard, Host, Networks, Storage, Events, or Domain Detail)        |
|                                                                             |
|                                                                             |
+-----------------------------------------------------------------------------+
| Bottom Chrome (Status Message · Which-Key Hints · Filter / Ex Input Line)  |
+-----------------------------------------------------------------------------+
```

1. **Top Bar (`ui::chrome::render_topbar`)**:
   - Hypervisor hostname and connection URI.
   - Polling refresh indicator (e.g. `1s`).
   - Numbered view tabs (`1:Domains`, `2:Host`, `3:Networks`, `4:Storage`, `5:Events`).
   - Current local time.
2. **Body**:
   - Houses the active primary view or full-screen overlay.
   - Automatically adapts layout grids to terminal width (standard 174-column full layout vs. compact 120-column layout).
3. **Bottom Bar (`ui::chrome::render_bottombar`)**:
   - Displays execution results, error notices, Polkit warnings, or action prompts.
   - When in `:` command mode or `/` filter mode, renders the interactive text input line with cursor positioning.
   - When a multi-key leader sequence is started (`Space`, `g`), displays the dynamic which-key popup hints.

---

## Primary Views

### 1. Domains Dashboard (`ui::views::dashboard.rs`)
- **Domain List**: Displays all virtual machines with state badges (`running`, `shut off`, `paused`), active vCPUs, memory allocation, IP addresses, and autostart status.
- **Filtering & Sorting**: Real-time fuzzy filtering (`/`) and column sorting (Name, State, Memory, CPU usage).
- **Side Panel**: When terminal width allows (>= 153 columns), renders the real-time telemetry panel for the selected domain:
  - Real-time CPU Braille chart.
  - Memory and balloon gauge.
  - Disk read/write throughput rates.
  - Network receive/transmit rates.
  - Key hint grid for instant lifecycle operations (`s` start, `S` shutdown, `D` destroy, `r` reboot).

### 2. Domain Detail (`ui::views::detail/`)
Opened by pressing `Enter` on a selected domain. Provides 6 sub-tabs:
- **Overview**: Complete domain metadata, boot devices, CPU architecture, hypervisor features, attached disks, and network interfaces.
- **Monitor**: High-resolution multi-window performance graphs with 60-second, 5-minute, and 1-hour historical resolutions.
- **Hardware Editor**: Interactive configuration editor for vCPUs, memory limits, and boot devices. Shows a real-time XML diff (`-` / `+`) before applying changes with `:w`.
- **Snapshots**: Hierarchical tree visualization of domain snapshots with creation dates and parent-child relationships. Offers snapshot creation, tree revert, and deletion.
- **Console**: Textual serial console attachment (`virsh console`) or guidance for launching graphical viewers (`virt-viewer`).
- **XML View**: Formatted, syntax-highlighted domain XML specification. Pressing `e` opens the configuration in `$EDITOR` with automatic validation upon save.

### 3. Host View (`ui::views::host.rs`)
Provides hypervisor-level resource monitoring:
- Per-thread CPU utilization meters with individual core percentages.
- Physical memory usage gauge, available memory, hugepages allocation, and Kernel Samepage Merging (KSM) statistics.
- Storage pool capacity summaries and active virtual networks overview.

### 4. Networks View (`ui::views::networks.rs`)
- Virtual network list with active states, autostart configuration, and bridge device names.
- Active DHCP leases table displaying IP addresses, MAC addresses, hostnames, and owning virtual machines.
- **Focus Navigation**: Pressing `Tab` or `Shift+Tab` cycles keyboard focus between the Networks table and the DHCP Leases table.
- Lease reservation: Pressing `Space l` pins an active dynamic DHCP lease as a persistent static reservation.

### 5. Storage View (`ui::views::storage.rs`)
- Storage pools list (dir, filesystem, LVM, etc.) with capacity, allocation, and available space gauges.
- Volumes list with format types (`qcow2`, `raw`), virtual capacity, physical allocation, and backing store chains.
- **Focus Navigation**: Pressing `Tab` or `Shift+Tab` cycles keyboard focus between the Storage Pools table and the Volumes table.
- Media management: Interactive ISO picker to insert or eject virtual CD-ROM media on running or shut-off domains.

### 6. Events View (`ui::views::events.rs`)
- Real-time scrollable event log fed by the libvirt event stream.
- Captures domain lifecycle changes, device attachment/detachment, storage pool reloads, and network alterations.
- Scope filtering allows filtering by domains, networks, storage, or internal application actions.

---

## Overlays & Modals

- **Help Overlay (`ui::overlays::help.rs`)**: Opened with `?`. Displays a categorized single-column list of all available key bindings with an interactive bottom fuzzy search input (like fzf) and keyboard navigation. Dynamically reflects any custom key remappings defined in `keymap.toml`.
- **Command Palette (`ui::overlays::palette.rs`)**: Opened with `Ctrl-p`. Provides a fuzzy-searchable list of all virsh-tui actions and virsh commands using the `nucleo-matcher` engine. Displays assigned shortcut badges.
- **New Domain Wizard (`ui::overlays::wizard.rs`)**: Opened with `Space n`. A 6-step guided modal to configure and launch a new virtual machine:
  1. Operating System & Source (ISO, URL, existing disk).
  2. Memory & vCPU allocation.
  3. Storage configuration (pool selection, disk size, format).
  4. Network selection (bridge or NAT virtual network).
  5. Architecture & Machine type.
  6. Final review and `virt-install` command preview.
- **Settings Screen (`ui::views::settings.rs`)**: Opened with `,`. Allows configuring appearance, polling intervals, confirmation policies, and keymap options directly from the UI. Pressing `Tab` cycles focus between category tabs and configuration fields.
- **Confirm Modal (`ui::overlays::confirm.rs`)**: Safety dialog presented before executing destructive actions (destroy, undefine, revert snapshot). Displays the exact command line about to run.

---

## Widgets & Visualizations

- **Braille Charts (`ui::widgets::charts.rs`, `braille.rs`)**: High-density 2D plots using Unicode 8-dot Braille characters (`U+2800` .. `U+28FF`), providing 2×4 sub-cell resolution per character position for smooth graphs.
- **Sparklines (`sparkline.rs`)**: Compact inline bar graphs for historical rates.
- **Gauges & Horizontal Bars (`gauge.rs`, `hbar.rs`)**: Visual percentage indicators with configurable threshold color gradients (green -> yellow -> red).
- **Dynamic Which-Key Popup (`whichkey.rs`)**: When a prefix key (such as `Space` or `g`) is pressed, an overlay automatically appears at the bottom of the screen displaying all valid continuations and descriptions.

---

## Theme Engine & Terminal Compatibility

virsh-tui includes a color theming engine in `src/theme/`:

### Built-in Themes
Includes 9 carefully balanced themes:
- Tokyo Night (default dark)
- Tokyo Night Day (light theme)
- Catppuccin Mocha
- Nord
- Dracula
- Gruvbox Dark
- Solarized Dark
- One Dark
- Everforest Dark

### Custom Themes
Custom themes can be created in `~/.config/virsh-tui/themes/<name>.toml` and selected via `:theme <name>` or `:set theme=<name>`.

### Truecolor & 256-Color Fallback
- **Truecolor Detection**: Inspects `COLORTERM`, `WT_SESSION` (Windows Terminal), `TERM_PROGRAM`, and 24-bit terminal flags.
- **Euclidean Color Mapping**: On terminals limited to 256 colors (xterm-256), RGB colors are dynamically mapped to the nearest indexed color using Euclidean distance matching across both the 6×6×6 color cube and the 24-step grayscale ramp.
- **Transparent Terminals**: Supports transparent terminal backgrounds by properly styling frame boundaries and panel gutters with `theme.base()` rather than resetting to terminal defaults.

