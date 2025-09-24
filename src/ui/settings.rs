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
        // Keep a bordered area for the title but do not render the label
        // inside the block border itself (user requested removing it).
        .block(Block::default().borders(Borders::ALL))
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

    // (Removed extra placeholder convenience settings per user request.)

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

    // Build visible rows and render each as its own bordered Block to create
    // a rectangular card for every setting. This improves readability when
    // there are only a few settings on the page.
    let end = cmp::min(offset + avail, total);
    let visible = &lines[offset..end];

    // Create vertical layout with one chunk per visible line.
    // Compute integer division so heights sum to <= area.height and then
    // distribute any leftover rows across the top-most chunks. This avoids
    // the last row taking all remaining space when using naive division.
    let mut cons: Vec<Constraint> = Vec::new();
    if visible.len() > 0 {
        // Use the same available height we used for paging (subtract 2 for
        // borders/title) so the constraints match the visible slice size.
        let total_h = if area.height > 2 { (area.height - 2) as usize } else { 1usize };
        let n = visible.len();
        // Use floor division so every row gets the same height. This may
        // leave a small unused gap at the bottom, but guarantees identical
        // box heights for all items which is the user's requirement.
        let base = std::cmp::max(1, total_h / n);
        for _ in 0..n {
            cons.push(Constraint::Length(base as u16));
        }
    }
    // Fallback: ensure at least one constraint so Layout::split doesn't panic
    if cons.is_empty() {
        cons.push(Constraint::Min(1));
    }

    let rows_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(cons.as_slice())
        .split(outer[1]);

    for (i, (s, ra)) in visible.iter().zip(rows_area.iter()).enumerate() {
        let idx = offset + i;
        let selected = idx == cursor;
        // Each setting is rendered inside a bordered block. We don't set a
        // title text here because the setting's content is displayed inside
        // the block; highlight is expressed via the border color only.
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(
                if selected {
                    Style::default().fg(app.theme.highlight.to_tui_color())
                } else {
                    Style::default().fg(app.theme.border.to_tui_color())
                }
            )
            .style(Style::default().bg(app.theme.background.to_tui_color()).fg(app.theme.foreground.to_tui_color()));

        // Render block, then render the text inside it as a Paragraph so it
        // wraps correctly if long.
        let inner = block.inner(*ra);
        f.render_widget(block, *ra);
        // Keep the text styling constant; only the block border indicates
        // selection. This avoids inverting/highlighting the text itself.
        let para = Paragraph::new(s.clone())
            .style(Style::default().bg(app.theme.background.to_tui_color()).fg(app.theme.foreground.to_tui_color()))
            .wrap(Wrap { trim: true });
        f.render_widget(para, inner);
    }
}
