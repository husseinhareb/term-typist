// src/ui/profile.rs

use chrono::{Datelike, Duration as ChronoDuration, NaiveDate, Utc, Weekday};
use rusqlite::Connection;
use std::collections::HashMap;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, Wrap},
    Frame,
};

use crate::graph;
use crate::ui::heatmap::draw_heatmap;

pub fn draw_profile<B: Backend>(f: &mut Frame<B>, conn: &Connection) {
    // ─── 1) SUMMARY METRICS ────────────────────────────────────────
    let mut stmt = conn.prepare(
        "SELECT 
            COUNT(*) FILTER (WHERE wpm>0) AS done,
            COUNT(*)               AS total,
            COALESCE(AVG(wpm) FILTER (WHERE wpm>0),0)      AS avg_wpm,
            COALESCE(AVG(accuracy),0)                     AS avg_acc
         FROM tests
        WHERE started_at >= date('now','-364 days')",
    ).unwrap();

    let (done, total, avg_wpm, avg_acc): (u32,u32,f64,f64) =
        stmt.query_row([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
            .unwrap_or((0,0,0.0,0.0));

    // ─── 2) HEATMAP DATA ───────────────────────────────────────────
    let mut heat: HashMap<NaiveDate,u32> = HashMap::new();
    let mut stmt = conn.prepare(
        "SELECT substr(started_at,1,10) AS day, COUNT(*) 
           FROM tests
          WHERE started_at >= date('now','-364 days')
          GROUP BY day",
    ).unwrap();

    for result in stmt.query_map([], |r| {
        let s: String = r.get(0)?;
        let day = NaiveDate::parse_from_str(&s, "%Y-%m-%d").unwrap();
        let cnt: u32     = r.get(1)?;
        Ok((day, cnt))
    }).unwrap() {
        let (day, cnt) = result.unwrap();
        heat.insert(day, cnt);
    }

    // ─── 3) LAYOUT ────────────────────────────────────────────────
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // summary
            Constraint::Length(7), // 7 rows for Mon→Sun
            Constraint::Min(10),   // rest: chart + table
        ])
        .split(f.size());

    // ─── 4) RENDER SUMMARY ────────────────────────────────────────
    let summary = Paragraph::new(vec![
        Spans::from(vec![
            Span::styled("Total: ", Style::default().fg(Color::Gray)),
            Span::raw(total.to_string()),
            Span::raw("   "),
            Span::styled("Done: ",  Style::default().fg(Color::Gray)),
            Span::raw(done.to_string()),
            Span::raw("   "),
            Span::styled("WPM:  ",  Style::default().fg(Color::Gray)),
            Span::raw(format!("{:.1}", avg_wpm)),
            Span::raw("   "),
            Span::styled("Acc:  ",  Style::default().fg(Color::Gray)),
            Span::raw(format!("{:.1}%", avg_acc)),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).title(" Profile "));
    f.render_widget(summary, chunks[0]);

    // ─── 5) RENDER HEATMAP ────────────────────────────────────────
    let heat_block = Block::default().borders(Borders::ALL).title(" Activity (365d) ");
    f.render_widget(heat_block, chunks[1]);

    let inner = heat_block.inner(chunks[1]);
    draw_heatmap(f, inner, &heat);

    // ─── 6) BOTTOM: WPM CHART + RECENT TESTS ─────────────────────
    let bottom = chunks[2];
    let bottom_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(4)])
        .split(bottom);

    // 6a) WPM over time
    let wpm_data: Vec<(u64, f64)> = conn.prepare(
        "SELECT duration_ms/1000 AS sec, wpm
           FROM tests
          WHERE started_at >= date('now','-364 days')
          ORDER BY sec ASC",
    ).unwrap()
    .query_map([], |r| {
        Ok((r.get::<_,i64>(0)? as u64, r.get(1)?))
    }).unwrap()
    .map(|r| r.unwrap())
    .collect();
    graph::draw_wpm_chart(f, bottom_chunks[0], &wpm_data);

    // 6b) Recent tests table
    let recent: Vec<(String,f64,f64,String)> = conn.prepare(
        "SELECT started_at, wpm, accuracy, mode
           FROM tests
          ORDER BY started_at DESC
          LIMIT 10",
    ).unwrap()
    .query_map([], |r| {
        let ts: String = r.get(0)?;
        Ok((
            ts.chars().take(10).collect(), 
            r.get(1)?, r.get(2)?, r.get(3)?
        ))
    }).unwrap()
    .map(|r| r.unwrap())
    .collect();

    let rows: Vec<Row> = recent.iter().map(|(d,w,a,m)| {
        let cells: Vec<Cell> = vec![
            Cell::from(d.clone()),
            Cell::from(format!("{:.0}", w)),
            Cell::from(format!("{:.0}%", a)),
            Cell::from(m.clone()),
        ];
        Row::new(cells)
    }).collect();

    let table = Table::new(rows)
        .header(
            Row::new(vec!["Date","WPM","Acc","Mode"])
                .style(Style::default().fg(Color::Yellow))
        )
        .block(Block::default().borders(Borders::ALL).title(" Recent Tests "))
        .widths(&[
            Constraint::Length(10),
            Constraint::Length(6),
            Constraint::Length(6),
            Constraint::Length(8),
        ])
        .column_spacing(1);

    f.render_widget(table, bottom_chunks[1]);
}
