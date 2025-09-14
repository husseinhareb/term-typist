// src/ui/settings.rs

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Margin},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
    text::{Span, Spans, Text},
};
use crate::app::state::App;
use crate::ui::keyboard::Keyboard;
use crate::audio;
use crate::themes_presets;

/// Draws the Settings screen, listing each boolean toggle.
pub fn draw_settings<B: Backend>(f: &mut Frame<B>, app: &App, _keyboard: &Keyboard) {
    // Top-level split: title (3 rows) and content
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.size());

    // Title
    let title = Paragraph::new("⚙ Settings")
        .block(Block::default().borders(Borders::ALL).title(Span::styled("⚙ Settings", Style::default().fg(app.theme.title_accent.to_tui_color()).add_modifier(Modifier::BOLD))))
        .alignment(tui::layout::Alignment::Center)
        .style(Style::default().fg(app.theme.title.to_tui_color()));
    f.render_widget(title, outer[0]);

    // Two-column layout for settings: toggles left, switches right
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)].as_ref())
        .split(outer[1]);

    // Build left column items (toggles) as aligned Spans so status boxes align
    let left_w = (cols[0].width as usize).saturating_sub(4); // account for borders/padding
    let mut lines: Vec<Spans> = Vec::new();

    let mut add_toggle = |label: &str, enabled: bool| {
        let status = if enabled { "[x]" } else { "[ ]" };
        // compute padding to align status to right
        let pad = if left_w > label.len() + status.len() {
            left_w - label.len() - status.len()
        } else { 1 };
        let style_on = Style::default().fg(app.theme.stats_value.to_tui_color()).add_modifier(Modifier::BOLD);
        let style_off = Style::default().fg(app.theme.stats_label.to_tui_color());
        lines.push(Spans::from(vec![
            Span::raw(label.to_string()),
            Span::raw(" ".repeat(pad)),
            if enabled { Span::styled(status.to_string(), style_on) } else { Span::styled(status.to_string(), style_off) },
        ]));
    };

    add_toggle("Show mode panel", app.show_mode);
    add_toggle("Show value panel", app.show_value);
    add_toggle("Show state panel", app.show_state);
    add_toggle("Show WPM/speed", app.show_speed);
    add_toggle("Show timer", app.show_timer);
    add_toggle("Show text area", app.show_text);
    add_toggle("Show on-screen keyboard", app.show_keyboard);

    // layout/display-only rows for layout and switch
    let layout_label = match app.keyboard_layout {
        crate::app::state::KeyboardLayout::Qwerty => "Keyboard layout: QWERTY",
        crate::app::state::KeyboardLayout::Azerty => "Keyboard layout: AZERTY",
        crate::app::state::KeyboardLayout::Dvorak => "Keyboard layout: Dvorak",
        crate::app::state::KeyboardLayout::Qwertz => "Keyboard layout: QWERTZ",
    };
    lines.push(Spans::from(vec![Span::raw(layout_label.to_string())]));
    lines.push(Spans::from(vec![Span::raw(format!("Keyboard switch: {}", app.keyboard_switch))]));

    // Theme preset display
    let theme_names = themes_presets::preset_names();
    let mut theme_lines: Vec<Spans> = Vec::new();
    theme_lines.push(Spans::from(Span::raw("Theme presets:")));
    for name in theme_names.iter() {
        if let Some(picked) = crate::themes_presets::theme_by_name(name) {
            // compare by converting to_tui_color of title as a cheap uniqueness test
            let selected = app.theme.title.to_tui_color() == picked.title.to_tui_color();
            let marker = if selected { "→ " } else { "  " };
            theme_lines.push(Spans::from(Span::raw(format!("{}{}", marker, name))));
        } else {
            theme_lines.push(Spans::from(Span::raw(format!("  {}", name))));
        }
    }
    // Render theme presets below other left column lines
    for tl in theme_lines.into_iter() {
        lines.push(tl);
    }

    // audio toggle (right-aligned like other toggles)
    let audio_label = "Audio enabled (press 'a' to toggle)";
    let audio_status = if app.audio_enabled { "[x]" } else { "[ ]" };
    let pad_audio = if left_w > audio_label.len() + audio_status.len() { left_w - audio_label.len() - audio_status.len() } else { 1 };
    lines.push(Spans::from(vec![
        Span::raw(audio_label.to_string()),
        Span::raw(" ".repeat(pad_audio)),
        if app.audio_enabled { Span::styled(audio_status.to_string(), Style::default().fg(app.theme.stats_value.to_tui_color()).add_modifier(Modifier::BOLD)) }
        else { Span::styled(audio_status.to_string(), Style::default().fg(app.theme.stats_label.to_tui_color())) },
    ]));

    let left_para = Paragraph::new(Text::from(lines))
        .block(Block::default()
            .borders(Borders::ALL)
            .title(Span::styled("Toggles", Style::default().fg(app.theme.title_accent.to_tui_color()).add_modifier(Modifier::BOLD)))
            .border_style(Style::default().fg(app.theme.border.to_tui_color()))
        )
    .style(Style::default().fg(app.theme.foreground.to_tui_color()))
        .wrap(Wrap { trim: true });

    f.render_widget(left_para, cols[0]);

    // Right column: available keyboard switches
    let switches = audio::list_switches();
    let mut switch_items: Vec<ListItem> = switches
        .iter()
        .map(|s| {
            // style each item with the theme's label color; the active one will be highlighted by the list
            ListItem::new(Span::styled(s.clone(), Style::default().fg(app.theme.stats_label.to_tui_color())))
        })
        .collect();

    if switch_items.is_empty() {
        switch_items.push(ListItem::new("(no switches found in assets)"));
    }

    // Mark current switch with a subtle marker
    let switch_list = List::new(switch_items)
        .block(Block::default().borders(Borders::ALL).title(Span::styled("Keyboard switches (press 'k' to cycle)", Style::default().fg(app.theme.title_accent.to_tui_color()).add_modifier(Modifier::BOLD))).border_style(Style::default().fg(app.theme.border.to_tui_color())))
        .highlight_style(Style::default().fg(app.theme.stats_value.to_tui_color()).add_modifier(Modifier::BOLD))
        .highlight_symbol("● ");
    f.render_widget(switch_list, cols[1]);

    // Footer/help at bottom of right column
    let help = Paragraph::new("Keys: l = cycle layout, k = cycle switch, a = toggle audio, Esc = back")
        .block(Block::default().borders(Borders::TOP).border_style(Style::default().fg(app.theme.border.to_tui_color())))
    .style(Style::default().fg(app.theme.info.to_tui_color()))
        .wrap(Wrap { trim: true });
    // render help with a small margin inside the right column
    f.render_widget(help, cols[1].inner(&Margin { vertical: 1, horizontal: 1 }));

    // Hint: theme cycling
    let theme_hint = Paragraph::new("When in Settings: press 't' to cycle themes instantly")
        .block(Block::default().borders(Borders::NONE))
        .style(Style::default().fg(app.theme.info.to_tui_color()));
    f.render_widget(theme_hint, cols[0].inner(&Margin { vertical: 1, horizontal: 1 }));
}
