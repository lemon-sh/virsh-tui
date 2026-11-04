//! Default keymap (DESIGN.md section 6) plus TOML overrides.

use super::engine::KeyAction;
use std::collections::HashMap;

/// Single source of truth for all default key bindings.
pub const DEFAULT_BINDINGS: &[(&[&str], KeyAction)] = &[
    // Navigation.
    (&["j"], KeyAction::MoveDown),
    (&["k"], KeyAction::MoveUp),
    (&["g", "g"], KeyAction::GoTop),
    (&["G"], KeyAction::GoBottom),
    (&["C-d"], KeyAction::HalfDown),
    (&["C-u"], KeyAction::HalfUp),
    (&["Enter"], KeyAction::Open),
    (&["Backspace"], KeyAction::Back),
    (&["q"], KeyAction::Back),
    (&["/"], KeyAction::Filter),
    (&["n"], KeyAction::NextMatch),
    (&["N"], KeyAction::PrevMatch),
    // Domain lifecycle.
    (&["s"], KeyAction::Start),
    (&["S"], KeyAction::Shutdown),
    (&["D"], KeyAction::Destroy),
    (&["r"], KeyAction::Reboot),
    (&["R"], KeyAction::Reset),
    (&["p"], KeyAction::Pause),
    (&["Z"], KeyAction::ManagedSave),
    (&["a"], KeyAction::Autostart),
    (&["c"], KeyAction::Console),
    (&["v"], KeyAction::Viewer),
    (&["e"], KeyAction::EditXml),
    (&["X"], KeyAction::Undefine),
    // Yank.
    (&["y", "y"], KeyAction::YankName),
    (&["y", "u"], KeyAction::YankUuid),
    (&["y", "i"], KeyAction::YankIp),
    (&["y", "c"], KeyAction::YankCmd),
    // Leader.
    (&["Space", "n"], KeyAction::NewDomain),
    (&["Space", "c"], KeyAction::Clone),
    (&["Space", "r"], KeyAction::Rename),
    (&["Space", "M"], KeyAction::Migrate),
    (&["Space", "s", "c"], KeyAction::SnapshotNew),
    (&["Space", "s", "r"], KeyAction::SnapshotRevert),
    (&["Space", "s", "d"], KeyAction::SnapshotDelete),
    (&["Space", "m", "i"], KeyAction::InsertMedia),
    (&["Space", "m", "e"], KeyAction::EjectMedia),
    (&["Space", "d", "a"], KeyAction::DiskAttach),
    (&["Space", "d", "r"], KeyAction::DiskResize),
    (&["Space", "i", "a"], KeyAction::NicAttach),
    (&["Space", "i", "l"], KeyAction::NicLink),
    (&["Space", "b"], KeyAction::BootOrder),
    (&["Space", "x"], KeyAction::ExportXml),
    (&["Space", "l"], KeyAction::LeasePin),
    (&["Space", "L"], KeyAction::LeaseUnpin),
    // Monitor / views.
    (&["t"], KeyAction::WindowCycle),
    (&["o"], KeyAction::SortCycle),
    (&["m"], KeyAction::Mark),
    (&["V"], KeyAction::Visual),
    (&["Tab"], KeyAction::CycleFocus),
    (&["S-Tab"], KeyAction::CycleFocus),
    (&["h"], KeyAction::PrevTab),
    (&["l"], KeyAction::NextTab),
    (&["H"], KeyAction::PrevTab),
    (&["L"], KeyAction::NextTab),
    (&["["], KeyAction::PrevTab),
    (&["]"], KeyAction::NextTab),
    (&["u"], KeyAction::UndoField),
    (&["g", "x"], KeyAction::GotoXml),
    // View tabs: instant 1-5 view switches.
    (&["1"], KeyAction::ViewDomains),
    (&["2"], KeyAction::ViewHost),
    (&["3"], KeyAction::ViewNetworks),
    (&["4"], KeyAction::ViewStorage),
    (&["5"], KeyAction::ViewEvents),
    // Modes.
    (&[":"], KeyAction::Ex),
    (&["C-p"], KeyAction::Palette),
    (&["?"], KeyAction::Help),
    (&[","], KeyAction::Settings),
    (&["C-r"], KeyAction::Refresh),
    (&["C-l"], KeyAction::Redraw),
];

/// Multi-key prefix groups with their human-readable labels.
pub const PREFIX_GROUPS: &[(&[&str], &str)] = &[
    (&["Space", "s"], "snapshot"),
    (&["Space", "m"], "media"),
    (&["Space", "d"], "disk"),
    (&["Space", "i"], "NIC"),
];

