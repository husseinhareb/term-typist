use std::{io, time::{Duration, Instant}};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
};
use tui::{backend::CrosstermBackend, Terminal};

mod app;         // src/app/mod.rs → state.rs, input.rs, config.rs
mod ui;          // src/ui/mod.rs → draw.rs, keyboard.rs
mod graph;       // src/graph.rs
mod wpm;         // src/wpm.rs
mod generator;   // src/generator.rs

use app::state::{App, Mode};
use app::input::handle_nav;
use ui::draw::draw;
use ui::keyboard::Keyboard;
use wpm::{accuracy, elapsed_seconds_since_start, net_wpm};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    // — Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // — Timing & WPM cache
    let tick_rate = Duration::from_millis(200);
    let mut last_tick     = Instant::now();
    let mut last_sample   = 0;
    let mut last_wpm_update = Instant::now();
    let mut cached_net    = 0.0;
    let mut cached_acc    = 0.0;

    // — App factory (e.g. 30‑word sentence)
    let make_app = || {
        let sentence = generator::generate_random_sentence(30);
        App::new(sentence)
    };
    let mut app      = make_app();
    let mut keyboard = Keyboard::new();

    'main: loop {
        // — Throttle WPM/accuracy updates once per second
        if let Mode::Insert = app.mode {
            if app.start.is_some() && last_wpm_update.elapsed() >= Duration::from_secs(1) {
                let real_secs = elapsed_seconds_since_start(app.start.unwrap());
                let idle      = Instant::now().duration_since(app.last_input).as_secs_f64();
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

        // — Draw UI
        terminal.draw(|f| draw(f, &app, &keyboard, cached_net, cached_acc))?;

        // — Handle input & toggles
        let timeout = tick_rate.checked_sub(last_tick.elapsed()).unwrap_or_default();
        if event::poll(timeout)? {
            if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                // Ctrl‐C quits
                if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
                    break 'main;
                }
                // Esc restarts
                if code == KeyCode::Esc && modifiers.is_empty() {
                    app = make_app();
                    last_sample = 0;
                    continue 'main;
                }

                // Highlight key in on‑screen keyboard
                keyboard.handle_key(&code);

                // ── SHIFT+NUMBER PANEL TOGGLES ─────────────────────
                if modifiers.contains(KeyModifiers::SHIFT) {
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
                }

                // Global navigation (tabs & values)
                handle_nav(&mut app, code);

                // Mode‑specific input
                match app.mode {
                    Mode::View => {
                        if code == KeyCode::Enter {
                            app.mode = Mode::Insert;
                            let now = Instant::now();
                            app.start = Some(now);
                            app.last_input = now;
                            app.locked = false;
                        }
                    }
                    Mode::Insert => {
                        match code {
                            KeyCode::Char(c)     => app.on_key(c),
                            KeyCode::Backspace   => app.backspace(),
                            _                    => {}
                        }
                    }
                    Mode::Finished => {
                        // TODO: show summary & await Esc/Ctrl‑C
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
