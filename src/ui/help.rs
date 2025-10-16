use tui::{
    backend::Backend,
    layout::{Alignment, Rect, Layout, Constraint, Direction},
    style::{Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Paragraph, Wrap, Clear, Table, Row, Cell}, // + Clear
    Frame,
};
use crate::app::state::App;

pub fn draw_help<B: Backend>(f: &mut Frame<B>, app: &App) {
    let area = f.size();
    let w = (area.width as f32 * 0.6) as u16;
    let h = (area.height as f32 * 0.6) as u16;
    let x = (area.width.saturating_sub(w)) / 2;
    let y = (area.height.saturating_sub(h)) / 2;
    let rect = Rect::new(x, y, w, h);

    // 1) Clear the popup region so nothing below shows through
    f.render_widget(Clear, rect);

    // 2) Draw the popup box (borders only) and paint background
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(app.theme.border.to_tui_color()))
        .style(Style::default().bg(app.theme.background.to_tui_color()));
    f.render_widget(block.clone(), rect);

    // inner area where we'll place header, table and footer
    let inner = block.inner(rect);

    // split inner into header (3), body (remaining - 4), footer (1)
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(4),
            Constraint::Length(1),
        ])
        .split(inner);

    // Header: centered title with accent color
    let title = Paragraph::new(Span::styled(
        "Help — Keybindings",
        Style::default()
            .fg(app.theme.title_accent.to_tui_color())
            .add_modifier(Modifier::BOLD),
    ))
    .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    // Build content (keys + descriptions)
    let bindings = vec![
        ("F1", "Open menu"),
        ("Tab", "Toggle menu (open/close)"),
        ("m", "Open menu"),
        ("Ctrl-C", "Quit application"),
        ("1 / 2 / 3", "Select mode tab (Time / Words / Zen)"),
        ("Left / Right", "Change selected value / setting"),
        ("Enter", "Start test / Activate menu item"),
        ("Esc", "Close popups or restart test"),
        ("Space", "Type a space (during test)"),
        ("Backspace", "Delete previous character"),
        ("p", "Open Profile view"),
        ("s", "Open Settings view"),
        ("t", "Cycle theme presets"),
        ("l", "Cycle keyboard layout"),
        ("a", "Toggle audio on/off"),
        ("k / j", "Navigate up/down (vim keys)"),
        ("Up / Down", "Navigate up/down in lists"),
        ("PageUp / PageDown", "Page navigation in lists"),
        ("Home / End", "Jump to start/end in lists"),
        ("! @ # $ % ^ &", "Toggle small UI panels (shift+1..7)"),
    ];

    // Prepare table rows
    let mut table_rows: Vec<Row> = Vec::with_capacity(bindings.len());
    for (key, desc) in bindings {
        let key_cell = Cell::from(Span::styled(
            key.to_string(),
            Style::default()
                .fg(app.theme.title_accent.to_tui_color())
                .add_modifier(Modifier::BOLD),
        ));
        let desc_cell = Cell::from(Span::styled(
            desc.to_string(),
            Style::default().fg(app.theme.foreground.to_tui_color()),
        ));
        table_rows.push(Row::new(vec![key_cell, desc_cell]));
    }

    let table = Table::new(table_rows)
        .block(Block::default())
        .widths(&[Constraint::Length(18), Constraint::Min(10)])
        .column_spacing(2)
        .style(Style::default().fg(app.theme.foreground.to_tui_color()));

    f.render_widget(table, chunks[1]);

    // Footer: small centered hint using muted theme color
    let footer = Paragraph::new(Span::styled(
        "Press Esc to close",
        Style::default()
            .fg(app.theme.stats_label.to_tui_color())
            .add_modifier(Modifier::ITALIC),
    ))
    .alignment(Alignment::Center);
    f.render_widget(footer, chunks[2]);
}
