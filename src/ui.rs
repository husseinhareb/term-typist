use std::io::{self, Write};
use std::thread;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::clear;
use crate::generator::generate_random_sentence;
use crate::config::read_nb_of_words;

const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const WHITE: &str = "\x1b[0m"; 

pub fn stopwatch() {
    let mut i = 0;
    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        i += 1;
        print!("\r{}s", i);
        io::stdout().flush().unwrap();
    }
}

pub fn listen_for_alphabets() {
    let nb_of_words = match read_nb_of_words() {
        Ok(num) => num,
        Err(err) => {
            eprintln!("Error reading number of words: {}", err);
            return;
        }
    };
    let initial_text = generate_random_sentence(nb_of_words as usize);
    let stdin = io::stdin();
    let mut stdout = io::stdout().into_raw_mode().expect("Failed to set raw mode");

    let mut i = 0;
    let mut char_count = 0;
    let mut char_status: Vec<char> = vec!['N'; initial_text.len()];
    let mut colored_text = String::new();

    print!("{}{}", clear::All, termion::cursor::Goto(1, 1));
    stdout.flush().unwrap();
    println!("{}", initial_text);

    print!("{}{}", termion::cursor::Goto(1, 5), clear::CurrentLine); // Move cursor to the fifth line

    for key in stdin.keys() {
        match key {
            Ok(key_event) => {
                match key_event {
                    termion::event::Key::Backspace => {
                        if i > 0 {
                            i -= 1;
                            char_status[i] = 'N';
                        }
                    }
                    termion::event::Key::Char(c) => {
                        if c == ' ' {
                            if c == initial_text.chars().nth(i).unwrap() {
                                char_status[i] = 'T';
                            } else {
                                char_status[i] = 'F';
                            }
                            i += 1;
                            char_count += 1;
                        }
                        if c.is_alphabetic() {
                            if c == initial_text.chars().nth(i).unwrap() {
                                char_status[i] = 'T';
                            } else {
                                char_status[i] = 'F';
                            }
                            char_count += 1;
                            i += 1;
                        }
                    }
                    _ => {}
                }

                colored_text.clear();
                for (index, char) in initial_text.chars().enumerate() {
                    match char_status[index] {
                        'N' => colored_text.push_str(WHITE),
                        'T' => colored_text.push_str(GREEN),
                        'F' => colored_text.push_str(RED),
                        _ => {}
                    }
                    colored_text.push(char);
                }
                colored_text.push_str(WHITE);
                print!("\r{}", colored_text); // Clear terminal and move cursor to beginning   
                io::stdout().flush().unwrap();      
            }
            Err(err) => {
                eprintln!("Error reading input: {}", err);
                break;
            }
        }
        if i == initial_text.len() {
            break;
        }
        stdout.flush().expect("Failed to flush stdout");
    }
}

pub fn run_simultaneously() {
    let stopwatch_handle = thread::spawn(|| {
        stopwatch();
    });

    let listener_handle = thread::spawn(|| {
        listen_for_alphabets();
    });

    stopwatch_handle.join().expect("Stopwatch thread panicked");
    listener_handle.join().expect("Listener thread panicked");

    // After the user finishes typing, print the timer on a separate line
    println!("\n\nTimer ended");
}

fn main() {
    run_simultaneously();
}
