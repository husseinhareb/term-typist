// src/ui/draw.rs

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
    Frame,
};
use crate::app::state::{App, Mode, Status};
use crate::ui::keyboard::Keyboard;

/// Main drawing function extracted from `lib.rs`'s terminal.draw closure.
/// - `cached_net` and `cached_acc` should be provided by the caller (throttled WPM/accuracy).
pub fn draw<B: Backend>(f: &mut Frame<B>, app: &App, keyboard: &Keyboard, cached_net: f64, cached_acc: f64) {
    let size = f.size();

    // 1) Build vertical constraints dynamically
    let mut v_cons = Vec::new();
    if app.show_mode || app.show_value {
        v_cons.push(Constraint::Length(3));
    }
    if app.show_state || app.show_speed || app.show_timer {
        v_cons.push(Constraint::Length(3));
    }
    // bottom always present
    v_cons.push(Constraint::Min(3));

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(v_cons)
        .split(size);

    let mut row_idx = 0;

    // ── Row 1: "1 Mode" & "2 Value"
    if app.show_mode || app.show_value {
        let area = rows[row_idx];
        row_idx += 1;
        let mut items = Vec::new();
        if app.show_mode { items.push(60u16); }
        if app.show_value { items.push(40u16); }
        let total: u16 = items.iter().sum();
        let h_cons = items.iter()
            .map(|&w| Constraint::Percentage(w * 100 / total))
            .collect::<Vec<_>>();
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(h_cons)
            .split(area);
        let mut col_idx = 0;

        // Mode tabs
        if app.show_mode {
            let titles = ["Time", "Words", "Zen"]
                .iter().map(|t| Spans::from(*t)).collect::<Vec<_>>();
            let tabs = Tabs::new(titles)
                .block(Block::default().borders(Borders::ALL).title("1 Mode"))
                .select(app.selected_tab)
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                .divider(Span::raw(" "));
            f.render_widget(tabs, cols[col_idx]);
            col_idx += 1;
        }

        // Value options
        if app.show_value {
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
                .block(Block::default().borders(Borders::ALL).title("2 Value"));
            f.render_widget(opts, cols[col_idx]);
        }
    }

    // ── Row 2: "3 State" | "4 Speed" | "5 Timer"
    if app.show_state || app.show_speed || app.show_timer {
        let area = rows[row_idx];
        row_idx += 1;
        let mut items = Vec::new();
        if app.show_state { items.push(30u16); }
        if app.show_speed { items.push(35u16); }
        if app.show_timer { items.push(35u16); }
        let total: u16 = items.iter().sum();
        let h_cons = items.iter()
            .map(|&w| Constraint::Percentage(w * 100 / total))
            .collect::<Vec<_>>();
        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(h_cons)
            .split(area);
        let mut col_idx = 0;

        // State box
        if app.show_state {
            let state_txt = match app.mode {
                Mode::View => "State: View",
                Mode::Insert => "State: Insert",
                Mode::Finished => "State: Finished",
            };
            let state = Paragraph::new(state_txt)
                .block(Block::default().borders(Borders::ALL).title("3 State"));
            f.render_widget(state, cols[col_idx]);
            col_idx += 1;
        }

        // Speed box
        if app.show_speed {
            let speed_txt = if app.start.is_some() {
                format!("WPM: {:.1}  Acc: {:.0}%", cached_net, cached_acc)
            } else {
                "WPM: --  Acc: --%".into()
            };
            let speed = Paragraph::new(speed_txt)
                .block(Block::default().borders(Borders::ALL).title("4 Speed"));
            f.render_widget(speed, cols[col_idx]);
            col_idx += 1;
        }

        // Timer box
        if app.show_timer {
            let timer_txt = match app.mode {
                Mode::View => "Press Enter to start".into(),
                Mode::Insert => {
                    if app.start.is_none() {
                        "Start typing…".into()
                    } else {
                        match app.selected_tab {
                            0 => {
                                let rem = (app.current_options()[app.selected_value] as i64
                                    - app.elapsed_secs() as i64).max(0);
                                format!("Time left: {}s", rem)
                            }
                            1 => {
                                let idx = app.status.iter()
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
                        }
                    }
                }
                Mode::Finished => {
                    let secs = app.elapsed_secs() as f64;
                    format!("🏁 Done! {}s • {:.1} WPM  Esc=Restart", app.elapsed_secs(), cached_net)
                }
            };
            let timer = Paragraph::new(timer_txt)
                .block(Block::default().borders(Borders::ALL).title("5 Timer"));
            f.render_widget(timer, cols[col_idx]);
        }
    }

    // ── Row 3: bottom area ────────────────────────────────
    let bottom_area = rows[row_idx];
    let b_cons = if app.show_text && app.show_keyboard {
        vec![Constraint::Percentage(50), Constraint::Percentage(50)]
    } else if app.show_text {
        vec![Constraint::Percentage(100)]
    } else {
        vec![Constraint::Percentage(100)]
    };
    let bottom_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(b_cons)
        .split(bottom_area);

    let mut bi = 0;
    // Text area
    if app.show_text {
        let area = bottom_chunks[bi];
        bi += 1;
        if app.selected_tab == 2 {
            let free = app.free_text.chars().map(|c| Span::raw(c.to_string())).collect::<Vec<_>>();
            f.render_widget(
                Paragraph::new(Spans::from(free))
                    .block(Block::default().borders(Borders::ALL).title("6 Text"))
                    .wrap(Wrap { trim: true }),
                area,
            );
        } else {
            let cur = app.status.iter().position(|&s| s == Status::Untyped).unwrap_or(app.status.len());
            let spans = app.target.chars().zip(app.status.iter().cloned())
                .enumerate()
                .map(|(i, (ch, st))| {
                    let base_style = match st {
                        Status::Untyped => Style::default().fg(Color::White),
                        Status::Correct => Style::default().fg(Color::Green),
                        Status::Incorrect => Style::default().fg(Color::Red),
                    };
                    if i == cur && app.mode == Mode::Insert {
                        Span::styled(ch.to_string(), base_style.bg(Color::Yellow).fg(Color::Black).add_modifier(Modifier::BOLD))
                    } else {
                        Span::styled(ch.to_string(), base_style)
                    }
                })
                .collect::<Vec<_>>();
            f.render_widget(
                Paragraph::new(Spans::from(spans))
                    .block(Block::default().borders(Borders::ALL).title("6 Text"))
                    .wrap(Wrap { trim: true }),
                area,
            );
        }
    }

    // Keyboard widget
    if app.show_keyboard {
        let area = bottom_chunks[bi];
        keyboard.draw(f, area);
    }
}
