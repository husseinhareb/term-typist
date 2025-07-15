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
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
    Frame, Terminal,
};

mod graph;
mod wpm;
use wpm::{accuracy, elapsed_seconds_since_start, net_wpm};

#[derive(PartialEq, Clone, Copy)]
enum Status {
    Untyped,
    Correct,
    Incorrect,
}

#[derive(PartialEq, Clone, Copy)]
enum Mode {
    View,
    Insert,
    Finished,
}

struct App {
    target: String,
    status: Vec<Status>,
    start: Option<Instant>,
    correct_chars: usize,
    incorrect_chars: usize,
    last_correct: Instant,
    locked: bool,
    free_text: String,
    selected_tab: usize,
    selected_value: usize,
    mode: Mode,
    samples: Vec<(u64, f64)>,
}

impl App {
    fn new(target: String) -> Self {
        let len = target.chars().count();
        let now = Instant::now();
        App {
            target,
            status: vec![Status::Untyped; len],
            start: None,
            correct_chars: 0,
            incorrect_chars: 0,
            last_correct: now,
            locked: false,
            free_text: String::new(),
            selected_tab: 0,
            selected_value: 0,
            mode: Mode::View,
            samples: Vec::new(),
        }
    }

    fn on_key(&mut self, key: char) {
        if self.selected_tab == 2 {
            // Zen mode
            self.free_text.push(key);
            self.correct_chars += 1;
            self.last_correct = Instant::now();
            return;
        }

        if self.locked {
            if let Some(i) = self.status.iter().position(|&s| s == Status::Untyped) {
                let expected = self.target.chars().nth(i).unwrap();
                if key != expected {
                    return;
                }
                self.locked = false;
            }
        }

        if let Some(i) = self.status.iter().position(|&s| s == Status::Untyped) {
            let expected = self.target.chars().nth(i).unwrap();
            let correct = key == expected;
            self.status[i] = if correct {
                self.correct_chars += 1;
                self.last_correct = Instant::now();
                Status::Correct
            } else {
                self.incorrect_chars += 1;
                Status::Incorrect
            };
        }

        if let Some(_) = self.start {
            if Instant::now().duration_since(self.last_correct) >= Duration::from_secs(1) {
                self.locked = true;
            }
        }
    }

    fn elapsed_secs(&self) -> u64 {
        self.start.map(|s| s.elapsed().as_secs()).unwrap_or(0)
    }

