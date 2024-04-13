use std::io::{self, Write};
use std::time::{Instant, Duration};

fn wpm() {
    println!("Type some text. Your typing speed will be calculated in real-time.");
    
    let mut input = String::new();
    let mut last_key_time = Instant::now();
    let mut char_count = 0;

    loop {
        input.clear();
        match io::stdin().read_line(&mut input) {
            Ok(_) => {
                let now = Instant::now();
                let elapsed = now - last_key_time;
                last_key_time = now;
                
                char_count += input.trim().chars().count();
                let elapsed_seconds = elapsed.as_secs_f64();
                let speed = (char_count as f64 / elapsed_seconds)/5.0 * 60.0; // Calculate speed in characters per minute

                println!("Speed: {:.2} words per minute", speed);
            }
            Err(error) => println!("Error reading input: {}", error),
        }
    }
}
