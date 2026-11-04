//! Process spawning: LC_ALL=C, explicit URI, timeouts, stderr capture.

use std::sync::atomic::{AtomicBool, Ordering};

use color_eyre::{Result, eyre::eyre};
use tokio::process::Command;

static DETACH_TTY: AtomicBool = AtomicBool::new(false);

/// Set whether child virsh processes should be detached from controlling TTY (setsid).
/// Active while the TUI is running so background virsh calls cannot trigger interactive
/// PolicyKit prompts on /dev/tty.
pub fn set_detach_tty(detach: bool) {
    DETACH_TTY.store(detach, Ordering::SeqCst);
}

/// Check if virsh processes should be detached from controlling TTY.
pub fn is_detach_tty() -> bool {
    DETACH_TTY.load(Ordering::SeqCst)
}

#[cfg(unix)]
unsafe extern "C" {
    fn setsid() -> i32;
}

/// Configure the async command so it does not attach to a controlling terminal
/// (preventing PolicyKit's pkttyagent from hijacking /dev/tty while the TUI is active).
pub fn detach_controlling_tty(cmd: &mut Command) {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.as_std_mut().pre_exec(|| {
                setsid();
                Ok(())
            });
        }
    }
    #[cfg(not(unix))]
    {
        let _ = cmd;
    }
}

/// Configure the synchronous command so it does not attach to a controlling terminal.
pub fn detach_controlling_tty_sync(cmd: &mut std::process::Command) {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        unsafe {
            cmd.pre_exec(|| {
                setsid();
                Ok(())
            });
        }
    }
    #[cfg(not(unix))]
    {
        let _ = cmd;
    }
}

/// Check if the `virsh` binary is available and executable in PATH.
pub fn is_virsh_available() -> bool {
    std::process::Command::new("virsh")
        .arg("-v")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

/// Verify that `virsh` is available in PATH, returning an actionable error if missing.
pub fn ensure_virsh_available() -> Result<()> {
    if is_virsh_available() {
        Ok(())
    } else {
        Err(eyre!(
            "'virsh' not found in PATH. Please install libvirt client tools (virsh) to use virsh-tui, or run with --demo for demonstration mode."
        ))
    }
}

/// How long a virsh subcommand may run. Long jobs (migration, cloning, wiping,
/// snapshots with memory, uploads) get hours; everything else 30 s.
pub fn timeout_for(program: &str, sub: &str) -> std::time::Duration {
    const LONG: &[&str] = &[
        "migrate",
        "vol-wipe",
        "vol-upload",
        "vol-download",
        "vol-clone",
        "vol-create-from",
        "blockcopy",
        "blockcommit",
        "blockpull",
        "snapshot-create-as",
        "snapshot-create",
        "snapshot-revert",
        "snapshot-delete",
        "managedsave",
        "save",
        "restore",
        "dump",
        "pool-build",
        "pool-delete",
    ];
    if program != "virsh" || LONG.contains(&sub) {
        std::time::Duration::from_secs(24 * 3600)
    } else {
        std::time::Duration::from_secs(30)
    }
}

/// Run `virsh -c <uri> <args>` and return stdout.
pub async fn run(uri: &str, args: &[&str]) -> Result<String> {
    if crate::command::exec::is_demo() {
        return Err(eyre!("demo mode: virsh is not executed"));
    }
    let mut cmd = Command::new("virsh");
    cmd.arg("-c").arg(uri).args(args);
    cmd.env("LC_ALL", "C");
    cmd.stdin(std::process::Stdio::null());
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    // A timed-out command must not keep running unattended.
    cmd.kill_on_drop(true);
    if is_detach_tty() {
        detach_controlling_tty(&mut cmd);
    }
    let limit = timeout_for("virsh", args.first().copied().unwrap_or(""));
    let output = tokio::time::timeout(limit, cmd.output())
        .await
        .map_err(|_| eyre!("virsh timed out"))?
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                eyre!("virsh not found in PATH")
            } else {
                eyre!("failed to spawn virsh: {e}")
            }
        })?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(eyre!("virsh failed: {}", friendly_first_line(&output.stderr)))
    }
}

/// First stderr line plus actionable hints (permissions, daemon down).
pub fn friendly_first_line(stderr: &[u8]) -> String {
    let first = String::from_utf8_lossy(stderr)
        .lines()
        .next()
        .unwrap_or("unknown error")
        .to_string();
    let lower = first.to_lowercase();
    if lower.contains("permission denied")
        || lower.contains("not authorized")
        || lower.contains("authentication")
        || lower.contains("polkit")
        || lower.contains("access denied")
    {
        format!("{first} (not in the libvirt group?)")
    } else if lower.contains("failed to connect") || lower.contains("no connection") {
        format!("{first} (is libvirtd running?)")
    } else {
        first
    }
}

/// Blocking variant for startup/cache paths (spawns `virsh` synchronously).
pub fn run_blocking(uri: &str, args: &[&str]) -> Result<String> {
    if crate::command::exec::is_demo() {
        return Err(eyre!("demo mode: virsh is not executed"));
    }
    let mut cmd = std::process::Command::new("virsh");
    cmd.arg("-c")
        .arg(uri)
        .args(args)
        .env("LC_ALL", "C")
        .stdin(std::process::Stdio::null());
    if is_detach_tty() {
        detach_controlling_tty_sync(&mut cmd);
    }
    let output = cmd.output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(eyre!("virsh failed"))
    }
}
