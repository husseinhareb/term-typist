// src/ui/keyboard.rs
use crossterm::event::KeyCode;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Style},
    text::Span,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::input::map_keycode;
use crate::theme::Theme;
use crate::app::state::KeyboardLayout;

/// On-screen keyboard widget with realistic key sizes and pressed‐key highlighting.
pub struct Keyboard {
    pub pressed_key: Option<String>,
}

impl Keyboard {
    /// Create a new, empty Keyboard widget.
    pub fn new() -> Self {
        Keyboard { pressed_key: None }
    }

    /// Update the widget's pressed key based on the raw `KeyCode`.
    pub fn handle_key(&mut self, code: &KeyCode) {
        self.pressed_key = map_keycode(code);
    }

    /// Draw the keyboard into the given `area`, splitting into rows and keys.
    pub fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Rect, theme: &Theme, layout: KeyboardLayout) {
        // Define rows per layout as arrays of (label, width units)
        let rows_qwerty: &[&[(&str, u16)]] = &[
            &[("Esc", 2), ("1", 1), ("2", 1), ("3", 1), ("4", 1), ("5", 1), ("6", 1), ("7", 1), ("8", 1), ("9", 1), ("0", 1), ("-", 1), ("=", 1), ("Backspace", 3)],
            &[("Tab", 3), ("Q", 1), ("W", 1), ("E", 1), ("R", 1), ("T", 1), ("Y", 1), ("U", 1), ("I", 1), ("O", 1), ("P", 1), ("[", 1), ("]", 1), ("\\", 2)],
            &[("CapsLk", 3), ("A", 1), ("S", 1), ("D", 1), ("F", 1), ("G", 1), ("H", 1), ("J", 1), ("K", 1), ("L", 1), (";", 1), ("'", 1), ("Enter", 3)],
            &[("Shift", 4), ("Z", 1), ("X", 1), ("C", 1), ("V", 1), ("B", 1), ("N", 1), ("M", 1), (",", 1), (".", 1), ("/", 1), ("Shift", 4)],
            &[ ("Space", 6) ],
        ];

        let rows_azerty: &[&[(&str, u16)]] = &[
            &[("Esc", 2), ("&", 1), ("é", 1), ("\"", 1), ("'", 1), ("(", 1), ("-", 1), ("è", 1), ("_", 1), ("ç", 1), ("à", 1), ("°", 1), ("=", 1), ("Backspace", 3)],
            &[("Tab", 3), ("A", 1), ("Z", 1), ("E", 1), ("R", 1), ("T", 1), ("Y", 1), ("U", 1), ("I", 1), ("O", 1), ("P", 1), ("^", 1), ("$", 1), ("\\", 2)],
            &[("CapsLk", 3), ("Q", 1), ("S", 1), ("D", 1), ("F", 1), ("G", 1), ("H", 1), ("J", 1), ("K", 1), ("L", 1), ("M", 1), ("ù", 1), ("Enter", 3)],
            &[("Shift", 4), (",", 1), (";", 1), (":", 1), ("!", 1), ("%", 1), ("?", 1), ("/", 1), (".", 1), ("-", 1), ("Shift", 4)],
            &[ ("Space", 6) ],
        ];

        let rows_dvorak: &[&[(&str, u16)]] = &[
            &[("Esc", 2), ("1", 1), ("2", 1), ("3", 1), ("4", 1), ("5", 1), ("6", 1), ("7", 1), ("8", 1), ("9", 1), ("0", 1), ("[", 1), ("]", 1), ("Backspace", 3)],
            &[("Tab", 3), ("'", 1), (",", 1), (".", 1), ("P", 1), ("Y", 1), ("F", 1), ("G", 1), ("C", 1), ("R", 1), ("L", 1), ("/", 1), ("=", 1), ("\\", 2)],
            &[("CapsLk", 3), ("A", 1), ("O", 1), ("E", 1), ("U", 1), ("I", 1), ("D", 1), ("H", 1), ("T", 1), ("N", 1), ("S", 1), ("-", 1), ("Enter", 3)],
            &[("Shift", 4), (";", 1), ("Q", 1), ("J", 1), ("K", 1), ("X", 1), ("B", 1), ("M", 1), ("W", 1), ("V", 1), ("Z", 1), ("Shift", 4)],
            &[ ("Space", 6) ],
        ];

        let rows_qwertz: &[&[(&str, u16)]] = &[
            &[("Esc", 2), ("1", 1), ("2", 1), ("3", 1), ("4", 1), ("5", 1), ("6", 1), ("7", 1), ("8", 1), ("9", 1), ("0", 1), ("ß", 1), ("´", 1), ("Backspace", 3)],
            &[("Tab", 3), ("Q", 1), ("W", 1), ("E", 1), ("R", 1), ("T", 1), ("Z", 1), ("U", 1), ("I", 1), ("O", 1), ("P", 1), ("ü", 1), ("+", 1), ("\\", 2)],
            &[("CapsLk", 3), ("A", 1), ("S", 1), ("D", 1), ("F", 1), ("G", 1), ("H", 1), ("J", 1), ("K", 1), ("L", 1), ("ö", 1), ("ä", 1), ("Enter", 3)],
            &[("Shift", 4), ("<", 1), ("Y", 1), ("X", 1), ("C", 1), ("V", 1), ("B", 1), ("N", 1), ("M", 1), (",", 1), (".", 1), ("-", 1), ("Shift", 4)],
            &[ ("Space", 6) ],
        ];

        let rows: &[&[(&str, u16)]] = match layout {
            KeyboardLayout::Qwerty => rows_qwerty,
            KeyboardLayout::Azerty => rows_azerty,
            KeyboardLayout::Dvorak => rows_dvorak,
            KeyboardLayout::Qwertz => rows_qwertz,
        };
        // Use the entire available area for the keyboard and distribute it
        // evenly between the rows so the keys occupy the full pane height.
        let keyboard_area = area;
        let row_count = rows.len() as u32;
        let row_areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Ratio(1, row_count); rows.len()])
            .split(keyboard_area);

    for (r, &row) in rows.iter().enumerate() {
            let row_area = row_areas[r];

            // Total “unit” width of this row
            let total_units: u16 = row.iter().map(|&(_, u)| u).sum();
            let mut remaining = row_area.width;

            // Compute each key’s absolute width
            let widths: Vec<u16> = row
                .iter()
                .enumerate()
                .map(|(i, &(_, units))| {
                    let w = if i + 1 < row.len() {
                        ((row_area.width as u32 * units as u32) / total_units as u32) as u16
                    } else {
                        remaining
                    };
                    remaining = remaining.saturating_sub(w);
                    w
                })
                .collect();

            // Split the row into key rectangles
            let key_areas = Layout::default()
                .direction(Direction::Horizontal)
                 .constraints(
        widths
            .iter()
            .map(|&w| Constraint::Length(w))
            .collect::<Vec<Constraint>>(),
    )
                .split(row_area);

            // Render each key
            for (i, &(label, _)) in row.iter().enumerate() {
                // If the final row contains only the Space key, render that key
                // using the entire keyboard_area so it becomes a single large
                // button that fills the keyboard section.
                // If the final row contains only the Space key, render that key
                // using the last row's area so it fills the bottom row only and
                // doesn't overlap the rows above.
                let key_area = if r + 1 == rows.len() && row.len() == 1 && label == "Space" {
                    row_area
                } else {
                    key_areas[i]
                };
                // Compare normalized forms (case/Unicode-aware) so labels with different
                // casing or accents still match the pressed key string.
                let is_pressed = match &self.pressed_key {
                    Some(pk) => pk.to_lowercase() == label.to_lowercase(),
                    None => false,
                };

                let bg = if is_pressed { 
                    theme.key_pressed_bg.to_tui_color() 
                } else { 
                    theme.key_normal_bg.to_tui_color() 
                };
                let fg = if is_pressed { 
                    theme.key_pressed_fg.to_tui_color() 
                } else { 
                    theme.key_normal_fg.to_tui_color() 
                };

                // 1) Draw the border block (so the visible border is produced)
                let border = Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.key_border.to_tui_color()))
                    .style(Style::default().bg(bg));
                // Render the border (clone so we can still call methods on `border`)
                f.render_widget(border.clone(), key_area);

                // 2) Fill only the block's inner rect so the background/highlight
                // does not overflow the border characters.
                let fill_area = border.inner(key_area);
                let fill = Block::default().style(Style::default().bg(bg));
                f.render_widget(fill, fill_area);

                // 3) Center the label inside the inner rect (no background on outer)
                let txt = Paragraph::new(Span::styled(label, Style::default().fg(fg).bg(bg)))
                    .alignment(Alignment::Center);
                f.render_widget(txt, fill_area);
            }
        }
    }
}
