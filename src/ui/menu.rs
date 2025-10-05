// ui/menu.rs
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
    // and small visual decorations (selection capsule and hint) on top.

    // Render small modal as a bordered box with transparent interior so the
    // underlying UI remains visible. We intentionally do not draw a shadow
    // behind it (shadow would cover content under some terminals).
    let modal_block = Block::default()
        .borders(Borders::ALL)
        // Transparent background; border colored with accent so the box is visible
        .style(Style::default().bg(Color::Reset).fg(app.theme.title_accent.to_tui_color()));
    // Draw the modal border (transparent interior)
    f.render_widget(modal_block.clone(), rect);

    // Render title at the top of the inner area (small accent title)
    let inner = modal_block.inner(rect);
    let title = Paragraph::new("Menu")
        .style(Style::default().fg(app.theme.title.to_tui_color()).add_modifier(Modifier::BOLD))
        .alignment(tui::layout::Alignment::Center);
    // place title in the first row of inner
    if inner.height >= 1 {
        let title_rect = Rect::new(inner.x, inner.y, inner.width, 1);
        f.render_widget(title, title_rect);
    }

    // compute the vertical region available for menu items: between the
    // title row and the hint/footer row. This allows showing as many items
    // as fit inside the modal without overlapping title/hint.
    let items_top = inner.y.saturating_add(1); // row just below title
    let items_bottom = inner.y.saturating_sub(0).saturating_add(inner.height).saturating_sub(1); // row index of hint (exclusive)
    // number of available rows for items
    let available_rows = if items_bottom > items_top { items_bottom - items_top } else { 0 };

    for (i, &label) in labels.iter().enumerate() {
        if i as u16 >= available_rows { break; }
        let item_y = items_top.saturating_add(i as u16);

        // center the label horizontally within inner with small horizontal padding
        let label_w = label.len() as u16 + 2; // padding inside capsule
        let x_off = if inner.width > label_w { (inner.width - label_w) / 2 } else { 0 };
        let item_x = inner.x.saturating_add(x_off);

        // clamp width so we never overflow
    let item_w = cmp::min(label_w, inner.width);
        if item_w == 0 { continue; }

        let item_rect = Rect::new(item_x, item_y, item_w, 1);

        let is_selected = (i == app.menu_cursor) || (app.menu_cursor > labels.len().saturating_sub(1) && i == 0);

        if is_selected {
            // Draw a small capsule background behind the selected label so it's visible
            let cap = Paragraph::new(" ")
                .style(Style::default().bg(app.theme.title_accent.to_tui_color()));
            // expand capsule by 1 column on both sides if possible
            let cap_x = item_rect.x.saturating_sub(1);
            let cap_w = (item_rect.width + 2).min(inner.width.saturating_sub(item_rect.x.saturating_sub(inner.x)));
            let cap_rect = Rect::new(cap_x, item_rect.y, cap_w, 1);
            f.render_widget(cap, cap_rect);

            // Render label text with background-contrasting foreground
            let sel_txt = Paragraph::new(label)
                .style(Style::default().fg(app.theme.background.to_tui_color()).add_modifier(Modifier::BOLD))
                .alignment(tui::layout::Alignment::Center);
            f.render_widget(sel_txt, item_rect);
        } else {
            let para = Paragraph::new(label)
                .style(Style::default().fg(app.theme.foreground.to_tui_color()))
                .alignment(tui::layout::Alignment::Center);
            f.render_widget(para, item_rect);
        }
    }

    // Render hint/footer inside modal (Esc/Enter)
    if inner.height >= 1 {
        let hint = Paragraph::new("Enter: select   Esc: close")
            .style(Style::default().fg(app.theme.stats_label.to_tui_color()))
            .alignment(tui::layout::Alignment::Center);
        let hint_rect = Rect::new(inner.x, inner.y + inner.height.saturating_sub(1), inner.width, 1);
        f.render_widget(hint, hint_rect);
    }
}
