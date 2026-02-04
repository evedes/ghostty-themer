use crate::color::Color;
use crate::pipeline::assign::AnsiPalette;

const RESET: &str = "\x1b[0m";

const SLOT_NAMES: [&str; 8] = ["Blk", "Red", "Grn", "Yel", "Blu", "Mag", "Cyn", "Wht"];

/// Set 24-bit foreground color.
fn fg(c: &Color) -> String {
    format!("\x1b[38;2;{};{};{}m", c.r, c.g, c.b)
}

/// Set 24-bit background color.
fn bg_esc(c: &Color) -> String {
    format!("\x1b[48;2;{};{};{}m", c.r, c.g, c.b)
}

/// Choose black or white text for maximum contrast against `bg`.
fn contrast_fg(bg: &Color) -> &'static str {
    if bg.relative_luminance() > 0.4 {
        "\x1b[38;2;0;0;0m"
    } else {
        "\x1b[38;2;255;255;255m"
    }
}

/// Print a colored terminal preview of the generated palette.
pub fn print_preview(palette: &AnsiPalette) {
    println!();

    // Row 1: normal colors (slots 0-7)
    print!("  ");
    for (i, name) in SLOT_NAMES.iter().enumerate() {
        let c = &palette.slots[i];
        print!("{}{} {name:^5} {RESET}", bg_esc(c), contrast_fg(c));
    }
    println!();

    // Row 2: bright colors (slots 8-15)
    print!("  ");
    for (i, name) in SLOT_NAMES.iter().enumerate() {
        let c = &palette.slots[i + 8];
        print!("{}{} {name:^5} {RESET}", bg_esc(c), contrast_fg(c));
    }
    println!();
    println!();

    // Sample foreground on background text
    let background = &palette.background;
    let foreground = &palette.foreground;
    println!(
        "  {}{}  The quick brown fox jumps over the lazy dog  {RESET}",
        bg_esc(background),
        fg(foreground)
    );
    println!();

    // Show accent colors on background
    print!("  {}  ", bg_esc(background));
    for (name, slot_color) in SLOT_NAMES[1..=6].iter().zip(&palette.slots[1..=6]) {
        print!("{}{name}{RESET}{} ", fg(slot_color), bg_esc(background));
    }
    println!("{RESET}");
    println!();

    // Contrast ratios
    let fg_ratio = Color::contrast_ratio(foreground, background);
    let min_accent_ratio = (1..=6)
        .chain(9..=14)
        .map(|i| Color::contrast_ratio(&palette.slots[i], background))
        .fold(f32::MAX, f32::min);

    println!("  Foreground contrast: {fg_ratio:.1}:1");
    println!("  Dimmest accent:      {min_accent_ratio:.1}:1");
    println!();
}
