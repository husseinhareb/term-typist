// src/ui/profile.rs

use chrono::{DateTime, FixedOffset};
use rusqlite::Connection;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};
use crate::graph;

/// Draws the “Profile” page: summary, WPM over time, and recent-tests table.
pub fn draw_profile<B: Backend>(f: &mut Frame<B>, conn: &Connection) {
    // 1) Summary metrics over last 365 days
    let mut stmt = conn.prepare(
        "SELECT COUNT(*) FILTER (WHERE wpm>0),
                COUNT(*),
                AVG(wpm) FILTER (WHERE wpm>0),
                AVG(accuracy)
           FROM tests
          WHERE started_at >= date('now','-364 days')",
    ).unwrap();
    let (done, total, avg_wpm, avg_acc): (u32, u32, f64, f64) =
        stmt.query_row([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
            .unwrap_or((0, 0, 0.0, 0.0));

    // Layout: [summary 3 rows] [rest]
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5)])
        .split(f.size());

    // ── SUMMARY ─────────────────────────────────────────────
    let summary = Paragraph::new(vec![Spans::from(vec![
        Span::styled("Total: ", Style::default().fg(Color::Gray)),
        Span::raw(total.to_string()),
        Span::raw("   "),
        Span::styled("Done:  ", Style::default().fg(Color::Gray)),
        Span::raw(done.to_string()),
        Span::raw("   "),
        Span::styled("WPM:   ", Style::default().fg(Color::Gray)),
        Span::raw(format!("{:.1}", avg_wpm)),
        Span::raw("   "),
        Span::styled("Acc:   ", Style::default().fg(Color::Gray)),
        Span::raw(format!("{:.1}%", avg_acc)),
    ])])
    .block(Block::default().borders(Borders::ALL).title(" Profile "));
    f.render_widget(summary, chunks[0]);

    // ── BOTTOM: chart (8 rows) + table ─────────────────────
    let bottom_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(3)])
        .split(chunks[1]);

    // 2) WPM-over-time chart
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
    graph::draw_wpm_chart(f, bottom_chunks[0], &data);

    // 3) Recent tests table (extended with raw & consistency, nulls -> 0)
    let mut stmt = conn.prepare(r#"
        SELECT
          t.started_at,
          t.wpm,
          COALESCE((t.correct_chars + t.incorrect_chars)*1.0/5.0
            / NULLIF(t.duration_ms/60000.0, 0), 0.0)    AS raw,
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
            // parse the RFC3339 timestamp and format "YYYY-MM-DD HH:MM:SS"
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
            let mode_str = format!("{} {}", mode, val);

            Ok((dt, net, raw, acc, cons, mode_str))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect();

    let table_rows: Vec<Row> = recent
        .into_iter()
        .map(|(d, net, raw, acc, cons, mode)| {
            Row::new(vec![
                Cell::from(d),
                Cell::from(format!("{:.1}", net)),
                Cell::from(format!("{:.1}", raw)),
                Cell::from(format!("{:.1}%", acc)),
                Cell::from(format!("{:.1}%", cons)),
                Cell::from(mode),
            ])
        })
        .collect();

    let table = Table::new(table_rows)
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

    f.render_widget(table, bottom_chunks[1]);
}
