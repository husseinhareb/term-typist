use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::PathBuf,
    time::{Duration, Instant},
};
use rand::seq::SliceRandom;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
    Terminal,
};

#[derive(PartialEq, Clone, Copy)]
enum Status {
    Untyped,
    Correct,
    Incorrect,
}

/// Two modes: View = nav only; Insert = typing & timer running
#[derive(PartialEq, Clone, Copy)]
enum Mode {
    View,
    Insert,
}

struct App {
    target: String,
    status: Vec<Status>,
    start: Option<Instant>,
    typed_count: usize,
    selected_tab: usize,   // 0=Time, 1=Words, 2=Zen
    selected_value: usize, // index into current mode's options
    mode: Mode,
}

impl App {
    fn new(target: String) -> Self {
        let len = target.chars().count();
        App {
            target,
            status: vec![Status::Untyped; len],
            start: None,
            typed_count: 0,
            selected_tab: 0,
            selected_value: 0,
            mode: Mode::View,
        }
    }

    /// Called once per character in Insert mode
    fn on_key(&mut self, key: char) {
        self.typed_count += 1;
        if let Some(i) = self.status.iter().position(|&s| s == Status::Untyped) {
            let expected = self.target.chars().nth(i).unwrap();
            self.status[i] =
                if key == expected { Status::Correct } else { Status::Incorrect };
        }
    }

    fn is_done(&self) -> bool {
        !self.status.iter().any(|&s| s == Status::Untyped)
    }

    /// Seconds elapsed since Enter was pressed
    fn elapsed_secs(&self) -> u64 {
        self.start.map(|s| s.elapsed().as_secs()).unwrap_or(0)
    }

