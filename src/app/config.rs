// src/app/config.rs
use std::fs::{ self, File };
use std::path::PathBuf;
use std::io::{ self, prelude::*, BufRead, Write, BufReader };

// (Removed unused helper functions: create_config, write_nb_of_words)

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

/// Write the selected test mode and value to config as: test_mode <tab> test_value <value>
pub fn write_selected_mode_value(tab: usize, value: usize) -> io::Result<()> {
    let file_path = config_file()?;
    let mut file_content = String::new();

    if file_path.exists() {
        let mut file = File::open(&file_path)?;
        file.read_to_string(&mut file_content)?;
    }

    let mut updated_content = String::new();
    let mut found_mode = false;
    let mut found_value = false;

    for line in file_content.lines() {
        if line.trim().starts_with("test_mode") {
            found_mode = true;
            updated_content.push_str(&format!("test_mode {}\n", tab));
        } else if line.trim().starts_with("test_value") {
            found_value = true;
            updated_content.push_str(&format!("test_value {}\n", value));
        } else {
            updated_content.push_str(line);
            updated_content.push('\n');
        }
    }

    if !found_mode {
        updated_content.push_str(&format!("test_mode {}\n", tab));
    }
    if !found_value {
        updated_content.push_str(&format!("test_value {}\n", value));
    }

    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = File::create(&file_path)?;
    file.write_all(updated_content.as_bytes())?;
    Ok(())
}

/// Read persisted test_mode and test_value from config. Returns (tab, value) if found,
/// otherwise returns None.
pub fn read_selected_mode_value() -> io::Result<Option<(usize, usize)>> {
    let file_path = config_file()?;
    if !file_path.exists() {
        return Ok(None);
    }
    let file = File::open(&file_path)?;
    let reader = BufReader::new(file);

    let mut tab: Option<usize> = None;
    let mut value: Option<usize> = None;

    for line in reader.lines() {
        let line = line?;
        if line.trim().starts_with("test_mode") {
            if let Some(tok) = line.split_whitespace().skip(1).next() {
                if let Ok(n) = tok.parse::<usize>() { tab = Some(n); }
            }
        }
        if line.trim().starts_with("test_value") {
            if let Some(tok) = line.split_whitespace().skip(1).next() {
                if let Ok(n) = tok.parse::<usize>() { value = Some(n); }
            }
        }
    }

    if let (Some(t), Some(v)) = (tab, value) {
        Ok(Some((t, v)))
    } else {
        Ok(None)
    }
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

// Helper: path of the config file is provided by `config_file()` above; no
// additional existence helpers required.

