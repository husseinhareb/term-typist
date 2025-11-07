// src/ui/settings.rs

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    text::Span,
    style::Style,
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};
use crate::app::state::App;
use crate::ui::keyboard::Keyboard;
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

    let title = Paragraph::new("Settings")
        // Keep a bordered area for the title but do not render the label
        // inside the block border itself (user requested removing it).
        .block(Block::default().borders(Borders::ALL))
        .alignment(tui::layout::Alignment::Center)
        .style(Style::default().bg(app.theme.background.to_tui_color()).fg(app.theme.foreground.to_tui_color()));
    f.render_widget(title, outer[0]);

    // Build a structured list of (label, value) pairs for nicer rendering
    let mut items: Vec<(String, String)> = Vec::new();
    items.push(("Show mode panel".into(), if app.show_mode { "On".into() } else { "Off".into() }));
    items.push(("Show value panel".into(), if app.show_value { "On".into() } else { "Off".into() }));
    items.push(("Show state panel".into(), if app.show_state { "On".into() } else { "Off".into() }));
    items.push(("Show WPM/speed".into(), if app.show_speed { "On".into() } else { "Off".into() }));
    items.push(("Show timer".into(), if app.show_timer { "On".into() } else { "Off".into() }));
    items.push(("Show text area".into(), if app.show_text { "On".into() } else { "Off".into() }));
    items.push(("Show on-screen keyboard".into(), if app.show_keyboard { "On".into() } else { "Off".into() }));

    let layout_label = match app.keyboard_layout {
        crate::app::state::KeyboardLayout::Qwerty => "QWERTY",
        crate::app::state::KeyboardLayout::Azerty => "AZERTY",
        crate::app::state::KeyboardLayout::Dvorak => "DVORAK",
        crate::app::state::KeyboardLayout::Qwertz => "QWERTZ",
    };
    items.push(("Keyboard layout".into(), format!("<<{}>>", layout_label)));

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
    items.push(("Theme".into(), format!("<<{}>>", cur_theme_name)));

    // Keyboard switch (audio)
    items.push(("Keyboard switch".into(), format!("<<{}>>", app.keyboard_switch)));
    // Audio
    items.push(("Audio enabled".into(), if app.audio_enabled { "On (press 'a')".into() } else { "Off (press 'a')".into() }));

    // (Removed extra placeholder convenience settings per user request.)

    // Determine paging: compute available rows inside the Settings block.
    // Use the full available area height so constraints sum to the area height
    // and there is no leftover that gets assigned unexpectedly to the last
    // chunk. Ensure at least 1 row is considered available.
    let area = outer[1];
    let avail = if area.height > 0 { area.height as usize } else { 1usize };
    let total = items.len();

    // Clamp app.settings_cursor to valid range (draw will clamp if it's larger)
    let mut cursor = app.settings_cursor;
    if total > 0 {
        cursor = cmp::min(cursor, total - 1);
    } else {
        cursor = 0;
    }

    // Compute offset so the selected item is visible and near the bottom when possible
    let max_offset = total.saturating_sub(avail);
    let mut offset = if cursor < avail { 0 } else { cursor.saturating_sub(avail - 1) };
    offset = cmp::min(offset, max_offset);

    // Build visible rows and render each as its own bordered Block to create
    // a rectangular card for every setting. This improves readability when
    // there are only a few settings on the page.
    let end = cmp::min(offset + avail, total);
    let visible = &items[offset..end];

    // Create vertical layout with one chunk per visible line.
    // Compute integer division so heights sum to <= area.height and then
    // distribute any leftover rows across the top-most chunks. This avoids
    // the last row taking all remaining space when using naive division.
    let mut cons: Vec<Constraint> = Vec::new();
    if !visible.is_empty() {
        // Compute constraints so the sum of the chunk heights equals the
        // available area height. Distribute the integer division remainder
        // across the top-most chunks so the layout appears balanced and
        // responsive when the terminal is resized.
        let total_h = area.height as usize;
        let n = visible.len();
        if n > 0 {
            if total_h >= n {
                // Give every visible card exactly the same height (base).
                // Any leftover rows are appended as a spacer chunk so cards
                // remain equal height instead of some gaining +1 row.
                let base = total_h / n;
                let rem = total_h - (base * n);
                for _ in 0..n {
                    cons.push(Constraint::Length(base as u16));
                }
                if rem > 0 {
                    // Add a spacer chunk to absorb leftover rows.
                    cons.push(Constraint::Length(rem as u16));
                }
            } else {
                // This branch should not happen because visible.len() <=
                // total_h (we slice `visible` with avail == total_h), but
                // keep a fallback that assigns 1 row to the first
                // `total_h` elements.
                for i in 0..n {
                    let h = if i < total_h { 1 } else { 0 };
                    cons.push(Constraint::Length(h as u16));
                }
            }
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
    for (i, ((label, value), ra)) in visible.iter().zip(rows_area.iter()).enumerate() {
        let idx = offset + i;
        let selected = idx == cursor;

        // Card block with accent border when selected
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(if selected {
                Style::default().fg(app.theme.highlight.to_tui_color())
            } else {
                Style::default().fg(app.theme.border.to_tui_color())
            })
            .style(Style::default().bg(app.theme.background.to_tui_color()).fg(app.theme.foreground.to_tui_color()));

        let inner = block.inner(*ra);
        f.render_widget(block, *ra);

        // Split inner into two columns: label (60%) and value (40%)
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(inner);

        // Left: label, smaller/muted color
        let lbl = Paragraph::new(Span::styled(label.clone(), Style::default().fg(app.theme.stats_label.to_tui_color())))
            .wrap(Wrap { trim: true });
        f.render_widget(lbl, cols[0]);

        // Right: value, highlighted; add a pointer when selected
        let val_text = if selected {
            format!("▶ {}", value)
        } else {
            value.clone()
        };
        let val = Paragraph::new(Span::styled(val_text, Style::default().fg(app.theme.stats_value.to_tui_color()).add_modifier(tui::style::Modifier::BOLD)))
            .alignment(tui::layout::Alignment::Right)
            .wrap(Wrap { trim: true });
        f.render_widget(val, cols[1]);
    }
}
