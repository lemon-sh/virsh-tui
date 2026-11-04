//! Private temporary XML files for `virsh define`/`attach-device`/`net-define`.
//!
//! Files are created with `O_EXCL` and mode 0600 under `$XDG_RUNTIME_DIR`
//! (per-user, 0700) when available, else the system temp dir. Names are random,
//! so other local users cannot pre-create or swap them (symlink/TOCTOU attacks),
//! and the file is removed when the returned handle is dropped.

use color_eyre::Result;
use std::io::Write;
use tempfile::NamedTempFile;

/// Write `xml` to a new private temp file. Keep the handle alive while the
/// command that reads it runs.
pub fn write(prefix: &str, xml: &str) -> Result<NamedTempFile> {
    let dir = std::env::var_os("XDG_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .filter(|p| p.is_dir())
        .unwrap_or_else(std::env::temp_dir);
    let safe_prefix: String = prefix
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let mut file = tempfile::Builder::new()
        .prefix(&format!("vt-{safe_prefix}-"))
        .suffix(".xml")
        .tempfile_in(dir)?;
    file.write_all(xml.as_bytes())?;
    file.flush()?;
    Ok(file)
}

/// Path of a temp file as an argv string.
pub fn path_arg(file: &NamedTempFile) -> String {
    file.path().to_string_lossy().into_owned()
}
