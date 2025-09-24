// src/ui/help.rs

use tui::{backend::Backend, widgets::{Block, Borders, Paragraph, Wrap}, layout::{Alignment, Constraint, Direction, Layout, Rect}, style::Style, text::Span, Frame};
use crate::app::state::App;

pub fn draw_help<B: Backend>(f: &mut Frame<B>, app: &App) {
    let area = f.size();
    let w = (area.width as f32 * 0.6) as u16;
    let h = (area.height as f32 * 0.6) as u16;
    let x = (area.width.saturating_sub(w)) / 2;
    let y = (area.height.saturating_sub(h)) / 2;
    let rect = Rect::new(x, y, w, h);

    let block = Block::default().borders(Borders::ALL).title("Help").style(Style::default().bg(app.theme.background.to_tui_color()).fg(app.theme.foreground.to_tui_color()));
    // render a clone so we can still borrow `block` for inner()
    f.render_widget(block.clone(), rect);

    let inner = block.inner(rect);
    let text = "Keys:\n  Shift+Esc: Open menu\n  Esc: Close menu/help or go back\n  p: Profile, s: Settings\n  a: toggle audio\n  Left/Right: change setting value when selected\n";
    let para = Paragraph::new(text).wrap(Wrap { trim: true });
    f.render_widget(para, inner);
}
