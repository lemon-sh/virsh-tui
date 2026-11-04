# Safety & Security Model

virsh-tui interacts with virtualization hypervisors and system resources. This document outlines the security architecture, input sanitization policies, and safety safeguards enforced across the codebase.

---

## Least Privilege & Non-Root Execution

virsh-tui adheres strictly to the principle of least privilege:

1. **No `sudo`**: The application never executes `sudo` or attempts privilege escalation internally.
2. **Standard Group Permissions**: To manage the system hypervisor (`qemu:///system`), the user simply needs standard membership in the host `libvirt` group.
3. **Rootless Operation**: Fully supports unprivileged per-user hypervisors (`qemu:///session`), allowing rootless virtual machine management without elevated system privileges.

---

## Untrusted Input Sanitization

Data originating from virtual machines, DHCP clients, or external event streams is treated as untrusted:

### 1. Control Character & Terminal Injection Prevention (`model::sanitize`)
Guest operating systems can provide arbitrary hostnames via DHCP requests or guest agent messages. If passed directly to the terminal, malicious ANSI escape sequences could compromise the host terminal or execute arbitrary commands.
- All external strings (hostnames, MAC labels, event payload strings) are filtered through `model::sanitize`.
- Removes non-printable characters, VT100/ANSI escape codes, and terminal manipulation sequences before rendering in Ratatui widgets.

### 2. XML Injection Prevention (`xml::escape`)
When constructing domain XML snippets, network configurations, or disk attachment specifications:
- Any user-supplied value (such as VM name, network bridge name, description text, or disk image path) is sanitized and escaped using `xml::escape`.
- Characters such as `<`, `>`, `&`, `"`, and `'` are properly encoded to prevent malformed XML structures or XML injection vulnerabilities.

---

## Polkit Authentication Handling

Connecting to the system hypervisor may require PolicyKit authorization:

- **Startup Prompts**: If authorization is required at startup, virsh-tui defers terminal raw mode until preflight connection checks complete. This allows PolicyKit's textual agent (`pkttyagent`) to safely prompt for password entry on standard `/dev/tty` with normal line buffering and character echoing.
- **Session TTY Detachment**: Once the TUI is active, background commands run in detached sessions (`setsid()` on Unix). Unauthorized background operations fail immediately with clean error messages rather than hanging the user interface or corrupting terminal graphics.
- **Actionable Diagnostics**: When permission is denied, virsh-tui detects Polkit error patterns and suggests practical remediation (e.g. checking whether the user is in the `libvirt` group).

---

## Destructive Mutation Guardrails

virsh-tui implements multiple layers of protection against accidental data destruction:

1. **Command Review**: Destructive actions (destroy, undefine, snapshot revert, volume delete) require confirmation in an interactive modal displaying the exact command to be run.
2. **Automatic Pre-Revert Snapshots**: Reverting a snapshot automatically takes a timestamped safety snapshot of the current state before applying the revert.
3. **Shutdown Tracking**: Rather than assuming `virsh shutdown` immediately turns off a guest, the application tracks the VM's state transition, displaying `Shutdown requested... waiting for guest...` and warning after a timeout if the guest refuses ACPI shutdown.
4. **Dry-Run Mode**: Passing `--dry-run` simulates execution without mutating any hypervisor state.
5. **Demo Safety Barrier**: In demo mode (`--demo`), `command::exec::set_demo(true)` acts as a hard switch in the command runner, immediately rejecting any attempt to spawn external mutating processes.

