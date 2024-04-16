
use std::time::{SystemTime, Duration};
use std::io::{self, Write};
pub fn elapsed_seconds_since_start(start_time: SystemTime) -> f64 {
    // Get the current time
    let current_time = SystemTime::now();

    // Calculate the duration since the code started running
    let elapsed_time = current_time.duration_since(start_time).expect("Time went backwards");

    // Convert the duration to seconds as a floating-point number
    elapsed_time.as_secs_f64()
}

