// ui/menu.rs
// std::cmp removed (was unused)

use tui::{
    backend::Backend,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::Paragraph,
    Frame,
};

use crate::app::state::App;

/// "term-typist" banner text lines. Colors are chosen at runtime from the
/// active theme so the banner adapts to the selected theme.
const BANNER_LINES: [&str; 6] = [
    "████████╗  ███████╗  ██████╗    ███╗   ███╗       ████████╗  ██╗   ██╗  ██████╗    ██╗  ███████╗  ████████╗",
    "╚══██╔══╝  ██╔════╝  ██╔══██╗   ████╗ ████║       ╚══██╔══╝  ╚██╗ ██╔╝  ██╔══██╗   ██║  ██╔════╝  ╚══██╔══╝",
    " ██║     █████╗    ██████╔╝   ██╔████╔██║  ███     ██║      ╚████╔╝   ██████╔╝   ██║  ███████╗     ██║   ",
    " ██║     ██╔══╝    ██╔══██╗   ██║╚██╔╝██║          ██║       ╚██╔╝    ██╔═══╝    ██║  ╚════██║     ██║   ",
    " ██║     ███████╗  ██║  ██║   ██║ ╚═╝ ██║          ██║        ██║     ██║        ██║  ███████║     ██║   ",
    " ╚═╝     ╚══════╝  ╚═╝  ╚═╝   ╚═╝     ╚═╝          ╚═╝        ╚═╝     ╚═╝        ╚═╝  ╚══════╝     ╚═╝   ",
];

/// Small 3-row ASCII words (unselected), btop++ style
const MENU_NORMAL: [[&str; 3]; 3] = [
    // Options
    [
        "┌─┐┌─┐┌┬┐┬┌─┐┌┐┌┌─┐",
        "│ │├─┘ │ ││ ││││└─┐",
        "└─┘┴   ┴ ┴└─┘┘└┘└─┘",
    ],
    // Help
    [
        "┬ ┬┌─┐┬  ┌─┐",
        "├─┤├┤ │  ├─┘",
        "┴ ┴└─┘┴─┘┴  ",
    ],
    // Quit
    [
        "┌─┐ ┬ ┬ ┬┌┬┐",
        "│─┼┐│ │ │ │ ",
        "└─┘└└─┘ ┴ ┴ ",
    ],
];

/// Small 3-row ASCII words (selected, double-line “╔═╗ … ╚═╝”)
const MENU_SELECTED: [[&str; 3]; 3] = [
    // Options
    [
        "╔═╗╔═╗╔╦╗╦╔═╗╔╗╔╔═╗",
        "║ ║╠═╝ ║ ║║ ║║║║╚═╗",
        "╚═╝╩   ╩ ╩╚═╝╝╚╝╚═╝",
    ],
    // Help
    [
        "╦ ╦╔═╗╦  ╔═╗",
        "╠═╣╠╣ ║  ╠═╝",
        "╩ ╩╚═╝╩═╝╩  ",
    ],
    // Quit
    [
        "╔═╗ ╦ ╦ ╦╔╦╗ ",
        "║═╬╗║ ║ ║ ║  ",
        "╚═╝╚╚═╝ ╩ ╩  ",
    ],
];

/// Visual widths of each item (for centering)
const MENU_WIDTH: [u16; 3] = [19, 12, 12];

