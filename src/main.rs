// src/main.rs

use std::{
    error::Error,
    fs::{self, File},
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
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

/// Per-character typing status
#[derive(Clone, Copy)]
enum Status {
    Untyped,
    Correct,
    Incorrect,
}

/// The application state
struct App {
    target: String,
    status: Vec<Status>,
    start: Instant,
}

impl App {
    fn new(target: String) -> Self {
        let len = target.chars().count();
        Self {
            target,
            status: vec![Status::Untyped; len],
            start: Instant::now(),
        }
    }

    /// Handle one keypress
    fn on_key(&mut self, key: char) {
        if let Some(i) = self.status.iter().position(|s| matches!(s, Status::Untyped)) {
            let expected = self.target.chars().nth(i).unwrap();
            self.status[i] = if key == expected {
                Status::Correct
            } else {
                Status::Incorrect
            };
        }
    }

    /// Have we finished the sentence?
    fn is_done(&self) -> bool {
        !self.status.iter().any(|s| matches!(s, Status::Untyped))
    }

    /// Seconds elapsed since start
    fn elapsed_secs(&self) -> u64 {
        self.start.elapsed().as_secs()
    }

    /// Compute WPM = (typed_chars/5) / (minutes elapsed)
    fn wpm(&self) -> f64 {
        let typed = self
            .status
            .iter()
            .filter(|&&s| s != Status::Untyped)
            .count() as f64;
        let mins = self.start.elapsed().as_secs_f64() / 60.0;
        if mins > 0.0 {
            (typed / 5.0) / mins
        } else {
            0.0
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // 1) Load word list → generate sentence
    let words = load_words()?;
    let sentence = generate_sentence(&words, 30);
    let mut app = App::new(sentence);

    // 2) Enter raw mode + alternate screen
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 3) Main event loop with periodic redraws
    let tick_rate = Duration::from_millis(200);
    let mut last_tick = Instant::now();

    loop {
        // Draw the UI
        terminal.draw(|f| {
            let size = f.size();
            // Four vertical chunks: header, WPM, text, footer
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([
                    Constraint::Length(2), // header
                    Constraint::Length(1), // wpm
                    Constraint::Min(3),    // text
                    Constraint::Length(1), // footer
                ])
                .split(size);

            // Header
            let header = Paragraph::new("Type the sentence below as fast and accurately as you can!  (Esc or Ctrl+C to quit)")
                .block(Block::default().borders(Borders::ALL).title("Term-Typist"));
            f.render_widget(header, chunks[0]);

            // WPM display
            let wpm_para = Paragraph::new(format!("WPM: {:.1}", app.wpm()))
                .block(Block::default().borders(Borders::ALL).title("Speed"));
            f.render_widget(wpm_para, chunks[1]);

            // Sentence text, character-by-character colored
            let spans: Vec<Span> = app
                .target
                .chars()
                .zip(&app.status)
                .map(|(ch, st)| {
                    let style = match st {
                        Status::Untyped => Style::default().fg(Color::White),
                        Status::Correct => Style::default().fg(Color::Green),
                        Status::Incorrect => Style::default().fg(Color::Red),
                    };
                    Span::styled(ch.to_string(), style)
                })
                .collect();
            let text = Paragraph::new(Spans::from(spans))
                .block(Block::default().borders(Borders::ALL).title("Text"));
            f.render_widget(text, chunks[2]);

            // Footer with elapsed time
            let footer = Paragraph::new(format!("Elapsed: {}s", app.elapsed_secs()))
                .block(Block::default().borders(Borders::ALL));
            f.render_widget(footer, chunks[3]);
        })?;

        // Handle input (with tick-based timeout)
        let timeout = tick_rate.checked_sub(last_tick.elapsed()).unwrap_or_default();
        if event::poll(timeout)? {
            if let Event::Key(KeyEvent { code, modifiers }) = event::read()? {
                // Ctrl+C or Esc to quit
                if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
                    break;
                }
                if code == KeyCode::Esc {
                    break;
                }
                // Normal character
                if let KeyCode::Char(c) = code {
                    app.on_key(c);
                    if app.is_done() {
                        break;
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    // 4) Restore terminal and exit
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen)?;
    println!("Final Time: {}s  ┃  Final WPM: {:.1}", app.elapsed_secs(), app.wpm());
    Ok(())
}

/// Load word list from `$XDG_DATA_HOME/term-typist/words/words.txt`,
/// falling back to an embedded `words/words.txt` at compile time.
fn load_words() -> io::Result<Vec<String>> {
    let mut data_path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    data_path.push("term-typist");
    data_path.push("words");
    data_path.push("words.txt");

    if data_path.exists() {
        let file = File::open(data_path)?;
        let reader = BufReader::new(file);
        Ok(reader.lines().flatten().collect())
    } else {
        // Embedded fallback
        let embedded = include_str!("../words/words.txt");
        Ok(embedded.lines().map(str::to_string).collect())
    }
}

/// Pick `n` random words and join them into a sentence
fn generate_sentence(words: &[String], n: usize) -> String {
    let mut rng = rand::thread_rng();
    words
        .choose_multiple(&mut rng, n)
        .cloned()
        .collect::<Vec<_>>()
        .join(" ")
}
