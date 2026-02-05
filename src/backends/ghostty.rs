use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::pipeline::assign::AnsiPalette;

use super::ThemeBackend;

/// Ghostty terminal theme backend.
pub struct GhosttyBackend;

impl ThemeBackend for GhosttyBackend {
    fn name(&self) -> &str {
        "Ghostty"
    }

    fn serialize(&self, palette: &AnsiPalette, _theme_name: &str) -> String {
        let p = palette;
        let mut out = String::new();

        out.push_str(&format!("background = {}\n", p.background.to_hex()));
        out.push_str(&format!("foreground = {}\n", p.foreground.to_hex()));
        out.push_str(&format!("cursor-color = {}\n", p.cursor_color.to_hex()));
        out.push_str(&format!("cursor-text = {}\n", p.cursor_text.to_hex()));
        out.push_str(&format!(
            "selection-background = {}\n",
            p.selection_bg.to_hex()
        ));
        out.push_str(&format!(
            "selection-foreground = {}\n",
            p.selection_fg.to_hex()
        ));

        for (i, color) in p.slots.iter().enumerate() {
            out.push_str(&format!("palette = {}={}\n", i, color.to_hex()));
        }

        out
    }

    fn install(&self, palette: &AnsiPalette, theme_name: &str) -> Result<PathBuf> {
        let dir = themes_dir()?;
        std::fs::create_dir_all(&dir)
            .with_context(|| format!("failed to create themes directory: {}", dir.display()))?;

        let path = dir.join(theme_name);
        self.write_to(palette, theme_name, &path)?;
        Ok(path)
    }

    fn write_to(&self, palette: &AnsiPalette, theme_name: &str, path: &Path) -> Result<()> {
        let content = self.serialize(palette, theme_name);
        std::fs::write(path, content)
            .with_context(|| format!("failed to write theme to {}", path.display()))?;
        Ok(())
    }
}

/// Resolve the Ghostty themes directory.
fn themes_dir() -> Result<PathBuf> {
    let config_home = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| "~".to_string());
            PathBuf::from(home).join(".config")
        });
    Ok(config_home.join("ghostty").join("themes"))
}

/// Resolve the full path where a theme with the given name would be installed.
pub fn theme_path(name: &str) -> Result<PathBuf> {
    Ok(themes_dir()?.join(name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::ThemeMode;
    use crate::color::Color;
    use crate::pipeline::assign::assign_slots;
    use crate::pipeline::extract::ExtractedColor;
    use palette::Oklch;

    fn make_extracted(l: f32, chroma: f32, hue: f32, weight: f32) -> ExtractedColor {
        ExtractedColor {
            color: Color::from_oklch(Oklch::new(l, chroma, hue)),
            weight,
        }
    }

    fn test_palette() -> AnsiPalette {
        let colors = vec![
            make_extracted(0.60, 0.20, 25.0, 0.12),
            make_extracted(0.60, 0.20, 145.0, 0.12),
            make_extracted(0.70, 0.20, 90.0, 0.12),
            make_extracted(0.55, 0.20, 260.0, 0.12),
            make_extracted(0.60, 0.20, 325.0, 0.12),
            make_extracted(0.65, 0.20, 195.0, 0.10),
            make_extracted(0.10, 0.01, 0.0, 0.15),
            make_extracted(0.95, 0.01, 0.0, 0.15),
        ];
        assign_slots(&colors, ThemeMode::Dark)
    }

    #[test]
    fn serialization_format_is_correct() {
        let backend = GhosttyBackend;
        let output = backend.serialize(&test_palette(), "test");
        let lines: Vec<&str> = output.lines().collect();

        assert_eq!(lines.len(), 22, "expected 22 lines, got {}", lines.len());

        assert!(lines[0].starts_with("background = #"));
        assert!(lines[1].starts_with("foreground = #"));
        assert!(lines[2].starts_with("cursor-color = #"));
        assert!(lines[3].starts_with("cursor-text = #"));
        assert!(lines[4].starts_with("selection-background = #"));
        assert!(lines[5].starts_with("selection-foreground = #"));

        for i in 0..16 {
            let line = lines[6 + i];
            let expected_prefix = format!("palette = {}=#", i);
            assert!(
                line.starts_with(&expected_prefix),
                "line {} should start with '{expected_prefix}', got '{line}'",
                6 + i
            );
        }
    }

    #[test]
    fn palette_lines_have_no_inner_space() {
        let backend = GhosttyBackend;
        let output = backend.serialize(&test_palette(), "test");

        for line in output.lines() {
            if line.starts_with("palette") {
                let after_eq = line.split(" = ").nth(1).unwrap();
                assert!(
                    after_eq.contains("=#"),
                    "palette line should have '=#' (no spaces): '{line}'"
                );
                assert!(
                    !after_eq.contains(" = "),
                    "palette value should not contain ' = ': '{line}'"
                );
            }
        }
    }

    #[test]
    fn hex_values_are_lowercase() {
        let backend = GhosttyBackend;
        let output = backend.serialize(&test_palette(), "test");

        for line in output.lines() {
            if let Some(hex_start) = line.find('#') {
                let hex = &line[hex_start..hex_start + 7];
                assert_eq!(
                    hex,
                    hex.to_lowercase(),
                    "hex values should be lowercase: '{line}'"
                );
            }
        }
    }

    #[test]
    fn all_hex_values_valid() {
        let backend = GhosttyBackend;
        let output = backend.serialize(&test_palette(), "test");

        for line in output.lines() {
            if let Some(hex_start) = line.find('#') {
                let hex = &line[hex_start..hex_start + 7];
                assert_eq!(hex.len(), 7);
                assert!(hex.starts_with('#'));
                assert!(
                    hex[1..].chars().all(|c| c.is_ascii_hexdigit()),
                    "invalid hex value in line: '{line}'"
                );
            }
        }
    }

    #[test]
    fn write_to_creates_file() {
        let backend = GhosttyBackend;
        let palette = test_palette();
        let dir = std::env::temp_dir().join("nuri-test-ghostty-backend");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test-theme");

        backend.write_to(&palette, "test-theme", &path).unwrap();

        let content = std::fs::read_to_string(&path).unwrap();
        assert_eq!(content, backend.serialize(&palette, "test-theme"));

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn install_creates_correct_path() {
        let temp_dir = std::env::temp_dir().join("nuri-test-ghostty-install");
        std::env::set_var("XDG_CONFIG_HOME", &temp_dir);

        let backend = GhosttyBackend;
        let palette = test_palette();
        let result = backend.install(&palette, "my-theme").unwrap();

        let expected_path = temp_dir.join("ghostty").join("themes").join("my-theme");
        assert_eq!(result, expected_path);
        assert!(expected_path.exists());

        let content = std::fs::read_to_string(&expected_path).unwrap();
        assert_eq!(content, backend.serialize(&palette, "my-theme"));

        std::fs::remove_dir_all(&temp_dir).unwrap();
        std::env::remove_var("XDG_CONFIG_HOME");
    }
}
