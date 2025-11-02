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
    text_corrected: ThemeColor::Rgb([255, 165, 0]),
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
    text_corrected: ThemeColor::Rgb([255, 165, 0]),
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
pub fn dracula() -> Theme {
    Theme {
        background: ThemeColor::Rgb([40, 42, 54]),
        foreground: ThemeColor::Rgb([248, 248, 242]),
        border: ThemeColor::Rgb([68, 71, 90]),
        title: ThemeColor::Rgb([189, 147, 249]),
        title_accent: ThemeColor::Rgb([139, 233, 253]),

        text_untyped: ThemeColor::Rgb([248, 248, 242]),
    text_correct: ThemeColor::Rgb([80, 250, 123]),
    text_corrected: ThemeColor::Rgb([255, 165, 0]),
        text_incorrect: ThemeColor::Rgb([255, 85, 85]),
        text_cursor_bg: ThemeColor::Rgb([255, 184, 108]),
        text_cursor_fg: ThemeColor::Rgb([40, 42, 54]),

        tab_active: ThemeColor::Rgb([189, 147, 249]),
        tab_inactive: ThemeColor::Rgb([120, 120, 140]),
        highlight: ThemeColor::Rgb([189, 147, 249]),
        stats_label: ThemeColor::Rgb([120, 120, 140]),
        stats_value: ThemeColor::Rgb([248, 248, 242]),

        key_normal_bg: ThemeColor::Rgb([48, 50, 62]),
        key_normal_fg: ThemeColor::Rgb([248, 248, 242]),
        key_pressed_bg: ThemeColor::Rgb([189, 147, 249]),
        key_pressed_fg: ThemeColor::Rgb([30, 30, 36]),
    key_border: ThemeColor::Rgb([48, 51, 70]),

        chart_line: ThemeColor::Rgb([80, 250, 123]),
        chart_axis: ThemeColor::Rgb([120, 120, 140]),
        chart_labels: ThemeColor::Rgb([180, 180, 200]),

        success: ThemeColor::Rgb([80, 250, 123]),
        warning: ThemeColor::Rgb([255, 184, 108]),
        error: ThemeColor::Rgb([255, 85, 85]),
        info: ThemeColor::Rgb([139, 233, 253]),
    }
}

pub fn solarized_dark() -> Theme {
    Theme {
        background: ThemeColor::Rgb([0, 43, 54]),
        foreground: ThemeColor::Rgb([131, 148, 150]),
        border: ThemeColor::Rgb([7, 54, 66]),
        title: ThemeColor::Rgb([38, 139, 210]),
        title_accent: ThemeColor::Rgb([42, 161, 152]),

        text_untyped: ThemeColor::Rgb([131, 148, 150]),
    text_correct: ThemeColor::Rgb([133, 153, 0]),
    text_corrected: ThemeColor::Rgb([255, 165, 0]),
        text_incorrect: ThemeColor::Rgb([220, 50, 47]),
        text_cursor_bg: ThemeColor::Rgb([181, 137, 0]),
        text_cursor_fg: ThemeColor::Rgb([0, 43, 54]),

        tab_active: ThemeColor::Rgb([181, 137, 0]),
        tab_inactive: ThemeColor::Rgb([88, 110, 117]),
        highlight: ThemeColor::Rgb([181, 137, 0]),
        stats_label: ThemeColor::Rgb([88, 110, 117]),
        stats_value: ThemeColor::Rgb([131, 148, 150]),

        key_normal_bg: ThemeColor::Rgb([7, 54, 66]),
        key_normal_fg: ThemeColor::Rgb([131, 148, 150]),
        key_pressed_bg: ThemeColor::Rgb([181, 137, 0]),
        key_pressed_fg: ThemeColor::Rgb([0, 43, 54]),
    key_border: ThemeColor::Rgb([10, 48, 60]),

        chart_line: ThemeColor::Rgb([38, 139, 210]),
        chart_axis: ThemeColor::Rgb([88, 110, 117]),
        chart_labels: ThemeColor::Rgb([101, 123, 131]),

        success: ThemeColor::Rgb([133, 153, 0]),
        warning: ThemeColor::Rgb([181, 137, 0]),
        error: ThemeColor::Rgb([220, 50, 47]),
        info: ThemeColor::Rgb([42, 161, 152]),
    }
}

