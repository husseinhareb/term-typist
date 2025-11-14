# term-typist

A feature-rich terminal-based typing speed test application written in Rust. Practice your typing skills with customizable tests, audio feedback, and detailed statistics tracking.

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)

## Features

- **Multiple Test Modes**
  - **Time Mode**: Type for a set duration (15s, 30s, 60s, 100s)
  - **Words Mode**: Complete a specific number of words (10, 25, 50, 100)
  - **Zen Mode**: Practice without constraints

- **Multi-Language Support**
  - Practice typing in multiple languages: English, German, Spanish, French, Japanese
  - 172,000+ real words from built-in dictionaries
  - No external word files needed - all embedded in the application
  - Switch languages instantly from settings

- **Keyboard Support**
  - Multiple keyboard layouts: QWERTY, AZERTY, DVORAK, QWERTZ
  - Visual on-screen keyboard with real-time key highlighting
  - Caps Lock detection and warning modal

- **Audio Feedback**
  - Mechanical keyboard sound samples
  - Multiple switch types: MX Black, MX Blue, MX Brown, Topre, Holy Pandas, and more
  - Toggle audio on/off

- **Statistics & Tracking**
  - Real-time WPM (Words Per Minute) and accuracy metrics
  - Historical test results stored in SQLite database
  - Profile view with recent test history
  - Leaderboard showing top performances
  - Graphical WPM progression during tests

- **Customization**
  - Fully themeable interface (see [THEMING.md](THEMING.md))
  - Multiple preset themes included
  - Toggle individual UI panels (mode, speed, timer, keyboard, etc.)
  - Persistent settings across sessions

- **Terminal UI**
  - Clean, distraction-free interface
  - Smooth rendering with crossterm backend
  - Color-coded typing feedback (correct/incorrect characters)

## Screenshots

<p align="center">
  <img width="1920" alt="Main typing interface" src="https://github.com/user-attachments/assets/896593f8-001a-484e-a14e-ca8063b12da2" />
  <em>Main Page</em>
</p>
<p align="center">
  <img width="1920" alt="Theme showcase 5" src="https://github.com/user-attachments/assets/3507cacb-5ed2-4fb5-a0e7-404b5dfc84e0" />
  <em>Menu Page</em>
</p>
<p align="center">
  <img width="1920" alt="Theme showcase 4" src="https://github.com/user-attachments/assets/6612a33f-c102-4838-b7ec-16ede011d8ef" />
  <em>Settings Page</em>
</p>
<p align="center">
  <img width="1920" alt="Profile and test history" src="https://github.com/user-attachments/assets/d56aae96-5f1a-413d-bc84-a5ad4d5ecf2d" />
  <em>Test results</em>
</p>

<p align="center">
  <img width="1920" alt="Theme showcase 1" src="https://github.com/user-attachments/assets/3dfd2d22-9dea-4809-b65d-1c3ede6ce2cc" />
  <em>Leaderboard</em>
</p>

<p align="center">
  <img width="1920" alt="Theme showcase 2" src="https://github.com/user-attachments/assets/34a07084-4322-4aa7-9eee-f470e0b65247" />
  <em>Profile Page</em>
</p>

<p align="center">
  <img width="1920" alt="Theme showcase 3" src="https://github.com/user-attachments/assets/1e44eabf-bffd-4c9a-bcc3-a3661f24888f" />
  <em>Help Menu</em>
</p>
<p align="center">
  <img width="1920" alt="Test results with WPM graph" src="https://github.com/user-attachments/assets/70ce0b84-a505-4b10-81d3-7b2f2f9d0c03" />
  <em>Other themes</em>
</p>
<p align="center">
  <img width="1920" alt="Settings view" src="https://github.com/user-attachments/assets/7fd64cba-c8b0-4f2f-8443-fdaff72e8204" />
</p>

## Installation

### Prerequisites

- Rust 1.70 or higher
- `pkg-config` (or `pkgconfig`)
- ALSA development libraries (for audio support on Linux)

