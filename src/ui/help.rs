// src/ui/help.rs

use tui::{backend::Backend, widgets::{Block, Borders, Paragraph, Wrap}, layout::{Alignment, Rect}, style::{Style, Modifier}, text::{Span, Spans, Text}, Frame};
use crate::app::state::App;

pub fn draw_help<B: Backend>(f: &mut Frame<B>, app: &App) {
    let area = f.size();
    let w = (area.width as f32 * 0.6) as u16;
    let h = (area.height as f32 * 0.6) as u16;
    let x = (area.width.saturating_sub(w)) / 2;
    let y = (area.height.saturating_sub(h)) / 2;
    let rect = Rect::new(x, y, w, h);

    let block = Block::default().borders(Borders::ALL).title("Help").style(Style::default().bg(app.theme.background.to_tui_color()).fg(app.theme.foreground.to_tui_color()));
    f.render_widget(block.clone(), rect);

    let inner = block.inner(rect);

    // Build the same bindings list we discovered in the codebase and render
    // with styled key labels (accent color) and normal text for descriptions.
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
        ("! @ # $ % ^ &", "Toggle small UI panels (shift+1..7)")
    ];

    let mut text_spans: Vec<Spans> = Vec::new();
    for (key, desc) in bindings.iter() {
        let key_span = Span::styled(
            format!("{:<12}", key),
            Style::default().fg(app.theme.title_accent.to_tui_color()).add_modifier(Modifier::BOLD),
        );
        let desc_span = Span::styled(
            format!("{}", desc),
            Style::default().fg(app.theme.foreground.to_tui_color()),
        );
        text_spans.push(Spans::from(vec![key_span, Span::raw("  "), desc_span]));
    }

    let para = Paragraph::new(Text::from(text_spans)).wrap(Wrap { trim: true }).alignment(Alignment::Left);
    f.render_widget(para, inner);
}
