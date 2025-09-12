// src/ui/settings.rs

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Modifier, Style},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};
use crate::app::state::App;
use crate::ui::keyboard::Keyboard;

/// Draws the Settings screen, listing each boolean toggle.
pub fn draw_settings<B: Backend>(f: &mut Frame<B>, app: &App, _keyboard: &Keyboard) {
    // Split the terminal into a title area (3 rows) and the list below
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Length(3), // title
                Constraint::Min(0),    // list
            ]
            .as_ref(),
        )
        .split(f.size());

    // Title block
    let title = Block::default()
        .title("⚙ Settings")
        .borders(Borders::ALL);
    f.render_widget(title, chunks[0]);

    // Build one ListItem per toggle, showing [x] or [ ]
    let items = vec![
        ListItem::new(format!(
            "[{}] Show mode panel",
            if app.show_mode { 'x' } else { ' ' }
        )),
        ListItem::new(format!(
            "[{}] Show value panel",
            if app.show_value { 'x' } else { ' ' }
        )),
        ListItem::new(format!(
            "[{}] Show state panel",
            if app.show_state { 'x' } else { ' ' }
        )),
        ListItem::new(format!(
            "[{}] Show WPM/speed",
            if app.show_speed { 'x' } else { ' ' }
        )),
        ListItem::new(format!(
            "[{}] Show timer",
            if app.show_timer { 'x' } else { ' ' }
        )),
        ListItem::new(format!(
            "[{}] Show text area",
            if app.show_text { 'x' } else { ' ' }
        )),
        ListItem::new(format!(
            "[{}] Show on-screen keyboard",
            if app.show_keyboard { 'x' } else { ' ' }
        )),
        ListItem::new(format!(
            "Keyboard layout: {}",
            match app.keyboard_layout {
                crate::app::state::KeyboardLayout::Qwerty => "QWERTY",
                crate::app::state::KeyboardLayout::Azerty => "AZERTY",
                crate::app::state::KeyboardLayout::Dvorak => "Dvorak",
                crate::app::state::KeyboardLayout::Qwertz => "QWERTZ",
            }
        )),
        ListItem::new(format!(
            "Keyboard switch: {} (press 'k' to cycle)",
            app.keyboard_switch
        )),
    ];

    // Wrap them in a List widget
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Toggles (press 'l' to cycle layout)"),
        )
        // style the title of the List to stand out
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    // Render the list into the bottom chunk
    f.render_widget(list, chunks[1]);
}
