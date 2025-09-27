use tui::{
    backend::Backend,
    layout::{ Constraint, Direction, Layout },
    style::{ Modifier, Style },
    text::{ Span, Spans },
    widgets::{ Block, Borders, Paragraph, Tabs, Wrap },
    Frame,
};
use crate::app::state::{ App, Mode, Status };
use crate::ui::keyboard::Keyboard;
use crate::graph;
use crate::wpm::accuracy;

/// Main drawing function.
/// - `cached_net` and `cached_acc` come from your throttled WPM/accuracy logic.
pub fn draw<B: Backend>(
    f: &mut Frame<B>,
    app: &App,
    keyboard: &Keyboard,
    cached_net: f64,
    cached_acc: f64
) {
    // ── Profile screen takes over entirely
    if app.mode == Mode::Profile {
        
        return;
    }
    let size = f.size();

    // 1) Build vertical constraints based on which panels are visible
    let mut v_cons = Vec::new();
    if app.show_mode || app.show_value {
        v_cons.push(Constraint::Length(3)); // Row 1
    }
    if app.show_state || app.show_speed || app.show_timer {
        v_cons.push(Constraint::Length(3)); // Row 2
    }
    v_cons.push(Constraint::Min(3)); // Row 3 (always present)

    let rows = Layout::default().direction(Direction::Vertical).constraints(v_cons).split(size);

    let mut row_idx = 0;

    // ── Row 1: "1 Mode" & "2 Value"
    if app.show_mode || app.show_value {
        let area = rows[row_idx];
        row_idx += 1;

        // decide widths (60/40 split) if both, or full width if only one
        let mut items = Vec::new();
        if app.show_mode {
            items.push(60u16);
        }
        if app.show_value {
            items.push(40u16);
        }
        let total: u16 = items.iter().sum();
        let h_cons = items
            .iter()
            .map(|&w| Constraint::Percentage((w * 100) / total))
            .collect::<Vec<_>>();

        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(h_cons)
            .split(area);

        let mut col_idx = 0;

        // Mode tabs
        if app.show_mode {
            let titles = ["Time", "Words", "Zen"]
                .iter()
                .map(|t| Spans::from(*t))
                .collect::<Vec<_>>();

            let tabs = Tabs::new(titles)
                .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.border.to_tui_color()))
                .style(Style::default().bg(app.theme.background.to_tui_color()).fg(app.theme.foreground.to_tui_color()))
                        .title(
                            Spans::from(
                                vec![
                                    Span::styled("¹", Style::default().fg(app.theme.title_accent.to_tui_color())),
                                    Span::raw(" Mode")
                                ]
                            )
                        )
                )
                .select(app.selected_tab)
                .style(Style::default().fg(app.theme.tab_inactive.to_tui_color()))
                .highlight_style(Style::default().fg(app.theme.tab_active.to_tui_color()).add_modifier(Modifier::BOLD))
                .divider(Span::raw(" "));

            f.render_widget(tabs, cols[col_idx]);
            col_idx += 1;
        }

        // Value options
        if app.show_value {
            let mut spans = vec![Span::raw("| ")];
            for (i, &v) in app.current_options().iter().enumerate() {
                let s = v.to_string();
                spans.push(
                    if i == app.selected_value {
                        Span::styled(
                            s,
                            Style::default().fg(app.theme.highlight.to_tui_color()).add_modifier(Modifier::BOLD)
                        )
                    } else {
                        Span::styled(s, Style::default().fg(app.theme.foreground.to_tui_color()))
                    }
                );
                spans.push(Span::raw(" "));
            }

            let opts = Paragraph::new(Spans::from(spans)).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(app.theme.border.to_tui_color()))
                    .style(Style::default().bg(app.theme.background.to_tui_color()).fg(app.theme.foreground.to_tui_color()))
                    .title(
                        Spans::from(
                            vec![
                                Span::styled("²", Style::default().fg(app.theme.title_accent.to_tui_color())),
                                Span::raw(" Value")
                            ]
                        )
                    )
            );

            f.render_widget(opts, cols[col_idx]);
        }
    }

    // ── Row 2: "3 State" | "4 Speed" | "5 Timer"
    if app.show_state || app.show_speed || app.show_timer {
        let area = rows[row_idx];
        row_idx += 1;

        let mut items = Vec::new();
        if app.show_state {
            items.push(30u16);
        }
        if app.show_speed {
            items.push(35u16);
        }
        if app.show_timer {
            items.push(35u16);
        }
        let total: u16 = items.iter().sum();
        let h_cons = items
            .iter()
            .map(|&w| Constraint::Percentage((w * 100) / total))
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
                _ => "", // covers Profile
            };
            let state = Paragraph::new(state_txt).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(app.theme.border.to_tui_color()))
                    .style(Style::default().bg(app.theme.background.to_tui_color()).fg(app.theme.foreground.to_tui_color()))
                    .title(
                        Spans::from(
                            vec![
                                Span::styled("³", Style::default().fg(app.theme.title_accent.to_tui_color())),
                                Span::raw(" State")
                            ]
                        )
                    )
            );
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
            let speed = Paragraph::new(speed_txt).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(app.theme.border.to_tui_color()))
                    .style(Style::default().bg(app.theme.background.to_tui_color()).fg(app.theme.foreground.to_tui_color()))
                    .title(
                        Spans::from(
                            vec![
                                Span::styled("⁴", Style::default().fg(app.theme.title_accent.to_tui_color())),
                                Span::raw(" Speed")
                            ]
                        )
                    )
            );
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
                                let rem = (
                                    (app.current_options()[app.selected_value] as i64) -
                                    (app.elapsed_secs() as i64)
                                ).max(0);
                                format!("Time left: {}s", rem)
                            }
                            1 => {
                                let idx = app.status
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
                        }
                    }
                }
                Mode::Finished =>
                    format!(
                        "🏁 Done! {}s • {:.1} WPM  Esc=Restart",
                        app.elapsed_secs(),
                        cached_net
                    ),
                _ => "".into(),
            };
            let timer = Paragraph::new(timer_txt).block(
                Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(app.theme.border.to_tui_color()))
                            .style(Style::default().bg(app.theme.background.to_tui_color()).fg(app.theme.foreground.to_tui_color()))
                    .title(
                        Spans::from(
                            vec![
                                Span::styled("⁵", Style::default().fg(app.theme.title_accent.to_tui_color())),
                                Span::raw(" Timer")
                            ]
                        )
                    )
            );
            f.render_widget(timer, cols[col_idx]);
        }
    }

    // ── Row 3: bottom area ────────────────────────────────
    let bottom_area = rows[row_idx];
    let b_cons = if app.show_text && app.show_keyboard {
        // 30% text on top, 70% keyboard below
        vec![Constraint::Percentage(42), Constraint::Percentage(58)]
    } else if app.show_text {
        // only text: full height
        vec![Constraint::Percentage(100)]
    } else {
        // only keyboard: full height
        vec![Constraint::Percentage(100)]
    };

    let bottom_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(b_cons)
        .split(bottom_area);

    let mut bi = 0;

    // 6 Text pane
    if app.show_text {
        let area = bottom_chunks[bi];
        bi += 1;
        if app.selected_tab == 2 {
            let free: Vec<Span> = app.free_text
                .chars()
                .map(|c| Span::raw(c.to_string()))
                .collect();
            f.render_widget(
                Paragraph::new(Spans::from(free))
                    .block(
                        Block::default()
                                    .borders(Borders::ALL)
                                    .border_style(Style::default().fg(app.theme.border.to_tui_color()))
                                    .style(Style::default().bg(app.theme.background.to_tui_color()).fg(app.theme.foreground.to_tui_color()))
                            .title(
                                Spans::from(
                                    vec![
                                        Span::styled("⁶", Style::default().fg(app.theme.title_accent.to_tui_color())),
                                        Span::raw(" Text")
                                    ]
                                )
                            )
                    )
                    .wrap(Wrap { trim: true }),
                area
            );
        } else {
            let cur = app.status
                .iter()
                .position(|&s| s == Status::Untyped)
                .unwrap_or(app.status.len());

            let spans: Vec<Span> = app.target
                .chars()
                .zip(app.status.iter().cloned())
                .enumerate()
                .map(|(i, (ch, st))| {
                    let base_style = match st {
                        Status::Untyped => Style::default().fg(app.theme.text_untyped.to_tui_color()),
                        Status::Correct => Style::default().fg(app.theme.text_correct.to_tui_color()),
                        Status::Incorrect => Style::default().fg(app.theme.text_incorrect.to_tui_color()),
                    };
                    if i == cur && app.mode == Mode::Insert {
                        Span::styled(
                            ch.to_string(),
                            base_style
                                .bg(app.theme.text_cursor_bg.to_tui_color())
                                .fg(app.theme.text_cursor_fg.to_tui_color())
                                .add_modifier(Modifier::BOLD)
                        )
                    } else {
                        Span::styled(ch.to_string(), base_style)
                    }
                })
                .collect();

            f.render_widget(
                Paragraph::new(Spans::from(spans))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .border_style(Style::default().fg(app.theme.border.to_tui_color()))
                            .style(Style::default().bg(app.theme.background.to_tui_color()).fg(app.theme.foreground.to_tui_color()))
                            .title(
                                Spans::from(
                                    vec![
                                        Span::styled("⁶", Style::default().fg(app.theme.title_accent.to_tui_color())),
                                        Span::raw(" Text")
                                    ]
                                )
                            )
                    )
                    .wrap(Wrap { trim: true }),
                area
            );
        }
    }

    // 7 Keyboard pane
    if app.show_keyboard {
        let area = bottom_chunks[bi];

        // 1) Build the Block
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(app.theme.border.to_tui_color()))
            .style(Style::default().bg(app.theme.background.to_tui_color()).fg(app.theme.foreground.to_tui_color()))
            .title(
                Spans::from(
                    vec![
                        Span::styled("⁷", Style::default().fg(app.theme.title_accent.to_tui_color())),
                        Span::raw(" Keyboard")
                    ]
                )
            );

        // 2) Compute the inner rect and render the block
        let inner = block.inner(area);
        f.render_widget(block, area);

    // 3) Draw the keyboard inside the inner rect
    keyboard.draw(f, inner, &app.theme, app.keyboard_layout);
    }
}

