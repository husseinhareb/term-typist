// src/generator.rs
use std::fs::File;
use std::io::{self, BufRead};
// PathBuf not needed here (we manipulate dirs::data_dir())
use rand::seq::SliceRandom;

/// Read the bundled word list from the XDG data directory.
/// Expected path: $XDG_DATA_HOME/term-typist/words/words.txt
fn read_words() -> io::Result<Vec<String>> {
    let mut file_path = match dirs::data_dir() {
        Some(p) => p,
        None => return Err(io::Error::new(io::ErrorKind::NotFound, "data directory not found")),
    };

    file_path.push("term-typist");
    file_path.push("words");
    file_path.push("words.txt");

    let file = File::open(&file_path)?;
    let reader = io::BufReader::new(file);
    let mut words = Vec::new();

    for line in reader.lines() {
        if let Ok(word) = line {
            words.push(word);
        }
    }

    Ok(words)
}

pub fn generate_random_sentence(num_words: usize) -> String {
    let words = match read_words() {
        Ok(words) => words,
        Err(err) => {
            eprintln!(
                "Error reading words: {}. Expected: $XDG_DATA_HOME/term-typist/words/words.txt",
                err
            );
            return String::new();
        }
    };

    let mut rng = rand::thread_rng();
    let mut sentence = String::new();

    for _ in 0..num_words {
        if let Some(random_word) = words.choose(&mut rng) {
            sentence.push_str(random_word);
            sentence.push(' ');
        }
    }

    sentence.trim().to_string()
}

/// Generate a target text sized according to the selected test mode and value.
///
/// - `selected_tab`: 0 = Time, 1 = Words, 2 = Zen
/// - `selected_value`: index into the options returned by `App::current_options()` for tabs 0/1
pub fn generate_for_mode(selected_tab: usize, selected_value: usize) -> String {
    // Conservative baseline typing speed used to size time-based tests (words per minute).
    // Raised from 40 to 60 so faster typists have more words available during the timer.
    const DEFAULT_WPM: f64 = 60.0;

    let num_words = match selected_tab {
        0 => {
            // Time mode: compute words = seconds * (WPM / 60)
            let options = [15u16, 30u16, 60u16, 100u16];
            let secs = options.get(selected_value).copied().unwrap_or(30u16) as f64;
            let words = (secs * (DEFAULT_WPM / 60.0)).ceil() as usize;
            // add a larger buffer so fast typists don't run out of words before the
            // timer expires. Ensure a reasonable minimum size.
            (words + 8).max(12)
        }
        1 => {
            // Words mode: generate roughly the requested number of words, plus a small buffer
            let options = [10u16, 25u16, 50u16, 100u16];
            let w = options.get(selected_value).copied().unwrap_or(25u16) as usize;
            let buffer = (w as f64 * 0.12).ceil() as usize + 2; // ~12% + 2 words
            w + buffer
        }
        _ => {
            // Zen mode: produce a long, continuous block
            200usize
        }
    };

    generate_random_sentence(num_words)
}
