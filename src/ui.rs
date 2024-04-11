extern crate termion;

use std::io::{self, Write};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use crate::generator::generate_random_sentence;

pub fn print_and_input_on_top(initial_text: String) -> io::Result<String> {

    io::stdout().flush()?;

    // Move the cursor to the leftmost position on the current line (column 1)
    print!("\r");
    io::stdout().flush()?;

    // Calculate the number of lines in the initial text
    let num_newlines = initial_text.chars().filter(|&c| c == '\n').count();
    // Move the cursor up by the number of lines
    print!("\x1B[{}A", num_newlines);
    io::stdout().flush()?;

    let stdin = io::stdin();
    let mut input = String::new();
    let mut cursor_position = initial_text.len();

    loop {
        // Get user input
        stdin.read_line(&mut input)?;
        // Remove newline character at the end
        input.pop();
        // Print the input
        print!("{}", input);
        io::stdout().flush()?;
        // Clear the rest of the line
        print!("\x1B[0K");
        io::stdout().flush()?;

        // Move cursor back to original position
        print!("\x1B[{}D", input.len());
        io::stdout().flush()?;

        // Check if Enter was pressed, then return the input
        if input.ends_with('\n') {
            return Ok(input);
        }
        // Update the cursor position
        cursor_position = input.len();
        // Clear the input for the next iteration
        input.clear();
    }
}
fn compare_and_colorize(input: &str, target: &str) -> String {
    let mut colored_output = String::new();

    for (i, (typed_char, target_char)) in input.chars().zip(target.chars()).enumerate() {
        if typed_char == target_char {
            // If the typed character matches the target character, color it green
            colored_output.push_str("\x1B[32m"); // Green color
        } else {
            // If the typed character doesn't match, color it red
            colored_output.push_str("\x1B[31m"); // Red color
        }

        // Append the character to the colored output
        colored_output.push(typed_char);

        // Reset color back to default
        colored_output.push_str("\x1B[0m");

        // If the target string is longer than the input, pad with spaces
        if i + 1 >= input.len() {
            if i + 1 < target.len() {
                colored_output.push(target.chars().nth(i + 1).unwrap());
            }
        }
    }

    // If the input string is longer than the target, trim the excess characters
    if input.len() > target.len() {
        colored_output.truncate(target.len());
    }

    colored_output
}

pub fn listen_for_alphabets() {
    let initial_text = generate_random_sentence(30).to_string();
    let red_color = "\x1b[31m";
    let reset_color = "\x1b[0m";

    let stdin = io::stdin();
    let mut stdout = io::stdout().into_raw_mode().expect("Failed to set raw mode");

    let mut i = 0;
    let mut colored_text = String::new(); // String to hold colored text
    let middle_index = initial_text.len() / 2; // Middle index of the string

    // Generate the initially colored text
    for (index, char) in initial_text.chars().enumerate() {
        if index == middle_index {
            colored_text.push_str(red_color);
            colored_text.push(char);
            colored_text.push_str(reset_color);
        } else {
            colored_text.push(char);
        }
    }

    // Print the initial colored text
    println!("{}", colored_text);

    for key in stdin.keys() {
        match key {
            Ok(key_event) => {
                match key_event {
                    termion::event::Key::Char(c) if c.is_alphabetic() => {
                        // Clear the screen
                        print!("{}{}{}", termion::clear::All, termion::cursor::Goto(1, 1), termion::cursor::Hide);
                        io::stdout().flush().unwrap();

                        if c == initial_text.chars().nth(i).unwrap() {
                            // If the pressed character matches, regenerate the colored text with the new match
                            colored_text.clear();
                            for (index, char) in initial_text.chars().enumerate() {
                                if index == i {
                                    colored_text.push_str(red_color);
                                    colored_text.push(char);
                                    colored_text.push_str(reset_color);
                                } else {
                                    colored_text.push(char);
                                }
                            }
                            println!("{}", colored_text);
                        } else {
                            // If the pressed character does not match, print the text as it is
                            println!("{}", initial_text);
                        }
                    }
                    // If 'q' is pressed, quit the loop
                    termion::event::Key::Char('q') => {
                        println!("Quitting...");
                        break;
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
        i += 1;
        if i == initial_text.len() {
            break;
        }
        stdout.flush().expect("Failed to flush stdout");
    }
}