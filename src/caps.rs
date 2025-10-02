// src/caps.rs
// Cross-platform helper to detect Caps Lock state.

use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use once_cell::sync::Lazy;

#[cfg(target_os = "windows")]
pub fn is_caps_lock_on() -> bool {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::GetKeyState;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::VK_CAPITAL;
    unsafe { (GetKeyState(VK_CAPITAL) & 0x0001) != 0 }
}

/// Return true if we have at least one system-backed method to detect CapsLock.
pub fn detection_available() -> bool {
    #[cfg(target_os = "windows")]
    {
        return true;
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;

        // sway (Wayland / sway): ensure outputs include xkb_active_leds
        if which::which("swaymsg").is_ok() {
            if let Ok(out) = Command::new("swaymsg").arg("-t").arg("get_inputs").output() {
                if out.status.success() {
                    let s = String::from_utf8_lossy(&out.stdout).to_lowercase();
                    // Presence of the LEDs array is a good signal that we can read it
                    if s.contains("xkb_active_leds") {
                        return true;
                    }
                }
            }
        }

        // X11 via xset: ensure it reports a "Caps Lock:" field
        if which::which("xset").is_ok() {
            if let Ok(out) = Command::new("xset").arg("q").output() {
                if out.status.success() {
                    let s = String::from_utf8_lossy(&out.stdout).to_lowercase();
                    if s.contains("caps lock:") {
                        return true;
                    }
                }
            }
        }

        // Sysfs LEDs — only consider entries that actually look like capslock LEDs
        if let Ok(entries) = std::fs::read_dir("/sys/class/leds") {
            for e in entries.flatten() {
                if let Ok(name) = e.file_name().into_string() {
                    let lname = name.to_lowercase();
                    if lname.contains("capslock") || lname.ends_with("::capslock") {
                        return true;
                    }
                }
            }
        }

        return false;
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        false
    }
}

// Cached value and polling support so the UI can query CapsLock quickly
static CACHED_CAPS_ON: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));
static POLLER_STARTED: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));

/// Start a background thread that polls the system Caps Lock state at the
/// given interval (milliseconds) and updates an in-process cache. Calling
/// this multiple times is safe; only the first call will spawn the thread.
pub fn start_polling(interval_ms: u64) {
    if !detection_available() {
        return;
    }
    // Ensure only one poller starts
    if POLLER_STARTED.swap(true, Ordering::SeqCst) {
        return;
    }

    // Prime the cache immediately
    let initial = is_caps_lock_on();
    CACHED_CAPS_ON.store(initial, Ordering::SeqCst);

    let interval = Duration::from_millis(interval_ms);
    std::thread::spawn(move || {
        loop {
            let v = is_caps_lock_on();
            CACHED_CAPS_ON.store(v, Ordering::SeqCst);
            std::thread::sleep(interval);
        }
    });
}

/// Read the last cached Caps Lock state set by the poller. If the poller
/// hasn't been started this returns the result of an immediate detection.
pub fn cached_is_caps_lock_on() -> bool {
    if POLLER_STARTED.load(Ordering::SeqCst) {
        CACHED_CAPS_ON.load(Ordering::SeqCst)
    } else {
        is_caps_lock_on()
    }
}

#[cfg(target_os = "linux")]
pub fn is_caps_lock_on() -> bool {
    use std::process::Command;

    // 1) sway: look for "Caps Lock" inside xkb_active_leds arrays
    if let Ok(out) = Command::new("swaymsg").arg("-t").arg("get_inputs").output() {
        if out.status.success() {
            if let Ok(s) = String::from_utf8(out.stdout) {
                let low = s.to_lowercase();
                let key = "\"xkb_active_leds\"";
                let mut from = 0;
                while let Some(idx) = low[from..].find(key) {
                    let start = from + idx;
                    if let Some(open_rel) = low[start..].find('[') {
                        let open = start + open_rel;
                        if let Some(close_rel) = low[open..].find(']') {
                            let close = open + close_rel;
                            // Slice of the array contents only
                            let arr = &low[open..close];
                            if arr.contains("caps lock") {
                                return true;
                            }
                            from = close;
                            continue;
                        }
                    }
                    break;
                }
            }
        }
    }

    // 2) X11/xset: parse the exact "Caps Lock: on|off" field
    if let Ok(out) = Command::new("xset").arg("q").output() {
        if out.status.success() {
            if let Ok(s) = String::from_utf8(out.stdout) {
                for line in s.lines() {
                    let l = line.to_lowercase();
                    if let Some(pos) = l.find("caps lock:") {
                        let after = &l[pos + "caps lock:".len()..];
                        if let Some(val) = after.split_whitespace().next() {
                            return val == "on";
                        }
                    }
                }
                // Optional: LED mask fallback (bit 1 commonly indicates CapsLock)
                if let Some(idx) = s.to_lowercase().find("led mask:") {
                    let tail = &s[idx..].to_lowercase();
                    if let Some(colon) = tail.find(':') {
                        let after = tail[colon + 1..].trim();
                        if let Some(tok) = after.split_whitespace().next() {
                            let mask = if let Some(hex) = tok.strip_prefix("0x") {
                                u64::from_str_radix(hex, 16).unwrap_or(0)
                            } else {
                                tok.parse::<u64>().unwrap_or(0)
                            };
                            if (mask & 0x02) != 0 {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }

    // 3) Sysfs LEDs: check only LEDs that look like capslock LEDs
    if let Ok(entries) = std::fs::read_dir("/sys/class/leds") {
        for e in entries.flatten() {
            if let Ok(name) = e.file_name().into_string() {
                let lname = name.to_lowercase();
                if lname.contains("capslock") || lname.ends_with("::capslock") {
                    let path = format!("/sys/class/leds/{}/brightness", name);
                    if let Ok(s) = std::fs::read_to_string(path) {
                        if s.trim() == "1" || s.trim() == "255" {
                            return true;
                        }
                    }
                }
            }
        }
    }

    false
}

#[cfg(target_os = "macos")]
pub fn is_caps_lock_on() -> bool {
    // macOS doesn't provide a simple command-line check; rely on heuristics.
    false
}

#[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
pub fn is_caps_lock_on() -> bool {
    false
}
