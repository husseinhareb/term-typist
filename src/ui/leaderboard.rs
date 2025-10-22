use chrono::{DateTime, FixedOffset, Local};
use crossterm::event::KeyCode;
use rusqlite::Connection;
use std::sync::atomic::{AtomicUsize, Ordering};
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Style, Modifier},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Row, Table, TableState, Clear, Paragraph, Wrap},
    Frame,
};

use crate::theme::Theme;

/// Cursor for leaderboard selection (absolute index into top-ranked list)
static LEADERBOARD_CURSOR: AtomicUsize = AtomicUsize::new(0);

/// Rows shown in the leaderboard
const TOP_N: u32 = 15;

/// Handle navigation keys when the leaderboard modal is active.
pub fn handle_leaderboard_key(code: KeyCode) {
    match code {
        KeyCode::Up | KeyCode::Char('k') => {
            let _ = LEADERBOARD_CURSOR.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |c| c.checked_sub(1));
        }
        KeyCode::Down | KeyCode::Char('j') => {
            LEADERBOARD_CURSOR.fetch_add(1, Ordering::Relaxed);
        }
        KeyCode::PageUp => {
            let _ = LEADERBOARD_CURSOR.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |c| c.checked_sub(TOP_N as usize));
        }
        KeyCode::PageDown => {
            LEADERBOARD_CURSOR.fetch_add(TOP_N as usize, Ordering::Relaxed);
        }
        KeyCode::Home => {
            LEADERBOARD_CURSOR.store(0, Ordering::Relaxed);
        }
        KeyCode::End => {
            LEADERBOARD_CURSOR.store(usize::MAX / 2, Ordering::Relaxed);
        }
        _ => {}
    }
}

/// Return the current leaderboard cursor (absolute index into the TOP_N ordering).
pub fn leaderboard_cursor() -> usize {
    LEADERBOARD_CURSOR.load(Ordering::Relaxed)
}