### Installing Dependencies

#### Automatic Installation (Recommended)

The provided script automatically detects your Linux distribution and installs the appropriate dependencies:

```bash
sudo ./scripts/install-deps.sh
```

This script supports the following distributions:
- **Debian-based**: Ubuntu, Debian, Pop!_OS, Linux Mint, Elementary OS
- **Red Hat-based**: Fedora, CentOS, RHEL, Rocky Linux, AlmaLinux
- **Arch-based**: Arch Linux, Manjaro, EndeavourOS, Garuda Linux
- **SUSE-based**: openSUSE Leap/Tumbleweed, SUSE Linux Enterprise
- **Others**: Alpine Linux, Gentoo, Void Linux

#### Manual Installation by Distribution

If the automatic script doesn't work or you prefer manual installation:

**Debian/Ubuntu/Pop!_OS/Mint:**
```bash
sudo apt-get update
sudo apt-get install -y pkg-config libasound2-dev
```

**Fedora/CentOS/RHEL/Rocky Linux:**
```bash
# Fedora 22+ (using dnf)
sudo dnf install -y pkgconfig alsa-lib-devel

# Older versions (using yum)
sudo yum install -y pkgconfig alsa-lib-devel
```

**Arch Linux/Manjaro/EndeavourOS:**
```bash
sudo pacman -S pkg-config alsa-lib
```

**openSUSE:**
```bash
sudo zypper install -y pkg-config alsa-devel
```

**Alpine Linux:**
```bash
sudo apk add pkgconfig alsa-lib-dev
```

**Gentoo:**
```bash
sudo emerge dev-util/pkgconfig media-libs/alsa-lib
```

**Void Linux:**
```bash
sudo xbps-install -y pkg-config alsa-lib-devel
```

### Building from Source

1. Clone the repository:
```bash
git clone https://github.com/husseinhareb/term-typist.git
cd term-typist
```

2. Build and install:
```bash
make build
sudo make install
```

This will:
- Build the release binary with embedded word dictionaries
- Install the binary to `/usr/bin/`

### Running

Simply run:
```bash
term-typist
```

## Usage

### Getting Started

1. Launch `term-typist`
2. Select your test mode and parameters using arrow keys
3. Press `Enter` to start typing
4. Type the displayed text as accurately as possible
5. Press `Esc` to restart or view results

### Keyboard Shortcuts

#### Global
- `Ctrl+C` - Quit application
- `Esc` - Restart test / Return to main view
- `Tab` - Toggle menu (from main view)
- `F1` or `m` - Open menu

#### Main View (Before Starting)
- `Arrow keys` / `hjkl` - Navigate mode and options
- `Enter` - Start typing test
- `p` - Open profile (test history)
- `l` - Open leaderboard
- `s` - Open settings

#### During Test (Insert Mode)
- `Backspace` - Delete previous character
- Type normally to complete the test

#### Menu
- `Arrow keys` / `hjkl` - Navigate menu items
- `Enter` - Activate selected menu item
- `Esc` - Close menu

#### Settings
- `Arrow keys` / `hjkl` - Navigate settings
- `Left/Right` - Change selected option
- `l` - Cycle keyboard layouts (QWERTY, AZERTY, DVORAK, QWERTZ)
- `Left/Right on Language` - Cycle test languages (English, German, Spanish, French, Japanese)
- `t` - Cycle themes
- `k` - Cycle keyboard switch sounds
- `a` - Toggle audio on/off
- `Esc` - Return to main view (applies language change)

#### Profile & Leaderboard
- `Arrow keys` / `hjkl` - Navigate test history
- `Page Up/Down` - Fast navigation
- `Home/End` - Jump to start/end
- `Esc` - Return to main view

#### Panel Toggles (Shift + Number)
- `Shift+1` - Toggle mode display
- `Shift+2` - Toggle value display
- `Shift+3` - Toggle state display
- `Shift+4` - Toggle speed display
- `Shift+5` - Toggle timer display
- `Shift+6` - Toggle text display
- `Shift+7` - Toggle keyboard display

