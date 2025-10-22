// src/theme.rs

use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf};
use tui::style::Color;

/// Complete theme configuration for the terminal typing test application.
/// All colors have sensible defaults and can be customized via config file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Theme {
    // General UI colors
    pub background: ThemeColor,
    pub foreground: ThemeColor,
    pub border: ThemeColor,
    pub title: ThemeColor,
    pub title_accent: ThemeColor,
    
    // Text area colors
    pub text_untyped: ThemeColor,
    pub text_correct: ThemeColor,
    /// Color used for characters that were typed incorrectly and later corrected
    pub text_corrected: ThemeColor,
    pub text_incorrect: ThemeColor,
    pub text_cursor_bg: ThemeColor,
    pub text_cursor_fg: ThemeColor,
    
    // UI element colors
    pub tab_active: ThemeColor,
    pub tab_inactive: ThemeColor,
    pub highlight: ThemeColor,
    pub stats_label: ThemeColor,
    pub stats_value: ThemeColor,
    
    // Keyboard colors
    pub key_normal_bg: ThemeColor,
    pub key_normal_fg: ThemeColor,
    pub key_pressed_bg: ThemeColor,
    pub key_pressed_fg: ThemeColor,
    pub key_border: ThemeColor,
    
    // Chart colors
    pub chart_line: ThemeColor,
    pub chart_axis: ThemeColor,
    pub chart_labels: ThemeColor,
    
    // Status colors
    pub success: ThemeColor,
    pub warning: ThemeColor,
    pub error: ThemeColor,
    pub info: ThemeColor,
}

/// Represents a color that can be serialized to/from TOML
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ThemeColor {
    /// Named color like "red", "blue", "yellow"
    Named(String),
    /// RGB color as [r, g, b] array
    Rgb([u8; 3]),
    /// Indexed color (0-255)
    Indexed(u8),
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            // General UI
            background: ThemeColor::Named("reset".to_string()),
            foreground: ThemeColor::Named("white".to_string()),
            border: ThemeColor::Named("white".to_string()),
            title: ThemeColor::Named("white".to_string()),
            title_accent: ThemeColor::Named("light_blue".to_string()),
            
            // Text area
            text_untyped: ThemeColor::Named("white".to_string()),
            // Correct text: green by default
            text_correct: ThemeColor::Named("green".to_string()),
            // Corrected (mistyped then fixed): orange for visibility
            text_corrected: ThemeColor::Rgb([255, 165, 0]),
            text_incorrect: ThemeColor::Named("red".to_string()),
            text_cursor_bg: ThemeColor::Named("yellow".to_string()),
            text_cursor_fg: ThemeColor::Named("black".to_string()),
            
            // UI elements
            tab_active: ThemeColor::Named("yellow".to_string()),
            tab_inactive: ThemeColor::Named("white".to_string()),
            highlight: ThemeColor::Named("yellow".to_string()),
            stats_label: ThemeColor::Named("gray".to_string()),
            stats_value: ThemeColor::Named("yellow".to_string()),
            
            // Keyboard
            key_normal_bg: ThemeColor::Named("reset".to_string()),
            key_normal_fg: ThemeColor::Named("white".to_string()),
            key_pressed_bg: ThemeColor::Named("yellow".to_string()),
            key_pressed_fg: ThemeColor::Named("black".to_string()),
            key_border: ThemeColor::Named("white".to_string()),
            
            // Chart
            chart_line: ThemeColor::Named("cyan".to_string()),
            chart_axis: ThemeColor::Named("white".to_string()),
            chart_labels: ThemeColor::Named("gray".to_string()),
            
            // Status
            success: ThemeColor::Named("green".to_string()),
            warning: ThemeColor::Named("yellow".to_string()),
            error: ThemeColor::Named("red".to_string()),
            info: ThemeColor::Named("light_blue".to_string()),
        }
    }
}

