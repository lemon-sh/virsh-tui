//! Keybindings overlay (PLAN.md section 9.8, P12): generated from the live keymap.

use crate::input::engine::KeyAction as A;
use crate::theme::Theme;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};

/// Live keymap: key sequence -> action.
pub type Keymap = std::collections::HashMap<Vec<String>, A>;

/// One section: title, section colour, rows (key, description, destructive).
#[derive(Debug, Clone)]
pub struct Section {
    pub title: String,
    pub color: &'static str,
    pub rows: Vec<(String, String, bool)>,
}

fn leader_title(_map: &Keymap) -> String {
    String::from("Leader")
}

/// Keys bound to `actions` in the live map, `a / b` joined; `None` if the
/// row has no actions (static rows) or nothing is bound.
fn live_label(map: &Keymap, actions: &[A]) -> Option<String> {
    if actions.is_empty() {
        return None;
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

    Some(parts.join(" / "))
}

/// Definition of a row in the help overlay.
#[derive(Debug, Clone, Copy)]
pub enum RowDef {
    /// Action-backed row: the key labels are derived 100% dynamically from `Keymap`.
    Action {
        actions: &'static [A],
        desc: &'static str,
        critical: bool,
    },
    /// Static informative row (e.g. ex commands, mode indicators) with no KeyAction binding.
    Static {
        key: &'static str,
        desc: &'static str,
        critical: bool,
    },
}

const fn act(actions: &'static [A], desc: &'static str) -> RowDef {
    RowDef::Action {
        actions,
        desc,
        critical: false,
    }
}

const fn act_crit(actions: &'static [A], desc: &'static str) -> RowDef {
    RowDef::Action {
        actions,
        desc,
        critical: true,
    }
}

const fn info(key: &'static str, desc: &'static str) -> RowDef {
    RowDef::Static {
        key,
        desc,
        critical: false,
    }
}

fn sec(map: &Keymap, title: &str, color: &'static str, rows: &[RowDef]) -> Section {
    Section {
        title: title.to_string(),
        color,
        rows: rows
            .iter()
            .map(|row| match row {
                RowDef::Action {
                    actions,
                    desc,
                    critical,
                } => (
                    live_label(map, actions).unwrap_or_else(|| String::from("—")),
                    desc.to_string(),
                    *critical,
                ),
                RowDef::Static { key, desc, critical } => (key.to_string(), desc.to_string(), *critical),
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
                act(&[A::MoveDown, A::MoveUp], "down / up"),
                act(&[A::GoTop, A::GoBottom], "top / bottom"),
                act(&[A::HalfDown, A::HalfUp], "half page"),
                act(&[A::CycleFocus], "cycle focus"),
                act(&[A::PrevTab, A::NextTab], "prev / next tab"),
                info("1 … 5", "switch view"),
                act(&[A::Open], "open"),
                act(&[A::Back], "back"),
                act(&[A::Filter], "filter"),
                act(&[A::NextMatch, A::PrevMatch], "next / prev match"),
            ],
        ),
        sec(
            map,
            "Modes",
            "green",
            &[
                act(&[A::EditField], "edit field (INSERT)"),
                info("⎋", "back to NORMAL"),
                act(&[A::Visual], "visual select"),
                act(&[A::Mark], "mark / unmark"),
                act(&[A::Ex], "ex command"),
                info(": ⇥", "complete in : line"),
                act(&[A::Palette], "command palette"),
            ],
        ),
        sec(
            map,
            "Domain",
            "magenta",
            &[
                act(&[A::Start], "start"),
                act(&[A::Shutdown], "shutdown (ACPI)"),
                act_crit(&[A::Destroy], "destroy (force off)"),
                act(&[A::Reboot], "reboot"),
                act_crit(&[A::Reset], "reset"),
                act(&[A::Pause], "pause / resume"),
                act(&[A::ManagedSave], "suspend to disk"),
                act(&[A::Autostart], "toggle autostart"),
                act(&[A::Console], "serial console"),
                act(&[A::Viewer], "graphical viewer"),
                act(&[A::EditXml], "edit XML ($EDITOR)"),
                act_crit(&[A::Undefine], "undefine"),
            ],
        ),
        sec(
            map,
            "Yank",
            "cyan",
            &[
                act(&[A::YankName], "name"),
                act(&[A::YankUuid], "uuid"),
                act(&[A::YankIp], "ip address"),
                act(&[A::YankCmd], "last virsh command"),
            ],
        ),
        sec(
            map,
            &leader_title(map),
            "orange",
            &[
                act(&[A::NewDomain], "new domain"),
                act(&[A::Clone], "clone"),
                act(&[A::Rename], "rename"),
                act(&[A::Migrate], "migrate"),
                act(
                    &[A::SnapshotNew, A::SnapshotRevert, A::SnapshotDelete],
                    "snapshot new/revert/del",
                ),
                act(&[A::InsertMedia, A::EjectMedia], "insert / eject ISO"),
                act(&[A::DiskAttach, A::DiskResize], "disk attach / resize"),
                act(&[A::NicAttach, A::NicLink], "NIC attach / link"),
                act(&[A::ExportXml], "export XML"),
                act(&[A::BootOrder], "boot order"),
            ],
        ),
        sec(
            map,
            "Monitor",
            "teal",
            &[
                act(&[A::WindowCycle], "60s · 5m · 1h"),
                act(&[A::SortCycle], "cycle sort"),
            ],
        ),
        sec(
            map,
            "Ex commands",
            "yellow",
            &[
                info(":start", "arch-dev"),
                info(":setmem", "arch-dev 8G --live"),
                info(":attach-iso", "arch-dev ~/iso/…"),
                info(":connect", "qemu+ssh://h/system"),
                info(":theme", "tokyo-night"),
                info(":sort", "cpu | mem | name"),
                info(":w / :q", "apply / close"),
                info(":!virsh", "raw passthrough"),
            ],
        ),
        sec(
            map,
            "App",
            "blue",
            &[
                act(&[A::Help], "this help"),
                act(&[A::Settings], "settings"),
                act(&[A::Refresh], "force refresh"),
                act(&[A::Redraw], "redraw"),
                info("ZZ / :qa", "quit"),
            ],
        ),
    ]
}

/// Help overlay state (fzf-style filter with single-line text input and scrolling).
#[derive(Debug, Clone, Default)]
pub struct HelpState {
    pub input: crate::input::textinput::TextInput,
    pub scroll: usize,
    pub max_scroll: std::cell::Cell<usize>,
    pub viewport_height: std::cell::Cell<usize>,
}

impl HelpState {
    pub fn new(filter: &str) -> Self {
        Self {
            input: crate::input::textinput::TextInput::with(filter),
            scroll: 0,
            max_scroll: std::cell::Cell::new(0),
            viewport_height: std::cell::Cell::new(25),
        }
    }

    pub fn query(&self) -> &str {
        self.input.value()
    }

    pub fn scroll_up(&mut self, delta: usize) {
        let max = self.max_scroll.get();
        self.scroll = self.scroll.min(max).saturating_sub(delta);
    }

    pub fn scroll_down(&mut self, delta: usize) {
        let max = self.max_scroll.get();
        self.scroll = self.scroll.min(max).saturating_add(delta).min(max);
    }

    pub fn page_step(&self) -> usize {
        (self.viewport_height.get() / 2).max(1)
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
    state.max_scroll.set(max_scroll);
    state.viewport_height.set(list_h);
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
