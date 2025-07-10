// src/wpm.rs

use std::time::Instant;

/// Returns elapsed time in seconds since `start`.
pub fn elapsed_seconds_since_start(start: Instant) -> f64 {
    start.elapsed().as_secs_f64()
}

/// Gross WPM: total keystrokes (correct + incorrect) ÷ 5, divided by minutes.
pub fn gross_wpm(chars_typed: usize, elapsed_secs: f64) -> f64 {
    if elapsed_secs <= 0.0 {
        return 0.0;
    }
    let minutes = elapsed_secs / 60.0;
    (chars_typed as f64 / 5.0) / minutes
}

/// Net WPM: (correct_chars – incorrect_chars) ÷ 5, divided by minutes, floored at zero.
/// This mirrors MonkeyType’s “word” penalty: each wrong keystroke cancels one correct one.
pub fn net_wpm(correct_chars: usize, incorrect_chars: usize, elapsed_secs: f64) -> f64 {
    if elapsed_secs <= 0.0 {
        return 0.0;
    }
    let minutes = elapsed_secs / 60.0;
    // Compute signed character difference
    let diff = correct_chars as i32 - incorrect_chars as i32;
    if diff <= 0 {
        return 0.0;
    }
    (diff as f64 / 5.0) / minutes
}

/// Accuracy percentage: correct_chars ÷ (correct_chars + incorrect_chars) × 100.
pub fn accuracy(correct_chars: usize, incorrect_chars: usize) -> f64 {
    let total = correct_chars + incorrect_chars;
    if total == 0 {
        100.0
    } else {
        (correct_chars as f64 / total as f64) * 100.0
    }
}
