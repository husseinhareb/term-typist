// src/lib.rs

use std::{ fs::File, io::{ self, BufRead, BufReader }, path::PathBuf, time::{ Duration, Instant } };
use rand::seq::SliceRandom;

use crossterm::{
    event::{ self, Event, KeyCode, KeyEvent, KeyModifiers },
    execute,
    terminal::{ disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen },
};
use tui::{
    backend::CrosstermBackend,
    layout::{ Constraint, Direction, Layout },
    style::{ Color, Modifier, Style },
    text::{ Span, Spans },
    widgets::{ Block, Borders, Paragraph, Tabs, Wrap },
    Terminal,
};

mod graph;
mod wpm;
use wpm::{ elapsed_seconds_since_start, net_wpm, accuracy };

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
    selected_tab: usize,
    selected_value: usize,
    mode: Mode,
    samples: Vec<(u64, f64)>,
}

impl App {
    fn new(target: String) -> Self {
        // compute length first, then move target into the struct
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
            selected_tab: 0,
            selected_value: 0,
            mode: Mode::View,
            samples: Vec::new(),
        }
    }

    fn on_key(&mut self, key: char) {
        // spam-lock logic
        if self.locked {
            if let Some(i) = self.status.iter().position(|&s| s == Status::Untyped) {
                let expected = self.target.chars().nth(i).unwrap();
                if key != expected {
                    return; // still locked
                }
                // unlock on correct
                self.locked = false;
            }
        }

        if let Some(i) = self.status.iter().position(|&s| s == Status::Untyped) {
            let expected = self.target.chars().nth(i).unwrap();
            let is_correct = key == expected;
            self.status[i] = if is_correct { Status::Correct } else { Status::Incorrect };
            if is_correct {
                self.correct_chars += 1;
                self.last_correct = Instant::now();
            } else {
                self.incorrect_chars += 1;
            }
        }

        // if testing started and 2s pass without a correct, lock
        if
            self.start.is_some() &&
            Instant::now().duration_since(self.last_correct) >= Duration::from_secs(1)
        {
            self.locked = true;
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
    let mut last_sample_sec = 0;

    // no `mut` needed here
    let make_app = || {
        let words = load_words().expect("load_words failed");
        let sentence = generate_sentence(&words, 30);
        App::new(sentence)
    };
    let mut app = make_app();

    'mainloop: loop {
        terminal.draw(|f| {
            let size = f.size();
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

            // Navbar
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

            let mut opts_spans = vec![Span::raw("| ")];
            for (i, &v) in app.current_options().iter().enumerate() {
                let txt = v.to_string();
                if i == app.selected_value {
                    opts_spans.push(
                        Span::styled(
                            txt,
                            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                        )
                    );
                } else {
                    opts_spans.push(Span::raw(txt));
                }
                opts_spans.push(Span::raw(" "));
            }
            let sel = Paragraph::new(Spans::from(opts_spans)).block(
                Block::default().borders(Borders::ALL).title("Value")
            );
            f.render_widget(sel, nav[1]);

            // Speed panel
            let speed = Layout::default()
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

            let speed_text = if app.mode == Mode::View {
                "WPM: --  Acc: --%".into()
            } else {
                format!("WPM: {:.1}  Acc: {:.0}%", net, acc)
            };
            f.render_widget(
                Paragraph::new(speed_text).block(
                    Block::default().borders(Borders::ALL).title("Speed")
                ),
                speed[0]
            );

            // Timer / count / summary
            let timer_txt = match app.mode {
                Mode::View => "Press Enter to start".into(),
                Mode::Insert =>
                    match app.selected_tab {
                        0 => {
                            let total = app.current_options()[app.selected_value] as i64;
                            let rem = (total - (app.elapsed_secs() as i64)).max(0);
                            format!("Time left: {}s", rem)
                        }
                        1 => {
                            let idx = app.status
                                .iter()
                                .position(|&s| s == Status::Untyped)
                                .unwrap_or(app.status.len());
                            let typed = app.target.chars().take(idx).collect::<String>();
                            let cnt = typed.split_whitespace().count();
                            format!("Words: {}/{}", cnt, app.current_options()[app.selected_value])
                        }
                        2 => "Zen mode".into(),
                        _ => String::new(),
                    }
                Mode::Finished =>
                    format!("🏁 Done! {}s • {:.1} WPM  Esc=Restart", app.elapsed_secs(), net),
            };
            f.render_widget(
                Paragraph::new(timer_txt).block(
                    Block::default().borders(Borders::ALL).title("Timer")
                ),
                speed[1]
            );

            // Text pane
            let current = app.status
                .iter()
                .position(|&s| s == Status::Untyped)
                .unwrap_or(app.status.len());
            let text_spans: Vec<Span> = app.target
                .chars()
                .zip(app.status.iter().cloned())
                .enumerate()
                .map(|(i, (ch, st))| {
                    let base = match st {
                        Status::Untyped => Style::default().fg(Color::White),
                        Status::Correct => Style::default().fg(Color::Green),
                        Status::Incorrect => Style::default().fg(Color::Red),
                    };
                    if i == current && app.mode == Mode::Insert {
                        Span::styled(
                            ch.to_string(),
                            base.bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD)
                        )
                    } else {
                        Span::styled(ch.to_string(), base)
                    }
                })
                .collect();
            f.render_widget(
                Paragraph::new(Spans::from(text_spans))
                    .block(Block::default().borders(Borders::ALL).title("Text"))
                    .wrap(Wrap { trim: true }),
                chunks[2]
            );

            // Footer / locked message
            let footer = if app.locked {
                Paragraph::new("Too many mistakes! Type correct letter to resume.").style(
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                )
            } else if app.mode == Mode::Insert {
                Paragraph::new(format!("Elapsed: {}s", app.elapsed_secs()))
            } else {
                Paragraph::new("")
            };
            f.render_widget(
                footer.block(Block::default().borders(Borders::ALL).title("Status")),
                chunks[3]
            );
        })?;

        // Input & logic
        let timeout = tick_rate.checked_sub(last_tick.elapsed()).unwrap_or_default();
        if event::poll(timeout)? {
            if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                // Ctrl+C quits
                if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
                    break 'mainloop;
                }
                // Esc restarts
                if code == KeyCode::Esc {
                    app = make_app();
                    last_sample_sec = 0;
                    continue 'mainloop;
                }
                match app.mode {
                    Mode::View => {
                        if code == KeyCode::Enter {
                            app.mode = Mode::Insert;
                            let now = Instant::now();
                            app.start = Some(now);
                            app.last_correct = now;
                            app.locked = false;
                        } else {
                            handle_nav_keys(&mut app, code);
                        }
                    }
                    Mode::Insert => {
                        if let KeyCode::Char(c) = code {
                            app.on_key(c);
                            // Auto-finish Time
                            if
                                app.selected_tab == 0 &&
                                app.elapsed_secs() >=
                                    (app.current_options()[app.selected_value] as u64)
                            {
                                app.mode = Mode::Finished;
                            }
                            // Auto-finish Words
                            if app.selected_tab == 1 {
                                let idx = app.status
                                    .iter()
                                    .position(|&s| s == Status::Untyped)
                                    .unwrap_or(app.status.len());
                                let typed = app.target.chars().take(idx).collect::<String>();
                                let cnt = typed.split_whitespace().count();
                                if cnt >= (app.current_options()[app.selected_value] as usize) {
                                    app.mode = Mode::Finished;
                                }
                            }
                        } else {
                            handle_nav_keys(&mut app, code);
                        }
                    }
                    Mode::Finished => handle_nav_keys(&mut app, code),
                }
            }
        }

        // Sample net WPM
        if app.mode == Mode::Insert {
            let sec = app.elapsed_secs();
            if sec > last_sample_sec {
                last_sample_sec = sec;
                let elapsed = elapsed_seconds_since_start(app.start.unwrap());
                let net = net_wpm(app.correct_chars, app.incorrect_chars, elapsed);
                app.samples.push((sec, net));
            }
        }

        // On finish → graph
        if app.mode == Mode::Finished {
            terminal.clear()?;
            terminal.draw(|f| graph::draw_wpm_chart(f, f.size(), &app.samples))?;
            loop {
                if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                    if code == KeyCode::Esc {
                        app = make_app();
                        last_sample_sec = 0;
                        break;
                    }
                    if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
                        break 'mainloop;
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    if let Some(start) = app.start {
        let elapsed = elapsed_seconds_since_start(start);
        let final_net = net_wpm(app.correct_chars, app.incorrect_chars, elapsed);
        println!("Final Time: {}s  ┃  Final WPM: {:.1}", app.elapsed_secs(), final_net);
    } else {
        println!("Goodbye!");
    }
    Ok(())
}

fn handle_nav_keys(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('1') => {
            app.selected_tab = 0;
        }
        KeyCode::Char('2') => {
            app.selected_tab = 1;
        }
        KeyCode::Char('3') => {
            app.selected_tab = 2;
        }
        KeyCode::Left if app.selected_value > 0 => {
            app.selected_value -= 1;
        }
        KeyCode::Right => {
            let len = app.current_options().len();
            if app.selected_value + 1 < len {
                app.selected_value += 1;
            }
        }
        _ => {}
    }
}

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

fn generate_sentence(words: &[String], n: usize) -> String {
    let mut rng = rand::thread_rng();
    words.choose_multiple(&mut rng, n).cloned().collect::<Vec<_>>().join(" ")
}
