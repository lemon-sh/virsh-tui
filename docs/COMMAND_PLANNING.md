# Command Planning & Mutation Safety

All modifications to libvirt resources within virsh-tui are encapsulated inside a declarative execution plan called a `CommandPlan`.

---

## The `CommandPlan` Architecture

Rather than executing ad-hoc commands when a user presses a key, virsh-tui constructs a `CommandPlan` before performing any change:

```rust
pub struct CommandPlan {
    pub display: String,              // Human-readable command description
    pub steps: Vec<CommandStep>,       // Concrete argument lists to execute
    pub destructive: bool,             // Requires confirmation modal
    pub target_domain: Option<String>, // Context domain if applicable
}
```

### Safety Guarantees
1. **Full Transparency**: The user can inspect the exact command line and flags that will be executed before confirming.
2. **Copyable & Reproducible**: Planned commands can be copied directly to the system clipboard (via `arboard`) to be saved in setup scripts or automation runbooks.
3. **Dry-Run Compatibility (`--dry-run`)**: When started with `--dry-run`, command execution is intercepted: the command is logged to history and rendered in the message line without calling the external binary.
4. **No Hidden Mutations**: virsh-tui never mutates libvirt state behind the user's back.

---

## Multi-Step Atomic Operations

Certain hypervisor modifications require multiple coordinated steps. `CommandPlan` allows executing sequential operations with rollback safety:

### 1. Snapshot Reversion Safety
Reverting a virtual machine to an earlier snapshot state discards all changes made since that snapshot was taken. To prevent accidental data loss:
- virsh-tui creates an automatic pre-revert safety snapshot before performing the revert:
  ```sh
  # Step 1: Create automatic safety snapshot with current timestamp
  virsh snapshot-create-as <domain> auto-before-revert-1438 --atomic
  # Step 2: Revert to desired target snapshot
  virsh snapshot-revert <domain> <target-snapshot>
  ```
- If the user explicitly opts out, the safety snapshot step can be skipped in the snapshot modal.

### 2. Hardware Configuration Editing
When adjusting vCPU counts, memory allocations, or boot order in the Hardware Editor:
1. The domain's inactive XML is retrieved and parsed into a mutable DOM using `xmltree`.
2. Modifications are applied in memory.
3. Both original and modified XML trees are formatted with deterministic, attribute-sorted pretty printing.
4. An XML unified diff is generated using `similar` and rendered to the user with syntax coloring (`-` red, `+` green).
5. Upon saving (`:w`), the modified XML is written to a secure temporary file and applied via:
   ```sh
   virsh define --validate <tempfile>
   ```
6. If the domain is running and supports live updates, native live commands (e.g. `setvcpus --live`) are executed following the definition.
7. If validation fails, the error message from `virsh` is captured and injected as an XML comment at the top of the file so the user can correct it without losing work.

---

## Secure Temporary Files (`command::tempxml`)

When writing domain XML, network XML, or device definitions:
- Files are created using `tempfile::NamedTempFile`.
- **Permissions**: Created with strict `0600` permissions (read/write by owner only) to prevent unauthorized reading of sensitive VM credentials or disk paths.
- **Randomization**: Uses unpredictable temporary filenames prefixed with `vt-` and sanitized target identifiers.
- **Automatic Deletion**: The temporary file implements `Drop`, ensuring immediate deletion upon plan completion or error abort.

---

## Confirmation Policies & Customization

virsh-tui classifies operations into standard and destructive actions:

- **Destructive Actions**: Destroying a running VM (`D`), undefining a domain (`u`), deleting a storage volume, deleting a virtual network, or deleting a snapshot.
- **Confirmation Modal**: Destructive operations present an interactive modal displaying:
  - Warning label.
  - Target object name.
  - The exact `virsh` command line to be executed.
  - `y` to confirm, `n` or `Esc` to cancel.
- **Policy Overrides**: Users can adjust confirmation behavior in Settings (`,`):
  - `confirm.destructive = true` (default: prompt for destructive actions).
  - `confirm.lifecycle = false` (allow instant starts, shutdowns, and reboots without prompt).

