// src/ui/profile.rs

use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::state::App;

/// Render the Profile page.  
/// Press Esc to return (that’s handled in your input loop).
pub fn draw_profile<B: Backend>(f: &mut Frame<B>, app: &App) {
    let size = f.size();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(size);

    // Header
    let header = Paragraph::new(Spans::from(vec![
        Span::styled(" PROFILE ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" (Esc to return)"),
    ]))
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, chunks[0]);

    // Stats summary (reuse whichever fields you like)
    let total = app.samples.len();
    let avg_wpm = if total>0 {
        app.samples.iter().map(|&(_,w)| w).sum::<f64>() / total as f64
    } else { 0.0 };
    let avg_acc = if total>0 {
        // crude: last sample accuracy
        let correct = app.correct_chars as f64;
        let total_chars = (app.correct_chars+app.incorrect_chars) as f64;
        if total_chars>0.0 { correct/total_chars*100.0 } else { 0.0 }
    } else { 0.0 };

    let middle = Paragraph::new(vec![
        Spans::from(vec![ Span::raw("Tests run: "),    Span::styled(total.to_string(), Style::default().fg(Color::Cyan)) ]),
        Spans::from(vec![ Span::raw("Avg WPM:   "),    Span::styled(format!("{:.1}", avg_wpm), Style::default().fg(Color::Cyan)) ]),
        Spans::from(vec![ Span::raw("Avg Acc:   "),    Span::styled(format!("{:.1}%", avg_acc), Style::default().fg(Color::Cyan)) ]),
    ])
    .block(Block::default().borders(Borders::ALL).title("Stats"));
    f.render_widget(middle, chunks[1]);

    // Footer: last few WPM samples
    let footer = if app.samples.is_empty() {
        Paragraph::new("No history").block(Block::default().borders(Borders::ALL).title("History"))
    } else {
        let lines = app.samples.iter().rev().take(5)
            .map(|&(t,w)| Spans::from(format!("{}s → {:.1} WPM", t, w)))
            .collect::<Vec<_>>();
        Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title("Last 5 Runs"))
    };
    f.render_widget(footer, chunks[2]);
}
