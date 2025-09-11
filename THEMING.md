# Theming Guide for term-typist

term-typist supports full theming through a configuration file located at `~/.config/term-typist/theme.toml`.

## How It Works

When you run term-typist for the first time, it automatically creates a default theme configuration file at `~/.config/term-typist/theme.toml`. You can edit this file to customize the appearance of the application.

## Color Specification

Colors can be specified in three ways:

### 1. Named Colors
Use standard terminal color names:
- Basic: `"black"`, `"red"`, `"green"`, `"yellow"`, `"blue"`, `"magenta"`, `"cyan"`, `"white"`
- Light variants: `"light_red"`, `"light_green"`, `"light_yellow"`, `"light_blue"`, `"light_magenta"`, `"light_cyan"`
- Special: `"gray"`, `"dark_gray"`, `"reset"`

### 2. RGB Colors
Use RGB arrays for precise color control:
```toml
text_correct = [50, 255, 50]    # Bright green
text_incorrect = [255, 100, 100] # Bright red
title_accent = [255, 165, 0]     # Orange
```

### 3. Indexed Colors
Use terminal color indices (0-255):
```toml
background = 235   # Dark gray background
foreground = 250   # Light gray text
```

## Customizable Elements

### General UI
- `background` - Main background color
- `foreground` - Default text color
- `border` - Border color for UI panels
- `title` - Title text color
- `title_accent` - Accent color for title numbers (¹, ², ³, etc.)

### Text Area
- `text_untyped` - Color for characters not yet typed
- `text_correct` - Color for correctly typed characters
- `text_incorrect` - Color for incorrectly typed characters
- `text_cursor_bg` - Background color for the typing cursor
- `text_cursor_fg` - Foreground color for the typing cursor

### UI Elements
- `tab_active` - Color for the active tab
- `tab_inactive` - Color for inactive tabs
- `highlight` - Color for highlighted/selected items
- `stats_label` - Color for statistic labels (WPM, ACC, etc.)
- `stats_value` - Color for statistic values

### Keyboard
- `key_normal_bg` - Background color for unpressed keys
- `key_normal_fg` - Text color for unpressed keys
- `key_pressed_bg` - Background color for pressed keys
- `key_pressed_fg` - Text color for pressed keys
- `key_border` - Border color for all keys

### Charts
- `chart_line` - Color for chart lines
- `chart_axis` - Color for chart axes
- `chart_labels` - Color for chart labels

### Status Colors
- `success` - Color for success messages
- `warning` - Color for warning messages
- `error` - Color for error messages
- `info` - Color for informational messages

## Example Themes

Check the `examples/` directory for example theme configurations:
- `theme-dark.toml` - A dark theme with muted colors
- `theme-colorful.toml` - A vibrant theme using RGB colors

## Applying Changes

After modifying your theme configuration:
1. Save the `~/.config/term-typist/theme.toml` file
2. Restart term-typist
3. Your new theme will be applied automatically

## Troubleshooting

If your theme file has errors:
- term-typist will fall back to the default theme
- Check the console for any error messages
- Ensure your TOML syntax is correct
- Verify color names or RGB values are valid

## Tips

- Use `"reset"` for transparent/default terminal background
- RGB colors provide the most precise control but may not work in all terminals
- Named colors are more portable across different terminal emulators
- Test your theme in different lighting conditions for optimal readability