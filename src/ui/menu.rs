use tui::{backend::Backend, widgets::{Block, Borders, Paragraph, List, ListItem, ListState}, layout::{Alignment, Constraint, Direction, Layout, Rect}, style::{Style, Modifier, Color}, text::{Span, Spans, Text}, Frame};
use crate::app::state::App;

pub fn draw_menu<B: Backend>(f: &mut Frame<B>, app: &App) {
        let area = f.size();

        // ASCII art header (user-provided). Place it before computing the
        // popup rect so we can size the popup to fit the art exactly.
        let ascii = r#"
 ▄▄▄▄▄▄▄ ▄▄▄▄▄▄▄ ▄▄▄▄▄▄   ▄▄   ▄▄    ▄▄▄▄▄▄▄ ▄▄   ▄▄ ▄▄▄▄▄▄▄ ▄▄▄ ▄▄▄▄▄▄▄ ▄▄▄▄▄▄▄ 
█       █       █   ▄  █ █  █▄█  █  █       █  █ █  █       █   █       █       █
█▄     ▄█    ▄▄▄█  █ █ █ █       █  █▄     ▄█  █▄█  █    ▄  █   █  ▄▄▄▄▄█▄     ▄█
  █   █ █   █▄▄▄█   █▄▄█▄█       █    █   █ █       █   █▄█ █   █ █▄▄▄▄▄  █   █  
  █   █ █    ▄▄▄█    ▄▄  █       █    █   █ █▄     ▄█    ▄▄▄█   █▄▄▄▄▄  █ █   █  
  █   █ █   █▄▄▄█   █  █ █ ██▄██ █    █   █   █   █ █   █   █   █▄▄▄▄▄█ █ █   █  
  █▄▄▄█ █▄▄▄▄▄▄▄█▄▄▄█  █▄█▄█   █▄█    █▄▄▄█   █▄▄▄█ █▄▄▄█   █▄▄▄█▄▄▄▄▄▄▄█ █▄▄▄█  
"#;

        let ascii_lines: Vec<&str> = ascii.lines().collect();
        let max_line_len = ascii_lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16;

        // padding around content inside the popup (per-side)
        let padding_h = 4_u16; // left/right padding in cells
        let padding_v = 2_u16; // top/bottom padding in cells

        // minimal area for list portion (title + items)
        let list_area_height = 6_u16;

        // compute width/height to exactly fit ASCII + list area, but clamp to terminal
        // include room for borders (2) and layout margin (2) so the ascii fits
        let required_w = max_line_len + padding_h * 2 + 4; // ascii + padding + borders + margins
        let w = required_w.min(area.width);
        let title_height = ascii_lines.len() as u16;
        let h = (title_height + list_area_height + padding_v * 2).min(area.height);

        let x = (area.width.saturating_sub(w)) / 2;
        let y = (area.height.saturating_sub(h)) / 2;
        let rect = Rect::new(x, y, w, h);

        // Build styled spans for the ASCII art once
        let mut spans_vec: Vec<Spans> = Vec::new();
        for (i, line) in ascii_lines.iter().enumerate() {
            let color = if i % 2 == 0 { app.theme.title_accent.to_tui_color() } else { app.theme.title.to_tui_color() };
            spans_vec.push(Spans::from(Span::styled(line.to_string(), Style::default().fg(color))));
        }

    // Render a dim overlay only on the regions outside the centered menu rect
    // so the menu area itself remains untouched and the UI underneath appears
    // "transparent" (terminals don't support true alpha blending).
    let overlay_style = Style::default().bg(Color::Indexed(236));

    // top area
    if y > 0 {
        let top = Rect::new(0, 0, area.width, y);
        let overlay = Paragraph::new("").style(overlay_style);
        f.render_widget(overlay, top);
    }

    // bottom area
    if y + h < area.height {
        let bottom = Rect::new(0, y + h, area.width, area.height - (y + h));
        let overlay = Paragraph::new("").style(overlay_style);
        f.render_widget(overlay, bottom);
    }

    // left area (between top and bottom)
    if x > 0 {
        let left = Rect::new(0, y, x, h);
        let overlay = Paragraph::new("").style(overlay_style);
        f.render_widget(overlay, left);
    }

    // right area
    if x + w < area.width {
        let right = Rect::new(x + w, y, area.width - (x + w), h);
        let overlay = Paragraph::new("").style(overlay_style);
        f.render_widget(overlay, right);
    }

    // Do not fill the centered rect; leave it transparent so the underlying
    // UI shows through. Draw only a border around the popup area below.

    let items = vec![
        ListItem::new("Settings"),
        ListItem::new("Help"),
        ListItem::new("Quit"),
    ];

    let mut state = ListState::default();
    state.select(Some(app.menu_cursor.min(items.len().saturating_sub(1))));

    // Use a bordered block without a background; the overlay will dim the
    // surroundings while the interior remains visible.
    let list_block = Block::default().borders(Borders::ALL);

    let list = List::new(items)
        .block(list_block)
        .highlight_style(Style::default().fg(app.theme.highlight.to_tui_color()).add_modifier(Modifier::BOLD));

    // place the list inside the rect with a small margin but do not clear

    let title_height = spans_vec.len() as u16;
    let chunks = Layout::default().direction(Direction::Vertical).constraints([
        Constraint::Length(title_height),
        Constraint::Min(0)
    ].as_ref()).margin(1).split(rect);

    let title = Paragraph::new(Text::from(spans_vec)).alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);
    f.render_stateful_widget(list, chunks[1], &mut state);
}
