//! Keybindings overlay (PLAN.md section 9.8, P12): generated from the live keymap.

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::theme::Theme;

use crate::input::engine::KeyAction as A;

/// Live keymap: key sequence -> action.
pub type Keymap = std::collections::HashMap<Vec<String>, A>;

/// One section: title, section colour, rows (key, description, destructive).
#[derive(Debug, Clone)]
pub struct Section {
    pub title: String,
    pub color: &'static str,
    pub rows: Vec<(String, String, bool)>,
}

fn leader_title(map: &Keymap) -> String {
    // Dynamic leader title e.g. "Leader  ␣" based on actual leader token bound to NewDomain
    if let Some((seq, _)) = map.iter().find(|(_, a)| **a == A::NewDomain)
        && seq.len() >= 2
    {
        let leader_sym = match seq[0].as_str() {
            "Space" => "␣",
            "Enter" => "⏎",
            other => other,
        };
        return format!("Leader  {leader_sym}");
    }
    String::from("Leader  ␣")
}

/// Keys bound to `actions` in the live map, `a / b` joined; `None` if the
/// row has no actions (static rows) or nothing is bound.
fn live_label(map: &Keymap, actions: &[A]) -> Option<String> {
    if actions.is_empty() {
        return None;
    }
    // Check if the actions represent the 5 view switching actions
    if actions.len() == 5
        && actions
            == [
                A::ViewDomains,
                A::ViewHost,
                A::ViewNetworks,
                A::ViewStorage,
                A::ViewEvents,
            ]
    {
        let seqs: Vec<Option<String>> = actions
            .iter()
            .map(|a| crate::input::keymap::action_key_label(map, *a))
            .collect();
        if seqs
            == [
                Some("1".into()),
                Some("2".into()),
                Some("3".into()),
                Some("4".into()),
                Some("5".into()),
            ]
        {
            return Some("1 … 5".to_string());
        }
    }

    let parts: Vec<String> = actions
        .iter()
        .map(|a| {
            let labels = crate::input::keymap::action_key_labels(map, *a);
            if actions.len() == 1 && labels.len() > 1 {
                labels.join(" / ")
            } else {
                labels.into_iter().next().unwrap_or_else(|| String::from("—"))
            }
        })
        .collect();

    // `␣sc / ␣sr / ␣sd` → `␣s c/r/d` when only the last key differs.
    if parts.len() > 2 {
        let prefix: String = parts[0]
            .chars()
            .take(parts[0].chars().count().saturating_sub(1))
            .collect();
        if !prefix.is_empty()
            && parts
                .iter()
                .all(|p| p.starts_with(&prefix) && p.chars().count() == prefix.chars().count() + 1)
        {
            let lasts: Vec<String> = parts
                .iter()
                .map(|p| p.chars().last().unwrap_or(' ').to_string())
                .collect();
            return Some(format!("{prefix} {}", lasts.join("/")));
        }
    }
    Some(parts.join(" / "))
}

fn sec(map: &Keymap, title: &str, color: &'static str, rows: &[(&str, &str, bool, &[A])]) -> Section {
    Section {
        title: title.to_string(),
        color,
        rows: rows
            .iter()
            .map(|(k, d, c, acts)| {
                (
                    live_label(map, acts).unwrap_or_else(|| k.to_string()),
                    d.to_string(),
                    *c,
                )
            })
            .collect(),
    }
}

