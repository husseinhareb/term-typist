use std::cmp;
use tui::{backend::Backend, widgets::{Block, Borders, Paragraph}, layout::Rect, style::{Style, Modifier, Color}, Frame};
use crate::app::state::App;

pub fn draw_menu<B: Backend>(f: &mut Frame<B>, app: &App) {
        let area = f.size();

                // Small centered modal (not a full-page menu). We deliberately keep this
                // compact so it overlays the current UI without replacing it.
    let labels = ["Settings", "Help", "Quit"];

                // Compute a compact modal size based on longest item + padding.
                let padding_h = 4u16; // left/right padding
                let padding_v = 1u16; // top/bottom padding
                let longest = labels.iter().map(|s| s.len()).max().unwrap_or(6) as u16;
                let w_calc = longest + padding_h * 2 + 2; // content + padding + borders
                let max_w = area.width.saturating_sub(10);
                let w = cmp::min(w_calc, max_w).max(12);
                let h = (labels.len() as u16) + padding_v * 2 + 2; // items + padding + border

                let x = (area.width.saturating_sub(w)) / 2;
                let y = (area.height.saturating_sub(h)) / 2;
                let rect = Rect::new(x, y, w, h);

    // NOTE: intentionally do not draw any dim overlay around the modal so the
    // underlying UI remains fully visible; we only draw the bordered modal
    // and the centered labels on top.

    // Render small modal as a bordered box with transparent interior so the
    // underlying UI remains visible. We intentionally do not draw a shadow
    // behind it (shadow would cover content under some terminals).
    let modal_block = Block::default()
        .borders(Borders::ALL)
        // Transparent background; border colored with accent so the box is visible
        .style(Style::default().bg(Color::Reset).fg(app.theme.title_accent.to_tui_color()));
    // Draw only the border (modal_block) so we don't fill the interior and
    // therefore preserve the underlying UI pixels where we don't draw text.
    f.render_widget(modal_block.clone(), rect);

    // Render each menu label as a single-line Paragraph centered inside the
    // inner rect. We avoid rendering full-width widgets (which would overwrite
    // the background) and instead draw only the characters of each label so the
    // underlying UI stays visible around them.
    let inner = modal_block.inner(rect);
    // compute vertical start for items (top padding)
    let start_y = inner.y.saturating_add(padding_v);

    for (i, &label) in labels.iter().enumerate() {
        let item_y = start_y.saturating_add(i as u16);
        if item_y >= inner.y.saturating_add(inner.height) { break; }

        // center the label horizontally within inner
        let label_w = label.len() as u16;
        let x_off = if inner.width > label_w { (inner.width - label_w) / 2 } else { 0 };
        let item_x = inner.x.saturating_add(x_off);

        // clamp width so we never overflow
        let item_w = cmp::min(label_w, inner.width);
        if item_w == 0 { continue; }

        let item_rect = Rect::new(item_x, item_y, item_w, 1);

        let is_selected = (i == app.menu_cursor) || (app.menu_cursor > labels.len().saturating_sub(1) && i == 0);
        let style = if is_selected {
            // Highlight with accent foreground and bold — avoid drawing a filled
            // background so the underlying UI remains visible around the label.
            Style::default().fg(app.theme.highlight.to_tui_color()).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(app.theme.foreground.to_tui_color())
        };

        let para = Paragraph::new(label).style(style).alignment(tui::layout::Alignment::Center);
        f.render_widget(para, item_rect);
    }
}