/// Draw the “finished” summary: left = WPM chart, right = stats.
pub fn draw_finished<B: Backend>(f: &mut Frame<B>, app: &App) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
        .split(size);

    // Left: WPM chart
    graph::draw_wpm_chart(f, chunks[0], &app.samples, &app.theme);

    // Right: stats
    let elapsed_secs = app.elapsed_secs();
    let elapsed_f = elapsed_secs as f64;
    // compute net using correct timestamps (penalizes mistakes via time gaps)
    let net = crate::wpm::net_wpm_from_correct_timestamps(&app.correct_timestamps, elapsed_f);
    let acc = accuracy(app.correct_chars, app.incorrect_chars);
    let raw = crate::wpm::raw_wpm_from_counts(app.correct_chars + app.incorrect_chars, elapsed_f);
    let errs = app.incorrect_chars;
    let test_type = match app.selected_tab {
        0 => format!("time {}s", app.current_options()[app.selected_value]),
        1 => format!("words {}", app.current_options()[app.selected_value]),
        _ => "zen".to_string(),
    };

    // Simple consistency metric
    let consistency = {
        let vs: Vec<f64> = app.samples
            .iter()
            .map(|&(_, w)| w)
            .collect();
        if vs.len() > 1 {
            let mean = vs.iter().sum::<f64>() / (vs.len() as f64);
            let var =
                vs
                    .iter()
                    .map(|v| (v - mean).powi(2))
                    .sum::<f64>() / (vs.len() as f64);
            let std = var.sqrt();
            format!("{:.0}%", (1.0 - std / (mean + 1.0)).max(0.0) * 100.0)
        } else {
            "--%".into()
        }
    };

    let items = vec![
        Spans::from(
            vec![
                Span::styled("WPM  ", Style::default().fg(app.theme.stats_label.to_tui_color())),
                Span::styled(
                    format!("{:.0}", net),
                    Style::default().fg(app.theme.stats_value.to_tui_color()).add_modifier(Modifier::BOLD)
                )
            ]
        ),
        Spans::from(
            vec![
                Span::styled("ACC  ", Style::default().fg(app.theme.stats_label.to_tui_color())),
                Span::styled(
                    format!("{:.0}%", acc),
                    Style::default().fg(app.theme.stats_value.to_tui_color()).add_modifier(Modifier::BOLD)
                )
            ]
        ),
        Spans::from(
            vec![
                Span::styled("RAW  ", Style::default().fg(app.theme.stats_label.to_tui_color())),
                Span::styled(raw.to_string(), Style::default().fg(app.theme.foreground.to_tui_color()))
            ]
        ),
        Spans::from(
            vec![
                Span::styled("ERR  ", Style::default().fg(app.theme.stats_label.to_tui_color())),
                Span::styled(errs.to_string(), Style::default().fg(app.theme.foreground.to_tui_color()))
            ]
        ),
        Spans::from(
            vec![
                Span::styled("TYPE ", Style::default().fg(app.theme.stats_label.to_tui_color())), 
                Span::styled(test_type, Style::default().fg(app.theme.foreground.to_tui_color()))
            ]
        ),
        Spans::from(
            vec![
                Span::styled("CONS ", Style::default().fg(app.theme.stats_label.to_tui_color())), 
                Span::styled(consistency, Style::default().fg(app.theme.foreground.to_tui_color()))
            ]
        ),
        Spans::from(
            vec![
                Span::styled("TIME ", Style::default().fg(app.theme.stats_label.to_tui_color())),
                Span::styled(format!("{}s", elapsed_secs), Style::default().fg(app.theme.foreground.to_tui_color()))
            ]
        )
    ];

    let stats = Paragraph::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(app.theme.border.to_tui_color()))
                .style(Style::default().bg(app.theme.background.to_tui_color()).fg(app.theme.foreground.to_tui_color()))
                .title("Summary")
        )
        .wrap(Wrap { trim: true });

    f.render_widget(stats, chunks[1]);
}
