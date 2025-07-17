use crossterm::event::KeyCode;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use crate::app::input::map_keycode;

/// On-screen keyboard widget with realistic key sizes and pressed-key highlighting.
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
    pub fn draw<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        // Define rows as arrays of (label, width units)
        let rows: &[&[(&str, u16)]] = &[
            &[("ESC",2),("1",1),("2",1),("3",1),("4",1),("5",1),("6",1),("7",1),("8",1),("9",1),("0",1),("-",1),("=",1),("BS",3)],
            &[("TAB",3),("Q",1),("W",1),("E",1),("R",1),("T",1),("Y",1),("U",1),("I",1),("O",1),("P",1),("[",1),("]",1),("\\",2)],
            &[("CAPS",3),("A",1),("S",1),("D",1),("F",1),("G",1),("H",1),("J",1),("K",1),("L",1),(";",1),("'",1),("ENTER",3)],
            &[("SHIFT",4),("Z",1),("X",1),("C",1),("V",1),("B",1),("N",1),("M",1),("," ,1),(".",1),("/",1),("SHIFT",4)],
            &[("SPC",12)],
        ];

        // Split the full area vertically into one stripe per row
        let row_areas = Layout::default()
            .direction(Direction::Vertical)
            .constraints(rows.iter().map(|_| Constraint::Length(3)).collect::<Vec<_>>())
            .split(area);

        for (r, &row) in rows.iter().enumerate() {
            let row_area = row_areas[r];
            let total_units: u16 = row.iter().map(|&(_, u)| u).sum();
            let mut remaining = row_area.width;

            // Compute each key's width
            let widths: Vec<u16> = row.iter().enumerate().map(|(i, &(_, units))| {
                let w = if i + 1 < row.len() {
                    ((row_area.width as u32 * units as u32) / total_units as u32) as u16
                } else {
                    remaining
                };
                remaining = remaining.saturating_sub(w);
                w
            }).collect();

            // Split horizontally by those widths
            let key_areas = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(widths.iter().map(|&w| Constraint::Length(w)).collect::<Vec<_>>())
                .split(row_area);

            // Render each key
            for (i, &(label, _)) in row.iter().enumerate() {
                let key_area = key_areas[i];
                let is_pressed = self.pressed_key.as_deref() == Some(label);
                let bg = if is_pressed { Color::Yellow } else { Color::Reset };
                let fg = if is_pressed { Color::Black } else { Color::White };
                let text = format!("{:^width$}", label, width = widths[i] as usize);
                let widget = Paragraph::new(Span::styled(text, Style::default().fg(fg).bg(bg)))
                    .block(Block::default().borders(Borders::ALL));
                f.render_widget(widget, key_area);
            }
        }
    }
}
