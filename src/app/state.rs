// /src/app/state.rs
use std::time::{Duration, Instant};
use crate::theme::Theme;

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
        Profile,
        Settings,
        Menu,
        Help,
        Leaderboard,
        TestDetail,
}

/// Supported keyboard layouts for on-screen keyboard and input mapping.
#[derive(PartialEq, Clone, Copy)]
pub enum KeyboardLayout {
    Qwerty,
    Azerty,
    Dvorak,
    Qwertz,
}

/// Supported languages for word generation.
#[derive(PartialEq, Clone, Copy)]
pub enum Language {
    English,
    German,
    Spanish,
    French,
    Japanese,
}

/// Application state tracking typing progress, timing, UI flags, and samples.
pub struct App {
    pub target: String,
    pub status: Vec<Status>,
    /// Whether a given character was ever typed incorrectly and later corrected.
    /// Indexed by character position in `target`.
    pub corrected: Vec<bool>,
    pub start: Option<Instant>,
    pub last_input: Instant,
    pub correct_chars: usize,
    pub incorrect_chars: usize,
    pub correct_timestamps: Vec<Instant>,
    pub incorrect_timestamps: Vec<Instant>,
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
    pub keyboard_layout: KeyboardLayout,
    pub keyboard_switch: String,
    pub audio_enabled: bool,
    pub language: Language,
    pub theme: Theme,
    // Cursor index for the Settings list (which line is selected)
    pub settings_cursor: usize,
    // Cursor for popup menu
    pub menu_cursor: usize,
    // Persistent Caps Lock state (best-effort heuristic in terminal environments)
    pub caps_lock_on: bool,
    // True when the platform provides a system-backed method to detect CapsLock
    pub caps_detection_available: bool,
    // Test ID being viewed in detail mode (from Profile/Leaderboard)
    pub viewing_test_id: Option<i64>,
    // Previous mode to return to when exiting test detail view
    pub previous_mode: Option<Mode>,
}

impl App {
    /// Construct a new App with a target sentence and default state.
    pub fn new(target: String) -> Self {
        let now = Instant::now();
        // Attempt to read persisted keyboard layout from config
        let kb_layout = match crate::app::config::read_keyboard_layout() {
            Ok(Some(name)) => match name.to_lowercase().as_str() {
                "qwerty" => KeyboardLayout::Qwerty,
                "azerty" => KeyboardLayout::Azerty,
                "dvorak" => KeyboardLayout::Dvorak,
                "qwertz" => KeyboardLayout::Qwertz,
                _ => KeyboardLayout::Qwerty,
            },
            _ => KeyboardLayout::Qwerty,
        };
        // Determine persisted keyboard switch, fallback to first available or "mxblack"
        let kb_switch = match crate::app::config::read_keyboard_switch() {
            Ok(Some(name)) => name,
            _ => {
                // try to list available switches from audio assets
                let list = crate::audio::list_switches();
                if !list.is_empty() { list[0].clone() } else { "mxblack".into() }
            }
        };

        // Read persisted audio setting (default true)
        let audio_enabled = match crate::app::config::read_audio_enabled() {
            Ok(Some(b)) => b,
            _ => true,
        };

        // Read persisted language setting (default English)
        let language = match crate::app::config::read_language() {
            Ok(Some(name)) => match name.to_lowercase().as_str() {
                "english" => Language::English,
                "german" => Language::German,
                "spanish" => Language::Spanish,
                "french" => Language::French,
                "japanese" => Language::Japanese,
                _ => Language::English,
            },
            _ => Language::English,
        };

        App {
            target: target.clone(),
            status: vec![Status::Untyped; target.chars().count()],
            corrected: vec![false; target.chars().count()],
            start: None,
            last_input: now,
            correct_chars: 0,
            incorrect_chars: 0,
            correct_timestamps: Vec::new(),
            incorrect_timestamps: Vec::new(),
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
            keyboard_layout: kb_layout,
            keyboard_switch: kb_switch,
            audio_enabled,
            language,
            theme: Theme::load(),
            settings_cursor: 0,
            menu_cursor: 0,
            caps_lock_on: false,
            caps_detection_available: false,
            viewing_test_id: None,
            previous_mode: None,
        }
    }

    /// Handle a typed character, updating status, counts, and locking logic.
    pub fn on_key(&mut self, key: char) {
        self.last_input = Instant::now();
        // audio handled by caller (lib.rs) to play per-KeyCode samples
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
            if correct {
                self.status[i] = Status::Correct;
                self.correct_chars += 1;
                let now = Instant::now();
                self.last_correct = now;
                self.correct_timestamps.push(now);
            } else {
                self.status[i] = Status::Incorrect;
                self.incorrect_chars += 1;
                self.incorrect_timestamps.push(Instant::now());
                // mark this character position as having been incorrect at
                // least once so later corrections can be highlighted.
                if i < self.corrected.len() {
                    self.corrected[i] = true;
                }
            }
        }
        // Re-lock if idle > 1s
        if self.start.is_some() && Instant::now().duration_since(self.last_correct) >= Duration::from_secs(1) {
            self.locked = true;
        }
    }

    /// Handle backspace: undo last typed char status or free-text.
    pub fn backspace(&mut self) {
        self.last_input = Instant::now();
        if self.audio_enabled {
            crate::audio::play_for(&self.keyboard_switch, "BACKSPACE");
        }
        if self.selected_tab == 2 {
            if self.free_text.pop().is_some() && self.correct_chars > 0 {
                self.correct_chars -= 1;
            }
            return;
        }
        if let Some(i) = self.status.iter().rposition(|&s| s != Status::Untyped) {
            match self.status[i] {
                Status::Correct if self.correct_chars > 0 => {
                    self.correct_chars -= 1;
                    // remove the last timestamp corresponding to the last correct char
                    self.correct_timestamps.pop();
                }
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