/// Sections in mockup order, with keys taken from the live keymap (so
/// `keymap.toml` remaps show up).
pub fn sections(map: &Keymap) -> Vec<Section> {
    vec![
        sec(
            map,
            "Navigation",
            "blue",
            &[
                ("j / k", "down / up", false, &[A::MoveDown, A::MoveUp]),
                ("gg / G", "top / bottom", false, &[A::GoTop, A::GoBottom]),
                ("C-d / C-u", "half page", false, &[A::HalfDown, A::HalfUp]),
                ("Tab", "cycle focus", false, &[A::CycleFocus]),
                ("H / L", "prev / next tab", false, &[A::PrevTab, A::NextTab]),
                (
                    "1 … 5",
                    "switch view",
                    false,
                    &[
                        A::ViewDomains,
                        A::ViewHost,
                        A::ViewNetworks,
                        A::ViewStorage,
                        A::ViewEvents,
                    ],
                ),
                ("⏎", "open", false, &[A::Open]),
                ("⌫ / q", "back", false, &[A::Back]),
                ("/", "filter", false, &[A::Filter]),
                ("n / N", "next / prev match", false, &[A::NextMatch, A::PrevMatch]),
            ],
        ),
        sec(
            map,
            "Modes",
            "green",
            &[
                ("i / a", "edit field (INSERT)", false, &[]),
                ("⎋", "back to NORMAL", false, &[]),
                ("V", "visual select", false, &[A::Visual]),
                ("m", "mark / unmark", false, &[A::Mark]),
                (":", "ex command", false, &[A::Ex]),
                (": ⇥", "complete in : line", false, &[]),
                ("C-p", "command palette", false, &[A::Palette]),
            ],
        ),
        sec(
            map,
            "Domain",
            "magenta",
            &[
                ("s", "start", false, &[A::Start]),
                ("S", "shutdown (ACPI)", false, &[A::Shutdown]),
                ("D", "destroy (force off)", true, &[A::Destroy]),
                ("r", "reboot", false, &[A::Reboot]),
                ("R", "reset", true, &[A::Reset]),
                ("p", "pause / resume", false, &[A::Pause]),
                ("Z", "suspend to disk", false, &[A::ManagedSave]),
                ("a", "toggle autostart", false, &[A::Autostart]),
                ("c", "serial console", false, &[A::Console]),
                ("v", "graphical viewer", false, &[A::Viewer]),
                ("e", "edit XML ($EDITOR)", false, &[A::EditXml]),
                ("X", "undefine", true, &[A::Undefine]),
            ],
        ),
        sec(
            map,
            "Yank",
            "cyan",
            &[
                ("yy", "name", false, &[A::YankName]),
                ("yu", "uuid", false, &[A::YankUuid]),
                ("yi", "ip address", false, &[A::YankIp]),
                ("yc", "last virsh command", false, &[A::YankCmd]),
            ],
        ),
        sec(
            map,
            &leader_title(map),
            "orange",
            &[
                ("␣n", "new domain", false, &[A::NewDomain]),
                ("␣c", "clone", false, &[A::Clone]),
                ("␣r", "rename", false, &[A::Rename]),
                ("␣M", "migrate", false, &[A::Migrate]),
                (
                    "␣s c/r/d",
                    "snapshot new/revert/del",
                    false,
                    &[A::SnapshotNew, A::SnapshotRevert, A::SnapshotDelete],
                ),
                (
                    "␣mi / ␣me",
                    "insert / eject ISO",
                    false,
                    &[A::InsertMedia, A::EjectMedia],
                ),
                (
                    "␣da / ␣dr",
                    "disk attach / resize",
                    false,
                    &[A::DiskAttach, A::DiskResize],
                ),
                (
                    "␣ia / ␣il",
                    "NIC attach / link",
                    false,
                    &[A::NicAttach, A::NicLink],
                ),
                ("␣x", "export XML", false, &[A::ExportXml]),
                ("␣b", "boot order", false, &[A::BootOrder]),
            ],
        ),
        sec(
            map,
            "Monitor",
            "teal",
            &[
                ("t", "60s · 5m · 1h", false, &[A::WindowCycle]),
                ("o", "cycle sort", false, &[A::SortCycle]),
                (
                    "f / F",
                    "cycle filter chips",
                    false,
                    &[A::ChipCycle, A::ChipCycleBack],
                ),
            ],
        ),
        sec(
            map,
            "Ex commands",
            "yellow",
            &[
                (":start", "arch-dev", false, &[]),
                (":setmem", "arch-dev 8G --live", false, &[]),
                (":attach-iso", "arch-dev ~/iso/…", false, &[]),
                (":connect", "qemu+ssh://h/system", false, &[]),
                (":theme", "tokyo-night", false, &[]),
                (":sort", "cpu | mem | name", false, &[]),
                (":w / :q", "apply / close", false, &[]),
                (":!virsh", "raw passthrough", false, &[]),
            ],
        ),
        sec(
            map,
            "App",
            "blue",
            &[
                ("?", "this help", false, &[A::Help]),
                (",", "settings", false, &[A::Settings]),
                ("C-r", "force refresh", false, &[A::Refresh]),
                ("C-l", "redraw", false, &[A::Redraw]),
                ("ZZ / :qa", "quit", false, &[]),
            ],
        ),
    ]
}

/// Help overlay state (fzf-style filter with single-line text input and scrolling).
#[derive(Debug, Clone, Default)]
pub struct HelpState {
    pub input: crate::input::textinput::TextInput,
    pub scroll: usize,
}

impl HelpState {
    pub fn new(filter: &str) -> Self {
        Self {
            input: crate::input::textinput::TextInput::with(filter),
            scroll: 0,
        }
    }

    pub fn query(&self) -> &str {
        self.input.value()
    }
}

