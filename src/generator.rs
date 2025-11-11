// src/generator.rs
use random_word::Lang;
use crate::app::state::Language;

pub fn generate_random_sentence(num_words: usize, language: Language) -> String {
    let lang = match language {
        Language::English => Lang::En,
        Language::German => Lang::De,
        Language::Spanish => Lang::Es,
        Language::French => Lang::Fr,
        Language::Japanese => Lang::Ja,
    };
    
    (0..num_words)
        .map(|_| random_word::gen(lang).to_lowercase())
        .collect::<Vec<_>>()
        .join(" ")
}

/// Generate a target text sized according to the selected test mode and value.
///
/// - `selected_tab`: 0 = Time, 1 = Words, 2 = Zen
/// - `selected_value`: index into the options returned by `App::current_options()` for tabs 0/1
/// - `language`: the language to use for word generation
pub fn generate_for_mode(selected_tab: usize, selected_value: usize, language: Language) -> String {
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

    generate_random_sentence(num_words, language)
}
