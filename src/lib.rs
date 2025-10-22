// src/ui/lib.rs

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    io,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use std::fs::OpenOptions;
use std::io::Write as _;
use tui::{
    backend::CrosstermBackend,
    style::{Style, Modifier},
    widgets::{Paragraph, Clear, Block, Borders, Wrap},
    layout::Alignment,
    text::{Span, Spans},
    Terminal,
};

mod app; // src/app/mod.rs → state.rs, input.rs, config.rs
mod audio;
mod caps; // src/caps.rs (platform helpers)
mod db; // src/db.rs
mod generator; // src/generator.rs
mod graph; // src/graph.rs
mod theme; // src/theme.rs
pub mod themes_presets; // src/themes_presets.rs (predefined themes)
mod ui; // src/ui/mod.rs → draw.rs, keyboard.rs
mod wpm; // src/wpm.rs // src/audio.rs

use crate::ui::profile::draw_profile;
use crate::ui::leaderboard::draw_leaderboard;
use crate::ui::settings::draw_settings;
use app::input::handle_nav;
use app::state::{App, Mode, Status};
use db::{open, save_test};
use std::cmp;
use ui::draw::{draw, draw_finished};
use ui::keyboard::Keyboard;
use wpm::{accuracy, elapsed_seconds_since_start};

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
    // Initialize caps lock state from system if possible
    app.caps_detection_available = caps::detection_available();
    app.caps_lock_on = if app.caps_detection_available {
        caps::is_caps_lock_on()
    } else {
        false
    };
    // Start a background poller to keep a fast in-process cached value so
    // the Caps Lock modal appears/disappears responsively.
    if app.caps_detection_available {
        caps::start_polling(150);
    }
    // No startup hint; Caps Lock state is driven by system detection or heuristics.
    let mut keyboard = Keyboard::new();
    // Initialize audio playback (background thread/stream)
    audio::init();

    'main: loop {
        // Poll OS Caps Lock state (best-effort) each tick to support toggles.
        // Only query the OS when a system-backed detection method is available.
        // If detection is available we trust the system value and do not allow
        // in-terminal heuristics to override it.
        if app.caps_detection_available {
            let os_caps = caps::is_caps_lock_on();
            if os_caps != app.caps_lock_on {
                app.caps_lock_on = os_caps;
            }
        }
        // No timeout anymore: modal overlay is only shown when system-backed detection is available
        // and the OS reports Caps Lock is actually on. Heuristics still update the `caps_lock_on`
        // flag but won't trigger the persistent modal.
        // — Throttle WPM/accuracy updates once per second
        if let Mode::Insert = app.mode {
            if app.start.is_some() && last_wpm_update.elapsed() >= Duration::from_secs(1) {
                let real_secs = elapsed_seconds_since_start(app.start.unwrap());
                let idle = Instant::now().duration_since(app.last_input).as_secs_f64();
                let _effective = real_secs + idle;
                // Raw WPM counts all typed chars as if correct
                // Net WPM must use the same time window as raw WPM (test start -> now).
                // Use the windowed function so raw >= net always holds.
                if let Some(start) = app.start {
                    let now = std::time::Instant::now();
                    cached_net = crate::wpm::net_wpm_from_correct_timestamps_window(
                        &app.correct_timestamps,
                        start,
                        now,
                    );
                } else {
                    cached_net = 0.0;
                }
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
            let bg = Paragraph::new("").style(
                Style::default()
                    .bg(app.theme.background.to_tui_color())
                    .fg(app.theme.foreground.to_tui_color()),
            );
            f.render_widget(bg, size);

            match app.mode {
                Mode::Profile => {
                    // draw_profile now accepts (&mut Frame, &Connection, &Theme)
                    draw_profile(f, &conn, &app.theme);
                }
                Mode::Leaderboard => {
                    // show the typing UI in the background and overlay the leaderboard modal
                    draw(f, &app, &keyboard, cached_net, cached_acc);
                    draw_leaderboard(f, &conn, &app.theme);
                }
                Mode::Settings => {
                    // draw_settings should render your settings UI
                    draw_settings(f, &app, &keyboard);
                }
                Mode::Menu => {
                    // Paint the normal typing UI as the backdrop, then overlay the small menu modal
                    draw(f, &app, &keyboard, cached_net, cached_acc);
                    let split_band = ui::draw::bottom_split_band(f, &app);
                    crate::ui::menu::draw_menu(f, &app, split_band);
                }
                Mode::Help => {
                    // Help is also a small overlay; keep the main UI visible behind it
                    draw(f, &app, &keyboard, cached_net, cached_acc);
                    crate::ui::help::draw_help(f, &app);
                }
                _ => {
                    draw(f, &app, &keyboard, cached_net, cached_acc);
                }
            }

            // Global overlay: Caps Lock modal — query the system fresh at render time.
            // Previously we relied on `app.caps_lock_on` which could be set by heuristics
            // or user toggles; that caused the modal to appear incorrectly. Here we
            // perform a fresh system query (when available) to determine whether to show
            // the persistent modal. Heuristics may still update the `app.caps_lock_on`
            // flag for internal logic, but they won't trigger the modal unless the
            // system actually reports CapsLock on.
            let show_caps_modal = if app.caps_detection_available {
                // Use the cached fast-read to make the UI responsive. The poller
                // is started at app init and updates quickly in the background.
                caps::cached_is_caps_lock_on()
            } else {
                false
            };

            if show_caps_modal {
                let area = f.size();

                // Keep it ASCII; emoji width varies across terminals.
                let title = "CAPS LOCK IS ON";
                let hint  = "Press Caps Lock to disable";

                // Size + position
                let max_line = title.chars().count().max(hint.chars().count()) as u16;
                let w = (max_line + 8).clamp(28, area.width.saturating_sub(2));
                let h = 5u16;
                let x = area.x + (area.width.saturating_sub(w)) / 2;
                let y = area.y + (area.height.saturating_sub(h)) / 2;
                let rect = tui::layout::Rect::new(x, y, w, h);

                // 1) Clear the region so underlying UI can't bleed through
                f.render_widget(Clear, rect);

                // 2) Modal box – neutral bg, accent border
                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(app.theme.title_accent.to_tui_color()))
                    .style(Style::default().bg(app.theme.background.to_tui_color()));
                f.render_widget(block.clone(), rect);

                // 4) Inner content (centered, two lines)
                let inner = block.inner(rect);
                f.render_widget(Clear, inner); // clear inner too, avoids leftover chars on short lines

                let lines = vec![
                    Spans::from(Span::styled(
                        title,
                        Style::default()
                            .fg(app.theme.title.to_tui_color())
                            .add_modifier(Modifier::BOLD),
                    )),
                    Spans::from(Span::styled(
                        hint,
                        Style::default().fg(app.theme.stats_label.to_tui_color()),
                    )),
                ];

                let para = Paragraph::new(lines)
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true })
                    .style(Style::default().bg(app.theme.background.to_tui_color()));
                f.render_widget(para, inner);
            }

            // Debug: append caps detection state to a temp log so we can see
            // whether opening/closing menus affects detection. This is a
            // non-invasive file append used only for debugging.
            let _ = (|| -> Result<(), std::io::Error> {
                let ts = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0);
                let os_caps = if app.caps_detection_available { caps::is_caps_lock_on() } else { false };
                let mut f = OpenOptions::new().create(true).append(true).open("/tmp/term_typist_caps.log")?;
                let mode_str = match app.mode {
                    Mode::View => "View",
                    Mode::Insert => "Insert",
                    Mode::Finished => "Finished",
                    Mode::Profile => "Profile",
                    Mode::Leaderboard => "Leaderboard",
                    Mode::Settings => "Settings",
                    Mode::Menu => "Menu",
                    Mode::Help => "Help",
                };
                writeln!(f, "{:.3} mode={} caps_detection_available={} os_caps={} app_caps={}", ts, mode_str, app.caps_detection_available, os_caps, app.caps_lock_on)?;
                Ok(())
            })();

            // (No startup test hint is shown; Caps Lock is driven by system detection,
            // terminal-reported CapsLock key, or heuristics.)
        })?;

        // — Handle input & toggles
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_default();
        if event::poll(timeout)? {
            if let Event::Key(KeyEvent {
                code, modifiers, ..
            }) = event::read()?
            {
                // Terminals don't reliably expose a CAPS_LOCK modifier via crossterm.
                // Rely on existing heuristics (uppercase letters typed without SHIFT imply CapsLock)
                // and on periodic OS polling when `caps_detection_available` is true.
                // ── SHIFT+NUMBER PANEL TOGGLES
                // Accept the number key regardless of layout by mapping several
                // common produced characters for the top-row number keys. Terminals
                // often report either the digit (when Shift produces digit), the
                // shifted symbol (e.g. '!' on QWERTY) or the unshifted AZERTY symbol
                // (e.g. '&'). We try a best-effort mapping:
                // 1) if a digit was produced use it (1..0 -> index 0..9)
                // 2) otherwise if SHIFT modifier is present prefer QWERTY shifted-symbol mapping
                // 3) otherwise prefer AZERTY unshifted mapping
                if let KeyCode::Char(c) = code {
                    // maps for shifted (QWERTY) symbols -> index
                    // '!','@','#','$','%','^','&','*','(',')' => 0..9
                    let shifted_map = |ch: char| match ch {
                        '!' => Some(0),
                        '@' => Some(1),
                        '#' => Some(2),
                        '$' => Some(3),
                        '%' => Some(4),
                        '^' => Some(5),
                        '&' => Some(6),
                        '*' => Some(7),
                        '(' => Some(8),
                        ')' => Some(9),
                        _ => None,
                    };

                    // common AZERTY unshifted symbols for keys 1..0 -> index 0..9
                    let azerty_map = |ch: char| match ch {
                        '&' => Some(0),
                        'é' => Some(1),
                        '"' => Some(2),
                        '\'' => Some(3),
                        '(' => Some(4),
                        '-' => Some(5),
                        'è' => Some(6),
                        '_' => Some(7),
                        'ç' => Some(8),
                        'à' => Some(9),
                        _ => None,
                    };

                    // 1) digit wins (map '1'..'9','0' -> indices 0..9)
                    let idx_opt: Option<usize> = if c.is_ascii_digit() {
                        Some(if c == '0' { 9 } else { (c as u8 - b'1') as usize })
                    } else if modifiers.contains(KeyModifiers::SHIFT) {
                        // prefer shifted symbols mapping when shift reported
                        shifted_map(c).or_else(|| azerty_map(c))
                    } else {
                        // prefer azerty/unshifted mapping, fallback to shifted_map
                        azerty_map(c).or_else(|| shifted_map(c))
                    };

                    if let Some(idx) = idx_opt {
                        match idx {
                            0 => { app.show_mode = !app.show_mode; continue 'main; }
                            1 => { app.show_value = !app.show_value; continue 'main; }
                            2 => { app.show_state = !app.show_state; continue 'main; }
                            3 => { app.show_speed = !app.show_speed; continue 'main; }
                            4 => { app.show_timer = !app.show_timer; continue 'main; }
                            5 => { app.show_text = !app.show_text; continue 'main; }
                            6 => { app.show_keyboard = !app.show_keyboard; continue 'main; }
                            _ => {}
                        }
                    }
                }

                // ── Ctrl-C quits
                if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
                    break 'main;
                }

                // ── Menu keys behavior:
                // - F1 and 'm' only open the menu when we're on the main page (Mode::View).
                // - Tab toggles the menu: if Menu -> close (to View); if View -> open.
                // This prevents accidental opens while typing or inside other popups.
                if code == KeyCode::Tab {
                    if app.mode == Mode::Menu {
                        // close menu
                        app.mode = Mode::View;
                        continue 'main;
                    }
                    if app.mode == Mode::View {
                        // open menu from main view
                        app.mode = Mode::Menu;
                        app.menu_cursor = 0;
                        continue 'main;
                    }
                    // update last_tick so poll timeout stays in sync
                    last_tick = Instant::now();
                }

                if app.mode == Mode::View
                    && (code == KeyCode::F(1)
                        || matches!(code, KeyCode::Char(c) if c.eq_ignore_ascii_case(&'m')))
                {
                    app.mode = Mode::Menu;
                    app.menu_cursor = 0;
                    continue 'main;
                }

                // Only save the test to the DB if it was actually started (app.start.is_some()).
                if code == KeyCode::Esc && modifiers.is_empty() {
                    match app.mode {
                        // When in small popups, Esc should just return to the main view.
                        Mode::Profile | Mode::Settings | Mode::Menu | Mode::Help | Mode::Leaderboard => {
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
                if app.mode == Mode::View {
                    handle_nav(&mut app, code);
                }

                // ── 'p' opens Profile, 's' opens Settings (accept upper/lower case)
                if let KeyCode::Char(c) = code {
                    let lc = c.to_ascii_lowercase();
                    // Heuristic: uppercase letter without SHIFT => CapsLock likely ON
                    if c.is_ascii_alphabetic()
                        && c.is_ascii_uppercase()
                        && !modifiers.contains(KeyModifiers::SHIFT)
                    {
                        app.caps_lock_on = true;
                    } else if c.is_ascii_alphabetic()
                        && c.is_ascii_lowercase()
                        && !modifiers.contains(KeyModifiers::SHIFT)
                    {
                        // typing lowercase without SHIFT suggests CapsLock OFF
                        app.caps_lock_on = false;
                    }
                    if lc == 'p' && app.mode == Mode::View {
                        app.mode = Mode::Profile;
                        continue 'main;
                    }
                    if lc == 's' && app.mode == Mode::View {
                        app.mode = Mode::Settings;
                        continue 'main;
                    }
                    if lc == 'l' && app.mode == Mode::View {
                        app.mode = Mode::Leaderboard;
                        continue 'main;
                    }
                }

                // ── Mode-specific input
                match app.mode {
                    Mode::Menu => {
                        match code {
                            // Wrap-around navigation: moving up from the first item goes to the last,
                            // moving down from the last item goes to the first. Keep the total
                            // in sync with the menu labels in `src/ui/menu.rs` (3 items).
                            KeyCode::Up | KeyCode::Char('k') => {
                                let total = 3usize;
                                if total > 0 {
                                    app.menu_cursor = if app.menu_cursor == 0 {
                                        total - 1
                                    } else {
                                        app.menu_cursor - 1
                                    };
                                }
                                continue 'main;
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                let total = 3usize;
                                if total > 0 {
                                    app.menu_cursor = (app.menu_cursor + 1) % total;
                                }
                                continue 'main;
                            }
                            KeyCode::Enter => {
                                match app.menu_cursor {
                                    0 => {
                                        app.mode = Mode::Settings;
                                        app.menu_cursor = 0;
                                    }
                                    1 => {
                                        app.mode = Mode::Help;
                                        app.menu_cursor = 0;
                                    }
                                    2 => {
                                        disable_raw_mode()?;
                                        execute!(io::stdout(), LeaveAlternateScreen)?;
                                        return Ok(());
                                    }
                                    _ => {}
                                }
                                continue 'main;
                            }
                            KeyCode::Esc => {
                                app.mode = Mode::View;
                                continue 'main;
                            }
                            _ => {}
                        }
                    }
                    Mode::Help => {
                        // any Esc returns to previous mode (View)
                        if code == KeyCode::Esc {
                            app.mode = Mode::View;
                            continue 'main;
                        }
                    }
                    Mode::View => {
                        if code == KeyCode::Enter {
                            // When entering Insert mode from View, prefer to keep the previewed
                            // `app.target` that the user already saw. Only regenerate if the
                            // target is empty (for example Zen mode cleared it) so the user
                            // experience is consistent and the text doesn't unexpectedly change
                            // on pressing Enter.
                            if app.target.is_empty() {
                                let new_target = generator::generate_for_mode(
                                    app.selected_tab,
                                    app.selected_value,
                                );
                                app.target = new_target.clone();
                                app.status = vec![
                                    crate::app::state::Status::Untyped;
                                    new_target.chars().count()
                                ];
                            } else {
                                // Ensure status length matches existing target
                                app.status = vec![
                                    crate::app::state::Status::Untyped;
                                    app.target.chars().count()
                                ];
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
                            KeyCode::Backspace => app.backspace(),
                            _ => {}
                        }
                        // Auto‐finish logic
                        match app.selected_tab {
                            0 => {
                                // Time mode
                                if app.elapsed_secs()
                                    >= (app.current_options()[app.selected_value] as u64)
                                {
                                    app.mode = Mode::Finished;
                                }
                            }
                            1 => {
                                // Words mode
                                let completed_chars = app
                                    .status
                                    .iter()
                                    .position(|&s| s == Status::Untyped)
                                    .unwrap_or(app.status.len());
                                let completed_words =
                                    app.target[..completed_chars].split_whitespace().count();
                                if completed_words
                                    >= (app.current_options()[app.selected_value] as usize)
                                {
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
                            if let Event::Key(KeyEvent {
                                code, modifiers, ..
                            }) = event::read()?
                            {
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
                                if code == KeyCode::Char('c')
                                    && modifiers.contains(KeyModifiers::CONTROL)
                                {
                                    disable_raw_mode()?;
                                    execute!(io::stdout(), LeaveAlternateScreen)?;
                                    return Ok(());
                                }
                            }
                        }
                    }

                    Mode::Profile => {
                        // Handle profile navigation keys and Enter to open a historical summary
                        use crate::ui::profile::{handle_profile_key, recent_cursor};
                        if code == KeyCode::Enter {
                            // Determine selected absolute index and fetch that test id
                            let sel = recent_cursor();
                            // Query the DB for the test at that offset (ordered by started_at DESC)
                            if let Ok(opt_id) = conn
                                .prepare("SELECT id FROM tests ORDER BY started_at DESC LIMIT 1 OFFSET ?")
                                .and_then(|mut s| s.query_row([sel as i64], |r| r.get::<_, i64>(0)))
                            {
                                // Build a temporary App populated from DB row and samples
                                if let Ok(temp_app) = (|| -> Result<app::state::App, Box<dyn std::error::Error>> {
                                    // Fetch the full test row
                                    let mut stmt = conn.prepare(
                                        "SELECT started_at, duration_ms, mode, target_text, target_value, correct_chars, incorrect_chars, target_statuses, target_corrected FROM tests WHERE id = ?",
                                    )?;
                                    let row = stmt.query_row([opt_id], |r| {
                                        Ok((
                                            r.get::<_, String>(0)?,
                                            r.get::<_, i64>(1)?,
                                            r.get::<_, String>(2)?,
                                            r.get::<_, String>(3)?,
                                            r.get::<_, i64>(4)?,
                                            r.get::<_, i64>(5)?,
                                            r.get::<_, i64>(6)?,
                                            r.get::<_, Option<String>>(7)?,
                                            r.get::<_, Option<String>>(8)?,
                                        ))
                                    })?;

                                    let (_started_at_str, duration_ms, mode_s, target_text, target_value, correct_chars, incorrect_chars, statuses_opt, corrected_opt) = row;

                                    // Load samples for this test
                                    let mut samp_stmt = conn.prepare("SELECT elapsed_s, wpm FROM samples WHERE test_id = ? ORDER BY elapsed_s ASC")?;
                                    let samples: Vec<(u64, f64)> = samp_stmt
                                        .query_map([opt_id], |r| Ok((r.get::<_, i64>(0)? as u64, r.get::<_, f64>(1)?)))?
                                        .filter_map(Result::ok)
                                        .collect();

                                    // Create a temporary App initialized with the target text (so draw_finished can reuse it)
                                    let mut a = app::state::App::new(target_text.clone());
                                    // Map mode string to selected_tab
                                    a.selected_tab = match mode_s.as_str() {
                                        "time" => 0,
                                        "words" => 1,
                                        _ => 2,
                                    };
                                    // Map target_value to the index for the corresponding options (best effort)
                                    let val = target_value as i64;
                                    a.selected_value = match a.selected_tab {
                                        0 => [15i64, 30, 60, 100].iter().position(|&x| x == val).unwrap_or(0),
                                        1 => [10i64, 25, 50, 100].iter().position(|&x| x == val).unwrap_or(0),
                                        _ => 0,
                                    };

                                    // Set theme to current app's theme so visuals match
                                    a.theme = app.theme.clone();

                                    // Populate counts
                                    a.correct_chars = correct_chars as usize;
                                    a.incorrect_chars = incorrect_chars as usize;

                                    // If per-character statuses were saved, restore them so the finished
                                    // view can render the colored text. Otherwise leave as Untyped.
                                    if let Some(sraw) = statuses_opt {
                                        let mut v = Vec::new();
                                        for ch in sraw.chars() {
                                            match ch {
                                                'C' => v.push(app::state::Status::Correct),
                                                'I' => v.push(app::state::Status::Incorrect),
                                                _ => v.push(app::state::Status::Untyped),
                                            }
                                        }
                                        a.status = v;
                                    }

                                    // If corrected flags were saved, restore them too.
                                    if let Some(craw) = corrected_opt {
                                        let mut cv = Vec::new();
                                        for ch in craw.chars() {
                                            cv.push(ch == '1');
                                        }
                                        a.corrected = cv;
                                    }

                                    // Reconstruct start Instant so elapsed matches duration_ms
                                    use std::time::{Duration, Instant};
                                    let dur = Duration::from_millis(duration_ms as u64);
                                    a.start = Some(Instant::now() - dur);

                                    // Build synthetic correct_timestamps evenly spread across duration
                                    a.correct_timestamps = Vec::new();
                                    if a.correct_chars > 0 {
                                        for i in 0..a.correct_chars {
                                            let frac = (i as f64) / (a.correct_chars as f64);
                                            let offset = frac * (dur.as_secs_f64());
                                            a.correct_timestamps.push(Instant::now() - Duration::from_secs_f64(dur.as_secs_f64() - offset));
                                        }
                                    }

                                    // Attach samples
                                    a.samples = samples;

                                    Ok(a)
                                })() {
                                    // Render the finished summary for the historic test and wait until Esc
                                    terminal.draw(|f| draw_finished(f, &temp_app))?;
                                    loop {
                                        if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                                            if code == KeyCode::Esc {
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
                            }
                            continue 'main;
                        }
                        // Otherwise handle navigation keys
                        handle_profile_key(code);
                    }

                    Mode::Leaderboard => {
                        use crate::ui::leaderboard::{handle_leaderboard_key, leaderboard_cursor};
                        if code == KeyCode::Enter {
                            // Fetch selected test id using ORDER BY wpm DESC LIMIT 1 OFFSET cursor
                            let sel = leaderboard_cursor();
                            if let Ok(opt_id) = conn
                                .prepare("SELECT id FROM tests ORDER BY wpm DESC LIMIT 1 OFFSET ?")
                                .and_then(|mut s| s.query_row([sel as i64], |r| r.get::<_, i64>(0)))
                            {
                                // Build temp app from DB and render finished summary (reuse same logic as Profile case)
                                if let Ok(temp_app) = (|| -> Result<app::state::App, Box<dyn std::error::Error>> {
                                    let mut stmt = conn.prepare(
                                        "SELECT started_at, duration_ms, mode, target_text, target_value, correct_chars, incorrect_chars, target_statuses, target_corrected FROM tests WHERE id = ?",
                                    )?;
                                    let row = stmt.query_row([opt_id], |r| {
                                        Ok((
                                            r.get::<_, String>(0)?,
                                            r.get::<_, i64>(1)?,
                                            r.get::<_, String>(2)?,
                                            r.get::<_, String>(3)?,
                                            r.get::<_, i64>(4)?,
                                            r.get::<_, i64>(5)?,
                                            r.get::<_, i64>(6)?,
                                            r.get::<_, Option<String>>(7)?,
                                            r.get::<_, Option<String>>(8)?,
                                        ))
                                    })?;

                                    let (_started_at_str, duration_ms, mode_s, target_text, target_value, correct_chars, incorrect_chars, statuses_opt, corrected_opt) = row;
                                    let mut samp_stmt = conn.prepare("SELECT elapsed_s, wpm FROM samples WHERE test_id = ? ORDER BY elapsed_s ASC")?;
                                    let samples: Vec<(u64, f64)> = samp_stmt
                                        .query_map([opt_id], |r| Ok((r.get::<_, i64>(0)? as u64, r.get::<_, f64>(1)?)))?
                                        .filter_map(Result::ok)
                                        .collect();

                                    let mut a = app::state::App::new(target_text.clone());
                                    a.selected_tab = match mode_s.as_str() {
                                        "time" => 0,
                                        "words" => 1,
                                        _ => 2,
                                    };
                                    a.selected_value = match a.selected_tab {
                                        0 => [15i64, 30, 60, 100].iter().position(|&x| x == target_value as i64).unwrap_or(0),
                                        1 => [10i64, 25, 50, 100].iter().position(|&x| x == target_value as i64).unwrap_or(0),
                                        _ => 0,
                                    };
                                    a.theme = app.theme.clone();
                                    a.correct_chars = correct_chars as usize;
                                    a.incorrect_chars = incorrect_chars as usize;
                                    if let Some(sraw) = statuses_opt {
                                        let mut v = Vec::new();
                                        for ch in sraw.chars() {
                                            match ch {
                                                'C' => v.push(app::state::Status::Correct),
                                                'I' => v.push(app::state::Status::Incorrect),
                                                _ => v.push(app::state::Status::Untyped),
                                            }
                                        }
                                        a.status = v;
                                    }
                                    if let Some(craw) = corrected_opt {
                                        let mut cv = Vec::new();
                                        for ch in craw.chars() {
                                            cv.push(ch == '1');
                                        }
                                        a.corrected = cv;
                                    }
                                    use std::time::{Duration, Instant};
                                    let dur = Duration::from_millis(duration_ms as u64);
                                    a.start = Some(Instant::now() - dur);
                                    a.correct_timestamps = Vec::new();
                                    if a.correct_chars > 0 {
                                        for i in 0..a.correct_chars {
                                            let frac = (i as f64) / (a.correct_chars as f64);
                                            let offset = frac * (dur.as_secs_f64());
                                            a.correct_timestamps.push(Instant::now() - Duration::from_secs_f64(dur.as_secs_f64() - offset));
                                        }
                                    }
                                    a.samples = samples;
                                    Ok(a)
                                })() {
                                    terminal.draw(|f| draw_finished(f, &temp_app))?;
                                    loop {
                                        if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                                            if code == KeyCode::Esc {
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
                            }
                            continue 'main;
                        }
                        handle_leaderboard_key(code);
                    }

                    Mode::Settings => {
                        // Navigation keys for settings: mirror profile's behavior but operate on app.settings_cursor
                        match code {
                            KeyCode::Up | KeyCode::Char('k') => {
                                // decrement if possible (like fetch_update with checked_sub)
                                if app.settings_cursor > 0 {
                                    app.settings_cursor = app.settings_cursor.saturating_sub(1);
                                }
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
                            KeyCode::Home => {
                                app.settings_cursor = 0;
                                continue 'main;
                            }
                            KeyCode::End => {
                                app.settings_cursor = usize::MAX / 2;
                                continue 'main;
                            }
                            _ => {}
                        }

                        // Allow changing the currently selected setting with Left/Right
                        if code == KeyCode::Left || code == KeyCode::Right {
                            let left = code == KeyCode::Left;
                            // number of settings rows (must match draw_settings ordering)
                            let total = 11usize;
                            let sel = if total > 0 {
                                cmp::min(app.settings_cursor, total - 1)
                            } else {
                                0
                            };
                            match sel {
                                0 => {
                                    app.show_mode = !app.show_mode;
                                }
                                1 => {
                                    app.show_value = !app.show_value;
                                }
                                2 => {
                                    app.show_state = !app.show_state;
                                }
                                3 => {
                                    app.show_speed = !app.show_speed;
                                }
                                4 => {
                                    app.show_timer = !app.show_timer;
                                }
                                5 => {
                                    app.show_text = !app.show_text;
                                }
                                6 => {
                                    app.show_keyboard = !app.show_keyboard;
                                }
                                7 => {
                                    app.keyboard_layout = match app.keyboard_layout {
                                        crate::app::state::KeyboardLayout::Qwerty => {
                                            if left {
                                                crate::app::state::KeyboardLayout::Qwertz
                                            } else {
                                                crate::app::state::KeyboardLayout::Azerty
                                            }
                                        }
                                        crate::app::state::KeyboardLayout::Azerty => {
                                            if left {
                                                crate::app::state::KeyboardLayout::Qwerty
                                            } else {
                                                crate::app::state::KeyboardLayout::Dvorak
                                            }
                                        }
                                        crate::app::state::KeyboardLayout::Dvorak => {
                                            if left {
                                                crate::app::state::KeyboardLayout::Azerty
                                            } else {
                                                crate::app::state::KeyboardLayout::Qwertz
                                            }
                                        }
                                        crate::app::state::KeyboardLayout::Qwertz => {
                                            if left {
                                                crate::app::state::KeyboardLayout::Dvorak
                                            } else {
                                                crate::app::state::KeyboardLayout::Qwerty
                                            }
                                        }
                                    };
                                    let _ = crate::app::config::write_keyboard_layout(
                                        match app.keyboard_layout {
                                            crate::app::state::KeyboardLayout::Qwerty => "qwerty",
                                            crate::app::state::KeyboardLayout::Azerty => "azerty",
                                            crate::app::state::KeyboardLayout::Dvorak => "dvorak",
                                            crate::app::state::KeyboardLayout::Qwertz => "qwertz",
                                        },
                                    );
                                }
                                8 => {
                                    let presets = crate::themes_presets::preset_names();
                                    if !presets.is_empty() {
                                        // find current index
                                        let mut idx = 0usize;
                                        for (i, &n) in presets.iter().enumerate() {
                                            if let Some(p) = crate::themes_presets::theme_by_name(n)
                                            {
                                                if p == app.theme {
                                                    idx = i;
                                                    break;
                                                }
                                            }
                                        }
                                        if left {
                                            idx = (idx + presets.len() - 1) % presets.len();
                                        } else {
                                            idx = (idx + 1) % presets.len();
                                        }
                                        if let Some(next) =
                                            crate::themes_presets::theme_by_name(presets[idx])
                                        {
                                            app.theme = next;
                                            let _ = app.theme.save_to_config();
                                        }
                                    }
                                }
                                9 => {
                                    // Keyboard switch: cycle available switch samples
                                    let list = crate::audio::list_switches();
                                    if !list.is_empty() {
                                        let mut idx = list
                                            .iter()
                                            .position(|s| s == &app.keyboard_switch)
                                            .unwrap_or(0);
                                        if left {
                                            idx = (idx + list.len() - 1) % list.len();
                                        } else {
                                            idx = (idx + 1) % list.len();
                                        }
                                        app.keyboard_switch = list[idx].clone();
                                        let _ = crate::app::config::write_keyboard_switch(
                                            &app.keyboard_switch,
                                        );
                                    }
                                }
                                10 => {
                                    app.audio_enabled = !app.audio_enabled;
                                    let _ =
                                        crate::app::config::write_audio_enabled(app.audio_enabled);
                                }
                                _ => {}
                            }
                            continue 'main;
                        }

                        // Allow cycling keyboard layout with 'l'
                        if code == KeyCode::Char('l') {
                            app.keyboard_layout = match app.keyboard_layout {
                                crate::app::state::KeyboardLayout::Qwerty => {
                                    crate::app::state::KeyboardLayout::Azerty
                                }
                                crate::app::state::KeyboardLayout::Azerty => {
                                    crate::app::state::KeyboardLayout::Dvorak
                                }
                                crate::app::state::KeyboardLayout::Dvorak => {
                                    crate::app::state::KeyboardLayout::Qwertz
                                }
                                crate::app::state::KeyboardLayout::Qwertz => {
                                    crate::app::state::KeyboardLayout::Qwerty
                                }
                            };
                            // Clear any pressed key highlight so it doesn't point to an unrelated key
                            keyboard.pressed_key = None;
                            // Persist layout choice to config
                            let _ = crate::app::config::write_keyboard_layout(
                                match app.keyboard_layout {
                                    crate::app::state::KeyboardLayout::Qwerty => "qwerty",
                                    crate::app::state::KeyboardLayout::Azerty => "azerty",
                                    crate::app::state::KeyboardLayout::Dvorak => "dvorak",
                                    crate::app::state::KeyboardLayout::Qwertz => "qwertz",
                                },
                            );
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
                                if let Some(next) =
                                    crate::themes_presets::theme_by_name(presets[idx])
                                {
                                    app.theme = next;
                                }
                            }
                            continue 'main;
                        }

                        // Cycle available keyboard switch samples with 'k' or 'K'
                        if matches!(code, KeyCode::Char('k') | KeyCode::Char('K')) {
                            let list = crate::audio::list_switches();
                            if !list.is_empty() {
                                // find current index
                                let mut idx = list
                                    .iter()
                                    .position(|s| s == &app.keyboard_switch)
                                    .unwrap_or(0);
                                idx = (idx + 1) % list.len();
                                app.keyboard_switch = list[idx].clone();
                                // persist selection
                                let _ =
                                    crate::app::config::write_keyboard_switch(&app.keyboard_switch);
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