/// btop-like per-row colors
// NOTE: selected/normal hard-coded color arrays removed. Menu now uses
// theme-derived colors for all items (generated at runtime).
pub fn draw_menu<B: Backend>(f: &mut Frame<B>, app: &App, _split_band: Option<Rect>) {
    let area = f.size();

    // Compute content width based on banner
    let banner_width = BANNER_LINES
        .iter()
        .map(|s| s.chars().count() as u16)
        .max()
        .unwrap_or(0);

    let padding_h = 4u16; // side breathing room
    let w = (banner_width + padding_h * 2).clamp(12, area.width);

    // Height: 6 banner + 1 gap + 3 items (3 rows each) + 2 gaps between = 18
    let needed_h = 6 + 1 + (3 * 3 + 2);
    let h = (needed_h as u16).min(area.height);

    // Centered rect
    let x = area.width.saturating_sub(w) / 2;
    let y = area.height.saturating_sub(h) / 2;
    let inner = Rect::new(x, y, w, h);

    // Row tracker
    let mut row_y = inner.y;

    // 1) Banner (transparent draw, theme gradient)
    let banner_colors_vec = generate_darker_shades(app.theme.title_accent.to_tui_color(), 6);
    for (i, line) in BANNER_LINES.iter().enumerate() {
        if row_y >= inner.y.saturating_add(inner.height) {
            break;
        }
        let color = banner_colors_vec
            .get(i)
            .copied()
            .unwrap_or(app.theme.title.to_tui_color());
        render_transparent_line(
            f,
            inner,
            row_y,
            line.trim_end(),
            color,
            Alignment::Center,
        );
        row_y = row_y.saturating_add(1);
    }

    // 2) Gap row
    if row_y < inner.y.saturating_add(inner.height) {
        row_y = row_y.saturating_add(1);
    }

    // 3) Items: Options / Help / Quit in 3-row blocks
    let selected = app.menu_cursor % 3;

    for i in 0..3 {
        if row_y + 3 > inner.y.saturating_add(inner.height) {
            break;
        }

        let item_w = MENU_WIDTH[i];
        let x_off = center_offset(inner.width, item_w);
        let base_rect_x = inner.x.saturating_add(x_off);

        let lines = if i == selected { &MENU_SELECTED[i] } else { &MENU_NORMAL[i] };

        // For each menu item select a base theme color and generate a 3-row
        // shade gradient from it. This applies whether the item is selected
        // or not, so Options/Help/Quit all follow the theme.
        let base_color = match i {
            0 => app.theme.title_accent.to_tui_color(), // Options
            1 => app.theme.info.to_tui_color(),         // Help
            2 => app.theme.error.to_tui_color(),        // Quit
            _ => app.theme.foreground.to_tui_color(),
        };
        let shades = generate_darker_shades(base_color, 3);
        let mut cols_arr = [base_color; 3];
        for (j, c) in shades.into_iter().enumerate().take(3) {
            cols_arr[j] = c;
        }

        for r in 0..3 {
            let rect = Rect::new(base_rect_x, row_y + r as u16, item_w.min(inner.width), 1);
            let mut style = Style::default().fg(cols_arr[r]);

            // For the help item we center the small ascii-art and provide a
            // slightly different selection cue (bold + underlined) to make it
            // stand out as an informational entry.
            let mut alignment = Alignment::Left;
            if i == 1 {
                alignment = Alignment::Center;
            }

            // keep a visual cue for selection (bold). Help gets an extra
            // underline when selected to emphasise it.
            if i == selected {
                // Keep bold as the primary selection cue. Remove UNDERLINED
                // so the Help item no longer shows an underline behind it.
                style = style.add_modifier(Modifier::BOLD);
            }

            let para = Paragraph::new(lines[r]).style(style).alignment(alignment);
            f.render_widget(para, rect);
        }

        row_y = row_y.saturating_add(3);
        if i != 2 {
            row_y = row_y.saturating_add(1); // gap between items
        }
    }

    // small hint block under the menu to explain controls
    if row_y < inner.y.saturating_add(inner.height) {
        // Single hint line centered. Use a muted theme color so it doesn't
        // compete with the menu items.
        let hint_txt = "Use ↑/↓ to select • Enter to open • Esc to exit";
        let hint_color = app.theme.stats_label.to_tui_color();
        let hint_style = Style::default().fg(hint_color).add_modifier(Modifier::ITALIC);
        let hint_x = inner.x;
        let hint_w = inner.width;
        let rect = Rect::new(hint_x, row_y, hint_w, 1);
        let para = Paragraph::new(hint_txt).style(hint_style).alignment(Alignment::Center);
        f.render_widget(para, rect);
    }
}