pub fn draw_leaderboard<B: Backend>(f: &mut Frame<B>, conn: &Connection, theme: &Theme) {
    let area = f.size();
    // modal size: 85% width, up to 22 rows height (or area.height - 6)
    let w = (area.width as f32 * 0.85) as u16;
    let max_h = area.height.saturating_sub(6);
    let h = (22u16).min(max_h).min(area.height);
    let x = area.x + (area.width.saturating_sub(w)) / 2;
    let y = area.y + (area.height.saturating_sub(h)) / 2;
    let rect = Rect::new(x, y, w, h);

    // Clear modal region
    f.render_widget(Clear, rect);

    // Draw block
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(" Leaderboard — Top Tests (Enter to view) ", Style::default().fg(theme.title.to_tui_color()).add_modifier(Modifier::BOLD)))
        .border_style(Style::default().fg(theme.border.to_tui_color()))
        .style(Style::default().bg(theme.background.to_tui_color()).fg(theme.foreground.to_tui_color()));
    f.render_widget(block.clone(), rect);

    let inner = block.inner(rect);

    // Fetch top N tests ordered by net WPM and include extra columns
    let sql = format!(
        "SELECT t.started_at, t.wpm,
                t.duration_ms, t.correct_chars, t.incorrect_chars,
                COALESCE((t.correct_chars+t.incorrect_chars)*1.0/5.0 / NULLIF(t.duration_ms/60000.0,0), 0.0) AS raw,
                t.accuracy,
                COALESCE(100.0*MIN(s.wpm)/NULLIF(MAX(s.wpm),0), 0.0) AS consistency,
                t.mode
         FROM tests t
         LEFT JOIN samples s ON s.test_id=t.id
         GROUP BY t.id
         ORDER BY t.wpm DESC
         LIMIT {}",
        TOP_N
    );

    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(_) => return,
    };

    let items: Vec<_> = stmt
        .query_map([], |r| {
            let ts: String = r.get(0)?;
            let dt_str = DateTime::parse_from_rfc3339(&ts)
                .map(|dt: DateTime<FixedOffset>| dt.with_timezone(&Local).format("%Y-%m-%d %H:%M:%S").to_string())
                .unwrap_or(ts.clone());
            Ok((
                dt_str,
                r.get::<_, f64>(1)?,               // net wpm
                r.get::<_, i64>(2)?,               // duration_ms
                r.get::<_, i64>(3)?,               // correct_chars
                r.get::<_, i64>(4)?,               // incorrect_chars
                r.get::<_, f64>(5)?,               // raw
                r.get::<_, f64>(6)?,               // accuracy
                r.get::<_, f64>(7)?,               // consistency
                r.get::<_, String>(8)?,            // mode
            ))
        })
        .unwrap()
        .filter_map(Result::ok)
        .collect();

    // Build table rows (rank, started at, net WPM, duration, correct/incorrect, raw, acc, consistency, mode)
    let rows: Vec<Row> = items
        .iter()
        .enumerate()
        .map(|(idx, (d, net, duration_ms, correct, incorrect, raw, acc, cons, mode))| {
            // duration formatting mm:ss
            let dur_s = (*duration_ms as u64).saturating_div(1000);
            let mins = dur_s / 60;
            let secs = dur_s % 60;
            let dur_str = format!("{:02}:{:02}", mins, secs);

            // Right-align numeric strings for nicer columns
            let net_s = format!("{:>6.1}", net);
            let correct_s = format!("{:>5}", correct);
            let incorrect_s = format!("{:>4}", incorrect);
            let raw_s = format!("{:>6.1}", raw);
            let acc_s = format!("{:>6.1}%", acc);
            let cons_s = format!("{:>6.1}%", cons);

            // Row style: alternate subtle background for readability
            let row_style = if idx % 2 == 0 {
                Style::default().bg(theme.background.to_tui_color())
            } else {
                // use `key_normal_bg` as a subtle alternate background when available
                Style::default().bg(theme.key_normal_bg.to_tui_color())
            };

            // Rank cell: highlight top 3 with accent color
            let rank_cell = if idx < 3 {
                Cell::from(Span::styled(format!("#{:>2}", idx + 1), Style::default().fg(theme.title_accent.to_tui_color()).add_modifier(Modifier::BOLD)))
            } else {
                Cell::from(format!("#{:>2}", idx + 1))
            };

            Row::new(vec![
                rank_cell,
                Cell::from(d.as_str()),
                Cell::from(Span::styled(net_s, Style::default().fg(theme.stats_value.to_tui_color()).add_modifier(Modifier::BOLD))),
                Cell::from(dur_str),
                Cell::from(Span::styled(correct_s, Style::default().fg(theme.foreground.to_tui_color()))),
                Cell::from(Span::styled(incorrect_s, Style::default().fg(theme.error.to_tui_color()))),
                Cell::from(raw_s),
                Cell::from(acc_s),
                Cell::from(cons_s),
                Cell::from(mode.as_str()),
            ]).style(row_style)
        })
        .collect();

    // Table stateful selection
    let mut state = TableState::default();
    let total = rows.len();
    let mut cursor = LEADERBOARD_CURSOR.load(Ordering::Relaxed);
    if total > 0 {
        cursor = cursor.min(total.saturating_sub(1));
    } else {
        cursor = 0;
    }
    LEADERBOARD_CURSOR.store(cursor, Ordering::Relaxed);
    let sel = if rows.is_empty() { None } else { Some(cursor) };
    state.select(sel);

    let header = Row::new(vec![
        Span::styled("#", Style::default().fg(theme.stats_value.to_tui_color()).add_modifier(Modifier::BOLD)),
        Span::styled("Started At", Style::default().fg(theme.stats_value.to_tui_color()).add_modifier(Modifier::BOLD)),
        Span::styled("WPM", Style::default().fg(theme.stats_value.to_tui_color()).add_modifier(Modifier::BOLD)),
        Span::styled("Dur", Style::default().fg(theme.stats_value.to_tui_color()).add_modifier(Modifier::BOLD)),
        Span::styled("Corr", Style::default().fg(theme.stats_value.to_tui_color()).add_modifier(Modifier::BOLD)),
        Span::styled("Err", Style::default().fg(theme.stats_value.to_tui_color()).add_modifier(Modifier::BOLD)),
        Span::styled("Raw", Style::default().fg(theme.stats_value.to_tui_color()).add_modifier(Modifier::BOLD)),
        Span::styled("Acc", Style::default().fg(theme.stats_value.to_tui_color()).add_modifier(Modifier::BOLD)),
        Span::styled("Cons", Style::default().fg(theme.stats_value.to_tui_color()).add_modifier(Modifier::BOLD)),
        Span::styled("Mode", Style::default().fg(theme.stats_value.to_tui_color()).add_modifier(Modifier::BOLD)),
    ]);

    let table = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::NONE))
        .widths(&[
            tui::layout::Constraint::Length(4),  // rank
            tui::layout::Constraint::Length(19), // started at
            tui::layout::Constraint::Length(7),  // wpm
            tui::layout::Constraint::Length(6),  // dur
            tui::layout::Constraint::Length(6),  // correct
            tui::layout::Constraint::Length(5),  // err
            tui::layout::Constraint::Length(7),  // raw
            tui::layout::Constraint::Length(7),  // acc
            tui::layout::Constraint::Length(7),  // cons
            tui::layout::Constraint::Length(8),  // mode
        ])
        .column_spacing(1)
        .highlight_style(Style::default().bg(theme.info.to_tui_color()))
        .highlight_symbol(" ");

    // Render table inside inner area but keep some padding: place table in a sub-rect with margin 1
    // Reserve 2 lines at the bottom of the modal for a footer with controls
    let footer_h = 2u16.min(inner.height.saturating_sub(2));
    let table_h = inner.height.saturating_sub(2 + footer_h);
    let table_rect = Rect::new(inner.x + 1, inner.y + 1, inner.width.saturating_sub(2), table_h.saturating_sub(0));
    f.render_stateful_widget(table, table_rect, &mut state);

    // Footer with help text
    let footer_lines = vec![
        Spans::from(vec![
            Span::styled("↑/↓: navigate", Style::default().fg(theme.stats_label.to_tui_color())),
            Span::raw("  "),
            Span::styled("Enter: view test", Style::default().fg(theme.stats_label.to_tui_color())),
            Span::raw("  "),
            Span::styled("Esc: close", Style::default().fg(theme.stats_label.to_tui_color())),
        ])
    ];
    let footer_para = Paragraph::new(footer_lines)
        .wrap(Wrap { trim: true })
        .style(Style::default().bg(theme.background.to_tui_color()).fg(theme.stats_label.to_tui_color()));
    let footer_rect = Rect::new(inner.x + 2, inner.y + 1 + table_h, inner.width.saturating_sub(4), footer_h);
    f.render_widget(footer_para, footer_rect);
}
