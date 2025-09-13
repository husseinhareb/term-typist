// src/app/config.rs
use std::fs::{ self, File };
use std::path::PathBuf;
use std::io::{ self, prelude::*, BufRead, Write, BufReader };

pub fn create_config() -> std::io::Result<()> {
    let config_dir = dirs::config_dir().expect("Unable to determine config directory");
    let folder_path = config_dir.join("term-typist");

    if !folder_exists(&folder_path) {
        fs::create_dir(&folder_path)?;
    }

    let file_path = folder_path.join("term-typist.conf");

    if !file_exists(&file_path) {
        fs::write(&file_path, "")?;
    }

    Ok(())
}

// Function to write number of words according to parameter into the config file
pub fn write_nb_of_words(nb_cmds: i32) -> io::Result<()> {
    let file_path = config_file()?;
    let mut file_content = String::new();

    if file_path.exists() {
        let mut file = File::open(&file_path)?;
        file.read_to_string(&mut file_content)?;
    }

    let mut updated_content = String::new();
    let mut nb_cmds_found = false;

    for line in file_content.lines() {
        if line.trim().starts_with("nb_of_words") {
            nb_cmds_found = true;
            updated_content.push_str(&format!("nb_of_words {}\n", nb_cmds));
        } else {
            updated_content.push_str(&line);
            updated_content.push('\n');
        }
    }

    if !nb_cmds_found {
        updated_content.push_str(&format!("nb_of_words {}\n", nb_cmds));
    }

    let mut file = File::create(&file_path)?;
    file.write_all(updated_content.as_bytes())?;

    Ok(())
}

/// Write the keyboard layout into the config file as: keyboard_layout <NAME>
pub fn write_keyboard_layout(layout: &str) -> io::Result<()> {
    let file_path = config_file()?;
    let mut file_content = String::new();

    if file_path.exists() {
        let mut file = File::open(&file_path)?;
        file.read_to_string(&mut file_content)?;
    }

    let mut updated_content = String::new();
    let mut found = false;

    for line in file_content.lines() {
        if line.trim().starts_with("keyboard_layout") {
            found = true;
            updated_content.push_str(&format!("keyboard_layout {}\n", layout));
        } else {
            updated_content.push_str(line);
            updated_content.push('\n');
        }
    }

    if !found {
        updated_content.push_str(&format!("keyboard_layout {}\n", layout));
    }

    // Ensure parent dir exists
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = File::create(&file_path)?;
    file.write_all(updated_content.as_bytes())?;
    Ok(())
}

/// Read keyboard_layout from config file. Returns Ok(name) if found.
pub fn read_keyboard_layout() -> io::Result<Option<String>> {
    let file_path = config_file()?;
    if !file_path.exists() {
        return Ok(None);
    }
    let file = File::open(&file_path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if line.trim().starts_with("keyboard_layout") {
            let name = line
                .split_whitespace()
                .skip(1)
                .next()
                .map(|s| s.to_string());
            return Ok(name);
        }
    }
    Ok(None)
}

// Function to read numberd of words from config file
pub fn read_nb_of_words() -> io::Result<i32> {
    let file_path = config_file()?;
    let file = File::open(&file_path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if line.trim().starts_with("nb_of_words") {
            let nb_cmds_str = line
                .split_whitespace()
                .skip(1)
                .next()
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "Invalid format for nb_of_words")
                })?;
            let nb_cmds = nb_cmds_str
                .parse::<i32>()
                .map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "Failed to parse nb_of_words")
                })?;
            return Ok(nb_cmds);
        }
    }

    // If nb_of_words variable is not found, return 30
    Ok(30)
}

/// Write the keyboard switch into the config file as: keyboard_switch <NAME>
pub fn write_keyboard_switch(switch: &str) -> io::Result<()> {
    let file_path = config_file()?;
    let mut file_content = String::new();

    if file_path.exists() {
        let mut file = File::open(&file_path)?;
        file.read_to_string(&mut file_content)?;
    }

    let mut updated_content = String::new();
    let mut found = false;

    for line in file_content.lines() {
        if line.trim().starts_with("keyboard_switch") {
            found = true;
            updated_content.push_str(&format!("keyboard_switch {}\n", switch));
        } else {
            updated_content.push_str(line);
            updated_content.push('\n');
        }
    }

    if !found {
        updated_content.push_str(&format!("keyboard_switch {}\n", switch));
    }

    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = File::create(&file_path)?;
    file.write_all(updated_content.as_bytes())?;
    Ok(())
}

