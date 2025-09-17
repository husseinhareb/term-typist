// src/wpm.rs

use std::time::Instant;

/// Returns elapsed time in seconds since `start`.
pub fn elapsed_seconds_since_start(start: Instant) -> f64 {
    start.elapsed().as_secs_f64()
}

/// Net WPM: (correct_chars – incorrect_chars) ÷ 5, divided by minutes, floored at zero.
pub fn net_wpm(correct_chars: usize, incorrect_chars: usize, elapsed_secs: f64) -> f64 {
    if elapsed_secs <= 0.0 {
        return 0.0;
    }
    let minutes = elapsed_secs / 60.0;
    // Compute signed character difference
    let diff = (correct_chars as i32) - (incorrect_chars as i32);
    if diff <= 0 {
        return 0.0;
    }
    (diff as f64) / 5.0 / minutes
}

/// Accuracy percentage: correct_chars ÷ (correct_chars + incorrect_chars) × 100.
pub fn accuracy(correct_chars: usize, incorrect_chars: usize) -> f64 {
    let total = correct_chars + incorrect_chars;
    if total == 0 {
        100.0
    } else {
        ((correct_chars as f64) / (total as f64)) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn net_wpm_zero_elapsed() {
        assert_eq!(net_wpm(10, 2, 0.0), 0.0);
    }

    #[test]
    fn net_wpm_negative_diff() {
        // more incorrect than correct
        assert_eq!(net_wpm(2, 5, 60.0), 0.0);
    }

    #[test]
    fn net_wpm_basic() {
        // 55 correct, 5 incorrect => diff = 50 chars => 10 words in 1 minute => 10 WPM
        let w = net_wpm(55, 5, 60.0);
        assert!((w - 10.0).abs() < 1e-6);
    }

    #[test]
    fn accuracy_zero_total() {
        assert_eq!(accuracy(0, 0), 100.0);
    }

    #[test]
    fn accuracy_basic() {
        // 80 correct, 20 incorrect => 80% accuracy
        let a = accuracy(80, 20);
        assert!((a - 80.0).abs() < 1e-6);
    }
}
