use crate::cli::ThemeMode;
use crate::color::Color;
use crate::pipeline::extract::ExtractedColor;

/// The full ANSI palette plus special Ghostty theme colors.
#[derive(Debug, Clone)]
pub struct AnsiPalette {
    /// ANSI colors 0-15.
    pub slots: [Color; 16],
    pub background: Color,
    pub foreground: Color,
    pub cursor_color: Color,
    pub cursor_text: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
}

/// Map extracted colors to the 16 ANSI palette slots plus special colors.
pub fn assign_slots(_colors: &[ExtractedColor], _mode: ThemeMode) -> AnsiPalette {
    todo!("Ticket 6: hue-based slot assignment")
}