/// Read keyboard_switch from config file. Returns Ok(name) if found.
pub fn read_keyboard_switch() -> io::Result<Option<String>> {
    let file_path = config_file()?;
    if !file_path.exists() {
        return Ok(None);
    }
    let file = File::open(&file_path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if line.trim().starts_with("keyboard_switch") {
            let name = line
                .split_whitespace()
                .skip(1)
                .next()
                .map(|s| s.to_string());
            return Ok(name);
        }
    }
    Ok(None)
}

/// Write audio enabled flag: audio_enabled 0|1
pub fn write_audio_enabled(enabled: bool) -> io::Result<()> {
    let file_path = config_file()?;
    let mut file_content = String::new();

    if file_path.exists() {
        let mut file = File::open(&file_path)?;
        file.read_to_string(&mut file_content)?;
    }

    let mut updated_content = String::new();
    let mut found = false;

    for line in file_content.lines() {
        if line.trim().starts_with("audio_enabled") {
            found = true;
            updated_content.push_str(&format!("audio_enabled {}\n", if enabled { 1 } else { 0 }));
        } else {
            updated_content.push_str(line);
            updated_content.push('\n');
        }
    }

    if !found {
        updated_content.push_str(&format!("audio_enabled {}\n", if enabled { 1 } else { 0 }));
    }

    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = File::create(&file_path)?;
    file.write_all(updated_content.as_bytes())?;
    Ok(())
}

/// Read audio_enabled from config. Returns Ok(Some(true/false)) if found.
pub fn read_audio_enabled() -> io::Result<Option<bool>> {
    let file_path = config_file()?;
    if !file_path.exists() {
        return Ok(None);
    }
    let file = File::open(&file_path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if line.trim().starts_with("audio_enabled") {
            let val = line
                .split_whitespace()
                .skip(1)
                .next()
                .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Invalid audio_enabled"))?;
            return Ok(Some(val != "0"));
        }
    }
    Ok(None)
}

// Function to get the path of the config file
fn config_file() -> Result<PathBuf, io::Error> {
    let config_dir = match dirs::config_dir() {
        Some(path) => path,
        None => {
            return Err(io::Error::new(io::ErrorKind::NotFound, "Config directory not found"));
        }
    };

    let file_path = config_dir.join("term-typist").join("term-typist.conf");
    Ok(file_path)
}

fn folder_exists(folder_path: &PathBuf) -> bool {
    if let Ok(metadata) = std::fs::metadata(folder_path) { metadata.is_dir() } else { false }
}

fn file_exists(file_path: &PathBuf) -> bool {
    if let Ok(metadata) = std::fs::metadata(file_path) { metadata.is_file() } else { false }
}
// src/app/input.rs
use crossterm::event::KeyCode;
use crate::app::state::{App, Mode, Status};

/// Handle navigation keys to switch tabs and adjust selected values.
pub fn handle_nav(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('1') => app.selected_tab = 0,
        KeyCode::Char('2') => app.selected_tab = 1,
        KeyCode::Char('3') => app.selected_tab = 2,
        KeyCode::Left if app.selected_value > 0 => app.selected_value -= 1,
        KeyCode::Right if app.selected_value + 1 < app.current_options().len() => {
            app.selected_value += 1;
        }
        _ => {}
    }
}

/// Map raw KeyCode into displayed keyboard labels (e.g., "Esc", "Backspace", "SPACE", or character).
pub fn map_keycode(code: &KeyCode) -> Option<String> {
    match code {
        KeyCode::Esc => Some("Esc".into()),
        KeyCode::Backspace => Some("Backspace".into()),
        KeyCode::Char(' ') => Some("Space".into()),
        KeyCode::Char(c) => Some(c.to_ascii_uppercase().to_string()),
        _ => None,
    }
}
