use std::io::{self, Write};

pub fn print_and_input_on_top(initial_text: String) -> io::Result<String> {
    // Print the initial string
    print!("{}", initial_text);
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