impl ThemeColor {
    /// Convert ThemeColor to tui::style::Color
    pub fn to_tui_color(&self) -> Color {
        match self {
            ThemeColor::Named(name) => match name.to_lowercase().as_str() {
                "reset" => Color::Reset,
                "black" => Color::Black,
                "red" => Color::Red,
                "green" => Color::Green,
                "yellow" => Color::Yellow,
                "blue" => Color::Blue,
                "magenta" => Color::Magenta,
                "cyan" => Color::Cyan,
                "gray" | "grey" => Color::Gray,
                "dark_gray" | "dark_grey" => Color::DarkGray,
                "light_red" => Color::LightRed,
                "light_green" => Color::LightGreen,
                "light_yellow" => Color::LightYellow,
                "light_blue" => Color::LightBlue,
                "light_magenta" => Color::LightMagenta,
                "light_cyan" => Color::LightCyan,
                "white" => Color::White,
                _ => Color::White, // fallback
            },
            ThemeColor::Rgb([r, g, b]) => Color::Rgb(*r, *g, *b),
            ThemeColor::Indexed(index) => Color::Indexed(*index),
        }
    }
}

impl Theme {
    /// Load theme from config file, falling back to defaults if file doesn't exist or has errors
    pub fn load() -> Self {
        match Self::load_from_config() {
            Ok(theme) => theme,
            Err(_) => {
                // If config loading fails, create default config file and return defaults
                let _ = Self::create_default_config();
                Self::default()
            }
        }
    }
    
    /// Load theme from ~/.config/term-typist/theme.toml
    fn load_from_config() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_file_path()?;
        let content = fs::read_to_string(&config_path)?;
        let theme: Theme = toml::from_str(&content)?;
        Ok(theme)
    }
    
    /// Create default theme config file if it doesn't exist
    pub fn create_default_config() -> Result<(), Box<dyn std::error::Error>> {
        let config_dir = Self::config_dir_path()?;
        let config_file = config_dir.join("theme.toml");
        
        // Create directory if it doesn't exist
        fs::create_dir_all(&config_dir)?;
        
        // Only create file if it doesn't exist
        if !config_file.exists() {
            let default_theme = Self::default();
            let toml_content = toml::to_string_pretty(&default_theme)?;
            
            // Add comments to make the config file more user-friendly
            let commented_content = Self::add_config_comments(&toml_content);
            fs::write(&config_file, commented_content)?;
        }
        
        Ok(())
    }
    
    /// Get the config directory path (~/.config/term-typist)
    fn config_dir_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let mut config_dir = dirs::config_dir()
            .ok_or("Could not determine config directory")?;
        config_dir.push("term-typist");
        Ok(config_dir)
    }
    
    /// Get the config file path (~/.config/term-typist/theme.toml)
    fn config_file_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let config_dir = Self::config_dir_path()?;
        Ok(config_dir.join("theme.toml"))
    }
    
    /// Add helpful comments to the generated TOML config
    fn add_config_comments(toml_content: &str) -> String {
        format!(r#"# term-typist theme configuration
# 
# This file controls the colors used throughout the term-typist application.
# Colors can be specified in three ways:
#   1. Named colors: "red", "blue", "green", "yellow", "cyan", "magenta", 
#      "white", "black", "gray", "light_red", "light_blue", etc.
#   2. RGB colors: [255, 128, 0] for orange
#   3. Indexed colors: 42 (for terminal color index 42)
#
# After making changes, restart term-typist to see the new theme.

{}"#, toml_content)
    }

    /// Save the current theme to the config file (~/.config/term-typist/theme.toml)
    pub fn save_to_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_dir = Self::config_dir_path()?;
        fs::create_dir_all(&config_dir)?;
        let config_file = config_dir.join("theme.toml");
        let toml_content = toml::to_string_pretty(self)?;
        let commented = Self::add_config_comments(&toml_content);
        fs::write(config_file, commented)?;
        Ok(())
    }
}