// ui/menu.rs
use std::cmp;
use tui::{backend::Backend, widgets::Paragraph, layout::Rect, style::{Style, Modifier}, Frame};
use crate::app::state::App;

pub fn draw_menu<B: Backend>(f: &mut Frame<B>, app: &App, _split_band: Option<Rect>) {
    let area = f.size();

    // Keep the overlay compact, but center the whole thing in the middle of the screen
    let labels = ["Settings", "Help", "Quit"];
    let title = "Menu";
    let hint_txt = "Enter: select   Esc: close   ↑/↓: navigate";

    let ascii_art = r#"  __                                   __                 .__          __   
    _/  |_  ___________  _____           _/  |_ ___.__.______ |__| _______/  |_ 
    \   __\/ __ \_  __ \/     \   ______ \   __<   |  |\____ \|  |/  ___/\   __\
     |  | \  ___/|  | \/  Y Y  \ /_____/  |  |  \___  ||  |_> >  |\___ \  |  |  
     |__|  \___  >__|  |__|_|  /          |__|  / ____||   __/|__/____  > |__|  
               \/            \/                 \/     |__|           \/        
        "#;

    // Trim trailing spaces so they don't paint over the background
    let art_lines: Vec<String> = ascii_art
        .lines()
        .map(|l| l.trim_end().to_string())
        .collect();

    // Compute width/height of our virtual modal
    let longest_label = labels.iter().map(|s| s.len()).max().unwrap_or(0) as u16;
    let longest_art = art_lines
        .iter()
        .map(|s| s.chars().count())
        .max()
        .unwrap_or(0) as u16;
    let title_w = title.len() as u16;
    let hint_w = hint_txt.lines().map(|l| l.len() as u16).max().unwrap_or(0);
    let content_w = longest_label.max(longest_art).max(title_w).max(hint_w);
    let padding_h = 4u16; // left/right visual breathing room for centering math
    let w_calc = (content_w + padding_h * 2).min(area.width).max(12);
    let w = w_calc;

    let art_h = art_lines.len() as u16;
    let items_h = labels.len() as u16;
    let hint_h = hint_txt.lines().count() as u16; // two lines: actions + "Space"
    let padding_v = 1u16; // top/bottom
    // art + title + items + hint + vertical padding + 1 gap row between art and title
    let mut h = art_h + 1 + items_h + hint_h + padding_v * 2;
    if h > area.height { h = area.height; }

    // Exact centering in the full screen
    let x = (area.width.saturating_sub(w)) / 2;
    let y = (area.height.saturating_sub(h)) / 2;
    let rect = Rect::new(x, y, w, h);
    let inner = rect;

    // 1) ASCII art at the top (non-space runs only, to remain transparent)
    for (idx, line) in art_lines.iter().enumerate() {
        let row_y = inner.y.saturating_add(idx as u16);
        if row_y >= inner.y.saturating_add(inner.height) { break; }
        let line_w = line.chars().count() as u16;
        if line_w == 0 { continue; }
        let x_off = if inner.width > line_w { (inner.width - line_w) / 2 } else { 0 };
        let line_x = inner.x.saturating_add(x_off);
        let chars: Vec<char> = line.chars().collect();
        let mut run_start: Option<usize> = None;
        for (i, ch) in chars.iter().enumerate() {
            if *ch != ' ' {
                if run_start.is_none() { run_start = Some(i); }
            } else if let Some(start) = run_start.take() {
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

    // 2) Title on the next row
    let title_row = inner.y.saturating_add(art_h);
    if title_row < inner.y.saturating_add(inner.height) {
        let tw = title.len() as u16;
        let x_off = if inner.width > tw { (inner.width - tw) / 2 } else { 0 };
        let title_rect = Rect::new(inner.x.saturating_add(x_off), title_row, cmp::min(tw, inner.width), 1);
        let title_para = Paragraph::new(title)
            .style(Style::default().fg(app.theme.title.to_tui_color()).add_modifier(Modifier::BOLD))
            .alignment(tui::layout::Alignment::Left);
        f.render_widget(title_para, title_rect);
    }

    // 3) Items just below title
    let items_top = inner.y.saturating_add(art_h + 1);
    let items_bottom = inner.y.saturating_add(inner.height).saturating_sub(hint_h); // reserve space for hint block
    let mut y_row = items_top;
    for (i, &label) in labels.iter().enumerate() {
        if y_row >= items_bottom { break; }
        let lw = label.len() as u16;
        let x_off = if inner.width > lw { (inner.width - lw) / 2 } else { 0 };
        let item_rect = Rect::new(inner.x.saturating_add(x_off), y_row, cmp::min(lw, inner.width), 1);

        let is_selected = i == app.menu_cursor || (app.menu_cursor > labels.len().saturating_sub(1) && i == 0);
        let para = if is_selected {
            Paragraph::new(label).style(
                Style::default()
                    .fg(app.theme.title_accent.to_tui_color())
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Paragraph::new(label).style(Style::default().fg(app.theme.foreground.to_tui_color()))
        };
        f.render_widget(para.alignment(tui::layout::Alignment::Left), item_rect);
        y_row = y_row.saturating_add(1);
    }

    // 4) Hint block at the very bottom rows of the inner rect
    let hint_start = inner.y.saturating_add(inner.height).saturating_sub(hint_h);
    for (i, line) in hint_txt.lines().enumerate() {
        let row_y = hint_start.saturating_add(i as u16);
        if row_y >= inner.y.saturating_add(inner.height) { break; }
        let lw = line.len() as u16;
        let x_off = if inner.width > lw { (inner.width - lw) / 2 } else { 0 };
        let rect = Rect::new(inner.x.saturating_add(x_off), row_y, cmp::min(lw, inner.width), 1);
        let para = Paragraph::new(line)
            .style(Style::default().fg(app.theme.stats_label.to_tui_color()))
            .alignment(tui::layout::Alignment::Left);
        f.render_widget(para, rect);
    }
}