## Configuration

### Configuration Files

term-typist stores its configuration and data in:
- **Config**: `~/.config/term-typist/`
- **Data**: `~/.local/share/term-typist/`

### Theme Customization

On first run, term-typist creates a default theme configuration at `~/.config/term-typist/theme.toml`. 

You can customize colors, UI elements, and visual appearance. See [THEMING.md](THEMING.md) for a comprehensive guide.

#### Built-in Themes

term-typist includes several predefined themes that you can switch between in the settings:

- **Catppuccin Mocha** - Soothing pastel theme with mauve and teal accents
- **Gruvbox Dark** - Retro groove with warm, earthy tones
- **Dracula** - Dark theme with vibrant purples and cyans
- **Solarized Dark** - Precision colors for optimal readability in low light
- **Solarized Light** - Precision colors for optimal readability in bright light
- **Nord** - Arctic, north-bluish color palette
- **One Dark** - Atom's iconic One Dark theme
- **Monokai** - Sublime Text's classic color scheme
- **Terminal** - Plain black and white terminal colors for a minimalist look

To change themes, press `s` to open settings and use `t` to cycle through themes, or use the Left/Right arrow keys on the Theme setting row.

#### Custom Themes

Example custom themes are available in the `examples/` directory:
- `theme-dark.toml` - Dark theme with muted colors
- `theme-colorful.toml` - Vibrant theme with RGB colors

### Audio Samples

Keyboard switch sound samples are embedded in the application. Available switches include:
- MX Black, MX Blue, MX Brown
- Topre, Holy Pandas
- Buckling Spring
- Alps variants (Blue, Cream, etc.)
- And more...

## Development

### Project Structure

```
term-typist/
├── src/
│   ├── main.rs           # Entry point
│   ├── lib.rs            # Main application loop
│   ├── app/              # Application state & input handling
│   ├── ui/               # UI rendering modules
│   ├── assets/audio/     # Embedded keyboard sound samples
│   ├── audio.rs          # Audio playback system
│   ├── caps.rs           # Caps Lock detection
│   ├── config.rs         # Configuration management
│   ├── db.rs             # SQLite database operations
│   ├── generator.rs      # Text generation for tests (using random-word crate)
│   ├── graph.rs          # WPM graph rendering
│   ├── theme.rs          # Theme system
│   ├── themes_presets.rs # Built-in themes
│   └── wpm.rs            # WPM calculation
├── examples/             # Example theme configurations
└── Cargo.toml            # Rust dependencies
```

### Building for Development

```bash
cargo build
cargo run
```

### Running Tests

```bash
cargo test
```

### Clean Build Artifacts

```bash
make clean
```

## Uninstall

To remove term-typist from your system:

```bash
sudo make uninstall
rm -rf ~/.config/term-typist
rm -rf ~/.local/share/term-typist
```

## Dependencies

- [crossterm](https://github.com/crossterm-rs/crossterm) - Cross-platform terminal manipulation
- [tui](https://github.com/fdehau/tui-rs) - Terminal UI framework
- [rusqlite](https://github.com/rusqlite/rusqlite) - SQLite database bindings
- [rodio](https://github.com/RustAudio/rodio) - Audio playback
- [random_word](https://github.com/ctsrc/random_word) - Multi-language word generation
- [serde](https://github.com/serde-rs/serde) - Serialization framework
- [toml](https://github.com/toml-rs/toml) - TOML parser
- [chrono](https://github.com/chronotope/chrono) - Date and time handling

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the [LICENCE](LICENCE) file for details.

## Author

**Hussein Hareb**

## Acknowledgments

- Inspired by [Monkeytype](https://monkeytype.com/) - A minimalistic typing test
- Keyboard sound samples inspired by [kbs.im](https://kbs.im/) - Keyboard sounds simulator
- Mechanical keyboard sound samples from [kbs.im](https://kbs.im/)
---