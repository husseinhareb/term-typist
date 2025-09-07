// src/generator.rs
use std::fs::File;
use std::io::{self, BufRead};
use std::path::PathBuf;
use rand::seq::SliceRandom;

/// Read the bundled word list from the XDG data directory.
/// Expected path: $XDG_DATA_HOME/term-typist/words/words.txt
fn read_words() -> io::Result<Vec<String>> {
    let mut file_path = match dirs::data_dir() {
        Some(p) => p,
        None => return Err(io::Error::new(io::ErrorKind::NotFound, "data directory not found")),
    };

    file_path.push("term-typist");
    file_path.push("words");
    file_path.push("words.txt");

    let file = File::open(&file_path)?;
    let reader = io::BufReader::new(file);
    let mut words = Vec::new();

    for line in reader.lines() {
        if let Ok(word) = line {
            words.push(word);
        }
    }

    Ok(words)
}

pub fn generate_random_sentence(num_words: usize) -> String {
    let words = match read_words() {
        Ok(words) => words,
        Err(err) => {
            eprintln!(
                "Error reading words: {}. Expected: $XDG_DATA_HOME/term-typist/words/words.txt",
                err
            );
            return String::new();
        }
    };

    let mut rng = rand::thread_rng();
    let mut sentence = String::new();

    for _ in 0..num_words {
        if let Some(random_word) = words.choose(&mut rng) {
            sentence.push_str(random_word);
            sentence.push(' ');
        }
    }

    sentence.trim().to_string()
}
