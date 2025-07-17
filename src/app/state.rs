// /src/app/state.rs
use std::time::{Duration, Instant};

/// Character typing status for each target character.
#[derive(PartialEq, Clone, Copy)]
pub enum Status {
    Untyped,
    Correct,
    Incorrect,
}

/// Current application mode: selecting, typing, or finished.
#[derive(PartialEq, Clone, Copy)]
pub enum Mode {
    View,
    Insert,
    Finished,
}

/// Application state tracking typing progress, timing, UI flags, and samples.
pub struct App {
    pub target: String,
    pub status: Vec<Status>,
    pub start: Option<Instant>,
    pub last_input: Instant,
    pub correct_chars: usize,
    pub incorrect_chars: usize,
    pub last_correct: Instant,
    pub locked: bool,
    pub free_text: String,
    pub selected_tab: usize,
    pub selected_value: usize,
    pub mode: Mode,
    pub samples: Vec<(u64, f64)>,

    // Visibility toggles for UI sections
    pub show_mode: bool,
    pub show_value: bool,
    pub show_state: bool,
    pub show_speed: bool,
    pub show_timer: bool,
    pub show_text: bool,
    pub show_keyboard: bool,
}

impl App {
    /// Construct a new App with a target sentence and default state.
    pub fn new(target: String) -> Self {
        let now = Instant::now();
        App {
            target: target.clone(),
            status: vec![Status::Untyped; target.chars().count()],
            start: None,
            last_input: now,
            correct_chars: 0,
            incorrect_chars: 0,
            last_correct: now,
            locked: false,
            free_text: String::new(),
            selected_tab: 0,
            selected_value: 0,
            mode: Mode::View,
            samples: Vec::new(),

            show_mode: true,
            show_value: true,
            show_state: true,
            show_speed: true,
            show_timer: true,
            show_text: true,
            show_keyboard: true,
        }
    }

    /// Handle a typed character, updating status, counts, and locking logic.
    pub fn on_key(&mut self, key: char) {
        self.last_input = Instant::now();
        // Free-text (zen) mode
        if self.selected_tab == 2 {
            self.free_text.push(key);
            self.correct_chars += 1;
            self.last_correct = Instant::now();
            return;
        }
        // Locked-after-idle: only accept correct next char
        if self.locked {
            if let Some(i) = self.status.iter().position(|&s| s == Status::Untyped) {
                if key != self.target.chars().nth(i).unwrap() {
                    return;
                }
                self.locked = false;
            }
        }
        // Mark correct or incorrect
        if let Some(i) = self.status.iter().position(|&s| s == Status::Untyped) {
            let expected = self.target.chars().nth(i).unwrap();
            let correct = key == expected;
            self.status[i] = if correct {
                self.correct_chars += 1;
                self.last_correct = Instant::now();
                Status::Correct
            } else {
                self.incorrect_chars += 1;
                Status::Incorrect
            };
        }
        // Re-lock if idle > 1s
        if let Some(_) = self.start {
            if Instant::now().duration_since(self.last_correct) >= Duration::from_secs(1) {
                self.locked = true;
            }
        }
    }

    /// Handle backspace: undo last typed char status or free-text.
    pub fn backspace(&mut self) {
        self.last_input = Instant::now();
        if self.selected_tab == 2 {
            if self.free_text.pop().is_some() && self.correct_chars > 0 {
                self.correct_chars -= 1;
            }
            return;
        }
        if let Some(i) = self.status.iter().rposition(|&s| s != Status::Untyped) {
            match self.status[i] {
                Status::Correct if self.correct_chars > 0 => self.correct_chars -= 1,
                Status::Incorrect if self.incorrect_chars > 0 => self.incorrect_chars -= 1,
                _ => {}
            }
            self.status[i] = Status::Untyped;
            self.locked = false;
        }
    }

    /// Seconds elapsed since start-of-typing.
    pub fn elapsed_secs(&self) -> u64 {
        self.start.map(|s| s.elapsed().as_secs()).unwrap_or(0)
    }

    /// Current options for time or word count, based on selected tab.
    pub fn current_options(&self) -> &'static [u16] {
        match self.selected_tab {
            0 => &[15, 30, 60, 100],
            1 => &[10, 25, 50, 100],
            _ => &[],
        }
    }
}
