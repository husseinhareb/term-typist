// src/ui/profile.rs

use chrono::{DateTime, FixedOffset};
use rusqlite::Connection;
use std::sync::atomic::{AtomicUsize, Ordering};
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
    Frame,
};
use crate::graph;
use crossterm::event::KeyCode;

/// Absolute cursor into your tests (0 = newest, 1 = next older, …).
static RECENT_CURSOR: AtomicUsize = AtomicUsize::new(0);

/// Rows per page in the Recent Tests table.
const PAGE_SIZE: u32 = 10;

/// Reset to cursor 0 (call when you open the Profile page).
pub fn reset_profile_page() {
    RECENT_CURSOR.store(0, Ordering::Relaxed);
}

/// Handle Up/Down arrows when Profile is active:
/// - Up   → older tests (cursor += 1)  
/// - Down → newer tests (cursor -= 1, clamped at 0)
pub fn handle_profile_scroll(key: &KeyCode) {
    match key {
        KeyCode::Up => {
            RECENT_CURSOR.fetch_add(1, Ordering::Relaxed);
        }
        KeyCode::Down => {
            let _ = RECENT_CURSOR.fetch_update(
                Ordering::Relaxed,
                Ordering::Relaxed,
                |c| c.checked_sub(1),
            );
        }
        _ => {}
    }
}

