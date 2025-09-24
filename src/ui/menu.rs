// src/ui/menu.rs

use tui::{backend::Backend, widgets::{Block, Borders, Paragraph, List, ListItem, ListState}, layout::{Alignment, Constraint, Direction, Layout, Rect}, style::{Style, Modifier}, text::{Span, Spans, Text}, Frame};
use crate::app::state::App;

pub fn draw_menu<B: Backend>(f: &mut Frame<B>, app: &App) {
    let area = f.size();
    // Draw a centered box roughly 40% width, 40% height
    let w = (area.width as f32 * 0.4) as u16;
    let h = (area.height as f32 * 0.4) as u16;
    let x = (area.width.saturating_sub(w)) / 2;
    let y = (area.height.saturating_sub(h)) / 2;
    let rect = Rect::new(x, y, w, h);

    // Do not clear or fill the popup area: render only the title and list
    // so the underlying UI remains visible and the menu text appears on top.
    // We'll avoid drawing a filled block so we don't overwrite background cells.

    let items = vec![
        ListItem::new("Settings"),
        ListItem::new("Help"),
        ListItem::new("Quit"),
    ];

    let mut state = ListState::default();
    state.select(Some(app.menu_cursor.min(items.len().saturating_sub(1))));

    let list = List::new(items)
        .block(Block::default())
        .highlight_style(Style::default().fg(app.theme.foreground.to_tui_color()).bg(app.theme.highlight.to_tui_color()).add_modifier(Modifier::BOLD));

    // place the list inside the rect with a small margin but do not clear
    // Prepare ASCII art header (user-provided). Use a raw string literal so backslashes
    // and spacing are preserved exactly as the user provided, then colorize alternating lines.
    let ascii = r#"  __                                   __                 .__          __   
_/  |_  ___________  _____           _/  |_ ___.__.______ |__| _______/  |_ 
\   __\/ __ \_  __ \/     \   ______ \   __<   |  |\____ \|  |/  ___/\   __\
 |  | \  ___/|  | \/  Y Y  \ /_____/  |  |  \___  ||  |_> >  |\___ \  |  |  
 |__|  \___  >__|  |__|_|  /          |__|  / ____||   __/|__/____  > |__|  
           \/            \/                 \/     |__|           \/         "#;

    let ascii_lines: Vec<&str> = ascii.lines().collect();
    let mut spans_vec: Vec<Spans> = Vec::new();
    for (i, line) in ascii_lines.iter().enumerate() {
        let color = if i % 2 == 0 { app.theme.title_accent.to_tui_color() } else { app.theme.title.to_tui_color() };
        spans_vec.push(Spans::from(Span::styled(line.to_string(), Style::default().fg(color))));
    }

    let title_height = spans_vec.len() as u16;
    let chunks = Layout::default().direction(Direction::Vertical).constraints([
        Constraint::Length(title_height),
        Constraint::Min(0)
    ].as_ref()).margin(1).split(rect);

    let title = Paragraph::new(Text::from(spans_vec)).alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);
    f.render_stateful_widget(list, chunks[1], &mut state);
}
