//! whichkey widget (PLAN.md section 7).
//! WhichKey popup (PLAN.md 7.16).

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};
use unicode_width::UnicodeWidthStr;

use crate::theme::Theme;

pub const TRIGGER_DELAY_MS: u64 = 300;

/// Return key and description pairs for a given pending prefix.
pub fn entries_for(prefix: &str) -> &'static [(&'static str, &'static str)] {
    match prefix {
        "Space" => &[
            ("n", "new domain"),
            ("c", "clone"),
            ("r", "rename"),
            ("M", "migrate"),
            ("s …", "snapshot"),
            ("m …", "media"),
            ("d …", "disk"),
            ("i …", "NIC"),
            ("b", "boot order"),
            ("x", "export XML"),
            ("l", "lease pin"),
            ("L", "lease unpin"),
        ],
        "Space s" => &[("c", "snapshot new"), ("r", "revert"), ("d", "delete")],
        "Space m" => &[("i", "insert ISO"), ("e", "eject")],
        "Space d" => &[("a", "disk attach"), ("r", "disk resize")],
        "Space i" => &[("a", "NIC attach"), ("l", "NIC link")],
        "g" => &[("g", "top"), ("x", "goto XML"), ("1-5", "views")],
        "y" => &[("y", "name"), ("u", "uuid"), ("i", "ip"), ("c", "command")],
        _ => &[("…", "…")],
    }
}

/// Render the which-key popup bottom-right for a pending prefix.
pub fn render(f: &mut Frame, theme: &Theme, prefix: &str) {
    let area = f.area();
    if area.width < 10 || area.height < 5 {
        return;
    }
    let t = &theme.tokens;
    let rows = entries_for(prefix);
    let lines: Vec<Line> = rows
        .iter()
        .map(|(k, d)| {
            Line::from(vec![
                Span::styled(*k, theme.key()),
                Span::styled(format!("  {d}"), theme.dim()),
            ])
        })
        .collect();

    let title = match prefix {
        "Space" => " Leader  ␣ ".to_string(),
        "Space s" => " Leader  ␣ s ".to_string(),
        "Space m" => " Leader  ␣ m ".to_string(),
        "Space d" => " Leader  ␣ d ".to_string(),
        "Space i" => " Leader  ␣ i ".to_string(),
        p if p.starts_with("Space ") => format!(" Leader  ␣ {} ", &p[6..]),
        _ => format!(" {prefix} "),
    };

    let max_len = rows
        .iter()
        .map(|(k, d)| UnicodeWidthStr::width(*k) + 2 + UnicodeWidthStr::width(*d))
        .max()
        .unwrap_or(20);
    let title_len = UnicodeWidthStr::width(title.as_str());
    let inner_w = (max_len.max(title_len) as u16) + 2;
    let w = (inner_w + 2).max(22).min(area.width.saturating_sub(2));
    let h = (lines.len() as u16 + 2).min(area.height.saturating_sub(2));

    let rect = Rect::new(
        area.x + area.width.saturating_sub(w + 1),
        area.y + area.height.saturating_sub(h + 2),
        w,
        h,
    );

    let title_line = Line::from(vec![Span::styled(
        title,
        Style::default().fg(t.orange).add_modifier(Modifier::BOLD),
    )]);

    let block = Block::default()
        .title(title_line)
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(t.gutter).bg(t.bg));

    f.render_widget(Clear, rect);
    f.render_widget(Paragraph::new(lines).block(block).style(theme.base()), rect);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;

    #[test]
    fn leader_entries_complete() {
        let rows = entries_for("Space");
        let keys: Vec<&str> = rows.iter().map(|(k, _)| *k).collect();
        assert!(keys.contains(&"n"));
        assert!(keys.contains(&"c"));
        assert!(keys.contains(&"r"));
        assert!(keys.contains(&"M"));
        assert!(keys.contains(&"s …"));
        assert!(keys.contains(&"m …"));
        assert!(keys.contains(&"d …"));
        assert!(keys.contains(&"i …"));
        assert!(keys.contains(&"b"));
        assert!(keys.contains(&"x"));
        assert!(keys.contains(&"l"));
        assert!(keys.contains(&"L"));
    }

    #[test]
    fn sub_prefix_entries() {
        assert_eq!(entries_for("Space s").len(), 3);
        assert_eq!(entries_for("Space m").len(), 2);
        assert_eq!(entries_for("Space d").len(), 2);
        assert_eq!(entries_for("Space i").len(), 2);
    }

    #[test]
    fn render_smoke_test() {
        let theme = crate::theme::builtin::by_name("tokyo-night");
        let backend = TestBackend::new(120, 36);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|f| {
                render(f, &theme, "Space");
            })
            .unwrap();
        let buffer = terminal.backend().buffer();
        let text = buffer
            .content()
            .iter()
            .map(|cell| cell.symbol())
            .collect::<String>();
        assert!(text.contains("Leader"));
        assert!(text.contains("new domain"));
        assert!(text.contains("clone"));
        assert!(text.contains("migrate"));
    }
}