pub fn solarized_light() -> Theme {
    Theme {
        background: ThemeColor::Rgb([253, 246, 227]),
        foreground: ThemeColor::Rgb([101, 123, 131]),
        border: ThemeColor::Rgb([238, 232, 213]),
        title: ThemeColor::Rgb([38, 139, 210]),
        title_accent: ThemeColor::Rgb([42, 161, 152]),

        text_untyped: ThemeColor::Rgb([101, 123, 131]),
    text_correct: ThemeColor::Rgb([133, 153, 0]),
    text_corrected: ThemeColor::Rgb([255, 165, 0]),
        text_incorrect: ThemeColor::Rgb([220, 50, 47]),
        text_cursor_bg: ThemeColor::Rgb([181, 137, 0]),
        text_cursor_fg: ThemeColor::Rgb([253, 246, 227]),

        tab_active: ThemeColor::Rgb([38, 139, 210]),
        tab_inactive: ThemeColor::Rgb([136, 123, 110]),
        highlight: ThemeColor::Rgb([38, 139, 210]),
        stats_label: ThemeColor::Rgb([136, 123, 110]),
        stats_value: ThemeColor::Rgb([101, 123, 131]),

        key_normal_bg: ThemeColor::Rgb([248, 241, 222]),
        key_normal_fg: ThemeColor::Rgb([101, 123, 131]),
        key_pressed_bg: ThemeColor::Rgb([38, 139, 210]),
        key_pressed_fg: ThemeColor::Rgb([253, 246, 227]),
        key_border: ThemeColor::Rgb([238, 232, 213]),

        chart_line: ThemeColor::Rgb([38, 139, 210]),
        chart_axis: ThemeColor::Rgb([136, 123, 110]),
        chart_labels: ThemeColor::Rgb([120, 100, 90]),

        success: ThemeColor::Rgb([133, 153, 0]),
        warning: ThemeColor::Rgb([181, 137, 0]),
        error: ThemeColor::Rgb([220, 50, 47]),
        info: ThemeColor::Rgb([42, 161, 152]),
    }
}

pub fn nord() -> Theme {
    Theme {
        background: ThemeColor::Rgb([46, 52, 64]),
        foreground: ThemeColor::Rgb([216, 222, 233]),
        border: ThemeColor::Rgb([59, 66, 82]),
        title: ThemeColor::Rgb([143, 188, 187]),
        title_accent: ThemeColor::Rgb([129, 161, 193]),

        text_untyped: ThemeColor::Rgb([216, 222, 233]),
    text_correct: ThemeColor::Rgb([163, 190, 140]),
    text_corrected: ThemeColor::Rgb([255, 165, 0]),
        text_incorrect: ThemeColor::Rgb([224, 108, 117]),
        text_cursor_bg: ThemeColor::Rgb([229, 192, 123]),
        text_cursor_fg: ThemeColor::Rgb([46, 52, 64]),

        tab_active: ThemeColor::Rgb([129, 161, 193]),
        tab_inactive: ThemeColor::Rgb([116, 125, 140]),
        highlight: ThemeColor::Rgb([129, 161, 193]),
        stats_label: ThemeColor::Rgb([116, 125, 140]),
        stats_value: ThemeColor::Rgb([216, 222, 233]),

        key_normal_bg: ThemeColor::Rgb([59, 66, 82]),
        key_normal_fg: ThemeColor::Rgb([216, 222, 233]),
        key_pressed_bg: ThemeColor::Rgb([129, 161, 193]),
        key_pressed_fg: ThemeColor::Rgb([30, 32, 38]),
    key_border: ThemeColor::Rgb([49, 56, 72]),

        chart_line: ThemeColor::Rgb([143, 188, 187]),
        chart_axis: ThemeColor::Rgb([116, 125, 140]),
        chart_labels: ThemeColor::Rgb([140, 150, 160]),

        success: ThemeColor::Rgb([163, 190, 140]),
        warning: ThemeColor::Rgb([229, 192, 123]),
        error: ThemeColor::Rgb([224, 108, 117]),
        info: ThemeColor::Rgb([143, 188, 187]),
    }
}

pub fn one_dark() -> Theme {
    Theme {
        background: ThemeColor::Rgb([40, 44, 52]),
        foreground: ThemeColor::Rgb([171, 178, 191]),
        border: ThemeColor::Rgb([60, 64, 72]),
        title: ThemeColor::Rgb([97, 175, 239]),
        title_accent: ThemeColor::Rgb([229, 192, 123]),

        text_untyped: ThemeColor::Rgb([171, 178, 191]),
    text_correct: ThemeColor::Rgb([152, 195, 121]),
    text_corrected: ThemeColor::Rgb([255, 165, 0]),
        text_incorrect: ThemeColor::Rgb([224, 108, 117]),
        text_cursor_bg: ThemeColor::Rgb([229, 192, 123]),
        text_cursor_fg: ThemeColor::Rgb([40, 44, 52]),

        tab_active: ThemeColor::Rgb([97, 175, 239]),
        tab_inactive: ThemeColor::Rgb([120, 124, 132]),
        highlight: ThemeColor::Rgb([97, 175, 239]),
        stats_label: ThemeColor::Rgb([120, 124, 132]),
        stats_value: ThemeColor::Rgb([171, 178, 191]),

        key_normal_bg: ThemeColor::Rgb([60, 64, 72]),
        key_normal_fg: ThemeColor::Rgb([171, 178, 191]),
        key_pressed_bg: ThemeColor::Rgb([97, 175, 239]),
        key_pressed_fg: ThemeColor::Rgb([30, 32, 38]),
    key_border: ThemeColor::Rgb([50, 54, 62]),

        chart_line: ThemeColor::Rgb([152, 195, 121]),
        chart_axis: ThemeColor::Rgb([120, 124, 132]),
        chart_labels: ThemeColor::Rgb([150, 150, 160]),

        success: ThemeColor::Rgb([152, 195, 121]),
        warning: ThemeColor::Rgb([229, 192, 123]),
        error: ThemeColor::Rgb([224, 108, 117]),
        info: ThemeColor::Rgb([97, 175, 239]),
    }
}