    fn current_options(&self) -> &'static [u16] {
        match self.selected_tab {
            0 => &[15, 30, 60, 100],
            1 => &[10, 25, 50, 100],
            _ => &[],
        }
    }
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(200);
    let mut last_tick = Instant::now();
    let mut last_sample = 0;

    let make_app = || {
        let words = load_words().expect("load_words failed");
        let sentence = generate_sentence(&words, 30);
        App::new(sentence)
    };
    let mut app = make_app();

    'main: loop {
        terminal.draw(|f| {
            // ===== top UI (mode & options) =====
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(3)])
                .split(size);

            let nav = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Min(10)])
                .split(chunks[0]);

            let titles = ["Time", "Words", "Zen"]
                .iter()
                .map(|t| Spans::from(*t))
                .collect::<Vec<_>>();
            let tabs = Tabs::new(titles)
                .block(Block::default().borders(Borders::ALL).title("Mode"))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .divider(Span::raw(" "));
            f.render_widget(tabs.select(app.selected_tab), nav[0]);

            let mut spans = vec![Span::raw("| ")];
            for (i, &v) in app.current_options().iter().enumerate() {
                let s = v.to_string();
                if i == app.selected_value {
                    spans.push(Span::styled(
                        s,
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                    ));
                } else {
                    spans.push(Span::raw(s));
                }
                spans.push(Span::raw(" "));
            }
            let opts = Paragraph::new(Spans::from(spans))
                .block(Block::default().borders(Borders::ALL).title("Value"));
            f.render_widget(opts, nav[1]);

            // ===== speed & timer =====
            let st = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[1]);

            let (net, acc) = if let Some(start) = app.start {
                let secs = elapsed_seconds_since_start(start);
                (
                    net_wpm(app.correct_chars, app.incorrect_chars, secs),
                    accuracy(app.correct_chars, app.incorrect_chars),
                )
            } else {
                (0.0, 0.0)
            };
            let speed_txt = if app.mode == Mode::View {
                "WPM: --  Acc: --%".into()
            } else {
                format!("WPM: {:.1}  Acc: {:.0}%", net, acc)
            };
            let speed = Paragraph::new(speed_txt)
                .block(Block::default().borders(Borders::ALL).title("Speed"));
            f.render_widget(speed, st[0]);

            let timer_txt = match app.mode {
                Mode::View => "Press Enter to start".into(),
                Mode::Insert => match app.selected_tab {
                    0 => {
                        let rem = (app.current_options()[app.selected_value] as i64
                            - app.elapsed_secs() as i64)
                            .max(0);
                        format!("Time left: {}s", rem)
                    }
                    1 => {
                        let idx = app
                            .status
                            .iter()
                            .position(|&s| s == Status::Untyped)
                            .unwrap_or(app.status.len());
                        let typed = app.target.chars().take(idx).collect::<String>();
                        format!(
                            "Words: {}/{}",
                            typed.split_whitespace().count(),
                            app.current_options()[app.selected_value]
                        )
                    }
                    _ => "Zen mode".into(),
                },
                Mode::Finished => {
                    let elapsed = app.elapsed_secs() as f64;
                    let net = net_wpm(app.correct_chars, app.incorrect_chars, elapsed);
                    format!("🏁 Done! {}s • {:.1} WPM  Esc=Restart", app.elapsed_secs(), net)
                }
            };
            let timer = Paragraph::new(timer_txt)
                .block(Block::default().borders(Borders::ALL).title("Timer"));
            f.render_widget(timer, st[1]);

            // ===== main text or zen =====
            if app.selected_tab == 2 {
                let free = app.free_text.chars().map(|c| Span::raw(c.to_string())).collect::<Vec<_>>();
                f.render_widget(
                    Paragraph::new(Spans::from(free))
                        .block(Block::default().borders(Borders::ALL).title("Zen"))
                        .wrap(Wrap { trim: true }),
                    chunks[2],
                );
            } else {
                let cur = app
                    .status
                    .iter()
                    .position(|&s| s == Status::Untyped)
                    .unwrap_or(app.status.len());
                let spans = app
                    .target
                    .chars()
                    .zip(app.status.iter().cloned())
                    .enumerate()
                    .map(|(i, (ch, st))| {
                        let base = match st {
                            Status::Untyped => Style::default().fg(Color::White),
                            Status::Correct => Style::default().fg(Color::Green),
                            Status::Incorrect => Style::default().fg(Color::Red),
                        };
                        if i == cur && app.mode == Mode::Insert {
                            Span::styled(
                                ch.to_string(),
                                base.bg(Color::Yellow)
                                    .fg(Color::Black)
                                    .add_modifier(Modifier::BOLD),
                            )
                        } else {
                            Span::styled(ch.to_string(), base)
                        }
                    })
                    .collect::<Vec<_>>();
                f.render_widget(
                    Paragraph::new(Spans::from(spans))
                        .block(Block::default().borders(Borders::ALL).title("Text"))
                        .wrap(Wrap { trim: true }),
                    chunks[2],
                );
            }
        })?;

        // ===== input & state machine =====
        let timeout = tick_rate.checked_sub(last_tick.elapsed()).unwrap_or_default();
        if event::poll(timeout)? {
            if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                // Ctrl-C to quit
                if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
                    break 'main;
                }
                // Esc to restart
                if code == KeyCode::Esc && modifiers.is_empty() {
                    app = make_app();
                    last_sample = 0;
                    continue 'main;
                }

                match app.mode {
                    Mode::View => {
                        if code == KeyCode::Enter {
                            let now = Instant::now();
                            app.start = Some(now);
                            app.last_correct = now;
                            app.locked = false;
                            app.mode = Mode::Insert;
                            if app.selected_tab == 2 {
                                app.free_text.clear();
                                app.correct_chars = 0;
                                app.incorrect_chars = 0;
                            }
                        } else {
                            handle_nav(&mut app, code);
                        }
                    }
                    Mode::Insert => {
                        if let KeyCode::Char(c) = code {
                            app.on_key(c);
                            // auto-finish by time
                            if app.selected_tab == 0
                                && app.elapsed_secs() >= app.current_options()[app.selected_value] as u64
                            {
                                app.mode = Mode::Finished;
                            }
                            // auto-finish by words
                            if app.selected_tab == 1 {
                                let idx = app
                                    .status
                                    .iter()
                                    .position(|&s| s == Status::Untyped)
                                    .unwrap_or(app.status.len());
                                let count = app
                                    .target
                                    .chars()
                                    .take(idx)
                                    .collect::<String>()
                                    .split_whitespace()
                                    .count();
                                if count >= app.current_options()[app.selected_value] as usize {
                                    app.mode = Mode::Finished;
                                }
                            }
                        } else {
                            handle_nav(&mut app, code);
                        }
                    }
                    Mode::Finished => {
                        // draw chart + stats
                        terminal.draw(|f| draw_finished(f, &app)).unwrap();
                        // wait for Esc or Ctrl-C
                        loop {
                            if let Event::Key(KeyEvent { code, modifiers, .. }) =
                                event::read()?
                            {
                                if code == KeyCode::Esc && modifiers.is_empty() {
                                    app = make_app();
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
                }
            }
        }

        // sampling
        if app.mode == Mode::Insert {
            let sec = app.elapsed_secs();
            if sec > last_sample {
                last_sample = sec;
                if let Some(start) = app.start {
                    let secs_f = elapsed_seconds_since_start(start);
                    let net = net_wpm(app.correct_chars, app.incorrect_chars, secs_f);
                    app.samples.push((sec, net));
                }
            }
        }

        last_tick = Instant::now();
    }

    // cleanup on normal exit
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

fn draw_finished<B: Backend>(f: &mut Frame<B>, app: &App) {
    let size = f.size();
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(size);

    // left: WPM graph
    graph::draw_wpm_chart(f, cols[0], &app.samples);

    // right: summary stats
    let elapsed_u = app.elapsed_secs();
    let elapsed_f = elapsed_u as f64;
    let net = net_wpm(app.correct_chars, app.incorrect_chars, elapsed_f);
    let acc = accuracy(app.correct_chars, app.incorrect_chars);
    let raw = app.correct_chars + app.incorrect_chars;
    let errs = app.incorrect_chars;
    let test_type = match app.selected_tab {
        0 => format!("time {}s", app.current_options()[app.selected_value]),
        1 => format!("words {}", app.current_options()[app.selected_value]),
        _ => "zen".into(),
    };
    // simple consistency
    let consistency = {
        let vs: Vec<f64> = app.samples.iter().map(|&(_, w)| w).collect();
        if vs.len() > 1 {
            let mean = vs.iter().sum::<f64>() / vs.len() as f64;
            let var = vs.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / vs.len() as f64;
            let std = var.sqrt();
            format!("{:.0}%", ((1.0 - std / (mean + 1.0)).max(0.0)) * 100.0)
        } else {
            "--%".into()
        }
    };

    let items = vec![
        Spans::from(vec![
            Span::styled("WPM  ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{:.0}", net), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Spans::from(vec![
            Span::styled("ACC  ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{:.0}%", acc), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        ]),
        Spans::from(vec![
            Span::styled("RAW  ", Style::default().fg(Color::Gray)),
            Span::raw(raw.to_string()),
        ]),
        Spans::from(vec![
            Span::styled("ERR  ", Style::default().fg(Color::Gray)),
            Span::raw(errs.to_string()),
        ]),
        Spans::from(vec![
            Span::styled("TYPE ", Style::default().fg(Color::Gray)),
            Span::raw(test_type),
        ]),
        Spans::from(vec![
            Span::styled("CONS ", Style::default().fg(Color::Gray)),
            Span::raw(consistency),
        ]),
        Spans::from(vec![
            Span::styled("TIME ", Style::default().fg(Color::Gray)),
            Span::raw(format!("{}s", elapsed_u)),
        ]),
    ];

    let para = Paragraph::new(items)
        .block(Block::default().borders(Borders::ALL).title("Summary"))
        .wrap(Wrap { trim: true });

    f.render_widget(para, cols[1]);
}

fn handle_nav(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('1') => app.selected_tab = 0,
        KeyCode::Char('2') => app.selected_tab = 1,
        KeyCode::Char('3') => app.selected_tab = 2,
        KeyCode::Left if app.selected_value > 0 => app.selected_value -= 1,
        KeyCode::Right if app.selected_value + 1 < app.current_options().len() => {
            app.selected_value += 1
        }
        _ => {}
    }
}

fn load_words() -> io::Result<Vec<String>> {
    let mut path = dirs::data_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("term-typist/words/words.txt");
    if path.exists() {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        return reader.lines().collect();
    }
    Ok(include_str!("../words/words.txt").lines().map(str::to_string).collect())
}

fn generate_sentence(words: &[String], n: usize) -> String {
    let mut rng = rand::thread_rng();
    words.choose_multiple(&mut rng, n).cloned().collect::<Vec<_>>().join(" ")
}
