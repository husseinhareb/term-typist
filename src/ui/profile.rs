// src/ui/profile.rs

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

/// Slim row from your `tests` table.
struct TestRecord {
    started_at: String,
    wpm:        f64,
    accuracy:   f64,
    mode:       String,
}

impl TestRecord {
    pub fn load_recent(conn: &Connection) -> rusqlite::Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT started_at, wpm, accuracy, mode
               FROM tests
              ORDER BY started_at DESC
              LIMIT 100",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(TestRecord {
                started_at: r.get(0)?,
                wpm:        r.get(1)?,
                accuracy:   r.get(2)?,
                mode:       r.get(3)?,
            })
        })?;
        rows.collect()
    }
}

pub fn draw_profile<B: Backend>(f: &mut Frame<B>, conn: &Connection) {
    let tests = TestRecord::load_recent(conn).unwrap_or_default();
    let total = tests.len();
    let completed: usize = tests.iter().filter(|t| t.wpm > 0.0).count();
    let avg_wpm = if completed > 0 {
        tests.iter().map(|t| t.wpm).sum::<f64>() / completed as f64
    } else { 0.0 };
    let avg_acc = if completed > 0 {
        tests.iter().map(|t| t.accuracy).sum::<f64>() / completed as f64
    } else { 0.0 };

    // Layout: summary / chart / table
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(4), Constraint::Length(7), Constraint::Min(5)])
        .split(f.size());

    // 1) SUMMARY
    let summary = vec![
        Spans::from(vec![
            Span::styled("Started: ", Style::default().fg(Color::Gray)),
            Span::raw(total.to_string()),
            Span::raw("   "),
            Span::styled("Done: ", Style::default().fg(Color::Gray)),
            Span::raw(completed.to_string()),
        ]),
        Spans::from(vec![
            Span::styled("Avg WPM: ", Style::default().fg(Color::Gray)),
            Span::raw(format!("{:.1}", avg_wpm)),
            Span::raw("   "),
            Span::styled("Avg Acc: ", Style::default().fg(Color::Gray)),
            Span::raw(format!("{:.1}%", avg_acc)),
        ]),
    ];
    let summary_widget = Paragraph::new(summary)
        .block(Block::default().borders(Borders::ALL).title(" Profile "));
    f.render_widget(summary_widget, chunks[0]);

    // 2) CHART (index -> wpm)
    let chart_data: Vec<(u64, f64)> = tests
        .iter()
        .enumerate()
        .map(|(i, t)| (i as u64, t.wpm))
        .collect();
    graph::draw_wpm_chart(f, chunks[1], &chart_data);

    // 3) TABLE of recent tests
    let rows = tests.iter().map(|t| {
        Row::new(vec![
            Cell::from(t.started_at.clone()),
            Cell::from(format!("{:.0}", t.wpm)),
            Cell::from(format!("{:.0}%", t.accuracy)),
            Cell::from(t.mode.clone()),
        ])
    });
    let table = Table::new(rows)
        .header(Row::new(vec!["Date", "WPM", "Acc", "Mode"]).style(Style::default().fg(Color::Yellow)))
        .block(Block::default().borders(Borders::ALL).title(" Recent Tests "))
        .widths(&[
            Constraint::Length(20),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(8),
        ])
        .column_spacing(1);
    f.render_widget(table, chunks[2]);
}