pub fn monokai() -> Theme {
    Theme {
        background: ThemeColor::Rgb([39, 40, 34]),
        foreground: ThemeColor::Rgb([248, 248, 242]),
        border: ThemeColor::Rgb([60, 60, 50]),
        title: ThemeColor::Rgb([249, 38, 114]),
        title_accent: ThemeColor::Rgb([166, 226, 46]),

        text_untyped: ThemeColor::Rgb([248, 248, 242]),
    text_correct: ThemeColor::Rgb([166, 226, 46]),
    text_corrected: ThemeColor::Rgb([255, 165, 0]),
        text_incorrect: ThemeColor::Rgb([249, 38, 114]),
        text_cursor_bg: ThemeColor::Rgb([253, 151, 31]),
        text_cursor_fg: ThemeColor::Rgb([39, 40, 34]),

        tab_active: ThemeColor::Rgb([249, 38, 114]),
        tab_inactive: ThemeColor::Rgb([120, 120, 100]),
        highlight: ThemeColor::Rgb([166, 226, 46]),
        stats_label: ThemeColor::Rgb([120, 120, 100]),
        stats_value: ThemeColor::Rgb([248, 248, 242]),

        key_normal_bg: ThemeColor::Rgb([60, 60, 50]),
        key_normal_fg: ThemeColor::Rgb([248, 248, 242]),
        key_pressed_bg: ThemeColor::Rgb([249, 38, 114]),
        key_pressed_fg: ThemeColor::Rgb([30, 30, 24]),
    key_border: ThemeColor::Rgb([100, 100, 90]),

        chart_line: ThemeColor::Rgb([166, 226, 46]),
        chart_axis: ThemeColor::Rgb([120, 120, 100]),
        chart_labels: ThemeColor::Rgb([180, 180, 160]),

        success: ThemeColor::Rgb([166, 226, 46]),
        warning: ThemeColor::Rgb([253, 151, 31]),
        error: ThemeColor::Rgb([249, 38, 114]),
        info: ThemeColor::Rgb([166, 226, 46]),
    }
}

pub fn terminal() -> Theme {
    Theme {
        background: ThemeColor::Named("black".into()),
        foreground: ThemeColor::Named("white".into()),
        border: ThemeColor::Named("white".into()),
        title: ThemeColor::Named("white".into()),
        title_accent: ThemeColor::Named("white".into()),

        text_untyped: ThemeColor::Named("dark_gray".into()),
    text_correct: ThemeColor::Named("white".into()),
    text_corrected: ThemeColor::Named("light_yellow".into()),
        text_incorrect: ThemeColor::Named("white".into()),
        text_cursor_bg: ThemeColor::Named("white".into()),
        text_cursor_fg: ThemeColor::Named("black".into()),

        tab_active: ThemeColor::Named("white".into()),
        tab_inactive: ThemeColor::Named("dark_gray".into()),
        highlight: ThemeColor::Named("white".into()),
        stats_label: ThemeColor::Named("gray".into()),
        stats_value: ThemeColor::Named("white".into()),

        key_normal_bg: ThemeColor::Named("black".into()),
        key_normal_fg: ThemeColor::Named("white".into()),
        key_pressed_bg: ThemeColor::Named("white".into()),
        key_pressed_fg: ThemeColor::Named("black".into()),
    key_border: ThemeColor::Named("white".into()),

        chart_line: ThemeColor::Named("white".into()),
        chart_axis: ThemeColor::Named("gray".into()),
        chart_labels: ThemeColor::Named("gray".into()),

        success: ThemeColor::Named("white".into()),
        warning: ThemeColor::Named("white".into()),
        error: ThemeColor::Named("white".into()),
        info: ThemeColor::Named("white".into()),
    }
}

/// Return a list of preset names available to the UI.
pub fn preset_names() -> Vec<&'static str> {
    vec![
        "Catppuccin Mocha",
        "Gruvbox Dark",
        "Dracula",
        "Solarized Dark",
        "Solarized Light",
        "Nord",
        "One Dark",
        "Monokai",
        "Terminal",
    ]
}

/// Get a Theme by a preset name (case-insensitive-ish).
pub fn theme_by_name(name: &str) -> Option<Theme> {
    match name.to_lowercase().as_str() {
        "catppuccin mocha" | "catppuccin" | "catppuccin_mocha" => Some(catppuccin_mocha()),
        "gruvbox dark" | "gruvbox" | "gruvbox_dark" => Some(gruvbox_dark()),
        "dracula" => Some(dracula()),
        "solarized dark" | "solarized_dark" => Some(solarized_dark()),
        "solarized light" | "solarized_light" => Some(solarized_light()),
        "nord" => Some(nord()),
        "one dark" | "one_dark" => Some(one_dark()),
        "monokai" => Some(monokai()),
        "terminal" => Some(terminal()),
        _ => None,
    }
}
