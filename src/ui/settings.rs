// src/ui/settings.rs

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Margin},
    style::{Modifier, Style},
    widgets::{Block, Borders, Paragraph, Wrap, Table, Row, Cell, TableState},
    Frame,
    text::{Span, Spans, Text},
};
use crate::app::state::App;
use crate::ui::keyboard::Keyboard;
use crate::audio;
use crate::themes_presets;
use std::cmp;

/// Draws the Settings screen, listing each boolean toggle.
pub fn draw_settings<B: Backend>(f: &mut Frame<B>, app: &App, _keyboard: &Keyboard) {
    // Title + content split
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());

    let title = Paragraph::new("⚙ Settings")
        .block(Block::default().borders(Borders::ALL).title(Span::styled("⚙ Settings", Style::default().fg(app.theme.title_accent.to_tui_color()).add_modifier(Modifier::BOLD))))
        .alignment(tui::layout::Alignment::Center)
        .style(Style::default().bg(app.theme.background.to_tui_color()).fg(app.theme.foreground.to_tui_color()));
    f.render_widget(title, outer[0]);

    // Build a list of logical settings lines (strings) so we can slice them for paging
    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("Show mode panel: {}", if app.show_mode { "On" } else { "Off" }));
    lines.push(format!("Show value panel: {}", if app.show_value { "On" } else { "Off" }));
    lines.push(format!("Show state panel: {}", if app.show_state { "On" } else { "Off" }));
    lines.push(format!("Show WPM/speed: {}", if app.show_speed { "On" } else { "Off" }));
    lines.push(format!("Show timer: {}", if app.show_timer { "On" } else { "Off" }));
    lines.push(format!("Show text area: {}", if app.show_text { "On" } else { "Off" }));
    lines.push(format!("Show on-screen keyboard: {}", if app.show_keyboard { "On" } else { "Off" }));

    let layout_label = match app.keyboard_layout {
        crate::app::state::KeyboardLayout::Qwerty => "QWERTY",
        crate::app::state::KeyboardLayout::Azerty => "AZERTY",
        crate::app::state::KeyboardLayout::Dvorak => "DVORAK",
        crate::app::state::KeyboardLayout::Qwertz => "QWERTZ",
    };
    lines.push(format!("Keyboard layout: <<{}>>", layout_label));

    // Theme
    let theme_names = themes_presets::preset_names();
    let mut cur_theme_name = "Custom".to_string();
    for &n in theme_names.iter() {
        if let Some(p) = crate::themes_presets::theme_by_name(n) {
            if p == app.theme {
                cur_theme_name = n.to_string();
                break;
            }
        }
    }
    lines.push(format!("Theme: <<{}>>", cur_theme_name));

    // Audio
    // Keyboard switch selection (audio sample set used for key sounds)
    lines.push(format!("Keyboard switch: <<{}>>", app.keyboard_switch));

    // Audio
    lines.push(format!("Audio enabled: {}  (press 'a' to toggle)", if app.audio_enabled { "On" } else { "Off" }));

    // Determine paging: compute available rows inside the Settings block (account for borders)
    let area = outer[1];
    // subtract 2 for borders/title (approx); ensure at least 1 row
    let avail = if area.height > 2 { (area.height - 2) as usize } else { 1usize };
    let total = lines.len();

    // Clamp app.settings_cursor to valid range (draw will clamp if it's larger)
    let mut cursor = app.settings_cursor;
    if total > 0 {
        cursor = cmp::min(cursor, total - 1);
    } else {
        cursor = 0;
    }

    // Compute offset so the selected item is visible and near the bottom when possible
    let max_offset = if total > avail { total - avail } else { 0 };
    let mut offset = if cursor < avail { 0 } else { cursor.saturating_sub(avail - 1) };
    offset = cmp::min(offset, max_offset);
    let selected_idx = cursor.saturating_sub(offset);

    // Build visible table rows for the current page so we can highlight the
    // entire row width (Table highlight fills the row background) rather
    // than only the text span.
    let end = cmp::min(offset + avail, total);
    let mut rows: Vec<Row> = Vec::with_capacity(end - offset);
    for s in lines[offset..end].iter() {
        rows.push(Row::new(vec![Cell::from(s.clone())]));
    }

    // Use a stateful Table so the highlight_style covers the full row area.
    let mut state = TableState::default();
    state.select(Some(selected_idx));

    let table = Table::new(rows)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.border.to_tui_color()))
                .style(Style::default().bg(app.theme.background.to_tui_color()).fg(app.theme.foreground.to_tui_color()))
                .title(Span::styled("Settings", Style::default().fg(app.theme.title.to_tui_color())))
        )
        .widths(&[Constraint::Percentage(100)])
        .column_spacing(0)
        .highlight_style(
            Style::default()
                .bg(app.theme.highlight.to_tui_color())
                .fg(app.theme.background.to_tui_color()),
        )
        .highlight_symbol(" ");

    f.render_stateful_widget(table, outer[1], &mut state);
}
