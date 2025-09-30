use std::cmp;
use tui::{backend::Backend, widgets::{Block, Borders, Paragraph, List, ListItem, ListState}, layout::Rect, style::{Style, Modifier, Color}, Frame};
use crate::app::state::App;

pub fn draw_menu<B: Backend>(f: &mut Frame<B>, app: &App) {
        let area = f.size();

                // Small centered modal (not a full-page menu). We deliberately keep this
                // compact so it overlays the current UI without replacing it.
        let labels = ["Settings", "Help", "Quit"];
        let items: Vec<ListItem> = labels.iter().map(|&s| ListItem::new(s)).collect();

                // Compute a compact modal size based on longest item + padding.
                let padding_h = 4u16; // left/right padding
                let padding_v = 1u16; // top/bottom padding
                let longest = labels.iter().map(|s| s.len()).max().unwrap_or(6) as u16;
                let w_calc = longest + padding_h * 2 + 2; // content + padding + borders
                let max_w = area.width.saturating_sub(10);
                let w = cmp::min(w_calc, max_w).max(12);
                let h = (items.len() as u16) + padding_v * 2 + 2; // items + padding + border

                let x = (area.width.saturating_sub(w)) / 2;
                let y = (area.height.saturating_sub(h)) / 2;
                let rect = Rect::new(x, y, w, h);

    // Render a dim overlay only on the regions outside the centered menu rect
    // so the menu area itself remains untouched and the UI underneath appears
    // "transparent" (terminals don't support true alpha blending).
    let overlay_style = Style::default().bg(Color::Indexed(236));

    // top area
    if y > 0 {
        let top = Rect::new(0, 0, area.width, y);
        let overlay = Paragraph::new("").style(overlay_style);
        f.render_widget(overlay, top);
    }

    // bottom area
    if y + h < area.height {
        let bottom = Rect::new(0, y + h, area.width, area.height - (y + h));
        let overlay = Paragraph::new("").style(overlay_style);
        f.render_widget(overlay, bottom);
    }

    // left area (between top and bottom)
    if x > 0 {
        let left = Rect::new(0, y, x, h);
        let overlay = Paragraph::new("").style(overlay_style);
        f.render_widget(overlay, left);
    }

    // right area
    if x + w < area.width {
        let right = Rect::new(x + w, y, area.width - (x + w), h);
        let overlay = Paragraph::new("").style(overlay_style);
        f.render_widget(overlay, right);
    }

    let mut state = ListState::default();
    state.select(Some(app.menu_cursor.min(items.len().saturating_sub(1))));

    // Render small modal as a bordered box with transparent interior so the
    // underlying UI remains visible. We intentionally do not draw a shadow
    // behind it (shadow would cover content under some terminals).
    let modal_block = Block::default()
        .borders(Borders::ALL)
        // Transparent background; border colored with accent so the box is visible
        .style(Style::default().bg(Color::Reset).fg(app.theme.title_accent.to_tui_color()));
    f.render_widget(modal_block.clone(), rect);

    // Render the items inside the modal without filling background
    let inner = modal_block.inner(rect);
    let list_area = Rect::new(inner.x + 1, inner.y + 1, inner.width.saturating_sub(2), inner.height.saturating_sub(2));

    let list = List::new(items)
        .block(Block::default())
        .style(Style::default().fg(app.theme.foreground.to_tui_color()).bg(Color::Reset))
        .highlight_style(Style::default().fg(app.theme.highlight.to_tui_color()).add_modifier(Modifier::BOLD));

    f.render_stateful_widget(list, list_area, &mut state);
}