/// Render a single line as multiple contiguous non-space runs so that background
/// remains transparent (only glyphs are drawn).
fn render_transparent_line<B: Backend>(
    f: &mut Frame<B>,
    bounds: Rect,
    y: u16,
    line: &str,
    color: Color,
    align: Alignment,
) {
    if y >= bounds.y.saturating_add(bounds.height) || bounds.height == 0 {
        return;
    }

    let chars: Vec<char> = line.chars().collect();
    let line_w = chars.len() as u16;
    if line_w == 0 {
        return;
    }

    // Horizontal alignment -> compute starting x for the full line, then each run is offset within it
    let x_off = match align {
        Alignment::Left => 0,
        Alignment::Center => center_offset(bounds.width, line_w),
        Alignment::Right => bounds.width.saturating_sub(line_w),
    };

    let base_x = bounds.x.saturating_add(x_off);
    let mut run_start: Option<usize> = None;

    for (i, ch) in chars.iter().enumerate() {
        if *ch != ' ' {
            if run_start.is_none() {
                run_start = Some(i);
            }
        } else if let Some(start) = run_start.take() {
            draw_run(f, bounds, y, &chars, start, i, base_x, color);
        }
    }
    if let Some(start) = run_start {
        draw_run(f, bounds, y, &chars, start, chars.len(), base_x, color);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_run<B: Backend>(
    f: &mut Frame<B>,
    bounds: Rect,
    y: u16,
    chars: &[char],
    start: usize,
    end: usize,
    base_x: u16,
    color: Color,
) {
    if end <= start {
        return;
    }
    let run: String = chars[start..end].iter().collect();
    let run_w = (end - start) as u16;
    if run_w == 0 {
        return;
    }
    let rx = base_x.saturating_add(start as u16);
    if rx >= bounds.x.saturating_add(bounds.width) {
        return;
    }
    let available = bounds
        .width
        .saturating_sub(rx.saturating_sub(bounds.x))
        .min(run_w);
    if available == 0 {
        return;
    }
    let rect = Rect::new(rx, y, available, 1);
    let para = Paragraph::new(run)
        .style(Style::default().fg(color))
        .alignment(Alignment::Left);
    f.render_widget(para, rect);
}

fn center_offset(container_w: u16, content_w: u16) -> u16 {
    if container_w > content_w {
        (container_w - content_w) / 2
    } else {
        0
    }
}

/// Try to convert a `tui::style::Color` to an RGB triple. For named/light
/// variants we approximate sensible RGB values so we can compute gradients.
fn tui_color_to_rgb(c: Color) -> Option<(u8, u8, u8)> {
    use Color::*;
    match c {
        Rgb(r, g, b) => Some((r, g, b)),
        Black => Some((0, 0, 0)),
        Red => Some((205, 49, 49)),
        Green => Some((13, 188, 121)),
        Yellow => Some((229, 229, 16)),
        Blue => Some((36, 114, 200)),
        Magenta => Some((188, 63, 188)),
        Cyan => Some((17, 168, 205)),
        Gray => Some((153, 153, 153)),
        DarkGray => Some((85, 85, 85)),
        LightRed => Some((255, 85, 85)),
        LightGreen => Some((75, 255, 135)),
        LightYellow => Some((255, 255, 85)),
        LightBlue => Some((135, 206, 250)),
        LightMagenta => Some((255, 135, 255)),
        LightCyan => Some((85, 255, 255)),
        White => Some((255, 255, 255)),
        Indexed(_) | Reset => None,
    }
}

/// Generate `n` shades by progressively darkening the base color toward
/// black. Returns a Vec of `tui::style::Color::Rgb` entries.
fn generate_darker_shades(base: Color, n: usize) -> Vec<Color> {
    let (r0, g0, b0) = tui_color_to_rgb(base).unwrap_or((200, 60, 60));
    let mut out = Vec::with_capacity(n);
    if n == 0 {
        return out;
    }

    for i in 0..n {
        // t goes from 0.0 (original color) to 1.0 (black)
        let t = if n == 1 { 0.0 } else { (i as f32) / ((n - 1) as f32) };
    // Interpolate toward black with slight easing. Avoid producing
    // near-black colors for the darkest shade (which makes the
    // bottom row of menu words look 'black' on many terminals).
    // Keep a reasonable minimum brightness so the glyphs remain
    // visible against dark backgrounds.
    let factor = 1.0 - (t * 0.65); // darkest keeps ~35% brightness
        let r = (r0 as f32 * factor).round().clamp(0.0, 255.0) as u8;
        let g = (g0 as f32 * factor).round().clamp(0.0, 255.0) as u8;
        let b = (b0 as f32 * factor).round().clamp(0.0, 255.0) as u8;
        out.push(Color::Rgb(r, g, b));
    }
    out
}