/// Draws the Profile screen: top stats grid, 365-day summary, WPM chart,
/// and a scrollable Recent Tests table with highlight.
pub fn draw_profile<B: Backend>(f: &mut Frame<B>, conn: &Connection) {
    // 0) Determine total_tests and clamp cursor
    let total_tests: u32 = conn
        .prepare("SELECT COUNT(*) FROM tests")
        .unwrap()
        .query_row([], |r| r.get(0))
        .unwrap_or(0);

    let mut cursor = RECENT_CURSOR.load(Ordering::Relaxed) as u32;
    if total_tests > 0 {
        cursor = cursor.min(total_tests - 1);
    } else {
        cursor = 0;
    }
    RECENT_CURSOR.store(cursor as usize, Ordering::Relaxed);

    // Compute which page and index on that page
    let page = cursor / PAGE_SIZE;
    let selected_idx = (cursor % PAGE_SIZE) as usize;

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
        ).unwrap();
    let (started, completed, est_words, avg_wpm, avg_raw, avg_acc, total_secs): (
        u32, u32, f64, f64, f64, f64, f64
    ) = agg
        .query_row([], |r| {
            Ok((
                r.get(0)?, r.get(1)?, r.get(2)?,
                r.get(3)?, r.get(4)?, r.get(5)?,
                r.get(6)?,
            ))
        })
        .unwrap_or((0, 0, 0.0, 0.0, 0.0, 0.0, 0.0));

    let mut hnet = conn
        .prepare("SELECT wpm, mode, target_value FROM tests ORDER BY wpm DESC LIMIT 1")
        .unwrap();
    let (h_wpm, h_mode, h_val): (f64, String, i64) =
        hnet.query_row([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap_or((0.0, "".into(), 0));

    let mut hraw = conn
        .prepare(
            "SELECT (correct_chars+incorrect_chars)*1.0/5.0
                / NULLIF(duration_ms/60000.0,0) AS raw
             FROM tests
             ORDER BY raw DESC LIMIT 1",
        )
        .unwrap();
    let h_raw: f64 = hraw.query_row([], |r| r.get(0)).unwrap_or(0.0);

    let mut hacc = conn
        .prepare("SELECT accuracy FROM tests ORDER BY accuracy DESC LIMIT 1")
        .unwrap();
    let h_acc: f64 = hacc.query_row([], |r| r.get(0)).unwrap_or(0.0);

    let mut hcons = conn
        .prepare(
            "SELECT MAX(cons) FROM (
               SELECT 100.0*MIN(wpm)/MAX(wpm) AS cons
                 FROM samples
                GROUP BY test_id
             )",
        )
        .unwrap();
    let h_cons: f64 = hcons.query_row([], |r| r.get(0)).unwrap_or(0.0);

    let mut avgcons = conn
        .prepare(
            "SELECT AVG(cons) FROM (
               SELECT 100.0*MIN(wpm)/MAX(wpm) AS cons
                 FROM samples
                GROUP BY test_id
             )",
        )
        .unwrap();
    let avg_cons: f64 = avgcons.query_row([], |r| r.get(0)).unwrap_or(0.0);

    // Last-10 summary
    let mut last10 = conn
        .prepare(
            r#"
        SELECT
          t.wpm,
          COALESCE((t.correct_chars+t.incorrect_chars)*1.0/5.0
            / NULLIF(t.duration_ms/60000.0,0), 0) AS raw,
          t.accuracy,
          COALESCE(100.0*MIN(s.wpm)/MAX(s.wpm), 0) AS cons
        FROM tests t
        LEFT JOIN samples s ON s.test_id = t.id
        GROUP BY t.id
        ORDER BY t.started_at DESC
        LIMIT 10
    "#,
        ).unwrap();

    let mut rows_iter = last10.query([]).unwrap();
    let mut cnt = 0;
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
        .constraints([Constraint::Percentage(33), Constraint::Percentage(34), Constraint::Percentage(33)])
        .split(chunks[0]);

    // Summary
    let left = Paragraph::new(vec![
        Spans::from(Span::styled("tests started", Style::default().fg(Color::Gray))),
        Spans::from(Span::raw(started.to_string())),
        Spans::from(Span::styled("highest wpm", Style::default().fg(Color::Gray))),
        Spans::from(Span::raw(format!("{:.0}", h_wpm))),
        Spans::from(Span::raw(format!("{} {}", h_mode, h_val))),
        Spans::from(Span::styled("highest raw wpm", Style::default().fg(Color::Gray))),
        Spans::from(Span::raw(format!("{:.0}", h_raw))),
        Spans::from(Span::styled("highest accuracy", Style::default().fg(Color::Gray))),
        Spans::from(Span::raw(format!("{:.0}%", h_acc))),
        Spans::from(Span::styled("highest consistency", Style::default().fg(Color::Gray))),
        Spans::from(Span::raw(format!("{:.0}%", h_cons))),
    ])
    .block(Block::default().borders(Borders::ALL).title(" Summary "));
    f.render_widget(left, cols[0]);

    // Overall
    let mid = Paragraph::new(vec![
        Spans::from(Span::styled("estimated words typed", Style::default().fg(Color::Gray))),
        Spans::from(Span::raw(format!("{:.0}", est_words))),
        Spans::from(Span::styled("tests completed", Style::default().fg(Color::Gray))),
        Spans::from(Span::raw(format!(
            "{} ({:.0}%)",
            completed,
            if started > 0 {
                completed as f64 / started as f64 * 100.0
            } else {
                0.0
            }
        ))),
        Spans::from(Span::styled("average wpm", Style::default().fg(Color::Gray))),
        Spans::from(Span::raw(format!("{:.0}", avg_wpm))),
        Spans::from(Span::styled("average raw wpm", Style::default().fg(Color::Gray))),
        Spans::from(Span::raw(format!("{:.0}", avg_raw))),
        Spans::from(Span::styled("avg accuracy", Style::default().fg(Color::Gray))),
        Spans::from(Span::raw(format!("{:.0}%", avg_acc))),
        Spans::from(Span::styled("avg consistency", Style::default().fg(Color::Gray))),
        Spans::from(Span::raw(format!("{:.0}%", avg_cons))),
    ])
    .block(Block::default().borders(Borders::ALL).title(" Overall "));
    f.render_widget(mid, cols[1]);

    // Recent (last-10 averages)
    let hrs = (total_secs as u64) / 3600;
    let mins = ((total_secs as u64) % 3600) / 60;
    let secs = (total_secs as u64) % 60;
    let right = Paragraph::new(vec![
        Spans::from(Span::styled("time typing", Style::default().fg(Color::Gray))),
        Spans::from(Span::raw(format!("{:02}:{:02}:{:02}", hrs, mins, secs))),
        Spans::from(Span::styled("avg wpm (last 10)", Style::default().fg(Color::Gray))),
        Spans::from(Span::raw(format!("{:.0}", avg10_wpm))),
        Spans::from(Span::styled("avg raw wpm (last 10)", Style::default().fg(Color::Gray))),
        Spans::from(Span::raw(format!("{:.0}", avg10_raw))),
        Spans::from(Span::styled("avg accuracy (last 10)", Style::default().fg(Color::Gray))),
        Spans::from(Span::raw(format!("{:.0}%", avg10_acc))),
        Spans::from(Span::styled("avg consistency (last 10)", Style::default().fg(Color::Gray))),
        Spans::from(Span::raw(format!("{:.0}%", avg10_cons))),
        Spans::from(Span::styled("Export CSV", Style::default().fg(Color::Gray))),
    ])
    .block(Block::default().borders(Borders::ALL).title(" Recent "));
    f.render_widget(right, cols[2]);

    // Last 365 Days summary
    let mut stmt_365 = conn
        .prepare(
            "SELECT
               COUNT(*) FILTER (WHERE wpm>0),
               COUNT(*),
               AVG(wpm) FILTER (WHERE wpm>0),
               AVG(accuracy)
             FROM tests
             WHERE started_at >= date('now','-364 days')",
        )
        .unwrap();
    let (done, total, avg_year_wpm, avg_year_acc): (u32, u32, f64, f64) =
        stmt_365
            .query_row([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
            .unwrap_or((0, 0, 0.0, 0.0));
    let summary = Paragraph::new(vec![Spans::from(vec![
        Span::styled("Total: ", Style::default().fg(Color::Gray)),
        Span::raw(total.to_string()),
        Span::raw("   "),
        Span::styled("Done:  ", Style::default().fg(Color::Gray)),
        Span::raw(done.to_string()),
        Span::raw("   "),
        Span::styled("WPM:   ", Style::default().fg(Color::Gray)),
        Span::raw(format!("{:.1}", avg_year_wpm)),
        Span::raw("   "),
        Span::styled("Acc:   ", Style::default().fg(Color::Gray)),
        Span::raw(format!("{:.1}%", avg_year_acc)),
    ])])
    .block(Block::default().borders(Borders::ALL).title(" Last 365 Days "))
    .wrap(Wrap { trim: true });
    f.render_widget(summary, chunks[1]);

    // Bottom: WPM chart + table
    let bottom = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(3)])
        .split(chunks[2]);

    // WPM-over-time chart
    let data: Vec<(u64, f64)> = conn
        .prepare(
            "SELECT duration_ms/1000 AS sec, wpm
               FROM tests
              WHERE started_at >= date('now','-364 days')
              ORDER BY sec ASC",
        )
        .unwrap()
        .query_map([], |r| Ok((r.get::<_, i64>(0)? as u64, r.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();
    graph::draw_wpm_chart(f, bottom[0], &data);

    // Scrollable Recent Tests table
    let offset = page * PAGE_SIZE;
    let sql = format!(
        "SELECT t.started_at, t.wpm, \
         COALESCE((t.correct_chars+t.incorrect_chars)*1.0/5.0/NULLIF(t.duration_ms/60000.0,0),0.0) AS raw, \
         t.accuracy, \
         COALESCE(100.0*MIN(s.wpm)/MAX(s.wpm),0.0) AS consistency, \
         t.mode, t.target_value \
         FROM tests t \
         LEFT JOIN samples s ON s.test_id=t.id \
         GROUP BY t.id \
         ORDER BY t.started_at DESC \
         LIMIT {} OFFSET {}",
        PAGE_SIZE, offset
    );

    let mut stmt = conn.prepare(&sql).unwrap();
    let recent: Vec<_> = stmt
        .query_map([], |r| {
            let ts: String = r.get(0)?;
            let dt = DateTime::parse_from_rfc3339(&ts)
                .map(|dt: DateTime<FixedOffset>| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or(ts.clone());
            Ok((
                dt,
                r.get::<_, f64>(1)?,
                r.get::<_, f64>(2)?,
                r.get::<_, f64>(3)?,
                r.get::<_, f64>(4)?,
                format!("{} {}", r.get::<_, String>(5)?, r.get::<_, i64>(6)?),
            ))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect();

    // Build and highlight rows
    let rows: Vec<Row> = recent
        .into_iter()
        .enumerate()
        .map(|(i, (d, net, raw, acc, cons, m))| {
            let mut row = Row::new(vec![
                Cell::from(d),
                Cell::from(format!("{:.1}", net)),
                Cell::from(format!("{:.1}", raw)),
                Cell::from(format!("{:.1}%", acc)),
                Cell::from(format!("{:.1}%", cons)),
                Cell::from(m),
            ]);
            if i == selected_idx {
                row = row.style(Style::default().bg(Color::Blue));
            }
            row
        })
        .collect();

    let table = Table::new(rows)
        .header(
            Row::new(vec![
                "Started At", "WPM", "Raw", "Accuracy", "Consistency", "Mode",
            ])
            .style(Style::default().fg(Color::Yellow)),
        )
        .block(Block::default().borders(Borders::ALL).title(" Recent Tests "))
        .widths(&[
            Constraint::Length(19),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(9),
            Constraint::Length(11),
            Constraint::Length(10),
        ])
        .column_spacing(1);

    f.render_widget(table, bottom[1]);
}
