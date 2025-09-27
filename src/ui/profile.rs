// src/ui/profile.rs

use chrono::{DateTime, FixedOffset, Local};
use crossterm::event::KeyCode;
use rusqlite::Connection;
use std::sync::atomic::{AtomicUsize, Ordering};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::Style,
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState, Wrap},
    Frame,
};

use crate::graph;
use crate::theme::Theme;

/// Absolute cursor into your tests (0 = newest, 1 = next older, …).
static RECENT_CURSOR: AtomicUsize = AtomicUsize::new(0);

/// Rows per page in the Recent Tests table.
const PAGE_SIZE: u32 = 10;

/// Core key handler. Works for both helpers above.
/// - Up      → newer tests (cursor -= 1, clamped at 0)
/// - Down    → older tests (cursor += 1)
/// - PageUp  → jump newer by a page
/// - PageDown→ jump older by a page
/// - Home    → newest
/// - End     → oldest (very large number; draw clamps to max)
pub fn handle_profile_key(code: KeyCode) {
    match code {
        KeyCode::Up => {
            let _ = RECENT_CURSOR.fetch_update(
                Ordering::Relaxed,
                Ordering::Relaxed,
                |c| c.checked_sub(1),
            );
        }
        KeyCode::Down => {
            RECENT_CURSOR.fetch_add(1, Ordering::Relaxed);
        }
        KeyCode::PageUp => {
            let _ = RECENT_CURSOR.fetch_update(
                Ordering::Relaxed,
                Ordering::Relaxed,
                |c| c.checked_sub(PAGE_SIZE as usize),
            );
        }
        KeyCode::PageDown => {
            RECENT_CURSOR.fetch_add(PAGE_SIZE as usize, Ordering::Relaxed);
        }
        KeyCode::Home => {
            // Newest
            RECENT_CURSOR.store(0, Ordering::Relaxed);
        }
        KeyCode::End => {
            // Oldest: we don't know total here; set a big number and let draw() clamp.
            RECENT_CURSOR.store(usize::MAX / 2, Ordering::Relaxed);
        }
        // Optional vim keys
        KeyCode::Char('k') => {
            let _ = RECENT_CURSOR.fetch_update(
                Ordering::Relaxed,
                Ordering::Relaxed,
                |c| c.checked_sub(1),
            );
        }
        KeyCode::Char('j') => {
            RECENT_CURSOR.fetch_add(1, Ordering::Relaxed);
        }
        _ => {}
    }
}

/// Simple thousands separator for positive integers.
fn sep_int(mut n: u64) -> String {
    if n == 0 {
        return "0".to_string();
    }
    let mut out = String::new();
    let mut digits = 0usize;
    while n > 0 {
        if digits > 0 && digits.is_multiple_of(3) {
            out.push(',');
        }
        out.push(char::from(b'0' + (n % 10) as u8));
        n /= 10;
        digits += 1;
    }
    out.chars().rev().collect()
}

/// Format a nonnegative f64 with 0 decimals and thousands separator.
fn sep_f64_0(f: f64) -> String {
    if !f.is_finite() || f <= 0.0 {
        "0".to_string()
    } else {
        sep_int(f.round() as u64)
    }
}

