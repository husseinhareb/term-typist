// src/app/input.rs
use crossterm::event::KeyCode;
use crate::app::state::{App, Mode, Status};

/// Handle navigation keys to switch tabs and adjust selected values.
pub fn handle_nav(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('1') => app.selected_tab = 0,
        KeyCode::Char('2') => app.selected_tab = 1,
        KeyCode::Char('3') => app.selected_tab = 2,
        KeyCode::Left if app.selected_value > 0 => app.selected_value -= 1,
        KeyCode::Right if app.selected_value + 1 < app.current_options().len() => {
            app.selected_value += 1;
        }
        _ => {}
    }
}

/// Map raw KeyCode into displayed keyboard labels (e.g., "ESC", "BS", "SPC", or character).
pub fn map_keycode(code: &KeyCode) -> Option<String> {
    match code {
        KeyCode::Esc => Some("ESC".into()),
        KeyCode::Backspace => Some("BS".into()),
        KeyCode::Char(' ') => Some("SPC".into()),
        KeyCode::Char(c) => Some(c.to_ascii_uppercase().to_string()),
        _ => None,
    }
}
