use std::env;
use std::fs::File;
use std::io::{ self, BufRead };
use std::path::PathBuf;
use rand::seq::SliceRandom;

fn read_words() -> io::Result<Vec<String>> {
    let mut file_path = PathBuf::new();

    if let Some(home_dir) = env::var_os("HOME") {
        file_path.push(home_dir);
        file_path.push(".local/share/term-typist/words/words.txt");
    } else {
        return Err(io::Error::new(io::ErrorKind::NotFound, "HOME environment variable not found"));
    }

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
            eprintln!("Error reading words: {}", err);
            return String::new();
        }
    };

    let mut rng = rand::thread_rng();
    let mut sentence = String::new();

    for _ in 0..num_words {
        let random_word = words.choose(&mut rng).unwrap();
        sentence.push_str(random_word);
        sentence.push(' ');
    }

    sentence.trim().to_string()
}
