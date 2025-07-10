use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Style},
    symbols,
    widgets::{Axis, Block, Borders, Chart, Dataset},
    Frame,
};
use tui::widgets::GraphType;
/// Draws a line chart of WPM samples over time.
pub fn draw_wpm_chart<B: Backend>(f: &mut Frame<B>, area: Rect, data: &[(u64, f64)]) {
    let points: Vec<(f64, f64)> = data.iter().map(|&(t, w)| (t as f64, w)).collect();
    let max_t = data.last().map(|&(t, _)| t as f64).unwrap_or(0.0).max(1.0);
    let max_w = data.iter().map(|&(_, w)| w).fold(0.0, f64::max).max(1.0) * 1.1;

let dataset = Dataset::default()
    .name("WPM")
    // Use Braille or Dot markers, but GraphType::Line will connect them
    .marker(symbols::Marker::Dot)
    .graph_type(GraphType::Line)               // <— draw lines between points :contentReference[oaicite:0]{index=0}
    .style(Style::default().fg(Color::Cyan))
    .data(&points);

let chart = Chart::new(vec![dataset])
    .block(Block::default().borders(Borders::ALL).title("WPM Over Time"))
    .x_axis(Axis::default().title("Seconds").bounds([0.0, max_t]))  
    .y_axis(Axis::default().title("WPM").bounds([0.0, max_w]));  
f.render_widget(chart, area);
}
