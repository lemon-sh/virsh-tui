# Backend Execution & Libvirt Integration

This document describes how virsh-tui interacts with the hypervisor, executes commands, parses libvirt output, handles remote connections, and provides an offline demonstration mode.

---

## Architecture Principle: CLI Subprocess Integration

virsh-tui interacts with libvirt exclusively by invoking the official `virsh` (and `virt-install`) command-line utilities via asynchronous subprocesses, rather than linking directly against the C `libvirt` library.

### Advantages of CLI Subprocess Architecture
1. **Security & ABI Independence**: Avoids complex C-FFI bindings and memory-safety concerns in user space.
2. **Identical Privilege Model**: Uses the exact authentication, Polkit rules, and SSH key configurations already established for the user's `virsh` CLI.
3. **Auditability & Scriptability**: Every action performed by virsh-tui is converted into a concrete command line that can be previewed, copied, or re-run directly in a shell.
4. **Resilience**: An unexpected error or crash in a libvirt driver does not crash the TUI process.

---

## Subprocess Execution Layer (`backend::virsh::exec`)

All command execution passes through `backend::virsh::exec`:

- **Argument Lists**: Commands are executed directly with vectors of arguments (`Command::new("virsh").args(...)`), never through a shell (`sh -c`). This guarantees immunity from shell injection.
- **Process Isolation (`setsid`)**: Background polling commands are executed in a new session (`setsid()` on Unix via `tokio::process::Command::pre_exec`). This disconnects the child from `/dev/tty` so background tasks cannot disrupt raw terminal input.
- **Timeouts**: Long-running operations (such as volume cloning, disk allocation, or migrations) are given extended timeouts (up to 24 hours), while standard operational queries time out after 30 seconds.
- **Polkit Awareness**: PolicyKit permission denials (e.g. non-root access to `qemu:///system`) are intercepted and reformatted into actionable guidance:
  ```text
  error: failed to connect to the hypervisor (not in the libvirt group?)
  ```

---

## Parsers

Libvirt CLI output is parsed into strongly typed domain models in `src/backend/virsh/`:

### 1. Domain & Resource Lists (`parse_list.rs`)
- `virsh list --all`: Parses domain ID, Name, and State (`running`, `shut off`, `paused`, `crashed`, etc.).
- `virsh net-list --all`: Parses virtual network names, active states, and autostart configurations.
- `virsh pool-list --all`: Parses storage pools, states, and autostart flags.
- `virsh vol-list --pool <P>`: Parses volume names and physical disk paths.
- `virsh snapshot-list <domain> --parent`: Parses snapshot tree relations (Name, Creation Time, State, Parent).

### 2. Live Telemetry (`parse_domstats.rs`)
- `virsh domstats --raw`: Emits key-value metrics for all running domains in a single invocation.
  - `cpu.time`: Nanoseconds of total vCPU time.
  - `balloon.current`: Current memory allocated to guest (KiB).
  - `balloon.maximum`: Maximum memory limit (KiB).
  - `block.<n>.rd.bytes`, `block.<n>.wr.bytes`: Disk read/write throughput.
  - `net.<n>.rx.bytes`, `net.<n>.tx.bytes`: Network receive/transmit throughput.
- These raw values are fed into `metrics::store::MetricsStore` to calculate moving rates and historical time-series.

### 3. Host Node Statistics (`parse_nodestats.rs`)
- `virsh nodecpustats --percent`: Total, user, system, and idle CPU utilization percentages.
- `virsh nodememstats`: Total, free, and available physical memory in KiB.
- `virsh nodeinfo`: CPU model, active core counts, NUMA nodes, and memory size.

### 4. Network Leases (`parse_leases.rs`)
- `virsh net-dhcp-leases <network>`: Parses active DHCP leases: expiry timestamp, MAC address, IP address with prefix, client hostname, and client ID.

### 5. XML Parsing with roxmltree (`parse_xml.rs`)
- Inspects inactive and active XML configurations using fast, read-only parsing (`roxmltree`):
  - Domain architecture, machine type, boot devices, and vCPU topology.
  - Attached disk targets (`vda`, `sda`), pool sources, format types (`qcow2`, `raw`), and backing stores.
  - Network interfaces (`bridge`, `network`, `macvtap`) and associated MAC addresses.
  - Network XML IP ranges, DHCP reservations, and bridge names.

### 6. Event Stream (`events.rs`)
- Spawns `virsh event --loop` as a streaming subprocess.
- Asynchronously parses lifecycle events (`Started`, `Stopped`, `Suspended`, `Resumed`), storage pool changes, and virtual network reconfigurations.
- Maps internal libvirt codes to user-readable event logs displayed in the Events view.

---

## Local vs. Remote Hypervisor Handling

virsh-tui supports remote hypervisors out of the box (e.g. `qemu+ssh://user@remote/system`):

| Feature | Local (`qemu:///system`, `qemu:///session`) | Remote (`qemu+ssh://...`, `qemu+tcp://...`) |
|---------|---------------------------------------------|---------------------------------------------|
| **URI Detection** | `backend::virsh::is_local_uri(uri)` returns `true` | `is_local_uri(uri)` returns `false` |
| **Host CPU & Memory** | Read directly from `/proc/stat` and `/proc/meminfo` | Polled via `virsh nodecpustats --percent` and `virsh nodememstats` |
| **Per-Thread Meters** | Real-time `/proc/stat` per-thread bars | Hidden gracefully (`remote host` indicator) |
| **Domain Uptime** | Read from QEMU PID creation time in `/run/libvirt/qemu/` | Estimated from session observation (`≥ 5m`, `≥ 1h 20m`) |
| **Host Network Traffic**| Read from `/sys/class/net/<bridge>/statistics/` | Derived from guest vNIC aggregations |
| **Hostname** | Queried via `hostname` crate or `virsh hostname` | Queried dynamically via `virsh hostname` |

### Dynamic Connection Switching (`:connect <uri>`)
Users can switch hypervisors at runtime without restarting virsh-tui using the `:connect <uri>` command:
1. Validates connection parameters.
2. Updates hypervisor URI across all views (Dashboard, Host, Networks, Storage, Events).
3. Terminates the active background poller and event stream.
4. Spawns new polling tasks against the target URI and displays the updated hostname in the top bar.

---

## Offline Demo Mode (`backend::demo`)

To enable local evaluation, UI verification, and safe testing without requiring a running libvirt daemon or root privileges, virsh-tui includes a built-in demo backend:

- Activated via `virsh-tui --demo`.
- Provides mock domain datasets (`arch-dev`, `win11-gaming`, `ubuntu-server`, etc.), network topologies, storage pools, and simulated real-time metrics.
- **Safety Barrier**: In demo mode, `command::exec::set_demo(true)` is activated globally. Any attempt to spawn an external mutating process is blocked and logged as an error, ensuring offline safety.

