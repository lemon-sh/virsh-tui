//! Panel container (PLAN.md section 7.1 / DESIGN.md "Panel").
//!
//! Views use borderless [`pane`]s separated by rules; modals and popups use the
//! boxed [`block`]: rounded border (`gutter`, or `blue` when focused), bold title
//! top-left, optional right title top-right, and optional footers on the bottom
//! border. Every view builds its chrome through these so it stays identical.

use crate::theme::Theme;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Padding, Paragraph};

/// Build a panel block.
pub fn block(
    theme: &Theme,
    focused: bool,
    title: &str,
    title_right: Option<Line<'static>>,
    footer_left: Option<&str>,
    footer_right: Option<&str>,
) -> Block<'static> {
    let mut b = Block::default()
        .borders(Borders::ALL)
        .border_type(theme.border)
        .border_style(theme.panel_border(focused))
        .padding(Padding::horizontal(1))
        .style(theme.base())
        .title(Span::styled(format!(" {title} "), theme.panel_title(focused)));
    if let Some(mut rt) = title_right {
        rt.spans.insert(0, Span::styled(" ", theme.dim()));
        rt.spans.push(Span::styled(" ", theme.dim()));
        b = b.title_top(rt.style(theme.dim()).right_aligned());
    }
    if let Some(fl) = footer_left {
        b = b.title_bottom(Line::from(Span::styled(format!(" {fl} "), theme.dim())).left_aligned());
    }
    if let Some(fr) = footer_right {
        b = b.title_bottom(Line::from(Span::styled(format!(" {fr} "), theme.dim())).right_aligned());
    }
    b
}

/// Build a borderless pane header.
///
/// Panes are separated by rules instead of being boxed: the block only has a
/// top rule carrying the title (and optional right title), which doubles as
/// the separator from whatever sits above. Side-by-side panes are separated
/// by a vertical rule drawn with [`vseps`]. Focus is shown by a blue rule, a
/// blue bold title and a leading `▌` marker; unfocused panes use the gutter
/// colour and a dim title.
pub fn pane(
    theme: &Theme,
    focused: bool,
    title: &str,
    title_right: Option<Line<'static>>,
    footer_left: Option<&str>,
    footer_right: Option<&str>,
) -> Block<'static> {
    let mut b = Block::default()
        .borders(Borders::TOP)
        .border_type(theme.border)
        .border_style(theme.panel_border(focused))
        .padding(Padding::horizontal(1))
        .style(theme.base());
    let marker = if focused { "▌" } else { " " };
    b = b.title(Line::from(vec![
        Span::styled(marker, theme.panel_border(true)),
        Span::styled(format!("{title} "), theme.panel_title(focused)),
    ]));
    // No bottom rule exists, so footers share the top rule with the right title.
    let mut right_spans: Vec<Span<'static>> = Vec::new();
    if let Some(rt) = title_right {
        right_spans.extend(rt.spans.into_iter().map(|s| s.patch_style(theme.dim())));
    }
    for f in [footer_left, footer_right].into_iter().flatten() {
        if !right_spans.is_empty() {
            right_spans.push(Span::styled(" · ", theme.dim()));
        }
        right_spans.push(Span::styled(f.to_string(), theme.dim()));
    }
    if !right_spans.is_empty() {
        right_spans.insert(0, Span::styled(" ", theme.dim()));
        right_spans.push(Span::styled(" ", theme.dim()));
        b = b.title_top(Line::from(right_spans).right_aligned());
    }
    b
}

/// Draw a vertical separator in the gap column between horizontally adjacent
/// panes. `areas` are the pane rects returned by a `.spacing(1)` split; the
/// rule joins the panes' top rules with a `┬` tee.
pub fn vseps(frame: &mut ratatui::Frame, theme: &Theme, areas: &[Rect]) {
    let set = theme.border.to_border_set();
    let style = theme.panel_border(false);
    let tee = match theme.border {
        ratatui::widgets::BorderType::Double => "╦",
        ratatui::widgets::BorderType::Thick => "┳",
        _ => "┬",
    };
    for pair in areas.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let x = a.right();
        if b.x <= x || a.height == 0 {
            continue;
        }
        let buf = frame.buffer_mut();
        for y in a.y..a.bottom() {
            let sym = if y == a.y { tee } else { set.vertical_left };
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_symbol(sym).set_style(style);
            }
        }
    }
}

/// Plain right-title helper (dim text).
pub fn right(text: impl Into<String>, theme: &Theme) -> Line<'static> {
    Line::from(Span::styled(text.into(), theme.dim()))
}

/// Render a panel with a plain-text body.
#[allow(clippy::too_many_arguments)]
pub fn render(
    frame: &mut ratatui::Frame,
    area: Rect,
    theme: &Theme,
    focused: bool,
    title: &str,
    title_right: Option<&str>,
    footer_left: Option<&str>,
    footer_right: Option<&str>,
    body: &str,
) {
    let b = block(
        theme,
        focused,
        title,
        title_right.map(|t| right(t, theme)),
        footer_left,
        footer_right,
    );
    frame.render_widget(
        Paragraph::new(body.to_string()).block(b).style(theme.base()),
        area,
    );
}
