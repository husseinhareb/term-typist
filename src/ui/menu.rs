// ui/menu.rs
use std::cmp;

use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::Paragraph,
    Frame,
};

use crate::app::state::App;

/// "term-typist" banner text lines. Colors are chosen at runtime from the
/// active theme so the banner adapts to the selected theme.
const BANNER_LINES: [&str; 6] = [
    "████████╗  ███████╗  ██████╗    ███╗   ███╗       ████████╗  ██╗   ██╗  ██████╗    ██╗  ███████╗  ████████╗",
    "╚══██╔══╝  ██╔════╝  ██╔══██╗   ████╗ ████║       ╚══██╔══╝  ╚██╗ ██╔╝  ██╔══██╗   ██║  ██╔════╝  ╚══██╔══╝",
    " ██║     █████╗    ██████╔╝   ██╔████╔██║  ███     ██║      ╚████╔╝   ██████╔╝   ██║  ███████╗     ██║   ",
    " ██║     ██╔══╝    ██╔══██╗   ██║╚██╔╝██║          ██║       ╚██╔╝    ██╔═══╝    ██║  ╚════██║     ██║   ",
    " ██║     ███████╗  ██║  ██║   ██║ ╚═╝ ██║          ██║        ██║     ██║        ██║  ███████║     ██║   ",
    " ╚═╝     ╚══════╝  ╚═╝  ╚═╝   ╚═╝     ╚═╝          ╚═╝        ╚═╝     ╚═╝        ╚═╝  ╚══════╝     ╚═╝   ",
];



pub fn draw_menu<B: Backend>(f: &mut Frame<B>, app: &App, _split_band: Option<Rect>) {
    let area = f.size();

    // Menu content
    let labels = ["Settings", "Help", "Quit"];
    let title = "Menu";
    let hint_txt = "Enter: select   Esc: close   ↑/↓: navigate";

    // Compute content metrics
    let banner_width = BANNER_LINES
        .iter()
        .map(|s| s.chars().count() as u16)
        .max()
        .unwrap_or(0);

    let longest_label = labels.iter().map(|s| s.len() as u16).max().unwrap_or(0);
    let title_w = title.len() as u16;
    let hint_w = hint_txt.len() as u16;

    // Width we need to center everything nicely
    let content_w = banner_width.max(longest_label).max(title_w).max(hint_w);
    let padding_h = 4u16; // side breathing room for nicer centering
    let w = (content_w + padding_h * 2).clamp(12, area.width);

    // Height calculation:
    // banner rows + 1 gap + title + labels + hint + top/bottom padding
    let banner_h = BANNER_LINES.len() as u16;
    let items_h = labels.len() as u16;
    let padding_v = 2u16;
    let h = (banner_h + 1 + 1 + items_h + 1 + padding_v).min(area.height);

    // Centered rect
    let x = area.width.saturating_sub(w) / 2;
    let y = area.height.saturating_sub(h) / 2;
    let inner = Rect::new(x, y, w, h);

    // Row tracker
    let mut row_y = inner.y;

    // 1) Draw banner lines (transparent: only paint non-space runs), colors are
    // derived from the current theme so the banner matches the active theme.
    // Use a single theme color for the entire banner so it appears uniform.
    let banner_color = app.theme.title_accent.to_tui_color();
    let banner_colors: [Color; 6] = [banner_color; 6];

    for (i, line) in BANNER_LINES.iter().enumerate() {
        if row_y >= inner.y.saturating_add(inner.height) {
            break;
        }
        let color = banner_colors
            .get(i)
            .copied()
            .unwrap_or(app.theme.title.to_tui_color());
        render_transparent_line(
            f,
            inner,
            row_y,
            line.trim_end(),
            color,
            Alignment::Center,
        );
        row_y = row_y.saturating_add(1);
    }

    // 2) Gap row
    if row_y < inner.y.saturating_add(inner.height) {
        row_y = row_y.saturating_add(1);
    }

    // 3) Title
    if row_y < inner.y.saturating_add(inner.height) {
        let tw = title.len() as u16;
        let x_off = center_offset(inner.width, tw);
        let title_rect = Rect::new(inner.x.saturating_add(x_off), row_y, tw.min(inner.width), 1);
        let title_para = Paragraph::new(title)
            .style(
                Style::default()
                    .fg(app.theme.title.to_tui_color())
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Left);
        f.render_widget(title_para, title_rect);
        row_y = row_y.saturating_add(1);
    }

    // 4) Items
    let selected = normalize_index(app.menu_cursor, labels.len());
    for (i, &label) in labels.iter().enumerate() {
        if row_y >= inner.y.saturating_add(inner.height).saturating_sub(2) {
            break;
        }
        let lw = label.len() as u16;
        let x_off = center_offset(inner.width, lw);
        let rect = Rect::new(inner.x.saturating_add(x_off), row_y, lw.min(inner.width), 1);

        let style = if i == selected {
            Style::default()
                .fg(app.theme.title_accent.to_tui_color())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(app.theme.foreground.to_tui_color())
        };

        let para = Paragraph::new(label).style(style).alignment(Alignment::Left);
        f.render_widget(para, rect);
        row_y = row_y.saturating_add(1);
    }

    // 5) Hint (anchored to the very bottom of `inner`)
    let hint_row = inner
        .y
        .saturating_add(inner.height)
        .saturating_sub(1); // single-line hint
    let hw = hint_txt.len() as u16;
    let x_off = center_offset(inner.width, hw);
    if hint_row < inner.y.saturating_add(inner.height) && inner.height > 0 {
        let rect = Rect::new(
            inner.x.saturating_add(x_off),
            hint_row,
            hw.min(inner.width),
            1,
        );
        let para = Paragraph::new(hint_txt)
            .style(Style::default().fg(app.theme.stats_label.to_tui_color()))
            .alignment(Alignment::Left);
        f.render_widget(para, rect);
    }
}

