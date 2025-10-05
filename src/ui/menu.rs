// ui/menu.rs
use std::cmp;
use tui::{backend::Backend, widgets::{Block, Borders, Paragraph}, layout::Rect, style::{Style, Modifier, Color}, Frame};
use crate::app::state::App;

pub fn draw_menu<B: Backend>(f: &mut Frame<B>, app: &App) {
        let area = f.size();

                // Small centered modal (not a full-page menu). We deliberately keep this
                // compact so it overlays the current UI without replacing it.
        // ASCII art header (centered at the top of the modal). Keep the lines
        // exactly as provided so the title looks beautiful. The art is rendered
        // with only foreground color so the underlying UI remains visible.
    let labels = ["Settings", "Help", "Quit"];

        let ascii_art = r#"  __                                   __                 .__          __   
    _/  |_  ___________  _____           _/  |_ ___.__.______ |__| _______/  |_ 
    \   __\/ __ \_  __ \/     \   ______ \   __<   |  |\____ \|  |/  ___/\   __\
     |  | \  ___/|  | \/  Y Y  \ /_____/  |  |  \___  ||  |_> >  |\___ \  |  |  
     |__|  \___  >__|  |__|_|  /          |__|  / ____||   __/|__/____  > |__|  
               \/            \/                 \/     |__|           \/        
                                                                            
                                                                            
                                                                            
                                                                            
                                                                                "#;

        let art_lines: Vec<&str> = ascii_art.lines().collect();

        // Compute a compact modal size based on longest item or ascii art line + padding.
        let padding_h = 4u16; // left/right padding
        let padding_v = 1u16; // top/bottom padding
        let longest_label = labels.iter().map(|s| s.len()).max().unwrap_or(6) as u16;
        let longest_art = art_lines.iter().map(|s| s.len()).max().unwrap_or(0) as u16;
        let longest = std::cmp::max(longest_label, longest_art);
        let w_calc = longest + padding_h * 2 + 2; // content + padding + borders
        let max_w = area.width.saturating_sub(10);
        let w = cmp::min(w_calc, max_w).max(12);

        // Height must accommodate ascii art lines, menu items, hint/footer and border
        let art_h = art_lines.len() as u16;
        let h = art_h + (labels.len() as u16) + padding_v * 2 + 2 + 1; // +1 small gap between art and items
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

    // Render ascii art lines at the top of the inner area, centered horizontally.
    for (idx, line) in art_lines.iter().enumerate() {
        let row_y = inner.y.saturating_add(idx as u16);
        if row_y >= inner.y.saturating_add(inner.height) { break; }
        let line_w = line.len() as u16;
        let x_off = if inner.width > line_w { (inner.width - line_w) / 2 } else { 0 };
        let line_x = inner.x.saturating_add(x_off);
        let line_w_clamped = cmp::min(line_w, inner.width);
        if line_w_clamped == 0 { continue; }
        let line_rect = Rect::new(line_x, row_y, line_w_clamped, 1);
        let art_para = Paragraph::new(*line)
            .style(Style::default().fg(app.theme.title.to_tui_color()))
            .alignment(tui::layout::Alignment::Left);
        f.render_widget(art_para, line_rect);
    }

    // After art, leave one empty row as a small gap before menu items (if space)
    let gap_top = inner.y.saturating_add(art_lines.len() as u16);
    let title = Paragraph::new("Menu")
        .style(Style::default().fg(app.theme.title.to_tui_color()).add_modifier(Modifier::BOLD))
        .alignment(tui::layout::Alignment::Center);
    // place title in the gap row if there's at least one row available
    if inner.height > art_lines.len() as u16 {
        let title_rect = Rect::new(inner.x, gap_top, inner.width, 1);
        f.render_widget(title, title_rect);
    }

    // compute the vertical region available for menu items: after the art + title
    // row and before the hint/footer row. This allows showing as many items as fit
    // inside the modal without overlapping title/hint.
    let items_top = inner.y.saturating_add(art_lines.len() as u16 + 1); // row just below title/gap
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
            // Render selected label with accent foreground and bold, but no background
            let sel_txt = Paragraph::new(label)
                .style(Style::default().fg(app.theme.title_accent.to_tui_color()).add_modifier(Modifier::BOLD))
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
