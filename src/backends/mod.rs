pub mod ghostty;
pub mod neovim;
pub mod zellij;

use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::pipeline::assign::AnsiPalette;

/// A theme output backend that serializes an `AnsiPalette` into a target format.
pub trait ThemeBackend {
    /// Human-readable name shown in CLI help and TUI (e.g., "Ghostty", "Zellij").
    fn name(&self) -> &str;

    /// Serialize the palette into the target format.
    fn serialize(&self, palette: &AnsiPalette, theme_name: &str) -> String;

    /// Install the theme to the target's standard config directory.
    /// Returns the path where the theme was written.
    fn install(&self, palette: &AnsiPalette, theme_name: &str) -> Result<PathBuf>;

    /// Write the theme to an arbitrary path.
    fn write_to(&self, palette: &AnsiPalette, theme_name: &str, path: &Path) -> Result<()>;

    /// File extension for this backend (e.g., ".kdl"), or empty string for none.
    fn extension(&self) -> &str;
}

/// Supported output targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Target {
    Ghostty,
    Zellij,
    Neovim,
}

/// Return the backend for a given target.
pub fn get_backend(target: Target) -> Box<dyn ThemeBackend> {
    match target {
        Target::Ghostty => Box::new(ghostty::GhosttyBackend),
        Target::Zellij => Box::new(zellij::ZellijBackend),
        Target::Neovim => Box::new(neovim::NeovimBackend),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_backend_returns_correct_name() {
        assert_eq!(get_backend(Target::Ghostty).name(), "Ghostty");
        assert_eq!(get_backend(Target::Zellij).name(), "Zellij");
        assert_eq!(get_backend(Target::Neovim).name(), "Neovim");
    }
}
