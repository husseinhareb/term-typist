use crate::theme::{Theme, ThemeColor};

/// Predefined theme presets users can pick from in settings.
/// Each function returns a complete Theme instance.
pub fn catppuccin_mocha() -> Theme {
    Theme {
        background: ThemeColor::Named("black".into()),
        foreground: ThemeColor::Rgb([205, 214, 244]),
        border: ThemeColor::Rgb([75, 84, 106]),
        title: ThemeColor::Rgb([198, 96, 251]), // mauve
        title_accent: ThemeColor::Rgb([139, 233, 253]), // teal

        text_untyped: ThemeColor::Rgb([205, 214, 244]),
        text_correct: ThemeColor::Rgb([165, 220, 134]),
        text_incorrect: ThemeColor::Rgb([255, 111, 105]),
        text_cursor_bg: ThemeColor::Rgb([250, 218, 88]),
        text_cursor_fg: ThemeColor::Rgb([20, 20, 24]),

        tab_active: ThemeColor::Rgb([139, 233, 253]),
        tab_inactive: ThemeColor::Rgb([120, 120, 140]),
        highlight: ThemeColor::Rgb([139, 233, 253]),
        stats_label: ThemeColor::Rgb([120, 120, 140]),
        stats_value: ThemeColor::Rgb([205, 214, 244]),

        key_normal_bg: ThemeColor::Rgb([30, 30, 36]),
        key_normal_fg: ThemeColor::Rgb([205, 214, 244]),
        key_pressed_bg: ThemeColor::Rgb([139, 233, 253]),
        key_pressed_fg: ThemeColor::Rgb([10, 10, 14]),
        key_border: ThemeColor::Rgb([75, 84, 106]),

        chart_line: ThemeColor::Rgb([139, 233, 253]),
        chart_axis: ThemeColor::Rgb([120, 120, 140]),
        chart_labels: ThemeColor::Rgb([150, 150, 170]),

        success: ThemeColor::Rgb([165, 220, 134]),
        warning: ThemeColor::Rgb([250, 218, 88]),
        error: ThemeColor::Rgb([255, 111, 105]),
        info: ThemeColor::Rgb([139, 233, 253]),
    }
}

pub fn gruvbox_dark() -> Theme {
    Theme {
        background: ThemeColor::Rgb([40, 40, 35]),
        foreground: ThemeColor::Rgb([235, 219, 178]),
        border: ThemeColor::Rgb([60, 60, 55]),
        title: ThemeColor::Rgb([255, 121, 198]), // pink
        title_accent: ThemeColor::Rgb([162, 190, 140]), // green

        text_untyped: ThemeColor::Rgb([235, 219, 178]),
        text_correct: ThemeColor::Rgb([166, 226, 46]),
        text_incorrect: ThemeColor::Rgb([251, 73, 52]),
        text_cursor_bg: ThemeColor::Rgb([250, 189, 47]),
        text_cursor_fg: ThemeColor::Rgb([40, 40, 35]),

        tab_active: ThemeColor::Rgb([250, 189, 47]),
        tab_inactive: ThemeColor::Rgb([150, 135, 120]),
        highlight: ThemeColor::Rgb([250, 189, 47]),
        stats_label: ThemeColor::Rgb([150, 135, 120]),
        stats_value: ThemeColor::Rgb([235, 219, 178]),

        key_normal_bg: ThemeColor::Rgb([55, 48, 48]),
        key_normal_fg: ThemeColor::Rgb([235, 219, 178]),
        key_pressed_bg: ThemeColor::Rgb([250, 189, 47]),
        key_pressed_fg: ThemeColor::Rgb([40, 40, 35]),
        key_border: ThemeColor::Rgb([60, 60, 55]),

        chart_line: ThemeColor::Rgb([166, 226, 46]),
        chart_axis: ThemeColor::Rgb([150, 135, 120]),
        chart_labels: ThemeColor::Rgb([180, 160, 140]),

        success: ThemeColor::Rgb([166, 226, 46]),
        warning: ThemeColor::Rgb([250, 189, 47]),
        error: ThemeColor::Rgb([251, 73, 52]),
        info: ThemeColor::Rgb([162, 190, 140]),
    }
}

/// Return a list of preset names available to the UI.
pub fn preset_names() -> Vec<&'static str> {
    vec!["Catppuccin Mocha", "Gruvbox Dark"]
}

/// Get a Theme by a preset name (case-insensitive-ish).
pub fn theme_by_name(name: &str) -> Option<Theme> {
    match name.to_lowercase().as_str() {
        "catppuccin mocha" | "catppuccin" | "catppuccin_mocha" => Some(catppuccin_mocha()),
        "gruvbox dark" | "gruvbox" | "gruvbox_dark" => Some(gruvbox_dark()),
        _ => None,
    }
}
