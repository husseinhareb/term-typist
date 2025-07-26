// src/ui/profile.rs

use chrono::{DateTime, FixedOffset};
use rusqlite::Connection;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
    Frame,
};
use crate::graph;

/// Draws the “Profile” page: top stats grid, 365-day summary, WPM over time chart, and recent-tests table.
pub fn draw_profile<B: Backend>(f: &mut Frame<B>, conn: &Connection) {
    //
    // 1) Compute all aggregates
    //

    // overall aggregates
    let mut agg = conn.prepare(r#"
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
    "#).unwrap();

    let (started, completed, est_words, avg_wpm, avg_raw, avg_acc, total_secs):
        (u32, u32, f64, f64, f64, f64, f64) =
        agg.query_row([], |r| Ok((
            r.get(0)?, r.get(1)?, r.get(2)?,
            r.get(3)?, r.get(4)?, r.get(5)?,
            r.get(6)?
        ))).unwrap_or((0,0,0.0,0.0,0.0,0.0,0.0));

    // highest net WPM
    let mut hnet = conn.prepare(
        "SELECT wpm, mode, target_value FROM tests ORDER BY wpm DESC LIMIT 1"
    ).unwrap();
    let (h_wpm, h_mode, h_val): (f64, String, i64) =
        hnet.query_row([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap_or((0.0, "".into(), 0));

    // highest raw WPM
    let mut hraw = conn.prepare(
        "SELECT (correct_chars+incorrect_chars)*1.0/5.0
            / NULLIF(duration_ms/60000.0,0) AS raw
         FROM tests
         ORDER BY raw DESC LIMIT 1"
    ).unwrap();
    let h_raw: f64 = hraw.query_row([], |r| r.get(0)).unwrap_or(0.0);

    // highest accuracy
    let mut hacc = conn.prepare(
        "SELECT accuracy FROM tests ORDER BY accuracy DESC LIMIT 1"
    ).unwrap();
    let h_acc: f64 = hacc.query_row([], |r| r.get(0)).unwrap_or(0.0);

    // highest consistency per single test
    let mut hcons = conn.prepare(
        "SELECT MAX(cons) FROM (
           SELECT 100.0*MIN(wpm)/MAX(wpm) AS cons
             FROM samples
            GROUP BY test_id
         )"
    ).unwrap();
    let h_cons: f64 = hcons.query_row([], |r| r.get(0)).unwrap_or(0.0);

    // average consistency across all tests
    let mut avgcons = conn.prepare(
        "SELECT AVG(cons) FROM (
           SELECT 100.0*MIN(wpm)/MAX(wpm) AS cons
             FROM samples
            GROUP BY test_id
         )"
    ).unwrap();
    let avg_cons: f64 = avgcons.query_row([], |r| r.get(0)).unwrap_or(0.0);

    // last 10 tests summary (for “last 10” averages)
    let mut last10 = conn.prepare(r#"
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
    "#).unwrap();

    let mut rows = last10.query([]).unwrap();
    let mut cnt = 0;
    let (mut sum_w, mut sum_r, mut sum_a, mut sum_c) = (0.0, 0.0, 0.0, 0.0);
    while let Ok(Some(r)) = rows.next() {
        cnt += 1;
        sum_w += r.get::<_, f64>(0).unwrap_or(0.0);
        sum_r += r.get::<_, f64>(1).unwrap_or(0.0);
        sum_a += r.get::<_, f64>(2).unwrap_or(0.0);
        sum_c += r.get::<_, f64>(3).unwrap_or(0.0);
    }
    let (avg10_wpm, avg10_raw, avg10_acc, avg10_cons) = if cnt > 0 {
        (sum_w/cnt as f64, sum_r/cnt as f64, sum_a/cnt as f64, sum_c/cnt as f64)
    } else {
        (0.0, 0.0, 0.0, 0.0)
    };

    //
    // 2) Layout: top grid (12 rows), 365-day summary (3), bottom (chart+table)
    //

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(12),
            Constraint::Length(3),
            Constraint::Min(5),
        ])
        .split(f.size());

    // ── TOP GRID (three equal columns) ─────────────────────
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(33),
            Constraint::Percentage(34),
            Constraint::Percentage(33),
        ])
        .split(chunks[0]);

    // Left column: overall highs + counts
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

    // Middle column: overall averages & estimated words
    let mid = Paragraph::new(vec![
        Spans::from(Span::styled("estimated words typed", Style::default().fg(Color::Gray))),
        Spans::from(Span::raw(format!("{:.0}", est_words))),
        Spans::from(Span::styled("tests completed", Style::default().fg(Color::Gray))),
        Spans::from(Span::raw(format!("{} ({:.0}%)",
            completed,
            if started>0 { completed as f64/started as f64*100.0 } else {0.0}
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

    // Right column: total time + last-10 averages
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

    //
    // 3) 365-day summary block
    //
    let mut stmt = conn.prepare(
        "SELECT
           COUNT(*) FILTER (WHERE wpm>0),
           COUNT(*),
           AVG(wpm) FILTER (WHERE wpm>0),
           AVG(accuracy)
         FROM tests
        WHERE started_at >= date('now','-364 days')"
    ).unwrap();

    let (done, total, avg_year_wpm, avg_year_acc): (u32, u32, f64, f64) =
        stmt.query_row([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
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

    //
    // 4) Bottom: chart + recent-tests table
    //
    let bottom = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(3)])
        .split(chunks[2]);

    // 4a) WPM-over-time chart
    let data: Vec<(u64, f64)> = conn
        .prepare(
            "SELECT duration_ms/1000 AS sec, wpm
               FROM tests
              WHERE started_at >= date('now','-364 days')
              ORDER BY sec ASC",
        ).unwrap()
        .query_map([], |r| Ok((r.get::<_, i64>(0)? as u64, r.get(1)?)))
        .unwrap()
        .map(Result::unwrap)
        .collect();

    graph::draw_wpm_chart(f, bottom[0], &data);

    // 4b) Recent-tests table, extended
    let mut stmt = conn.prepare(r#"
        SELECT
          t.started_at,
          t.wpm,
          COALESCE((t.correct_chars + t.incorrect_chars)*1.0/5.0
            / NULLIF(t.duration_ms/60000.0,0), 0.0)    AS raw,
          t.accuracy,
          COALESCE(100.0 * MIN(s.wpm) / MAX(s.wpm), 0.0)   AS consistency,
          t.mode,
          t.target_value
        FROM tests t
        LEFT JOIN samples s ON s.test_id = t.id
        GROUP BY t.id
        ORDER BY t.started_at DESC
        LIMIT 10
    "#).unwrap();

    let recent: Vec<(String, f64, f64, f64, f64, String)> = stmt
        .query_map([], |r| {
            let ts: String = r.get(0)?;
            let dt = DateTime::parse_from_rfc3339(&ts)
                .map(|dt: DateTime<FixedOffset>| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or(ts.clone());
            let net: f64 = r.get(1)?;
            let raw: f64 = r.get(2)?;
            let acc: f64 = r.get(3)?;
            let cons: f64 = r.get(4)?;
            let mode: String = r.get(5)?;
            let val: i64 = r.get(6)?;
            Ok((dt, net, raw, acc, cons, format!("{} {}", mode, val)))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect();

    let rows: Vec<Row> = recent.into_iter()
        .map(|(d, net, raw, a, c, m)| {
            Row::new(vec![
                Cell::from(d),
                Cell::from(format!("{:.1}", net)),
                Cell::from(format!("{:.1}", raw)),
                Cell::from(format!("{:.1}%", a)),
                Cell::from(format!("{:.1}%", c)),
                Cell::from(m),
            ])
        })
        .collect();

    let table = Table::new(rows)
        .header(
            Row::new(vec![
                "Started At", "WPM", "Raw", "Accuracy", "Consistency", "Mode",
            ]).style(Style::default().fg(Color::Yellow))
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
