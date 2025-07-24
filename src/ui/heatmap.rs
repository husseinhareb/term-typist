// src/ui/heatmap.rs

use chrono::{Datelike, Duration as ChronoDuration, NaiveDate, Utc};
use std::collections::HashMap;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

/// `data` maps each day to how many tests were run on that date.
/// Renders a 7×N grid of 2‑char‑wide squares (plus 1‑char spacing)
/// covering up to the last 365 days.
pub fn draw_heatmap<B: Backend>(
    f: &mut Frame<B>,
    area: Rect,
    data: &HashMap<NaiveDate, u32>,
) {
    // Each “cell” is 2 chars wide + 1 char padding = 3 total columns
    let cell_w = 3;
    let max_weeks = (area.width as usize / cell_w)
        .max(1)   // at least one column
        .min(53); // no more than 53 weeks

    // Find the Monday on or before today:
    let today = Utc::now().date_naive();
    let monday_offset = today.weekday().num_days_from_monday() as i64;
    let last_monday = today - ChronoDuration::days(monday_offset);

    // Start at (last_monday - (max_weeks-1) weeks):
    let start = last_monday - ChronoDuration::weeks((max_weeks as i64) - 1);

    // Build one row per weekday (Mon → Sun):
    let mut heat_rows = Vec::with_capacity(7);
    for dow in 0..7 {
        let mut spans = Vec::with_capacity(max_weeks * 2);
        for w in 0..max_weeks {
            let date = start
                + ChronoDuration::weeks(w as i64)
                + ChronoDuration::days(dow);
            let count = data.get(&date).copied().unwrap_or(0);

            // Bucket counts into colors:
            let bg = match count {
                0        => Color::DarkGray,
                1..=2    => Color::Green,
                3..=5    => Color::LightGreen,
                6..=10   => Color::Yellow,
                _        => Color::Red,
            };

            // two spaces for a “square” + one space
            spans.push(Span::styled("  ", Style::default().bg(bg)));
            spans.push(Span::raw(" "));
        }
        heat_rows.push(Spans::from(spans));
    }

    // Render the surrounding block, computing its inner rect first:
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Activity (365d) ");
    let inner = block.inner(area);
    f.render_widget(block, area);

    // Render the heatmap inside that inner rect:
    let heatmap = Paragraph::new(heat_rows)
        .wrap(Wrap { trim: false });
    f.render_widget(heatmap, inner);
}
