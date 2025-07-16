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
    layout::{Constraint, Direction, Layout, Rect},
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
    last_input: Instant,
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
        let now = Instant::now();
        App {
            target: target.clone(),
            status: vec![Status::Untyped; target.chars().count()],
            start: None,
            last_input: now,
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
        self.last_input = Instant::now();
        if self.selected_tab == 2 {
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

    fn backspace(&mut self) {
        self.last_input = Instant::now();
        if self.selected_tab == 2 {
            if self.free_text.pop().is_some() && self.correct_chars > 0 {
                self.correct_chars -= 1;
            }
            return;
        }
        if let Some(i) = self.status.iter().rposition(|&s| s != Status::Untyped) {
            match self.status[i] {
                Status::Correct => {
                    if self.correct_chars > 0 {
                        self.correct_chars -= 1;
                    }
                }
                Status::Incorrect => {
                    if self.incorrect_chars > 0 {
                        self.incorrect_chars -= 1;
                    }
                }
                _ => {}
            }
            self.status[i] = Status::Untyped;
            self.locked = false;
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

/// On-screen keyboard widget.
struct Keyboard {
    pressed_key: Option<String>,
}

impl Keyboard {
    fn new() -> Self {
        Keyboard { pressed_key: None }
    }

    fn handle_key(&mut self, code: &KeyCode) {
        self.pressed_key = map_keycode(code);
    }

    fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let rows = vec![
            vec!["ESC","1","2","3","4","5","6","7","8","9","0","-","=","BS"],
            vec!["Q","W","E","R","T","Y","U","I","O","P","[","]","\\"],
            vec!["A","S","D","F","G","H","J","K","L",";","'"],
            vec!["Z","X","C","V","B","N","M",",",".","/"],
            vec!["SPC"],
        ];

        let row_areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(rows.iter().map(|_| Constraint::Length(3)).collect::<Vec<_>>())
            .split(area);

        for (r, keys) in rows.iter().enumerate() {
            let row_area = row_areas[r];
            let constraints = if keys.len() == 1 && keys[0] == "SPC" {
                vec![Constraint::Percentage(100)]
            } else {
                keys.iter().map(|_| Constraint::Length(5)).collect()
            };
            let key_areas = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(constraints)
                .split(row_area);

            for (i, &label) in keys.iter().enumerate() {
                let is_pressed = self.pressed_key.as_deref() == Some(label);
                let (fg, bg) = if is_pressed {
                    (Color::Black, Color::Yellow)
                } else {
                    (Color::White, Color::Reset)
                };
                let width = key_areas[i].width as usize;
                let text = format!("{:^1$}", label, width);
                let paragraph = Paragraph::new(Span::styled(text, Style::default().fg(fg).bg(bg)))
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(paragraph, key_areas[i]);
            }
        }
    }
}

fn map_keycode(code: &KeyCode) -> Option<String> {
    match code {
        KeyCode::Esc => Some("ESC".into()),
        KeyCode::Backspace => Some("BS".into()),
        KeyCode::Char(' ') => Some("SPC".into()),
        KeyCode::Char(c) => Some(c.to_ascii_uppercase().to_string()),
        _ => None,
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

    // For throttling WPM display updates
    let mut last_wpm_update = Instant::now();
    let mut cached_net = 0.0;
    let mut cached_acc = 0.0;

    let make_app = || {
        let words = load_words().expect("load_words failed");
        let sentence = generate_sentence(&words, 30);
        App::new(sentence)
    };
    let mut app = make_app();
    let mut keyboard = Keyboard::new();

    'main: loop {
        // Before drawing, update cached WPM/accuracy at most once per second
        if let Mode::Insert = app.mode {
            if last_wpm_update.elapsed() >= Duration::from_secs(1) {
                let real_elapsed = elapsed_seconds_since_start(app.start.unwrap());
                let idle = Instant::now().duration_since(app.last_input).as_secs_f64();
                let effective = real_elapsed + idle;
                cached_net = net_wpm(app.correct_chars, app.incorrect_chars, effective);
                cached_acc = accuracy(app.correct_chars, app.incorrect_chars);
                last_wpm_update = Instant::now();
                // also record sample for charting once per second
                let secs = app.elapsed_secs();
                if secs > last_sample {
                    last_sample = secs;
                    app.samples.push((secs, cached_net));
                }
            }
        }

        terminal.draw(|f| {
            let size = f.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(3), Constraint::Length(3), Constraint::Min(3)])
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

            // Value selector
            let mut spans = vec![Span::raw("| ")];
            for (i, &v) in app.current_options().iter().enumerate() {
                let s = v.to_string();
                spans.push(if i == app.selected_value {
                    Span::styled(s, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                } else {
                    Span::raw(s)
                });
                spans.push(Span::raw(" "));
            }
            let opts = Paragraph::new(Spans::from(spans))
                .block(Block::default().borders(Borders::ALL).title("Value"));
            f.render_widget(opts, nav[1]);

            // Speed & Timer
            let st = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[1]);

            // Use cached_net/cached_acc instead of recalculating every frame
            let speed_txt = if app.mode == Mode::View {
                "WPM: --  Acc: --%".into()
            } else {
                format!("WPM: {:.1}  Acc: {:.0}%", cached_net, cached_acc)
            };
            let speed = Paragraph::new(speed_txt)
                .block(Block::default().borders(Borders::ALL).title("Speed"));
            f.render_widget(speed, st[0]);

            // Timer text
            let timer_txt = match app.mode {
                Mode::View => "Press Enter to start".into(),
                Mode::Insert => match app.selected_tab {
                    0 => {
                        let rem = (app.current_options()[app.selected_value] as i64 - app.elapsed_secs() as i64)
                            .max(0);
                        format!("Time left: {}s", rem)
                    }
                    1 => {
                        let idx = app.status.iter().position(|&s| s == Status::Untyped)
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
                    let real_elapsed = elapsed_seconds_since_start(app.start.unwrap());
                    format!(
                        "🏁 Done! {}s • {:.1} WPM  Esc=Restart",
                        app.elapsed_secs(),
                        net_wpm(app.correct_chars, app.incorrect_chars, real_elapsed)
                    )
                }
            };
            let timer = Paragraph::new(timer_txt)
                .block(Block::default().borders(Borders::ALL).title("Timer"));
            f.render_widget(timer, st[1]);

            // Bottom split: text/zen and keyboard
            let bottom = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(chunks[2]);

            // Left pane: Zen or Text
            if app.selected_tab == 2 {
                let free = app.free_text.chars().map(|c| Span::raw(c.to_string())).collect::<Vec<_>>();
                f.render_widget(
                    Paragraph::new(Spans::from(free))
                        .block(Block::default().borders(Borders::ALL).title("Zen"))
                        .wrap(Wrap { trim: true }),
                    bottom[0],
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
                                base.bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD),
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
                    bottom[0],
                );
            }

            // Right pane: keyboard
            keyboard.draw(f, bottom[1]);
        })?;

        // Input & navigation handling
        let timeout = tick_rate.checked_sub(last_tick.elapsed()).unwrap_or_default();
        if event::poll(timeout)? {
            if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                keyboard.handle_key(&code);

                // Quit or restart
                if code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL) {
                    break 'main;
                }
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
                            app.last_input = now;
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
                    Mode::Insert => match code {
                        KeyCode::Char(c) => {
                            app.on_key(c);
                            // auto-finish on time or word count
                            if app.selected_tab == 0
                                && app.elapsed_secs() >= app.current_options()[app.selected_value] as u64
                            {
                                app.mode = Mode::Finished;
                            }
                            if app.selected_tab == 1 {
                                let idx = app
                                    .status
                                    .iter()
                                    .position(|&s| s == Status::Untyped)
                                    .unwrap_or(app.status.len());
                                let cnt = app
                                    .target
                                    .chars()
                                    .take(idx)
                                    .collect::<String>()
                                    .split_whitespace()
                                    .count();
                                if cnt >= app.current_options()[app.selected_value] as usize {
                                    app.mode = Mode::Finished;
                                }
                            }
                        }
                        KeyCode::Backspace => {
                            app.backspace();
                        }
                        other => handle_nav(&mut app, other),
                    },
                    Mode::Finished => {
                        terminal.draw(|f| draw_finished(f, &app))?;
                        // wait for Esc to restart or Ctrl-C to quit
                        loop {
                            if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                                if code == KeyCode::Esc && modifiers.is_empty() {
                                    app = make_app();
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
                }
            }
        }

        last_tick = Instant::now();
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}


// draw_finished, handle_nav, load_words, generate_sentence unchanged

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
