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
use tui::widgets::Wrap;
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
    words: Option<&str>,
) {
    // Split the provided area into top (chart) and bottom (words) sections.
    // Bottom section takes ~20% of the total height and the same width.
    let split_h = ((area.height as f64) * 0.80).round() as u16;
    let split_h = split_h.clamp(3, area.height.saturating_sub(1));
    let top_rect = Rect::new(area.x, area.y, area.width, split_h);
    let bottom_rect = Rect::new(area.x, area.y.saturating_add(split_h), area.width, area.height.saturating_sub(split_h));

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

    // Keep only the WPM dataset for the chart. We'll draw error markers as
    // explicit red 'X' glyphs overlaid on top of the chart so they are always
    // visible and look like Monkeytype's X markers.
    let datasets = vec![wpm_ds];

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
    if max_per_sec > 0 && top_rect.width > 16 {
        let gutter = 6u16;
        let chart_area = Rect::new(top_rect.x, top_rect.y, top_rect.width.saturating_sub(gutter), top_rect.height);
        f.render_widget(chart, chart_area);

        // Overlay red 'X' markers for each error point by mapping chart
        // coordinates -> terminal cells and rendering a 1x1 Paragraph with
        // a heavy multiplication glyph. This gives us a consistent red X
        // regardless of the Chart's built-in marker set.
    if !err_pts.is_empty() {
            use tui::widgets::Paragraph;

            let x_min = 0.0f64;
            let x_max = max_t.max(1.0);
            let y_min = 0.0f64;
            let y_max = max_w.max(1.0);

            // Use the chart's inner plotting area (shrink by 1 on all sides to
            // avoid the Block border and title row). This prevents markers from
            // being drawn on the widget frame.
            let plot_x = chart_area.x.saturating_add(1);
            let plot_y = chart_area.y.saturating_add(1);
            let plot_w = chart_area.width.saturating_sub(2);
            let plot_h = chart_area.height.saturating_sub(2);

            let cw = plot_w.saturating_sub(1) as f64;
            let ch = plot_h.saturating_sub(1) as f64;

            for &(x, y) in &err_pts {
                // normalized ratios (clamp to [0,1])
                let xr = if x_max > x_min { ((x - x_min) / (x_max - x_min)).clamp(0.0, 1.0) } else { 0.0 };
                let yr = if y_max > y_min { ((y - y_min) / (y_max - y_min)).clamp(0.0, 1.0) } else { 0.0 };

                // terminal coordinates inside plotting area. x grows right, y grows down.
                let px = plot_x.saturating_add((xr * cw).round() as u16);
                // invert y because chart origin is bottom-left for plotting
                let py = plot_y.saturating_add(plot_h.saturating_sub(1))
                    .saturating_sub((yr * ch).round() as u16);

                // ensure inside area
                if px >= chart_area.x && px < chart_area.x + chart_area.width
                    && py >= chart_area.y && py < chart_area.y + chart_area.height
                {
                    let x_span = Span::styled("✕", Style::default().fg(theme.error.to_tui_color()).add_modifier(Modifier::BOLD));
                    let p = Paragraph::new(x_span).style(Style::default().bg(theme.background.to_tui_color()));
                    // render a 1x1 cell paragraph at the computed position
                    f.render_widget(p, Rect::new(px, py, 1, 1));
                }
            }
        }

        // (manual legend removed — the UI already shows a Summary box with labels)

        // Right gutter area
    let right = Rect::new(chart_area.x + chart_area.width, chart_area.y, gutter, chart_area.height);

        // --- Place labels so they align exactly with the plotted X markers ---
        // We'll compute the screen row for y = max, y = half, and y = 0 using
        // the same plotting -> screen transform as the X markers so both line
        // up precisely.
        let h = right.height as i32;
        if h >= 3 {
            use tui::widgets::Paragraph;
            use tui::text::Spans;

            // Inner plotting rect (must match the one used for plotting Xs).
            let plot_y = chart_area.y.saturating_add(1);
            let plot_h = chart_area.height.saturating_sub(2);
            let ch = (plot_h.saturating_sub(1)) as f64;

            // Compute band top value and per-step height exactly like when
            // creating err_pts so positions are consistent.
            let band_top = max_w * err_band_ratio;
            let step = if max_per_sec > 0 { band_top / (max_per_sec.max(1) as f64) } else { 0.0 };

            // Values to map to rows
            let v_max = step * (max_per_sec as f64);
            let half_count = ((max_per_sec as f64) / 2.0).round() as usize;
            let v_half = step * (half_count as f64);
            let v_zero = 0.0f64;

            let y_min = 0.0f64;
            let y_max = max_w.max(1.0f64);

            let map_to_row = |val: f64| -> usize {
                let yr = if y_max > y_min { ((val - y_min) / (y_max - y_min)).clamp(0.0, 1.0) } else { 0.0 };
                let py = plot_y.saturating_add(plot_h.saturating_sub(1))
                    .saturating_sub((yr * ch).round() as u16);
                // convert into index relative to right area (right.y == chart_area.y)
                let idx = py.saturating_sub(right.y) as i32;
                idx.clamp(0, h - 1) as usize
            };

            let top_idx = map_to_row(v_max);
            let mid_idx = map_to_row(v_half);
            let bot_idx = map_to_row(v_zero);

            let mut lines: Vec<Spans> = vec![Spans::from(Span::raw("")); h as usize];

            // avoid accidental collisions: ensure distinct indices if possible
            let mid_idx = if mid_idx == top_idx && bot_idx != top_idx {
                (top_idx + bot_idx) / 2
            } else { mid_idx };

            lines[top_idx] = Spans::from(
                Span::styled(
                    format!("{}", max_per_sec),
                    Style::default().fg(theme.error.to_tui_color()).add_modifier(Modifier::BOLD),
                )
            );
            lines[mid_idx] = Spans::from(
                Span::styled(
                    format!("{}", half_count),
                    Style::default().fg(theme.error.to_tui_color()),
                )
            );
            lines[bot_idx] = Spans::from(
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
        // Fallback: render chart in the top rect
        f.render_widget(chart, top_rect);
    }

    // Bottom panel: show the test words (if provided)
    if let Some(text) = words {
        use tui::widgets::Paragraph;
        let para = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border.to_tui_color()))
                    .style(Style::default().bg(theme.background.to_tui_color()).fg(theme.foreground.to_tui_color()))
                    .title("Text"),
            )
            .wrap(Wrap { trim: true });
        f.render_widget(para, bottom_rect);
    }
}