/// Human-readable label for a multi-key group prefix.
pub fn prefix_group_description(prefix: &[String]) -> &'static str {
    for &(group, desc) in PREFIX_GROUPS {
        if prefix.len() == group.len() && prefix.iter().zip(group.iter()).all(|(a, b)| a == *b) {
            return desc;
        }
    }
    match prefix.last().map(|s| s.as_str()) {
        Some("s") => "snapshot",
        Some("m") => "media",
        Some("d") => "disk",
        Some("i") => "NIC",
        _ => "…",
    }
}

/// Format key sequence tokens into display string (`["Space", "n"]` -> `"␣n"`).
pub fn format_seq(seq: &[String]) -> String {
    seq.iter()
        .map(|t| match t.as_str() {
            "Space" => "␣",
            "Enter" => "⏎",
            "Backspace" => "⌫",
            "Esc" => "⎋",
            other => other,
        })
        .collect()
}

/// Primary sequence display string for an action in a map (e.g. `␣n`).
pub fn action_key_label(map: &HashMap<Vec<String>, KeyAction>, action: KeyAction) -> Option<String> {
    let seqs = action_key_labels(map, action);
    seqs.into_iter().next()
}

/// All sequence display strings for an action in a map (e.g. `["⌫", "q"]`).
pub fn action_key_labels(map: &HashMap<Vec<String>, KeyAction>, action: KeyAction) -> Vec<String> {
    let mut seqs: Vec<(usize, String)> = map
        .iter()
        .filter(|(_, a)| **a == action)
        .map(|(k, _)| {
            let pos = DEFAULT_BINDINGS
                .iter()
                .position(|(keys, a)| {
                    *a == action && keys.len() == k.len() && keys.iter().zip(k.iter()).all(|(x, y)| x == y)
                })
                .unwrap_or(usize::MAX);
            (pos, format_seq(k))
        })
        .collect();
    seqs.sort_by_key(|(pos, s)| (*pos, s.chars().count(), s.clone()));
    seqs.into_iter().map(|(_, s)| s).collect()
}

fn seq(keys: &[&str]) -> Vec<String> {
    keys.iter().map(|s| s.to_string()).collect()
}

/// Default bindings: sequence -> action.
pub fn default_map() -> HashMap<Vec<String>, KeyAction> {
    let mut m: HashMap<Vec<String>, KeyAction> = HashMap::new();
    for &(keys, a) in DEFAULT_BINDINGS {
        m.insert(seq(keys), a);
    }
    // Confirm modal keys (y/n) are handled by the modal itself, not the global map,
    // so that yank sequences (yy/yu/yi/yc) keep working.
    // Quit keys (q/ZZ/C-c) are handled at the app level for the same reason:
    // Z is managed-save and ZZ is quit.
    m
}

/// Apply `keymap.toml` overrides on top of the defaults.
///
/// ```toml
/// [normal]                       # any [normal] or [normal.<view>] table
/// shutdown = ["S", "Space q"]    # key sequences, space separated tokens
/// snapshot_new = ["Space s n"]
/// ```
///
/// An overridden action loses its default keys. Unknown actions and
/// unparsable files are reported in the returned warnings.
pub fn apply_overrides(
    mut map: HashMap<Vec<String>, KeyAction>,
    toml_text: &str,
) -> (HashMap<Vec<String>, KeyAction>, Vec<String>) {
    let mut warnings = Vec::new();
    let value: toml::Value = match toml::from_str(toml_text) {
        Ok(v) => v,
        Err(e) => {
            warnings.push(format!("keymap.toml: {e}"));
            return (map, warnings);
        }
    };
    let mut tables: Vec<&toml::value::Table> = Vec::new();
    if let Some(normal) = value.get("normal").and_then(|v| v.as_table()) {
        tables.push(normal);
        for v in normal.values() {
            if let Some(t) = v.as_table() {
                tables.push(t);
            }
        }
    }
    for table in tables {
        for (name, keys) in table {
            let Some(keys) = keys.as_array() else {
                continue;
            };
            let Some(action) = KeyAction::from_name(name) else {
                warnings.push(format!("keymap.toml: unknown action `{name}`"));
                continue;
            };
            map.retain(|_, a| *a != action);
            for k in keys.iter().filter_map(|k| k.as_str()) {
                let seq: Vec<String> = k
                    .split_whitespace()
                    .map(|t| super::key::KeySeq::parse(t).token().to_string())
                    .collect();
                if !seq.is_empty() {
                    map.insert(seq, action);
                }
            }
        }
    }
    (map, warnings)
}

/// Load `~/.config/virsh-tui/keymap.toml` (if any) over the defaults.
pub fn load() -> (HashMap<Vec<String>, KeyAction>, Vec<String>) {
    let path =
        directories::ProjectDirs::from("", "", "virsh-tui").map(|d| d.config_dir().join("keymap.toml"));
    match path.and_then(|p| std::fs::read_to_string(p).ok()) {
        Some(text) => apply_overrides(default_map(), &text),
        None => (default_map(), Vec::new()),
    }
}
