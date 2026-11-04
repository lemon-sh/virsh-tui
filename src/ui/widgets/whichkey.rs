//! whichkey widget (PLAN.md section 7).
//! WhichKey popup (PLAN.md 7.16).

use std::collections::HashMap;

use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph},
};
use unicode_width::UnicodeWidthStr;

use crate::input::engine::KeyAction;
use crate::theme::Theme;

pub const TRIGGER_DELAY_MS: u64 = 300;

/// Return key and description pairs for a given pending prefix from a live keymap.
pub fn entries_for_map(map: &HashMap<Vec<String>, KeyAction>, prefix: &str) -> Vec<(String, String)> {
    let prefix_tokens: Vec<&str> = prefix.split_whitespace().collect();
    if prefix_tokens.is_empty() {
        return Vec::new();
    }

    // Collect all candidate tokens immediately following prefix_tokens
    let mut next_tokens: Vec<String> = Vec::new();
    for seq in map.keys() {
        if seq.len() > prefix_tokens.len()
            && seq[..prefix_tokens.len()]
                .iter()
                .zip(prefix_tokens.iter())
                .all(|(a, b)| a == *b)
        {
            let tok = seq[prefix_tokens.len()].clone();
            if !next_tokens.contains(&tok) {
                next_tokens.push(tok);
            }
        }
    }

    if next_tokens.is_empty() {
        return vec![("…".to_string(), "…".to_string())];
    }

    // Rank next_tokens using crate::input::keymap::DEFAULT_BINDINGS order so that
    // standard layout (n, c, r, M, s …, m …, etc.) is preserved.
    next_tokens.sort_by_key(|tok| {
        let pos = crate::input::keymap::DEFAULT_BINDINGS
            .iter()
            .position(|(keys, _)| {
                keys.len() > prefix_tokens.len()
                    && keys[..prefix_tokens.len()] == prefix_tokens[..]
                    && keys[prefix_tokens.len()] == tok.as_str()
            })
            .unwrap_or(usize::MAX);
        (pos, tok.clone())
    });

    let mut result = Vec::new();
    for tok in next_tokens {
        // Check if all bindings with this tok are leaf bindings or sub-prefixes
        let mut sub_seqs: Vec<(&Vec<String>, &KeyAction)> = map
            .iter()
            .filter(|(k, _)| {
                k.len() > prefix_tokens.len()
                    && k[..prefix_tokens.len()]
                        .iter()
                        .zip(prefix_tokens.iter())
                        .all(|(a, b)| a == *b)
                    && k[prefix_tokens.len()] == tok
            })
            .collect();

        sub_seqs.sort_by_key(|(k, _)| k.len());

        let is_subgroup = sub_seqs.iter().any(|(k, _)| k.len() > prefix_tokens.len() + 1);

        if is_subgroup {
            let mut full_sub = prefix_tokens.iter().map(|s| s.to_string()).collect::<Vec<_>>();
            full_sub.push(tok.clone());
            let group_name = crate::input::keymap::prefix_group_description(&full_sub);
            result.push((format!("{tok} …"), group_name.to_string()));
        } else if let Some((_, action)) = sub_seqs.first() {
            result.push((tok, action.description().to_string()));
        }
    }

    result
}

/// Return key and description pairs for a given pending prefix using the default keymap.
pub fn entries_for(prefix: &str) -> Vec<(String, String)> {
    entries_for_map(&crate::input::keymap::default_map(), prefix)
}

/// Render the which-key popup bottom-right for a pending prefix and live keymap.
pub fn render(f: &mut Frame, theme: &Theme, prefix: &str, map: &HashMap<Vec<String>, KeyAction>) {
    let area = f.area();
    if area.width < 10 || area.height < 5 {
        return;
    }
    let t = &theme.tokens;
    let rows = entries_for_map(map, prefix);
    let lines: Vec<Line> = rows
        .iter()
        .map(|(k, d)| {
            Line::from(vec![
                Span::styled(k.as_str(), theme.key()),
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
        .map(|(k, d)| UnicodeWidthStr::width(k.as_str()) + 2 + UnicodeWidthStr::width(d.as_str()))
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
