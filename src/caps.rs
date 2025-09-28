// src/caps.rs
// Cross-platform helper to detect Caps Lock state.

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
        // check for swaymsg, xset or /sys/class/leds
        if which::which("swaymsg").is_ok() { return true; }
        if which::which("xset").is_ok() { return true; }
        if std::path::Path::new("/sys/class/leds").exists() { return true; }
        return false;
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        false
    }
}

#[cfg(target_os = "linux")]
pub fn is_caps_lock_on() -> bool {
    // 1) Try `swaymsg` (Wayland - sway) to read input states
    if let Ok(out) = std::process::Command::new("swaymsg").arg("-t").arg("get_inputs").output() {
        if out.status.success() {
            if let Ok(s) = String::from_utf8(out.stdout) {
                let lower = s.to_lowercase();
                if lower.contains("caps_lock") && lower.contains("true") {
                    return true;
                }
            }
        }
    }

    // 2) Try X11 via `xset q` (works on many X11 setups)
    if let Ok(out) = std::process::Command::new("xset").arg("q").output() {
        if out.status.success() {
            if let Ok(s) = String::from_utf8(out.stdout) {
                let lower = s.to_lowercase();
                if lower.contains("caps lock") && lower.contains("on") {
                    return true;
                }
                // Fallback: parse LED mask containing 2 (second bit) which often indicates CapsLock
                if let Some(idx) = lower.find("led mask") {
                    let tail = &lower[idx..];
                    if let Some(colon) = tail.find(':') {
                        let after = &tail[colon+1..];
                        if let Some(num_str) = after.split_whitespace().next() {
                            // parse hex or decimal
                            let num_str = num_str.trim();
                            if num_str.starts_with("0x") {
                                if let Ok(mask) = u64::from_str_radix(num_str.trim_start_matches("0x"), 16) {
                                    return (mask & 0x02) != 0;
                                }
                            } else if let Ok(mask) = num_str.parse::<u64>() {
                                return (mask & 0x02) != 0;
                            }
                        }
                    }
                }
            }
        }
    }

    // 3) Try sysfs LED entries (/sys/class/leds/*caps*/brightness)
    if let Ok(entries) = std::fs::read_dir("/sys/class/leds") {
        for e in entries.flatten() {
            if let Ok(name) = e.file_name().into_string() {
                let lname = name.to_lowercase();
                if lname.contains("caps") {
                    let path = format!("/sys/class/leds/{}/brightness", name);
                    if let Ok(s) = std::fs::read_to_string(path) {
                        if let Ok(val) = s.trim().parse::<u32>() {
                            if val > 0 {
                                return true;
                            }
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
