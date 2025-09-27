// src/audio.rs

use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::mpsc::{self, Sender};
use std::thread;
use once_cell::sync::OnceCell;

/// Loads audio samples from `src/assets/audio/<switch>/press/*.mp3` at startup and
/// plays them non-blocking when requested. This uses an in-memory cache of the
/// files so playback is fast.
pub struct Player {
    tx: Sender<(String, String)>, // (switch, keyname)
    // store available switches for UI
    switches: Vec<String>,
}

impl Player {
    pub fn new() -> Self {
        // Channel carries (switch_name, keyname)
        let (tx, rx) = mpsc::channel::<(String, String)>();

        // Build asset base path at compile-time relative to manifest dir
        let mut assets_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        assets_root.push("src/assets/audio");

        // Preload files into a map: switch -> keyname -> bytes
        let mut store: HashMap<String, HashMap<String, Vec<u8>>> = HashMap::new();
        let mut switches: Vec<String> = Vec::new();

        if let Ok(entries) = fs::read_dir(&assets_root) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Some(switch_name_os) = entry.file_name().to_str() {
                        let switch_name = switch_name_os.to_string();
                        let mut map: HashMap<String, Vec<u8>> = HashMap::new();
                        let press_dir = entry.path().join("press");
                        if press_dir.exists() && press_dir.is_dir() {
                            if let Ok(files) = fs::read_dir(&press_dir) {
                                for f in files.flatten() {
                                    if f.path().is_file() {
                                        if let Some(fname) = f.file_name().to_str() {
                                            if let Ok(bytes) = fs::read(f.path()) {
                                                // normalize keyname from filename (strip extension)
                                                if let Some(stem) = PathBuf::from(fname).file_stem().and_then(|s| s.to_str()) {
                                                    map.insert(stem.to_string(), bytes);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        if !map.is_empty() {
                            store.insert(switch_name.clone(), map);
                            switches.push(switch_name);
                        }
                    }
                }
            }
        }

        // Spawn thread that plays decoded bytes on request
        thread::spawn(move || {
            // try to get default output stream; if fails, we simply drop playback
            if let Ok((_stream, stream_handle)) = rodio::OutputStream::try_default() {
                // keep store local to thread for playback
                let store = store;
                while let Ok((switch, key)) = rx.recv() {
                    if let Some(map) = store.get(&switch) {
                        // exact match (SPACE, ENTER, BACKSPACE)
                        if let Some(bytes) = map.get(&key) {
                            let cursor = Cursor::new(bytes.clone());
                            if let Ok(dec) = rodio::Decoder::new_mp3(cursor) {
                                if let Ok(sink) = rodio::Sink::try_new(&stream_handle) {
                                    sink.append(dec);
                                    sink.detach();
                                }
                            }
                            continue;
                        }

                        // fallback to GENERIC variants: prefer GENERIC, then GENERIC_R0..R4 randomly
                        if let Some(bytes) = map.get("GENERIC") {
                            let cursor = Cursor::new(bytes.clone());
                            if let Ok(dec) = rodio::Decoder::new_mp3(cursor) {
                                if let Ok(sink) = rodio::Sink::try_new(&stream_handle) {
                                    sink.append(dec);
                                    sink.detach();
                                }
                            }
                            continue;
                        }

                        // try GENERIC_R0..R4
                        for i in 0..5 {
                            let keyn = format!("GENERIC_R{}", i);
                            if let Some(bytes) = map.get(&keyn) {
                                let cursor = Cursor::new(bytes.clone());
                                if let Ok(dec) = rodio::Decoder::new_mp3(cursor) {
                                    if let Ok(sink) = rodio::Sink::try_new(&stream_handle) {
                                        sink.append(dec);
                                        sink.detach();
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            }
        });

        Player { tx, switches }
    }

    pub fn available_switches(&self) -> Vec<String> {
        self.switches.clone()
    }

    pub fn play_for(&self, switch: &str, key: &str) {
        let _ = self.tx.send((switch.to_string(), key.to_string()));
    }
}

static GLOBAL_PLAYER: OnceCell<Player> = OnceCell::new();

/// Initialize the global audio player (preload assets). Safe to call multiple times.
pub fn init() {
    let _ = GLOBAL_PLAYER.get_or_init(Player::new);
}

/// Returns list of available switch names (directories found in assets)
pub fn list_switches() -> Vec<String> {
    if let Some(p) = GLOBAL_PLAYER.get() {
        p.available_switches()
    } else {
        // If not initialized, try to init and return
        init();
        GLOBAL_PLAYER.get().map(|p| p.available_switches()).unwrap_or_default()
    }
}

/// Play the sample for a given switch and key label (e.g., "SPACE", "BACKSPACE", "GENERIC")
pub fn play_for(switch: &str, key: &str) {
    if let Some(p) = GLOBAL_PLAYER.get() {
        p.play_for(switch, key);
    }
}
