// src/ui/lib.rs

use std::{ io, time::{ Duration, Instant } };
use crossterm::{
    execute,
    terminal::{ disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen },
    event::{ self, Event, KeyCode, KeyEvent, KeyModifiers },
};
use tui::{ backend::CrosstermBackend, Terminal, widgets::Paragraph, style::Style };

mod app;       // src/app/mod.rs → state.rs, input.rs, config.rs
mod ui;        // src/ui/mod.rs → draw.rs, keyboard.rs
mod graph;     // src/graph.rs
mod wpm;       // src/wpm.rs
mod generator; // src/generator.rs
mod db;        // src/db.rs
mod theme;     // src/theme.rs
pub mod themes_presets; // src/themes_presets.rs (predefined themes)
mod audio;     // src/audio.rs

use app::state::{ App, Mode, Status };
use std::cmp;
use app::input::handle_nav;
use ui::draw::{ draw, draw_finished };
use ui::keyboard::Keyboard;
use wpm::{ accuracy, elapsed_seconds_since_start, net_wpm };
use db::{ open, save_test };
use theme::Theme;
use crate::ui::profile::draw_profile;
use crate::ui::settings::draw_settings; 

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    // — Open (or create) the SQLite DB under ~/.local/share/term-typist/term_typist.db
    let mut conn = open()?;

    // — Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // — Timing & WPM cache
    let tick_rate = Duration::from_millis(200);
    let mut last_tick = Instant::now();
    let mut last_sample = 0;
    let mut last_wpm_update = Instant::now();
    let mut cached_net = 0.0;
    let mut cached_acc = 0.0;

    // — App factory: generate target text sized to the selected mode/value
    // Attempt to honour a persisted `nb_of_words` config: if present and matching one
    // of the Words options, start in Words mode with that selected value. Otherwise
    // default to Time mode 15s (selected_tab=0, selected_value=0).
    let make_app = || {
        // Default to Time 15s
        let mut default_tab = 0usize;
        let mut default_value = 0usize;

        // First, prefer persisted explicit selection (test_mode/test_value)
        if let Ok(Some((tab, val))) = crate::app::config::read_selected_mode_value() {
            default_tab = tab;
            default_value = val;
        } else if let Ok(nb) = crate::app::config::read_nb_of_words() {
            // Backwards-compat: if nb_of_words was saved (older config), map it to Words options
            let words_options = [10i32, 25i32, 50i32, 100i32];
            if let Some(idx) = words_options.iter().position(|&w| w == nb) {
                default_tab = 1; // Words mode
                default_value = idx;
            }
        }

        let sentence = generator::generate_for_mode(default_tab, default_value);
        let mut a = App::new(sentence);
        a.selected_tab = default_tab;
        a.selected_value = default_value;
        a
    };
    let mut app = make_app();
    let mut keyboard = Keyboard::new();
    // Initialize audio playback (background thread/stream)
    audio::init();

    'main: loop {
        // — Throttle WPM/accuracy updates once per second
        if let Mode::Insert = app.mode {
            if app.start.is_some() && last_wpm_update.elapsed() >= Duration::from_secs(1) {
                let real_secs = elapsed_seconds_since_start(app.start.unwrap());
                let idle = Instant::now().duration_since(app.last_input).as_secs_f64();
                let effective = real_secs + idle;
                cached_net = net_wpm(app.correct_chars, app.incorrect_chars, effective);
                cached_acc = accuracy(app.correct_chars, app.incorrect_chars);
                last_wpm_update = Instant::now();

                let secs = app.elapsed_secs();
                if secs > last_sample {
                    last_sample = secs;
                    app.samples.push((secs, cached_net));
                }
            }
        }

        // — Draw typing UI, Profile, or Settings
        terminal.draw(|f| {
            // First, paint a full-screen themed background so no terminal
            // pixels show through at the edges (this fills every cell).
            let size = f.size();
            let bg = Paragraph::new("").style(Style::default().bg(app.theme.background.to_tui_color()).fg(app.theme.foreground.to_tui_color()));
            f.render_widget(bg, size);

            match app.mode {
                Mode::Profile => {
                    // draw_profile now accepts (&mut Frame, &Connection, &Theme)
                    draw_profile(f, &conn, &app.theme);
                }
                Mode::Settings => {
                    // draw_settings should render your settings UI
                    draw_settings(f, &app, &keyboard);
                }
                _ => {
                    draw(f, &app, &keyboard, cached_net, cached_acc);
                }
            }
        })?;

        // — Handle input & toggles
        let timeout = tick_rate.checked_sub(last_tick.elapsed()).unwrap_or_default();
        if event::poll(timeout)? {
            if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                // ── SHIFT+NUMBER PANEL TOGGLES
                match code {
                    KeyCode::Char('!') => { app.show_mode     = !app.show_mode;     continue 'main; }
                    KeyCode::Char('@') => { app.show_value    = !app.show_value;    continue 'main; }
                    KeyCode::Char('#') => { app.show_state    = !app.show_state;    continue 'main; }
                    KeyCode::Char('$') => { app.show_speed    = !app.show_speed;    continue 'main; }
                    KeyCode::Char('%') => { app.show_timer    = !app.show_timer;    continue 'main; }
                    KeyCode::Char('^') => { app.show_text     = !app.show_text;     continue 'main; }
                    KeyCode::Char('&') => { app.show_keyboard = !app.show_keyboard; continue 'main; }
                    _ => {}
                }

                // ── Ctrl-C quits
                if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
                    break 'main;
                }

                // ── Esc: in Profile or Settings → back to View; else restart test
                // Only save the test to the DB if it was actually started (app.start.is_some()).
                if code == KeyCode::Esc && modifiers.is_empty() {
                    match app.mode {
                        Mode::Profile | Mode::Settings => {
                            app.mode = Mode::View;
                            continue 'main;
                        }
                        _ => {
                            if app.start.is_some() {
                                // Test was started; persist stats
                                save_test(&mut conn, &app)?;
                            }
                            // Restart the test regardless of whether it started
                            // Preserve UI choices (layout, switch, and selected mode/value)
                            let cur_layout = app.keyboard_layout;
                            let cur_switch = app.keyboard_switch.clone();
                            let cur_tab = app.selected_tab;
                            let cur_value = app.selected_value;

                            // Build a new App using a target that matches the current selection
                            let new_target = if cur_tab == 2 {
                                // Zen mode: start with empty target
                                String::new()
                            } else {
                                generator::generate_for_mode(cur_tab, cur_value)
                            };
                            app = App::new(new_target);
                            // restore preserved UI state
                            app.keyboard_layout = cur_layout;
                            app.keyboard_switch = cur_switch;
                            app.selected_tab = cur_tab;
                            app.selected_value = cur_value;
                            // Clear any pressed key highlight
                            keyboard.pressed_key = None;
                            last_sample = 0;
                            continue 'main;
                        }
                    }
                }

                // ── Highlight key in on-screen keyboard
                keyboard.handle_key(&code);

                // ── Global navigation (tabs & values)
                handle_nav(&mut app, code);

                // ── 'p' opens Profile, 's' opens Settings (accept upper/lower case)
                if let KeyCode::Char(c) = code {
                    let lc = c.to_ascii_lowercase();
                    if lc == 'p' && app.mode == Mode::View {
                        app.mode = Mode::Profile;
                        continue 'main;
                    }
                    if lc == 's' && app.mode == Mode::View {
                        app.mode = Mode::Settings;
                        continue 'main;
                    }
                }

                // ── Mode-specific input
                match app.mode {
                    Mode::View => {
                        if code == KeyCode::Enter {
                            // When entering Insert mode from View, prefer to keep the previewed
                            // `app.target` that the user already saw. Only regenerate if the
                            // target is empty (for example Zen mode cleared it) so the user
                            // experience is consistent and the text doesn't unexpectedly change
                            // on pressing Enter.
                            if app.target.is_empty() {
                                let new_target = generator::generate_for_mode(app.selected_tab, app.selected_value);
                                app.target = new_target.clone();
                                app.status = vec![crate::app::state::Status::Untyped; new_target.chars().count()];
                            } else {
                                // Ensure status length matches existing target
                                app.status = vec![crate::app::state::Status::Untyped; app.target.chars().count()];
                            }

                            app.mode = Mode::Insert;
                            let now = Instant::now();
                            app.start = Some(now);
                            app.last_input = now;
                            app.locked = false;
                            if app.audio_enabled {
                                crate::audio::play_for(&app.keyboard_switch, "ENTER");
                            }
                        }
                    }

                    Mode::Insert => {
                        match code {
                            KeyCode::Char(' ') => {
                                if app.audio_enabled {
                                    crate::audio::play_for(&app.keyboard_switch, "SPACE");
                                }
                                app.on_key(' ');
                            }
                            KeyCode::Char(c) if !c.is_control() => {
                                if app.audio_enabled {
                                    crate::audio::play_for(&app.keyboard_switch, "GENERIC");
                                }
                                app.on_key(c)
                            }
                            KeyCode::Backspace  => app.backspace(),
                            _                   => {}
                        }
                        // Auto‐finish logic
                        match app.selected_tab {
                            0 => {
                                // Time mode
                                if app.elapsed_secs() >= (app.current_options()[app.selected_value] as u64) {
                                    app.mode = Mode::Finished;
                                }
                            }
                            1 => {
                                // Words mode
                                let completed_chars = app.status
                                    .iter()
                                    .position(|&s| s == Status::Untyped)
                                    .unwrap_or(app.status.len());
                                let completed_words = app.target[..completed_chars]
                                    .split_whitespace()
                                    .count();
                                if completed_words >= (app.current_options()[app.selected_value] as usize) {
                                    app.mode = Mode::Finished;
                                }
                            }
                            _ => {}
                        }
                    }

                    Mode::Finished => {
                        // Finished tests are results of a started test, so save unconditionally
                        save_test(&mut conn, &app)?;
                        terminal.draw(|f| draw_finished(f, &app))?;
                        // Await Esc (restart) or Ctrl-C (quit)
                        loop {
                            if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                                if code == KeyCode::Esc {
                                    // When viewing finished results, Esc restarts without saving again.
                                    // Preserve selected tab/value and other UI settings, and
                                    // regenerate a target that matches the user's selection so
                                    // the restarted test length/time corresponds to what was selected.
                                    let cur_layout = app.keyboard_layout;
                                    let cur_switch = app.keyboard_switch.clone();
                                    let cur_tab = app.selected_tab;
                                    let cur_value = app.selected_value;

                                    let new_target = if cur_tab == 2 {
                                        String::new()
                                    } else {
                                        generator::generate_for_mode(cur_tab, cur_value)
                                    };
                                    app = App::new(new_target);
                                    app.keyboard_layout = cur_layout;
                                    app.keyboard_switch = cur_switch;
                                    app.selected_tab = cur_tab;
                                    app.selected_value = cur_value;
                                    keyboard.pressed_key = None;
                                    last_sample = 0;
                                    break;
                                }
                                if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
                                    disable_raw_mode()?;
                                    execute!(io::stdout(), LeaveAlternateScreen)?;
                                    return Ok(());
                                }
                            }
                        }
                    }

                    Mode::Profile => {
                        // Handle profile navigation keys
                        use crate::ui::profile::handle_profile_key;
                        handle_profile_key(code);
                    }

                    Mode::Settings => {
                        use std::num::NonZeroUsize;
                        // Navigation keys for settings: mirror profile's behavior but operate on app.settings_cursor
                        match code {
                            KeyCode::Up | KeyCode::Char('k') => {
                                // decrement if possible (like fetch_update with checked_sub)
                                if app.settings_cursor > 0 { app.settings_cursor = app.settings_cursor.saturating_sub(1); }
                                continue 'main;
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                app.settings_cursor = app.settings_cursor.saturating_add(1);
                                continue 'main;
                            }
                            KeyCode::PageUp => {
                                app.settings_cursor = app.settings_cursor.saturating_sub(10);
                                continue 'main;
                            }
                            KeyCode::PageDown => {
                                app.settings_cursor = app.settings_cursor.saturating_add(10);
                                continue 'main;
                            }
                            KeyCode::Home => { app.settings_cursor = 0; continue 'main; }
                            KeyCode::End => { app.settings_cursor = usize::MAX / 2; continue 'main; }
                            _ => {}
                        }

                        // Allow changing the currently selected setting with Left/Right
                        if code == KeyCode::Left || code == KeyCode::Right {
                            let left = code == KeyCode::Left;
                            // number of settings rows (must match draw_settings ordering)
                            let total = 10usize;
                            let sel = if total > 0 { cmp::min(app.settings_cursor, total - 1) } else { 0 };
                            match sel {
                                0 => { app.show_mode = !app.show_mode; }
                                1 => { app.show_value = !app.show_value; }
                                2 => { app.show_state = !app.show_state; }
                                3 => { app.show_speed = !app.show_speed; }
                                4 => { app.show_timer = !app.show_timer; }
                                5 => { app.show_text = !app.show_text; }
                                6 => { app.show_keyboard = !app.show_keyboard; }
                                7 => {
                                    app.keyboard_layout = match app.keyboard_layout {
                                        crate::app::state::KeyboardLayout::Qwerty => if left { crate::app::state::KeyboardLayout::Qwertz } else { crate::app::state::KeyboardLayout::Azerty },
                                        crate::app::state::KeyboardLayout::Azerty => if left { crate::app::state::KeyboardLayout::Qwerty } else { crate::app::state::KeyboardLayout::Dvorak },
                                        crate::app::state::KeyboardLayout::Dvorak => if left { crate::app::state::KeyboardLayout::Azerty } else { crate::app::state::KeyboardLayout::Qwertz },
                                        crate::app::state::KeyboardLayout::Qwertz => if left { crate::app::state::KeyboardLayout::Dvorak } else { crate::app::state::KeyboardLayout::Qwerty },
                                    };
                                    let _ = crate::app::config::write_keyboard_layout(match app.keyboard_layout {
                                        crate::app::state::KeyboardLayout::Qwerty => "qwerty",
                                        crate::app::state::KeyboardLayout::Azerty => "azerty",
                                        crate::app::state::KeyboardLayout::Dvorak => "dvorak",
                                        crate::app::state::KeyboardLayout::Qwertz => "qwertz",
                                    });
                                }
                                8 => {
                                    let presets = crate::themes_presets::preset_names();
                                    if !presets.is_empty() {
                                        // find current index
                                        let mut idx = 0usize;
                                        for (i, &n) in presets.iter().enumerate() {
                                            if let Some(p) = crate::themes_presets::theme_by_name(n) {
                                                    if p == app.theme { idx = i; break; }
                                                }
                                        }
                                        if left { idx = (idx + presets.len() - 1) % presets.len(); } else { idx = (idx + 1) % presets.len(); }
                                        if let Some(next) = crate::themes_presets::theme_by_name(presets[idx]) { app.theme = next; let _ = app.theme.save_to_config(); }
                                    }
                                }
                                9 => { app.audio_enabled = !app.audio_enabled; let _ = crate::app::config::write_audio_enabled(app.audio_enabled); }
                                _ => {}
                            }
                            continue 'main;
                        }

                        // Allow cycling keyboard layout with 'l'
                        if code == KeyCode::Char('l') {
                            app.keyboard_layout = match app.keyboard_layout {
                                crate::app::state::KeyboardLayout::Qwerty => crate::app::state::KeyboardLayout::Azerty,
                                crate::app::state::KeyboardLayout::Azerty => crate::app::state::KeyboardLayout::Dvorak,
                                crate::app::state::KeyboardLayout::Dvorak => crate::app::state::KeyboardLayout::Qwertz,
                                crate::app::state::KeyboardLayout::Qwertz => crate::app::state::KeyboardLayout::Qwerty,
                            };
                            // Clear any pressed key highlight so it doesn't point to an unrelated key
                            keyboard.pressed_key = None;
                            // Persist layout choice to config
                            let _ = crate::app::config::write_keyboard_layout(match app.keyboard_layout {
                                crate::app::state::KeyboardLayout::Qwerty => "qwerty",
                                crate::app::state::KeyboardLayout::Azerty => "azerty",
                                crate::app::state::KeyboardLayout::Dvorak => "dvorak",
                                crate::app::state::KeyboardLayout::Qwertz => "qwertz",
                            });
                            continue 'main;
                        }

                        // Cycle themes with 't' and apply immediately
                        if code == KeyCode::Char('t') {
                            // get preset names
                            let presets = crate::themes_presets::preset_names();
                            if !presets.is_empty() {
                                // find index of current theme by matching title color
                                let mut idx = 0usize;
                                for (i, name) in presets.iter().enumerate() {
                                    if let Some(p) = crate::themes_presets::theme_by_name(name) {
                                        if p == app.theme {
                                            idx = i;
                                            break;
                                        }
                                    }
                                }
                                idx = (idx + 1) % presets.len();
                                if let Some(next) = crate::themes_presets::theme_by_name(presets[idx]) {
                                    app.theme = next;
                                }
                            }
                            continue 'main;
                        }

                        // Cycle available keyboard switch samples with 'k'
                        if code == KeyCode::Char('K') || code == KeyCode::Char('K') {
                            let list = crate::audio::list_switches();
                            if !list.is_empty() {
                                // find current index
                                let mut idx = list.iter().position(|s| s == &app.keyboard_switch).unwrap_or(0);
                                idx = (idx + 1) % list.len();
                                app.keyboard_switch = list[idx].clone();
                                // persist selection
                                let _ = crate::app::config::write_keyboard_switch(&app.keyboard_switch);
                                // clear pressed highlight
                                keyboard.pressed_key = None;
                            }
                            continue 'main;
                        }

                        // Toggle audio on/off with 'a'
                        if code == KeyCode::Char('a') {
                            app.audio_enabled = !app.audio_enabled;
                            let _ = crate::app::config::write_audio_enabled(app.audio_enabled);
                            continue 'main;
                        }
                        // Other keys do nothing here; Esc is already handled above
                    }
                }
            }
        }

        last_tick = Instant::now();
    }

    // — Teardown
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}
