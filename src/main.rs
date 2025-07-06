// src/main.rs

use std::{
    error::Error,
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
    widgets::{Block, Borders, Paragraph, Tabs},
    Terminal,
};

#[derive(PartialEq, Clone, Copy)]
enum Status { Untyped, Correct, Incorrect }

struct App {
    target: String,
    status: Vec<Status>,
    start: Instant,
    typed_count: usize,
    selected_tab: usize,        // 0=Time,1=Words,2=Zen
    selected_value: usize,      // index into the current mode's options
}

impl App {
    fn new(target: String) -> Self {
        let len = target.chars().count();
        Self {
            target,
            status: vec![Status::Untyped; len],
            start: Instant::now(),
            typed_count: 0,
            selected_tab: 0,
            selected_value: 0,
        }
    }

    fn on_key(&mut self, key: char) {
        self.typed_count += 1;
        if let Some(i) = self.status.iter().position(|&s| s == Status::Untyped) {
            let expected = self.target.chars().nth(i).unwrap();
            self.status[i] = if key == expected { Status::Correct } else { Status::Incorrect };
        }
    }

    fn is_done(&self) -> bool {
        !self.status.iter().any(|&s| s == Status::Untyped)
    }

    fn elapsed_secs(&self) -> u64 {
        self.start.elapsed().as_secs()
    }

    fn wpm(&self) -> f64 {
        let mins = self.start.elapsed().as_secs_f64() / 60.0;
        if mins > 0.0 { (self.typed_count as f64 / 5.0) / mins } else { 0.0 }
    }

    /// Returns the numeric options for the current tab
    fn current_options(&self) -> &'static [u16] {
        match self.selected_tab {
            0 => &[15, 30, 60, 100],   // Time
            1 => &[10, 25, 50, 100],   // Words
            _ => &[],                  // Zen has none
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let words = load_words()?;
    let sentence = generate_sentence(&words, 30);
    let mut app = App::new(sentence);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(200);
    let mut last_tick = Instant::now();

    'mainloop: loop {
        terminal.draw(|f| {
            let size = f.size();
            // Reserve: navbar (3), speed (3), text (min3), footer (1)
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(3),
                              Constraint::Length(3),
                              Constraint::Min(3),
                              Constraint::Length(1)])
                .split(size);

            // --- Navbar with Tabs + Numeric Options ---
            let navbar = chunks[0];
            let inner = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Min(10)])
                .split(navbar);

            // 1) Mode tabs
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
            f.render_widget(tabs.select(app.selected_tab), inner[0]);

            // 2) Separator + numeric options
            let opts = app.current_options();
            let mut spans = Vec::<Span>::new();
            spans.push(Span::raw("| "));
            for (i, &val) in opts.iter().enumerate() {
                if i == app.selected_value {
                    spans.push(Span::styled(
                        val.to_string(),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    ));
                } else {
                    spans.push(Span::raw(val.to_string()));
                }
                spans.push(Span::raw(" "));
            }
            let opts_para = Paragraph::new(Spans::from(spans))
                .block(Block::default().borders(Borders::ALL).title("Value"));
            f.render_widget(opts_para, inner[1]);

            // --- Speed panel ---
            let wpm_para = Paragraph::new(format!("WPM: {:.1}", app.wpm()))
                .block(Block::default().borders(Borders::ALL).title("Speed"));
            f.render_widget(wpm_para, chunks[1]);

            // --- Typing text with marker ---
            let current = app.status.iter()
                .position(|&s| s == Status::Untyped)
                .unwrap_or(app.status.len());
            let spans: Vec<Span> = app.target.chars()
                .zip(app.status.iter().cloned())
                .enumerate()
                .map(|(idx, (ch, st))| {
                    let base = match st {
                        Status::Untyped   => Style::default().fg(Color::White),
                        Status::Correct   => Style::default().fg(Color::Green),
                        Status::Incorrect => Style::default().fg(Color::Red),
                    };
                    let style = if idx == current {
                        base.patch(Style::default()
                            .fg(Color::Black)
                            .bg(Color::Yellow)
                            .add_modifier(Modifier::BOLD))
                    } else {
                        base
                    };
                    Span::styled(ch.to_string(), style)
                })
                .collect();
            let text = Paragraph::new(Spans::from(spans))
                .block(Block::default().borders(Borders::ALL).title("Text"));
            f.render_widget(text, chunks[2]);

            // --- Footer ---
            let footer = Paragraph::new(format!("Elapsed: {}s", app.elapsed_secs()))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[3]);
        })?;

        // Input & tick
        let timeout = tick_rate.checked_sub(last_tick.elapsed()).unwrap_or_default();
        if event::poll(timeout)? {
            if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                // Quit
                if code == KeyCode::Esc
                   || (code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL)) {
                    break 'mainloop;
                }
                // Mode switch or typing/numeric adjust
                match code {
                    KeyCode::Char('1') => { app.selected_tab = 0; app.selected_value = 0; }
                    KeyCode::Char('2') => { app.selected_tab = 1; app.selected_value = 0; }
                    KeyCode::Char('3') => { app.selected_tab = 2; app.selected_value = 0; }
                    KeyCode::Left  => {
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
                    KeyCode::Char(c) => {
                        // Only type when in a mode that uses typing
                        if app.selected_tab != 2 {
                            app.on_key(c);
                            if app.is_done() { break 'mainloop; }
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

    // Restore and print final stats
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

/// Load words or embedded fallback
fn load_words() -> io::Result<Vec<String>> {
    let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("term-typist/words/words.txt");
    if path.exists() {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
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
