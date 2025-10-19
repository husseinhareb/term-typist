// src/graph.rs


use tui::{
    backend::Backend,
    layout::Rect,
    symbols,
    style::{Modifier, Style},
    text::{Span, Spans},
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Frame,
};
use crate::theme::Theme;

/// Draws a WPM over time chart with styled axes, labels, and smooth line.
/// Draw WPM chart with optional error markers.
///
/// `data` is a slice of (elapsed_seconds, wpm).
/// `errors` if provided is a slice of elapsed seconds where errors occured.
pub fn draw_wpm_chart<B: Backend>(
    f: &mut Frame<B>,
    area: Rect,
    data: &[(u64, f64)],
    theme: &Theme,
    errors: Option<&[u64]>,
) {
    // Convert data points to f64
    let pts: Vec<(f64, f64)> = data.iter().map(|&(t, w)| (t as f64, w)).collect();
    let max_t = data.last().map(|&(t, _)| t as f64).unwrap_or(1.0).max(1.0);
    let max_w = data.iter().map(|&(_, w)| w).fold(0.0, f64::max).max(1.0) * 1.1;

    // Dataset with Braille markers for smoother line
    let dataset = Dataset::default()
        .name("WPM")
        .marker(symbols::Marker::Braille)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(theme.chart_line.to_tui_color()))
        .data(&pts);

    // Build an errors dataset if error timestamps were provided.
    // Aggregate errors per-second: produce one point per second with a count.
    use std::collections::BTreeMap;
    let mut per_sec: BTreeMap<u64, usize> = BTreeMap::new();
    if let Some(errs) = errors {
        for &et in errs.iter() {
            *per_sec.entry(et).or_insert(0) += 1;
        }
    }

    let mut errs_pts: Vec<(f64, f64)> = Vec::new();
    let mut max_per_second = 0usize;
    if !per_sec.is_empty() {
        max_per_second = *per_sec.values().max().unwrap_or(&0);
        for (&sec, &count) in per_sec.iter() {
            // Map count -> y coordinate proportionally into WPM axis range so
            // markers appear vertically relative to the WPM chart. Right axis
            // will display counts (0..max_per_second).
            let y = if max_per_second > 0 {
                (count as f64 / max_per_second as f64) * max_w
            } else {
                0.0
            };
            errs_pts.push((sec as f64, y));
        }
    }

    let errors_dataset = if !errs_pts.is_empty() {
        Some(
            Dataset::default()
                .name("Errors")
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(theme.error.to_tui_color()))
                .data(&errs_pts),
        )
    } else {
        None
    };

    // Helper to format seconds into a human-friendly label (s, XmYs, or X.Xh)
    fn fmt_time_label(secs: f64) -> String {
        if secs.is_nan() || !secs.is_finite() || secs <= 0.0 {
            return "0s".to_string();
        }
        if secs >= 3600.0 {
            // show hours with one decimal
            format!("{:.1}h", secs / 3600.0)
        } else if secs >= 60.0 {
            let mins = (secs / 60.0).floor() as u64;
            let s = (secs % 60.0).round() as u64;
            if s == 0 {
                format!("{}m", mins)
            } else {
                format!("{}m{}s", mins, s)
            }
        } else {
            format!("{}s", secs.round() as u64)
        }
    }

    // Generate axis labels at min, mid, max using friendly time units
    let x_labels = vec![
        Span::styled(
            fmt_time_label(0.0),
            Style::default().fg(theme.chart_axis.to_tui_color()).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            fmt_time_label(max_t / 2.0),
            Style::default().fg(theme.chart_axis.to_tui_color()),
        ),
        Span::styled(
            fmt_time_label(max_t),
            Style::default().fg(theme.chart_axis.to_tui_color()).add_modifier(Modifier::BOLD),
        ),
    ];
    let y_labels = vec![
        Span::styled(
            "0",
            Style::default().fg(theme.chart_axis.to_tui_color()).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("{}", (max_w / 2.0).round()),
            Style::default().fg(theme.chart_axis.to_tui_color()),
        ),
        Span::styled(
            format!("{}", max_w.round()),
            Style::default().fg(theme.chart_axis.to_tui_color()).add_modifier(Modifier::BOLD),
        ),
    ];

    // Build the chart
    // Include errors dataset if present so markers are drawn on top of the WPM line.
    let mut datasets = vec![dataset];
    if let Some(ed) = errors_dataset {
        datasets.push(ed);
    }

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(Span::styled(
                    "WPM Over Time",
                    Style::default().fg(theme.stats_value.to_tui_color()).add_modifier(Modifier::BOLD),
                ))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border.to_tui_color()))
                .style(Style::default().bg(theme.background.to_tui_color()).fg(theme.foreground.to_tui_color())),
        )
        .style(Style::default().bg(theme.background.to_tui_color()).fg(theme.foreground.to_tui_color()))
        .x_axis(
            Axis::default()
                .title(Span::styled("Seconds", Style::default().fg(theme.chart_labels.to_tui_color())))
                .style(Style::default().fg(theme.chart_axis.to_tui_color()))
                .bounds([0.0, max_t])
                .labels(x_labels),
        )
        .y_axis(
            Axis::default()
                .title(Span::styled("WPM", Style::default().fg(theme.chart_labels.to_tui_color())))
                .style(Style::default().fg(theme.chart_axis.to_tui_color()))
                .bounds([0.0, max_w])
                .labels(y_labels),
        );

    // Reserve a small gutter on the right for an errors Y-axis if errors provided
    if max_per_second > 0 && area.width > 10 {
        let gutter = 6u16;
        if area.width > gutter + 10 {
            let chart_area = Rect::new(area.x, area.y, area.width.saturating_sub(gutter), area.height);
            f.render_widget(chart, chart_area);

            // Right axis area
            let right = Rect::new(chart_area.x + chart_area.width, chart_area.y, gutter, chart_area.height);
            // Build labels: top, mid, bottom
            let top = format!("{}", max_per_second);
            let mid = format!("{}", (max_per_second as f64 / 2.0).round() as usize);
            let bot = "0".to_string();
            // Create lines with spacing so values align top/mid/bottom roughly
            let mut lines: Vec<Spans> = Vec::new();
            lines.push(Spans::from(Span::styled(top, Style::default().fg(theme.error.to_tui_color()).add_modifier(Modifier::BOLD))));
            // middle spacer: approximate by inserting blank lines
            let spacer_lines = (right.height.saturating_sub(3)).saturating_div(2);
            for _ in 0..spacer_lines { lines.push(Spans::from(Span::raw(""))); }
            lines.push(Spans::from(Span::styled(mid, Style::default().fg(theme.error.to_tui_color()))));
            for _ in 0..spacer_lines { lines.push(Spans::from(Span::raw(""))); }
            lines.push(Spans::from(Span::styled(bot, Style::default().fg(theme.error.to_tui_color()).add_modifier(Modifier::BOLD))));

            use tui::widgets::Paragraph;
            use tui::layout::Alignment;
            let para = Paragraph::new(lines).alignment(Alignment::Center).style(Style::default().bg(theme.background.to_tui_color()));
            f.render_widget(para, right);
            return;
        }
    }

    // Fallback: render chart full width
    f.render_widget(chart, area);
}
