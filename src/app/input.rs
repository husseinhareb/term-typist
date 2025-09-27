// src/app/input.rs
use crossterm::event::KeyCode;
use crate::app::state::{App, Status};
use crate::generator;

/// Handle navigation keys to switch tabs and adjust selected values.
pub fn handle_nav(app: &mut App, code: KeyCode) {
    // Do not allow navigation to change selected tab/value while typing (Insert mode).
    // Changing the selection during an active test can cause the preview/target to
    // become inconsistent with the user's last visible preview and lead to unexpected
    // target lengths on restart. Navigation is still allowed in View/Settings/Profile.
    use crate::app::state::Mode;
    if app.mode == Mode::Insert {
        return;
    }

    // capture previous selection to detect changes
    let prev_tab = app.selected_tab;
    let prev_value = app.selected_value;

    match code {
        KeyCode::Char('1') => app.selected_tab = 0,
        KeyCode::Char('2') => app.selected_tab = 1,
        KeyCode::Char('3') => app.selected_tab = 2,
        KeyCode::Left if app.selected_value > 0 => app.selected_value -= 1,
    KeyCode::Right if !app.current_options().is_empty() && app.selected_value + 1 < app.current_options().len() => {
            app.selected_value += 1;
        }
        _ => {}
    }

    // If tab changed, ensure selected_value is within new options' bounds
    if app.selected_tab != prev_tab {
        let opts_len = app.current_options().len();
        if opts_len == 0 {
            app.selected_value = 0;
        } else {
            app.selected_value = std::cmp::min(app.selected_value, opts_len - 1);
        }
    }

    // If selection changed and we're not currently typing, regenerate the preview target so
    // the new number-of-words or time choice is visible immediately. Persist the
    // selection to config so it survives restarts.
    if (app.selected_tab != prev_tab || app.selected_value != prev_value) && app.mode != Mode::Insert {
        if app.selected_tab == 2 {
            // Zen mode: clear the generated target and reset free_text
            app.target = String::new();
            app.status = vec![];
            app.free_text.clear();
        } else {
            let new_target = generator::generate_for_mode(app.selected_tab, app.selected_value);
            app.target = new_target.clone();
            app.status = vec![Status::Untyped; new_target.chars().count()];
        }
        // Persist the selection so closing and reopening the app restores it.
        let _ = crate::app::config::write_selected_mode_value(app.selected_tab, app.selected_value);
    }
}

/// Map raw KeyCode into displayed keyboard labels (e.g., "Esc", "Backspace", "Space", or character).
pub fn map_keycode(code: &KeyCode) -> Option<String> {
    match code {
        KeyCode::Esc => Some("Esc".into()),
        KeyCode::Backspace => Some("Backspace".into()),
        KeyCode::Char(' ') => Some("Space".into()),
        KeyCode::Char(c) => Some(c.to_ascii_uppercase().to_string()),
        _ => None,
    }
}
