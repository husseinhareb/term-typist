use std::io::{self, Write};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use crate::generator::generate_random_sentence;

const GREEN: &str = "\x1b[32m";
const RED: &str = "\x1b[31m";
const WHITE: &str = "\x1b[0m"; 

pub fn listen_for_alphabets() {
    let initial_text = generate_random_sentence(30).to_string();

    let stdin = io::stdin();
    let mut stdout = io::stdout().into_raw_mode().expect("Failed to set raw mode");

    let mut i = 0;
    let mut char_status: Vec<char> = vec!['N'; initial_text.len()]; // 'N' means neutral, 'T' means true (correct), 'F' means false (incorrect)
    let mut colored_text = String::new(); 

    println!("{}", initial_text);

    for key in stdin.keys() {
        match key {
            Ok(key_event) => {
                match key_event {
                    termion::event::Key::Backspace => {
                        if i > 0 {
                            i -= 1;
                            char_status[i] = 'N';

                            // Regenerate colored text
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
                            print!("{}{}", termion::cursor::Goto(1, 1), colored_text); // Move cursor to beginning of line before printing
                        }
                    }
                    termion::event::Key::Char(c) if c.is_alphabetic() => {
                        print!("{}{}{}", termion::clear::All, termion::cursor::Goto(1, 1), termion::cursor::Hide);
                        io::stdout().flush().unwrap();

                        if c == initial_text.chars().nth(i).unwrap() {
                            char_status[i] = 'T';
                        } else {
                            char_status[i] = 'F';
                        }

                        // Regenerate colored text
                        colored_text.clear();
                        for (index, char) in initial_text.chars().enumerate() {
                            match char_status[index] {
                                'N' => colored_text.push_str(WHITE), // Neutral (not reached yet) is white
                                'T' => colored_text.push_str(GREEN), // True (correctly matched) is green
                                'F' => colored_text.push_str(RED),   // False (incorrectly matched) is red
                                _ => {}
                            }
                            colored_text.push(char);
                        }
                        colored_text.push_str(WHITE);
                        println!("{}", colored_text);

                        i += 1; // Increment i for alphabetic keys
                    }
                    // Handle other key presses
                    _ => {}
                }
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
