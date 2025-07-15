use tui::{
    backend::Backend,
    layout::Rect,
    symbols,
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Frame,
};

/// Draws a WPM over time chart with styled axes, labels, and smooth line.
pub fn draw_wpm_chart<B: Backend>(f: &mut Frame<B>, area: Rect, data: &[(u64, f64)]) {
    // Convert data points to f64
    let pts: Vec<(f64, f64)> = data.iter().map(|&(t, w)| (t as f64, w)).collect();
    let max_t = data.last().map(|&(t, _)| t as f64).unwrap_or(1.0).max(1.0);
    let max_w = data.iter().map(|&(_, w)| w).fold(0.0, f64::max).max(1.0) * 1.1;

    // Dataset with Braille markers for smoother line
    let dataset = Dataset::default()
        .name("WPM")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Cyan))
        .data(&pts);

    // Generate axis labels at min, mid, max
    let x_labels = vec![
        Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(format!("{}", (max_t / 2.0).round())),
        Span::styled(
            format!("{}", max_t.round()),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ];
    let y_labels = vec![
        Span::styled("0", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(format!("{}", (max_w / 2.0).round())),
        Span::styled(
            format!("{}", max_w.round()),
            Style::default().add_modifier(Modifier::BOLD),
        ),
    ];

    // Build the chart
    let chart = Chart::new(vec![dataset])
        .block(
            Block::default().
                title(Span::styled("WPM Over Time", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))).
                borders(Borders::ALL).
                border_style(Style::default().fg(Color::Gray)),
        )
        .x_axis(
            Axis::default()
                .title(Span::styled("Seconds", Style::default().fg(Color::Gray)))
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, max_t])
                .labels(x_labels),
        )
        .y_axis(
            Axis::default()
                .title(Span::styled("WPM", Style::default().fg(Color::Gray)))
                .style(Style::default().fg(Color::Gray))
                .bounds([0.0, max_w])
                .labels(y_labels),
        );

    // Render the chart in the given area
    f.render_widget(chart, area);
}