/// Render the help modal (single-column categorized list with fzf-style text input at bottom).
pub fn render(frame: &mut Frame, theme: &Theme, state: &HelpState, map: &Keymap) {
    let t = &theme.tokens;
    let area = frame.area();
    let w = 82u16.min(area.width.saturating_sub(4));
    let h = 34u16.min(area.height.saturating_sub(4));
    let modal = Rect::new(
        area.x + (area.width - w) / 2,
        area.y + (area.height - h) / 2,
        w,
        h,
    );
    frame.render_widget(Clear, modal);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(theme.border)
        .border_style(Style::default().fg(t.blue).bg(t.bg))
        .style(theme.base())
        .title(Span::styled(" ? Keybindings ", theme.panel_title(true)))
        .title_top(Line::from(Span::styled(" ⎋ close ", theme.dim())).right_aligned());
    frame.render_widget(block, modal);

    let inner = Rect::new(
        modal.x + 1,
        modal.y + 1,
        modal.width.saturating_sub(2),
        modal.height.saturating_sub(2),
    );
    if inner.width < 4 || inner.height < 3 {
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    let query = state.query().trim();
    let sections = sections(map);
    let mut matcher = nucleo_matcher::Matcher::new(nucleo_matcher::Config::DEFAULT);

    let mut total_rows = 0;
    let mut matching_rows = 0;
    let mut lines: Vec<Line> = Vec::new();

    let mut first_sec = true;
    for sec in &sections {
        let mut matched_in_sec: Vec<&(String, String, bool)> = Vec::new();
        for row in &sec.rows {
            total_rows += 1;
            let (k, d, _) = row;
            let haystack = format!("{} {} {}", sec.title, k, d);
            let matches = if query.is_empty() || haystack.to_lowercase().contains(&query.to_lowercase()) {
                true
            } else {
                let mut haystack_buf = Vec::new();
                let mut needle_buf = Vec::new();
                let h_utf = nucleo_matcher::Utf32Str::new(&haystack, &mut haystack_buf);
                let n_utf = nucleo_matcher::Utf32Str::new(query, &mut needle_buf);
                matcher.fuzzy_match(h_utf, n_utf).is_some()
            };

            if matches {
                matching_rows += 1;
                matched_in_sec.push(row);
            }
        }

        if !matched_in_sec.is_empty() {
            if !first_sec {
                lines.push(Line::from(""));
            }
            first_sec = false;

            let color = match sec.color {
                "blue" => t.blue,
                "green" => t.green,
                "magenta" => t.magenta,
                "cyan" => t.cyan,
                "orange" => t.orange,
                "teal" => t.teal,
                "yellow" => t.yellow,
                _ => t.fg,
            };

            lines.push(Line::from(vec![Span::styled(
                format!("  {}", sec.title),
                Style::default().fg(color).bg(t.bg).add_modifier(Modifier::BOLD),
            )]));

            for (k, d, destructive) in matched_in_sec {
                let k_len = unicode_width::UnicodeWidthStr::width(k.as_str());
                let pad = " ".repeat(16usize.saturating_sub(k_len));
                lines.push(Line::from(vec![
                    Span::raw("    "),
                    Span::styled(format!("{k}{pad}"), theme.key()),
                    Span::styled(
                        d.clone(),
                        if *destructive {
                            Style::default().fg(t.red).bg(t.bg)
                        } else {
                            theme.secondary()
                        },
                    ),
                ]));
            }
        }
    }

    if lines.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "    no matching keybindings",
            theme.dim(),
        )));
    }

    // Scrollable list area
    let list_h = rows[0].height as usize;
    let max_scroll = lines.len().saturating_sub(list_h);
    let scroll = state.scroll.min(max_scroll);
    let visible_lines: Vec<Line> = lines.into_iter().skip(scroll).take(list_h).collect();
    frame.render_widget(Paragraph::new(visible_lines).style(theme.base()), rows[0]);

    // Separator with legend
    let legend = "── ● asks for confirmation ";
    let sep_len = (inner.width as usize).saturating_sub(legend.chars().count());
    let sep_line = Line::from(vec![
        Span::styled("── ", Style::default().fg(t.hl)),
        Span::styled("● asks for confirmation ", Style::default().fg(t.red)),
        Span::styled("─".repeat(sep_len), Style::default().fg(t.hl)),
    ]);
    frame.render_widget(Paragraph::new(sep_line), rows[1]);

    // Bottom single-line text input
    let text = state.input.value();
    let cur = state.input.cursor().min(text.len());
    let (before, rest) = text.split_at(cur);
    let mut rest_chars = rest.chars();
    let at_cursor = rest_chars.next().map_or(String::from(" "), String::from);
    let after: String = rest_chars.collect();

    let left = vec![
        Span::styled(
            "  ❯ ",
            Style::default().fg(t.magenta).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            before.to_string(),
            Style::default().fg(t.fg).add_modifier(Modifier::BOLD),
        ),
        Span::styled(at_cursor, Style::default().fg(t.bg).bg(t.fg)),
        Span::styled(after, Style::default().fg(t.fg).add_modifier(Modifier::BOLD)),
    ];
    let count = format!("{} / {}  ", matching_rows, total_rows);
    let right = vec![Span::styled(count, Style::default().fg(t.comment).bg(t.bg2))];
    crate::ui::chrome::render_split(frame, rows[2], left, right, Style::default().bg(t.bg2));
}
