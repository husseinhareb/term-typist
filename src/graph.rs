// src/graph.rs

use tui::{
    backend::Backend,
    layout::Rect,
    symbols,
    style::{Modifier, Style},
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType},
    Frame,
};
use std::collections::BTreeMap;
use tui::layout::Alignment;
use crate::theme::Theme;

/// Draw WPM chart with optional error markers.
/// `data`: (elapsed_seconds, wpm)
/// `errors`: elapsed seconds where errors occurred.
pub fn draw_wpm_chart<B: Backend>(
    f: &mut Frame<B>,
    area: Rect,
    data: &[(u64, f64)],
    theme: &Theme,
    errors: Option<&[u64]>,
) {
    // ---- WPM data ----
    let pts: Vec<(f64, f64)> = data.iter().map(|&(t, w)| (t as f64, w)).collect();
    let max_t = data.last().map(|&(t, _)| t as f64).unwrap_or(1.0).max(1.0);
    let max_w = data.iter().map(|&(_, w)| w).fold(0.0, f64::max).max(1.0) * 1.1;
    // Error band ratio: 0.18 keeps errors in a small bottom band (Monkeytype look).
    // Set to 1.0 to allow errors to use the full chart height.
    let err_band_ratio: f64 = 1.0; // change this to 0.18 for the small bottom band

    let wpm_ds = Dataset::default()
        .name("WPM")
        .marker(symbols::Marker::Braille) // smoother line
        .graph_type(GraphType::Line)
        .style(Style::default().fg(theme.chart_line.to_tui_color()))
        .data(&pts);

    // ---- Error markers (Monkeytype-style) ----
    // Aggregate by second. Auto-detect whether incoming timestamps are in
    // seconds or milliseconds (or higher resolution) by comparing the max
    // error timestamp against `max_t` (which is in seconds). If timestamps
    // look like milliseconds, divide by 1000 before bucketing.
    let mut per_sec: BTreeMap<u64, usize> = BTreeMap::new();
    if let Some(es) = errors {
        if !es.is_empty() {
            let &max_err = es.iter().max().unwrap();
            // Try divisors for: seconds(1), milliseconds(1_000), microseconds(1_000_000), nanoseconds(1_000_000_000)
            let candidates: [u64; 4] = [1, 1_000, 1_000_000, 1_000_000_000];
            // pick the smallest divisor that makes max_err/div <= max_t * 10
            let mut divisor = *candidates.last().unwrap();
            for &d in &candidates {
                if (max_err as f64) / (d as f64) <= max_t * 10.0 {
                    divisor = d;
                    break;
                }
            }

            for &t in es {
                // floor to the second after applying divisor so multiple
                // sub-second timestamps in the same second bucket together.
                let sec = if divisor > 1 {
                    ((t as f64) / (divisor as f64)).floor() as u64
                } else {
                    t
                };
                *per_sec.entry(sec).or_insert(0) += 1;
            }

            // If we ended up with an empty map (unexpected), try a couple of
            // fallback divisors to be more forgiving with the input unit.
            if per_sec.is_empty() {
                for &alt in &[1000u64, 1u64, 1_000_000u64] {
                    let mut alt_map: BTreeMap<u64, usize> = BTreeMap::new();
                    for &t in es {
                        let sec = if alt > 1 { ((t as f64) / (alt as f64)).floor() as u64 } else { t };
                        *alt_map.entry(sec).or_insert(0) += 1;
                    }
                    if !alt_map.is_empty() {
                        per_sec = alt_map;
                        break;
                    }
                }
            }
        }
    }
    let max_per_sec = per_sec.values().copied().max().unwrap_or(0);
    // Debug: print buckets when running non-release builds so you can verify
    // that multiple errors in the same second are being counted.
    // debug printing removed

    // Map error counts into a small band at the bottom of the chart (≈ 18% of Y range).
    let mut err_pts: Vec<(f64, f64)> = Vec::new();
    if max_per_sec > 0 {
        let band_top = max_w * err_band_ratio;
        let step = band_top / (max_per_sec.max(1) as f64); // stack multiple errors in the same second
        for (&sec, &count) in &per_sec {
            let y = step * count as f64;         // stay in bottom band
            err_pts.push((sec as f64, y));
        }
    }

    let mut datasets = vec![wpm_ds];
    if !err_pts.is_empty() {
        datasets.push(
            Dataset::default()
                .name("Errors")
                // use a solid block marker so it won't visually disappear when
                // overlapping braille/line markers; keep only fg + bold (no bg)
                .marker(symbols::Marker::Block)
                .graph_type(GraphType::Scatter) // points, not a connected line
                .style(Style::default().fg(theme.error.to_tui_color()).add_modifier(Modifier::BOLD))
                .data(&err_pts),
        );
    }

    // ---- Axis label helpers ----
    fn fmt_time_label(secs: f64) -> String {
        if secs.is_nan() || !secs.is_finite() || secs <= 0.0 { return "0s".into(); }
        if secs >= 3600.0 { format!("{:.1}h", secs / 3600.0) }
        else if secs >= 60.0 {
            let m = (secs / 60.0).floor() as u64;
            let s = (secs % 60.0).round() as u64;
            if s == 0 { format!("{m}m") } else { format!("{m}m{s}s") }
        } else { format!("{}s", secs.round() as u64) }
    }

    let x_labels = vec![
        Span::styled(fmt_time_label(0.0), Style::default().fg(theme.chart_axis.to_tui_color()).add_modifier(Modifier::BOLD)),
        Span::styled(fmt_time_label(max_t / 2.0), Style::default().fg(theme.chart_axis.to_tui_color())),
        Span::styled(fmt_time_label(max_t), Style::default().fg(theme.chart_axis.to_tui_color()).add_modifier(Modifier::BOLD)),
    ];
    let y_labels = vec![
        Span::styled("0", Style::default().fg(theme.chart_axis.to_tui_color()).add_modifier(Modifier::BOLD)),
        Span::styled(format!("{}", (max_w / 2.0).round()), Style::default().fg(theme.chart_axis.to_tui_color())),
        Span::styled(format!("{}", max_w.round()), Style::default().fg(theme.chart_axis.to_tui_color()).add_modifier(Modifier::BOLD)),
    ];

    let chart = Chart::new(datasets)
        .block(
            Block::default()
                .title(Span::styled("WPM Over Time",
                    Style::default().fg(theme.stats_value.to_tui_color()).add_modifier(Modifier::BOLD)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border.to_tui_color()))
                .style(Style::default()
                    .bg(theme.background.to_tui_color())
                    .fg(theme.foreground.to_tui_color())),
        )
        .style(Style::default()
            .bg(theme.background.to_tui_color())
            .fg(theme.foreground.to_tui_color()))
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

    // Reserve a slim right gutter for an error count scale that aligns with the error band.
    if max_per_sec > 0 && area.width > 16 {
        let gutter = 6u16;
        let chart_area = Rect::new(area.x, area.y, area.width.saturating_sub(gutter), area.height);
        f.render_widget(chart, chart_area);

        // (manual legend removed — the UI already shows a Summary box with labels)

        // Right gutter area
        let right = Rect::new(chart_area.x + chart_area.width, chart_area.y, gutter, chart_area.height);

        // --- Place labels inside the same vertical band as the dots ---
        // Anchor the label band to the bottom of the gutter so it lines up with the
        // error points. Use the same `err_band_ratio` as used to map error points.
        let band_ratio = err_band_ratio; // must match the ratio used to build the dots
        let h = right.height;
        if h >= 3 {
            let mut band_rows = ((h as f64) * band_ratio).round() as u16;
            band_rows = band_rows.clamp(3, h);

            let band_start = h.saturating_sub(band_rows); // first row of band (0 is top)
            let band_mid = band_start + (band_rows / 2);
            let band_end = h.saturating_sub(1);

            use tui::widgets::Paragraph;
            use tui::text::Spans;

            let mut lines: Vec<Spans> = vec![Spans::from(Span::raw("")); h as usize];

            // Top of band => max errors/sec
            lines[band_start as usize] = Spans::from(
                Span::styled(
                    format!("{}", max_per_sec),
                    Style::default().fg(theme.error.to_tui_color()).add_modifier(Modifier::BOLD),
                )
            );

            // Middle of band => half
            let half = ((max_per_sec as f64) / 2.0).round() as usize;
            lines[band_mid as usize] = Spans::from(
                Span::styled(
                    format!("{}", half),
                    Style::default().fg(theme.error.to_tui_color()),
                )
            );

            // Bottom of band => 0
            lines[band_end as usize] = Spans::from(
                Span::styled(
                    "0",
                    Style::default().fg(theme.error.to_tui_color()).add_modifier(Modifier::BOLD),
                )
            );

            let para = Paragraph::new(lines)
                .alignment(Alignment::Center)
                .style(Style::default().bg(theme.background.to_tui_color()));
            f.render_widget(para, right);
        }
    } else {
        // Fallback: render chart full width
        f.render_widget(chart, area);
    }
}