    fn wpm(&self) -> f64 {
        if let Some(s) = self.start {
            let mins = s.elapsed().as_secs_f64() / 60.0;
            if mins > 0.0 {
                (self.typed_count as f64 / 5.0) / mins
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Numeric options for each mode
    fn current_options(&self) -> &'static [u16] {
        match self.selected_tab {
            0 => &[15, 30, 60, 100],  // Time (seconds)
            1 => &[10, 25, 50, 100],  // Words (count)
            _ => &[],                 // Zen: no options
        }
    }
}

/// Library entry point (called from `main.rs`)
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    // 1) Prepare test sentence
    let words = load_words()?;
    let sentence = generate_sentence(&words, 30);
    let mut app = App::new(sentence);

    // 2) Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 3) Main loop with periodic redraws
    let tick_rate = Duration::from_millis(200);
    let mut last_tick = Instant::now();

    'mainloop: loop {
        terminal.draw(|f| {
            let size = f.size();
            // Vertical layout: navbar, speed, text, footer
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Length(3),
                    Constraint::Min(3),
                    Constraint::Length(1),
                ])
                .split(size);

            // --- Navbar ---
            let nav = chunks[0];
            let nav_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Min(10)])
                .split(nav);

            // Mode tabs
            let titles = ["Time", "Words", "Zen"]
                .iter()
                .map(|t| Spans::from(*t))
                .collect::<Vec<_>>();
            let tabs = Tabs::new(titles)
                .block(Block::default().borders(Borders::ALL).title("Mode"))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD))
                .divider(Span::raw(" "));
            f.render_widget(tabs.select(app.selected_tab), nav_chunks[0]);

            // Separator + numeric selectors
            let opts = app.current_options();
            let mut spans = vec![Span::raw("| ")];
            for (i, &val) in opts.iter().enumerate() {
                let s = val.to_string();
                if i == app.selected_value {
                    spans.push(Span::styled(s, Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)));
                } else {
                    spans.push(Span::raw(s));
                }
                spans.push(Span::raw(" "));
            }
            let opts_para = Paragraph::new(Spans::from(spans))
                .block(Block::default().borders(Borders::ALL).title("Value"));
            f.render_widget(opts_para, nav_chunks[1]);

            // --- Speed panel: WPM | Timer ---
            let speed_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[1]);

            // WPM
            let wpm_para = Paragraph::new(format!("WPM: {:.1}", app.wpm()))
                .block(Block::default().borders(Borders::ALL).title("WPM"));
            f.render_widget(wpm_para, speed_chunks[0]);

            // Timer / counter
            let timer_text = if app.mode == Mode::Insert {
                match app.selected_tab {
                    1 => {
                        // Words: how many typed
                        let idx = app.status.iter()
                            .position(|&s| s == Status::Untyped)
                            .unwrap_or(app.status.len());
                        let typed = app.target.chars().take(idx).collect::<String>();
                        let count = typed.split_whitespace().count();
                        let total = app.current_options()[app.selected_value] as usize;
                        format!("Words: {}/{}", count, total)
                    }
                    0 => {
                        // Time: remaining
                        let tot = app.current_options()[app.selected_value] as i64;
                        let rem = (tot - app.elapsed_secs() as i64).max(0);
                        format!("Time: {}s", rem)
                    }
                    _ => String::new(),
                }
            } else {
                // View mode prompt
                "Press Enter to start".into()
            };
            let timer_para = Paragraph::new(timer_text)
                .block(Block::default().borders(Borders::ALL).title("Timer"));
            f.render_widget(timer_para, speed_chunks[1]);

            // --- Text pane (with wrapping) ---
            let current = app.status.iter()
                .position(|&s| s == Status::Untyped)
                .unwrap_or(app.status.len());
            let spans: Vec<Span> = app.target.chars()
                .zip(app.status.iter().cloned())
                .enumerate()
                .map(|(i, (ch, st))| {
                    let base = match st {
                        Status::Untyped => Style::default().fg(Color::White),
                        Status::Correct => Style::default().fg(Color::Green),
                        Status::Incorrect => Style::default().fg(Color::Red),
                    };
                    if i == current && app.mode == Mode::Insert {
                        Span::styled(ch.to_string(), base
                            .bg(Color::Yellow)
                            .fg(Color::Black)
                            .add_modifier(Modifier::BOLD))
                    } else {
                        Span::styled(ch.to_string(), base)
                    }
                })
                .collect();
            let text_para = Paragraph::new(Spans::from(spans))
                .block(Block::default().borders(Borders::ALL).title("Text"))
                .wrap(Wrap { trim: true });
            f.render_widget(text_para, chunks[2]);

            // --- Footer (total elapsed) ---
            let footer_text = if app.mode == Mode::Insert {
                format!("Elapsed: {}s", app.elapsed_secs())
            } else {
                "".into()
            };
            let footer = Paragraph::new(footer_text)
                .block(Block::default().borders(Borders::ALL).title("Elapsed"));
            f.render_widget(footer, chunks[3]);
        })?;

        // Handle input & ticking
        let timeout = tick_rate.checked_sub(last_tick.elapsed()).unwrap_or_default();
        if event::poll(timeout)? {
            if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                // Quit
                if code == KeyCode::Esc
                    || (code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL))
                {
                    break 'mainloop;
                }
                match code {
                    KeyCode::Char('1') => { app.selected_tab = 0; app.selected_value = 0; }
                    KeyCode::Char('2') => { app.selected_tab = 1; app.selected_value = 0; }
                    KeyCode::Char('3') => { app.selected_tab = 2; app.selected_value = 0; }
                    KeyCode::Left => {
                        if app.selected_value > 0 && !app.current_options().is_empty() {
                            app.selected_value -= 1;
                        }
                    }
                    KeyCode::Right => {
                        let len = app.current_options().len();
                        if app.selected_value + 1 < len {
                            app.selected_value += 1;
                        }
                    }
                    KeyCode::Enter if app.mode == Mode::View => {
                        app.mode = Mode::Insert;
                        app.start = Some(Instant::now());
                    }
                    KeyCode::Char(c) if app.mode == Mode::Insert => {
                        app.on_key(c);
                        if app.is_done() {
                            break 'mainloop;
                        }
                    }
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen)?;
    println!(
        "Final Time: {}s  ┃  Final WPM: {:.1}",
        app.elapsed_secs(),
        app.wpm()
    );
    Ok(())
}

/// Load words from config dir or fallback to embedded
fn load_words() -> io::Result<Vec<String>> {
    let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("term-typist/words/words.txt");
    if path.exists() {
        let f = File::open(path)?;
        let reader = BufReader::new(f);
        return Ok(reader.lines().flatten().collect());
    }
    let embedded = include_str!("../words/words.txt");
    Ok(embedded.lines().map(str::to_string).collect())
}

/// Generate a random sentence of `n` words
fn generate_sentence(words: &[String], n: usize) -> String {
    let mut rng = rand::thread_rng();
    words
        .choose_multiple(&mut rng, n)
        .cloned()
        .collect::<Vec<_>>()
        .join(" ")
}
