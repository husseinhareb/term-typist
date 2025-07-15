// src/keyboard.rs

use crossterm::event::KeyCode;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// A little widget that draws an on‑screen keyboard and highlights the last pressed key.
pub struct Keyboard {
    /// Label of the last key pressed (e.g. "A", "BS", "SPC", etc.)
    pub pressed_key: Option<String>,
}

impl Keyboard {
    /// Construct a new, empty keyboard widget.
    pub fn new() -> Self {
        Keyboard { pressed_key: None }
    }

    /// Call on each KeyCode event to update which key is highlighted.
    pub fn handle_key(&mut self, code: &KeyCode) {
        self.pressed_key = map_keycode(code);
    }

    /// Draw the keyboard into the given `area` of the frame.
    pub fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        // Define keyboard rows (you can tweak these strings to exactly match your layout)
        let rows = vec![
            vec!["ESC","1","2","3","4","5","6","7","8","9","0","-","=","BS"],
            vec!["Q","W","E","R","T","Y","U","I","O","P","[","]","\\"],
            vec!["A","S","D","F","G","H","J","K","L",";","'"],
            vec!["Z","X","C","V","B","N","M",",",".","/"],
            vec!["SPC"],
        ];

        // Split the overall area into one horizontal band per row
        let row_areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(rows.iter().map(|_| Constraint::Length(3)).collect::<Vec<_>>())
            .split(area);

        for (r, keys) in rows.iter().enumerate() {
            let row_area = row_areas[r];
            // If it's just the space bar, give it full width; else equal fixed width
            let constraints = if keys.len() == 1 && keys[0] == "SPC" {
                vec![Constraint::Percentage(100)]
            } else {
                keys.iter().map(|_| Constraint::Length(5)).collect()
            };
            let key_areas = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(constraints)
                .split(row_area);

            for (i, &label) in keys.iter().enumerate() {
                let is_pressed = self.pressed_key.as_deref() == Some(label);
                let (fg, bg) = if is_pressed {
                    (Color::Black, Color::Yellow)
                } else {
                    (Color::White, Color::Reset)
                };

                // Center the label in the cell
                let width = key_areas[i].width as usize;
                let text = format!("{:^1$}", label, width);

                let paragraph = Paragraph::new(Span::styled(text, Style::default().fg(fg).bg(bg)))
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(paragraph, key_areas[i]);
            }
        }
    }
}

/// Map a crossterm KeyCode to one of our on-screen key labels.
fn map_keycode(code: &KeyCode) -> Option<String> {
    match code {
        KeyCode::Esc           => Some("ESC".into()),
        KeyCode::Backspace     => Some("BS".into()),
        KeyCode::Char(' ')     => Some("SPC".into()),
        KeyCode::Char(c)       => Some(c.to_ascii_uppercase().to_string()),
        _                      => None,
    }
}
