// ui/menu.rs
use std::cmp;
use tui::{backend::Backend, widgets::Paragraph, layout::Rect, style::{Style, Modifier}, Frame};
use crate::app::state::App;

pub fn draw_menu<B: Backend>(f: &mut Frame<B>, app: &App, split_band: Option<Rect>) {
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

    // Trim trailing spaces so they don't paint over the background
    let art_lines: Vec<String> = ascii_art.lines().map(|l| l.trim_end().to_string()).collect();

        // Compute a compact modal size based on longest item or ascii art line + padding.
    let padding_h = 4u16; // left/right padding
    let padding_v = 1u16; // top/bottom padding
        let longest_label = labels.iter().map(|s| s.len()).max().unwrap_or(6) as u16;
        let longest_art = art_lines.iter().map(|s| s.len()).max().unwrap_or(0) as u16;
        let longest = std::cmp::max(longest_label, longest_art);
    // content + padding (no borders since we render only text)
    let w_calc = longest + padding_h * 2; 
        let max_w = area.width.saturating_sub(10);
        let w = cmp::min(w_calc, max_w).max(12);

        // Height must accommodate ascii art lines, menu items, hint/footer and border
    let art_h = art_lines.len() as u16;
    // art + items + vertical padding + gap + hint/footer
    let mut h = art_h + (labels.len() as u16) + padding_v * 2 + 2;
    let x = (area.width.saturating_sub(w)) / 2;

    // Reserve a small vertical margin so the modal doesn't cover UI section
    // borders (top of keyboard / bottom of text). If the terminal is small,
    // shrink the modal height to fit within margins.
    let top_margin = 2u16;
    let bottom_margin = 2u16;
    let available_vert = if area.height > top_margin + bottom_margin {
        area.height - top_margin - bottom_margin
    } else {
        0
    };
    if h > available_vert {
        h = available_vert;
    }

    // Center vertically within the safe region and clamp to margins
    let mut y = (area.height.saturating_sub(h)) / 2;
    if y < top_margin {
        y = top_margin;
    }
    let max_y = area.height.saturating_sub(h).saturating_sub(bottom_margin);
    if y > max_y {
        y = max_y;
    }

    let mut rect = Rect::new(x, y, w, h);

    // If we know the split band (the horizontal border between text and keyboard),
    // nudge the modal fully above or below to avoid intersecting that row, picking the
    // position that stays le plus proche du centre vertical.
    if let Some(band) = split_band {
        let band_y = band.y;
        let band_h = band.height.max(1);
        let band_end = band_y.saturating_add(band_h - 1);
        let rect_end = rect.y.saturating_add(rect.height.saturating_sub(1));
        let intersects = !(rect_end < band_y || rect.y > band_end);
        if intersects {
            // Two candidates: fully above or fully below the band
            let mut up_y = band_y.saturating_sub(rect.height);
            let mut down_y = band_end.saturating_add(1);

            // Respect top/bottom margins by clamping candidates
            let safe_min_y = top_margin;
            let safe_max_y = area
                .height
                .saturating_sub(bottom_margin)
                .saturating_sub(rect.height);
            if up_y < safe_min_y { up_y = safe_min_y; }
            if down_y > safe_max_y { down_y = safe_max_y; }

            // Choose the candidate whose center is closest to the screen center
            let area_mid = area.y + area.height / 2;
            let up_mid = up_y + rect.height / 2;
            let down_mid = down_y + rect.height / 2;
            let d_up = if up_mid > area_mid { up_mid - area_mid } else { area_mid - up_mid };
            let d_down = if down_mid > area_mid { down_mid - area_mid } else { area_mid - down_mid };
            rect.y = if d_up <= d_down { up_y } else { down_y };
        }
    }

    // We render the menu as text-only overlay: no border, no background.
    // Use the computed `rect` as the inner drawing area.
    let inner = rect;

    // Render ascii art lines at the top of the inner area, centered horizontally.
    for (idx, line) in art_lines.iter().enumerate() {
        let row_y = inner.y.saturating_add(idx as u16);
        if row_y >= inner.y.saturating_add(inner.height) { break; }
        let line_w = line.chars().count() as u16;
        if line_w == 0 { continue; }
        let x_off = if inner.width > line_w { (inner.width - line_w) / 2 } else { 0 };
        let line_x = inner.x.saturating_add(x_off);
        // Render only non-space runs so spaces are fully transparent.
        let chars: Vec<char> = line.chars().collect();
        let mut run_start: Option<usize> = None;
        for (i, ch) in chars.iter().enumerate() {
            if *ch != ' ' {
                if run_start.is_none() { run_start = Some(i); }
            } else if let Some(start) = run_start {
                let run: String = chars[start..i].iter().collect();
                let run_w = (i - start) as u16;
                if run_w > 0 {
                    let rx = line_x.saturating_add(start as u16);
                    let rw = cmp::min(run_w, inner.width.saturating_sub(start as u16));
                    if rw > 0 {
                        let rect = Rect::new(rx, row_y, rw, 1);
                        let para = Paragraph::new(run)
                            .style(Style::default().fg(app.theme.title.to_tui_color()))
                            .alignment(tui::layout::Alignment::Left);
                        f.render_widget(para, rect);
                    }
                }
                run_start = None;
            }
        }
        if let Some(start) = run_start {
            let run: String = chars[start..].iter().collect();
            let run_w = (chars.len() - start) as u16;
            if run_w > 0 {
                let rx = line_x.saturating_add(start as u16);
                let rw = cmp::min(run_w, inner.width.saturating_sub(start as u16));
                if rw > 0 {
                    let rect = Rect::new(rx, row_y, rw, 1);
                    let para = Paragraph::new(run)
                        .style(Style::default().fg(app.theme.title.to_tui_color()))
                        .alignment(tui::layout::Alignment::Left);
                    f.render_widget(para, rect);
                }
            }
        }
    }

    // After art, leave one empty row as a small gap before menu items (if space)
    let gap_top = inner.y.saturating_add(art_lines.len() as u16);
    // title paragraph is constructed later when drawing the centered text
    // place title in the gap row if there's at least one row available
    if inner.height > art_lines.len() as u16 {
        let t = "Menu";
        let tw = t.len() as u16;
        let x_off = if inner.width > tw { (inner.width - tw) / 2 } else { 0 };
        let title_rect = Rect::new(inner.x.saturating_add(x_off), gap_top, cmp::min(tw, inner.width), 1);
        let title_para = Paragraph::new(t)
            .style(Style::default().fg(app.theme.title.to_tui_color()).add_modifier(Modifier::BOLD))
            .alignment(tui::layout::Alignment::Left);
        f.render_widget(title_para, title_rect);
    }

    // compute the vertical region available for menu items: after the art + title
    // row and before the hint/footer row. This avoids overlaps.
    let items_top = inner.y.saturating_add(art_lines.len() as u16 + 1); // row just below title/gap
    let items_bottom = inner.y.saturating_add(inner.height).saturating_sub(1); // last row is hint/footer
    // number of available rows for items
    let available_rows = if items_bottom > items_top { items_bottom - items_top } else { 0 };

    for (i, &label) in labels.iter().enumerate() {
        if i as u16 >= available_rows { break; }
        let item_y = items_top.saturating_add(i as u16);

        // Render label only in a tight rect sized to the label text; center it
        // by computing an x offset so we don't overwrite surrounding cells.
        let lw = label.len() as u16;
        if lw == 0 { continue; }
        let x_off = if inner.width > lw { (inner.width - lw) / 2 } else { 0 };
        let item_x = inner.x.saturating_add(x_off);
        let item_w = cmp::min(lw, inner.width);
        let item_rect = Rect::new(item_x, item_y, item_w, 1);

        let is_selected = (i == app.menu_cursor) || (app.menu_cursor > labels.len().saturating_sub(1) && i == 0);

        if is_selected {
            let sel_txt = Paragraph::new(label)
                .style(Style::default().fg(app.theme.title_accent.to_tui_color()).add_modifier(Modifier::BOLD))
                .alignment(tui::layout::Alignment::Left);
            f.render_widget(sel_txt, item_rect);
        } else {
            let para = Paragraph::new(label)
                .style(Style::default().fg(app.theme.foreground.to_tui_color()))
                .alignment(tui::layout::Alignment::Left);
            f.render_widget(para, item_rect);
        }
    }

    // Render hint/footer inside modal (Esc/Enter)
    if inner.height >= 1 {
        let hint_txt = "Enter: select   Esc: close";
        let hw = hint_txt.len() as u16;
        let hx_off = if inner.width > hw { (inner.width - hw) / 2 } else { 0 };
        let hint_rect = Rect::new(inner.x.saturating_add(hx_off), inner.y + inner.height.saturating_sub(1), cmp::min(hw, inner.width), 1);
        let hint = Paragraph::new(hint_txt)
            .style(Style::default().fg(app.theme.stats_label.to_tui_color()))
            .alignment(tui::layout::Alignment::Left);
        f.render_widget(hint, hint_rect);
    }
}