/// Draws the Profile screen: top stats grid, 365-day summary, WPM chart,
/// and a scrollable Recent Tests table with highlight.
pub fn draw_profile<B: Backend>(f: &mut Frame<B>, conn: &Connection, theme: &Theme) {
    // 0) Determine total_tests and clamp cursor
    let total_tests: u32 = conn
        .prepare("SELECT COUNT(*) FROM tests")
        .and_then(|mut s| s.query_row([], |r| r.get(0)))
        .unwrap_or(0);

    let mut cursor = RECENT_CURSOR.load(Ordering::Relaxed) as u32;
    if total_tests > 0 {
        cursor = cursor.min(total_tests.saturating_sub(1));
    } else {
        cursor = 0;
    }
    RECENT_CURSOR.store(cursor as usize, Ordering::Relaxed);

    // Compute a sliding-window offset so arrow keys shift the visible list by
    // one item (remove the first, append the next) rather than jumping in
    // PAGE_SIZE blocks. We choose an offset so the selected item is at
    // `selected_idx = cursor - offset` and 0 <= selected_idx < PAGE_SIZE.
    let max_offset = total_tests.saturating_sub(PAGE_SIZE);
    let mut offset = if cursor < PAGE_SIZE { 0 } else { cursor.saturating_sub(PAGE_SIZE - 1) };
    offset = offset.min(max_offset);
    let selected_idx = (cursor.saturating_sub(offset)) as usize;

    //
    // 1) Compute all aggregates
    //
    let mut agg = conn
        .prepare(
            r#"
            SELECT
              COUNT(*) AS started,
              SUM(CASE WHEN wpm>0 THEN 1 ELSE 0 END) AS completed,
              SUM((correct_chars+incorrect_chars)*1.0/5.0) AS est_words,
              AVG(wpm) FILTER (WHERE wpm>0) AS avg_wpm,
              AVG((correct_chars+incorrect_chars)*1.0/5.0
                  / NULLIF(duration_ms/60000.0,0)) AS avg_raw,
              AVG(accuracy) AS avg_acc,
              SUM(duration_ms)/1000.0 AS total_secs
            FROM tests
        "#,
        )
        .unwrap();
    let (started, completed, est_words, avg_wpm, avg_raw, avg_acc, total_secs): (
        u32,
        u32,
        f64,
        f64,
        f64,
        f64,
        f64,
    ) = agg
        .query_row([], |r| {
            Ok((
                r.get(0)?,
                r.get(1)?,
                r.get(2)?,
                r.get(3)?,
                r.get(4)?,
                r.get(5)?,
                r.get(6)?,
            ))
        })
        .unwrap_or((0, 0, 0.0, 0.0, 0.0, 0.0, 0.0));

    // Highest net WPM (+ mode/value)
    let mut hnet = conn
        .prepare("SELECT wpm, mode, target_value FROM tests ORDER BY wpm DESC LIMIT 1")
        .unwrap();
    let (h_wpm, h_mode, h_val): (f64, String, i64) = hnet
        .query_row([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap_or((0.0, "".into(), 0));

    // Highest raw WPM
    let mut hraw = conn
        .prepare(
            "SELECT (correct_chars+incorrect_chars)*1.0/5.0
                 / NULLIF(duration_ms/60000.0,0) AS raw
              FROM tests
              ORDER BY raw DESC LIMIT 1",
        )
        .unwrap();
    let h_raw: f64 = hraw.query_row([], |r| r.get(0)).unwrap_or(0.0);

    // Highest accuracy
    let mut hacc = conn
        .prepare("SELECT accuracy FROM tests ORDER BY accuracy DESC LIMIT 1")
        .unwrap();
    let h_acc: f64 = hacc.query_row([], |r| r.get(0)).unwrap_or(0.0);

    // Highest/average consistency (safe: guard MAX against 0/NULL)
    let mut hcons = conn
        .prepare(
            "SELECT COALESCE(MAX(cons), 0.0) FROM (
               SELECT 100.0*MIN(wpm)/NULLIF(MAX(wpm),0) AS cons
                 FROM samples
                GROUP BY test_id
             )",
        )
        .unwrap();
    let h_cons: f64 = hcons.query_row([], |r| r.get(0)).unwrap_or(0.0);

    let mut avgcons = conn
        .prepare(
            "SELECT COALESCE(AVG(cons), 0.0) FROM (
               SELECT 100.0*MIN(wpm)/NULLIF(MAX(wpm),0) AS cons
                 FROM samples
                GROUP BY test_id
             )",
        )
        .unwrap();
    let avg_cons: f64 = avgcons.query_row([], |r| r.get(0)).unwrap_or(0.0);

    // Last-10 summary (safe consistency and raw)
    let mut last10 = conn
        .prepare(
            r#"
            SELECT
              t.wpm,
              COALESCE((t.correct_chars+t.incorrect_chars)*1.0/5.0
                / NULLIF(t.duration_ms/60000.0,0), 0) AS raw,
              t.accuracy,
              COALESCE(100.0*MIN(s.wpm)/NULLIF(MAX(s.wpm),0), 0) AS cons
            FROM tests t
            LEFT JOIN samples s ON s.test_id = t.id
            GROUP BY t.id
            ORDER BY t.started_at DESC
            LIMIT 10
        "#,
        )
        .unwrap();

    let mut rows_iter = last10.query([]).unwrap();
    let mut cnt = 0usize;
    let (mut sum_w, mut sum_r, mut sum_a, mut sum_c) = (0.0, 0.0, 0.0, 0.0);
    while let Ok(Some(r)) = rows_iter.next() {
        cnt += 1;
        sum_w += r.get::<_, f64>(0).unwrap_or(0.0);
        sum_r += r.get::<_, f64>(1).unwrap_or(0.0);
        sum_a += r.get::<_, f64>(2).unwrap_or(0.0);
        sum_c += r.get::<_, f64>(3).unwrap_or(0.0);
    }
    let (avg10_wpm, avg10_raw, avg10_acc, avg10_cons) = if cnt > 0 {
        (
            sum_w / cnt as f64,
            sum_r / cnt as f64,
            sum_a / cnt as f64,
            sum_c / cnt as f64,
        )
    } else {
        (0.0, 0.0, 0.0, 0.0)
    };

    //
    // 2) Layout: top grid + 365-day summary + bottom (chart + table)
    //
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(12), Constraint::Length(3), Constraint::Min(5)])
        .split(f.size());

    // Top grid (Summary, Overall, Recent)
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(chunks[0]);

    // Summary
    let left = Paragraph::new(vec![
        Spans::from(Span::styled(
            "tests started",
            Style::default().fg(theme.stats_label.to_tui_color()),
        )),
        Spans::from(Span::styled(sep_int(started as u64), Style::default().fg(theme.foreground.to_tui_color()))),
        Spans::from(Span::styled(
            "highest wpm",
            Style::default().fg(theme.stats_label.to_tui_color()),
        )),
        Spans::from(Span::styled(format!("{:.0}", h_wpm), Style::default().fg(theme.foreground.to_tui_color()))),
        Spans::from(Span::styled(format!("{} {}", h_mode, h_val), Style::default().fg(theme.foreground.to_tui_color()))),
        Spans::from(Span::styled(
            "highest raw wpm",
            Style::default().fg(theme.stats_label.to_tui_color()),
        )),
        Spans::from(Span::styled(format!("{:.0}", h_raw), Style::default().fg(theme.foreground.to_tui_color()))),
        Spans::from(Span::styled(
            "highest accuracy",
            Style::default().fg(theme.stats_label.to_tui_color()),
        )),
        Spans::from(Span::styled(format!("{:.0}%", h_acc), Style::default().fg(theme.foreground.to_tui_color()))),
        Spans::from(Span::styled(
            "highest consistency",
            Style::default().fg(theme.stats_label.to_tui_color()),
        )),
        Spans::from(Span::styled(format!("{:.0}%", h_cons), Style::default().fg(theme.foreground.to_tui_color()))),
    ])
    .block(Block::default().borders(Borders::ALL).title(" Summary ").border_style(Style::default().fg(theme.border.to_tui_color())).style(Style::default().bg(theme.background.to_tui_color()).fg(theme.foreground.to_tui_color())));
    f.render_widget(left, cols[0]);

    // Overall
    let completion_pct = if started > 0 {
        (completed as f64 / started as f64) * 100.0
    } else {
        0.0
    };

    let mid = Paragraph::new(vec![
        Spans::from(Span::styled(
            "estimated words typed",
            Style::default().fg(theme.stats_label.to_tui_color()),
        )),
        Spans::from(Span::styled(sep_f64_0(est_words), Style::default().fg(theme.foreground.to_tui_color()))),
        Spans::from(Span::styled(
            "tests completed",
            Style::default().fg(theme.stats_label.to_tui_color()),
        )),
        Spans::from(Span::styled(
            format!("{} ({:.0}%)", sep_int(completed as u64), completion_pct),
            Style::default().fg(theme.foreground.to_tui_color()),
        )),
        Spans::from(Span::styled(
            "average wpm",
            Style::default().fg(theme.stats_label.to_tui_color()),
        )),
        Spans::from(Span::styled(format!("{:.0}", avg_wpm), Style::default().fg(theme.foreground.to_tui_color()))),
        Spans::from(Span::styled(
            "average raw wpm",
            Style::default().fg(theme.stats_label.to_tui_color()),
        )),
        Spans::from(Span::styled(format!("{:.0}", avg_raw), Style::default().fg(theme.foreground.to_tui_color()))),
        Spans::from(Span::styled(
            "avg accuracy",
            Style::default().fg(theme.stats_label.to_tui_color()),
        )),
        Spans::from(Span::styled(format!("{:.0}%", avg_acc), Style::default().fg(theme.foreground.to_tui_color()))),
        Spans::from(Span::styled(
            "avg consistency",
            Style::default().fg(theme.stats_label.to_tui_color()),
        )),
        Spans::from(Span::styled(format!("{:.0}%", avg_cons), Style::default().fg(theme.foreground.to_tui_color()))),
    ])
    .block(Block::default().borders(Borders::ALL).title(" Overall ").border_style(Style::default().fg(theme.border.to_tui_color())).style(Style::default().bg(theme.background.to_tui_color()).fg(theme.foreground.to_tui_color())));
    f.render_widget(mid, cols[1]);

    // Recent (last-10 averages)
    let hrs = (total_secs as u64) / 3600;
    let mins = ((total_secs as u64) % 3600) / 60;
    let secs = (total_secs as u64) % 60;
    let right = Paragraph::new(vec![
        Spans::from(Span::styled(
            "time typing",
            Style::default().fg(theme.stats_label.to_tui_color()),
        )),
        Spans::from(Span::styled(format!("{:02}:{:02}:{:02}", hrs, mins, secs), Style::default().fg(theme.foreground.to_tui_color()))),
        Spans::from(Span::styled(
            "avg wpm (last 10)",
            Style::default().fg(theme.stats_label.to_tui_color()),
        )),
        Spans::from(Span::styled(format!("{:.0}", avg10_wpm), Style::default().fg(theme.foreground.to_tui_color()))),
        Spans::from(Span::styled(
            "avg raw wpm (last 10)",
            Style::default().fg(theme.stats_label.to_tui_color()),
        )),
        Spans::from(Span::styled(format!("{:.0}", avg10_raw), Style::default().fg(theme.foreground.to_tui_color()))),
        Spans::from(Span::styled(
            "avg accuracy (last 10)",
            Style::default().fg(theme.stats_label.to_tui_color()),
        )),
        Spans::from(Span::styled(format!("{:.0}%", avg10_acc), Style::default().fg(theme.foreground.to_tui_color()))),
        Spans::from(Span::styled(
            "avg consistency (last 10)",
            Style::default().fg(theme.stats_label.to_tui_color()),
        )),
        Spans::from(Span::styled(format!("{:.0}%", avg10_cons), Style::default().fg(theme.foreground.to_tui_color()))),
    ])
    .block(Block::default().borders(Borders::ALL).title(" Recent ").border_style(Style::default().fg(theme.border.to_tui_color())).style(Style::default().bg(theme.background.to_tui_color()).fg(theme.foreground.to_tui_color())));
    f.render_widget(right, cols[2]);

    // Last 365 Days summary — use datetime window (no midnight snap)
    let mut stmt_365 = conn
        .prepare(
            "SELECT
               COUNT(*) FILTER (WHERE wpm>0),
               COUNT(*),
               AVG(wpm) FILTER (WHERE wpm>0),
               AVG(accuracy)
             FROM tests
             WHERE started_at >= datetime('now','-365 days')",
        )
        .unwrap();
    let (done, total, avg_year_wpm, avg_year_acc): (u32, u32, f64, f64) = stmt_365
        .query_row([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
        .unwrap_or((0, 0, 0.0, 0.0));
    let summary = Paragraph::new(vec![Spans::from(vec![
        Span::styled("Total: ", Style::default().fg(theme.stats_label.to_tui_color())),
        Span::styled(sep_int(total as u64), Style::default().fg(theme.foreground.to_tui_color())),
        Span::raw("   "),
        Span::styled("Done:  ", Style::default().fg(theme.stats_label.to_tui_color())),
        Span::styled(sep_int(done as u64), Style::default().fg(theme.foreground.to_tui_color())),
        Span::raw("   "),
        Span::styled("WPM:   ", Style::default().fg(theme.stats_label.to_tui_color())),
        Span::styled(format!("{:.1}", avg_year_wpm), Style::default().fg(theme.foreground.to_tui_color())),
        Span::raw("   "),
        Span::styled("Acc:   ", Style::default().fg(theme.stats_label.to_tui_color())),
        Span::styled(format!("{:.1}%", avg_year_acc), Style::default().fg(theme.foreground.to_tui_color())),
    ])])
    .block(Block::default().borders(Borders::ALL).title(" Last 365 Days ").border_style(Style::default().fg(theme.border.to_tui_color())).style(Style::default().bg(theme.background.to_tui_color()).fg(theme.foreground.to_tui_color())))
    .wrap(Wrap { trim: true });
    f.render_widget(summary, chunks[1]);

    // Bottom: WPM chart + table. Anchor the table to the bottom by giving
    // it a fixed height and allowing the chart area to take the remaining
    // space above it. This prevents the table from floating up when the
    // profile pane is taller than expected.
    let bottom = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(8)])
        .split(chunks[2]);

    // WPM-over-time chart (chronological by started_at)
    // NOTE: the DB stores epoch seconds; plotting raw epoch values makes the
    // numeric x-axis huge (e.g. 1_7e9). Convert to relative seconds since the
    // first timestamp in the window so the chart shows elapsed time instead.
    let raw_data: Vec<(u64, f64)> = conn
        .prepare(
            "SELECT CAST(strftime('%s', started_at) AS INTEGER) AS ts, wpm
               FROM tests
              WHERE started_at >= datetime('now','-365 days')
              ORDER BY ts ASC",
        )
        .unwrap()
        .query_map([], |r| Ok((r.get::<_, i64>(0)? as u64, r.get(1)?)))
        .unwrap()
        .filter_map(Result::ok)
        .collect();

    let data: Vec<(u64, f64)> = if raw_data.is_empty() {
        raw_data
    } else {
        let min_ts = raw_data.first().unwrap().0;
        raw_data.into_iter().map(|(t, w)| (t.saturating_sub(min_ts), w)).collect()
    };

    graph::draw_wpm_chart(f, bottom[0], &data, theme);

    // Scrollable Recent Tests table
        let sql = format!(
                "SELECT t.started_at, t.wpm,
                                COALESCE((t.correct_chars+t.incorrect_chars)*1.0/5.0
                                    / NULLIF(t.duration_ms/60000.0,0), 0.0) AS raw,
                                t.accuracy,
                                COALESCE(100.0*MIN(s.wpm)/NULLIF(MAX(s.wpm),0), 0.0) AS consistency,
                                t.mode, t.target_value
                     FROM tests t
            LEFT JOIN samples s ON s.test_id=t.id
                    GROUP BY t.id
                    ORDER BY t.started_at DESC
                    LIMIT {} OFFSET {}",
                PAGE_SIZE, offset
        );

    let mut stmt = conn.prepare(&sql).unwrap();
    let recent: Vec<_> = stmt
        .query_map([], |r| {
            let ts: String = r.get(0)?;
            // Display in local time if timestamp parses; otherwise fall back to raw string.
            let dt_str = DateTime::parse_from_rfc3339(&ts)
                .map(|dt: DateTime<FixedOffset>| {
                    dt.with_timezone(&Local)
                        .format("%Y-%m-%d %H:%M:%S")
                        .to_string()
                })
                .unwrap_or(ts.clone());

            Ok((
                dt_str,
                r.get::<_, f64>(1)?,
                r.get::<_, f64>(2)?,
                r.get::<_, f64>(3)?,
                r.get::<_, f64>(4)?,
                format!("{} {}", r.get::<_, String>(5)?, r.get::<_, i64>(6)?),
            ))
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect();

    // Build rows
    let rows: Vec<Row> = recent
        .into_iter()
        .map(|(d, net, raw, acc, cons, m)| {
            Row::new(vec![
                Cell::from(d),
                Cell::from(format!("{:.1}", net)),
                Cell::from(format!("{:.1}", raw)),
                Cell::from(format!("{:.1}%", acc)),
                Cell::from(format!("{:.1}%", cons)),
                Cell::from(m),
            ])
        })
        .collect();

    // Use a *stateful* table for selection so highlight always reflects the cursor.
    let mut state = TableState::default();
    // In sliding window mode, selected_idx should always be valid within the
    // fetched rows unless we have no data at all. The sliding window logic
    // above ensures selected_idx is computed correctly relative to the offset.
    let display_selected = if rows.is_empty() {
        None
    } else {
        // selected_idx should be valid since it's computed as cursor - offset
        // and we fetched rows starting from offset. Only clamp if necessary.
        Some(selected_idx.min(rows.len().saturating_sub(1)))
    };
    state.select(display_selected);

    let table = Table::new(rows)
        .header(
            Row::new(vec![
                "Started At (local)",
                "WPM",
                "Raw",
                "Accuracy",
                "Consistency",
                "Mode",
            ])
            .style(Style::default().fg(theme.stats_value.to_tui_color())),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    " Recent Tests  (↑ older, ↓ newer, PgUp/PgDn page, Home/End) ",
                    Style::default().fg(theme.title.to_tui_color()),
                ))
                .border_style(Style::default().fg(theme.border.to_tui_color()))
                .style(Style::default().bg(theme.background.to_tui_color()).fg(theme.foreground.to_tui_color())),
        )
        .widths(&[
            Constraint::Length(19),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(9),
            Constraint::Length(11),
            Constraint::Length(12),
        ])
        .column_spacing(1)
    .highlight_style(Style::default().bg(theme.info.to_tui_color()))
        .highlight_symbol(" ");

    // IMPORTANT: render as stateful to activate highlight behavior
    f.render_stateful_widget(table, bottom[1], &mut state);
}
