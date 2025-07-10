// src/wpm.rs

use std::time::Instant;

/// Returns the elapsed time in seconds since `start`.
pub fn elapsed_seconds_since_start(start: Instant) -> f64 {
    start.elapsed().as_secs_f64()
}

/// Gross WPM: total characters (including errors) ÷ 5, divided by minutes.
pub fn gross_wpm(chars_typed: usize, elapsed_secs: f64) -> f64 {
    if elapsed_secs <= 0.0 {
        return 0.0;
    }
    let minutes = elapsed_secs / 60.0;
    (chars_typed as f64 / 5.0) / minutes
}

/// Net WPM: gross WPM minus error penalty (errors ÷ minutes), floored at zero.
pub fn net_wpm(correct_chars: usize, incorrect_chars: usize, elapsed_secs: f64) -> f64 {
    if elapsed_secs <= 0.0 {
        return 0.0;
    }
    let minutes = elapsed_secs / 60.0;
    let gross = gross_wpm(correct_chars + incorrect_chars, elapsed_secs);
    let penalty = incorrect_chars as f64 / minutes;
    (gross - penalty).max(0.0)
}

/// Accuracy percentage: correct_chars ÷ total_chars × 100.
pub fn accuracy(correct_chars: usize, incorrect_chars: usize) -> f64 {
    let total = correct_chars + incorrect_chars;
    if total == 0 {
        100.0
    } else {
        (correct_chars as f64 / total as f64) * 100.0
    }
}