/// Render a single line as multiple contiguous non-space runs so that background
/// remains transparent (only glyphs are drawn).
fn render_transparent_line<B: Backend>(
    f: &mut Frame<B>,
    bounds: Rect,
    y: u16,
    line: &str,
    color: Color,
    align: Alignment,
) {
    if y >= bounds.y.saturating_add(bounds.height) || bounds.height == 0 {
        return;
    }

    let chars: Vec<char> = line.chars().collect();
    let line_w = chars.len() as u16;
    if line_w == 0 {
        return;
    }

    // Horizontal alignment -> compute starting x for the full line, then each run is offset within it
    let x_off = match align {
        Alignment::Left => 0,
        Alignment::Center => center_offset(bounds.width, line_w),
        Alignment::Right => bounds.width.saturating_sub(line_w),
    };

    let base_x = bounds.x.saturating_add(x_off);
    let mut run_start: Option<usize> = None;

    for (i, ch) in chars.iter().enumerate() {
        if *ch != ' ' {
            if run_start.is_none() {
                run_start = Some(i);
            }
        } else if let Some(start) = run_start.take() {
            draw_run(f, bounds, y, &chars, start, i, base_x, color);
        }
    }
    if let Some(start) = run_start {
        draw_run(f, bounds, y, &chars, start, chars.len(), base_x, color);
    }
}

fn draw_run<B: Backend>(
    f: &mut Frame<B>,
    bounds: Rect,
    y: u16,
    chars: &[char],
    start: usize,
    end: usize,
    base_x: u16,
    color: Color,
) {
    if end <= start {
        return;
    }
    let run: String = chars[start..end].iter().collect();
    let run_w = (end - start) as u16;
    if run_w == 0 {
        return;
    }
    let rx = base_x.saturating_add(start as u16);
    if rx >= bounds.x.saturating_add(bounds.width) {
        return;
    }
    let available = bounds
        .width
        .saturating_sub(rx.saturating_sub(bounds.x))
        .min(run_w);
    if available == 0 {
        return;
    }
    let rect = Rect::new(rx, y, available, 1);
    let para = Paragraph::new(run)
        .style(Style::default().fg(color))
        .alignment(Alignment::Left);
    f.render_widget(para, rect);
}

fn center_offset(container_w: u16, content_w: u16) -> u16 {
    if container_w > content_w {
        (container_w - content_w) / 2
    } else {
        0
    }
}

fn normalize_index(cursor: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else {
        cursor % len
    }
}

fn color_from_hex(hex: &str) -> Option<Color> {
    // Accepts "#RRGGBB" or "RRGGBB"
    let s = hex.trim();
    let s = s.strip_prefix('#').unwrap_or(s);
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some(Color::Rgb(r, g, b))
}
