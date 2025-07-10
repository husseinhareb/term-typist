use tui::{
    symbols,
    widgets::{Dataset, GraphType},
    style::Style,
    style::Color,
    Frame, backend::Backend,
    layout::Rect,
    widgets::{Chart, Axis, Block, Borders},
};

pub fn draw_wpm_chart<B: Backend>(f: &mut Frame<B>, area: Rect, data: &[(u64,f64)]) {
    let pts: Vec<(f64,f64)> = data.iter().map(|&(t,w)| (t as f64, w)).collect();
    let max_t = data.last().map(|&(t,_)| t as f64).unwrap_or(1.0).max(1.0);
    let max_w = data.iter().map(|&(_,w)| w).fold(0.0,f64::max).max(1.0)*1.1;

    let dataset = Dataset::default()
        .name("WPM")
        // Option A: use a small solid block for each point
        .marker(symbols::Marker::Block)
        .graph_type(GraphType::Line)
        .style(Style::default().fg(Color::Cyan))
        .data(&pts);

    let chart = Chart::new(vec![dataset])
        .block(Block::default().title("WPM Over Time").borders(Borders::ALL))
        .x_axis(Axis::default().title("Seconds").bounds([0.0, max_t]))
        .y_axis(Axis::default().title("WPM").bounds([0.0, max_w]));

    f.render_widget(chart, area);
}
