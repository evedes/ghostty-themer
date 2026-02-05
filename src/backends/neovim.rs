use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::pipeline::assign::AnsiPalette;

use super::ThemeBackend;

/// Neovim colorscheme backend (Lua format).
pub struct NeovimBackend;

impl ThemeBackend for NeovimBackend {
    fn name(&self) -> &str {
        "Neovim"
    }

    fn serialize(&self, _palette: &AnsiPalette, _theme_name: &str) -> String {
        todo!("Neovim backend serialization (ticket #22)")
    }

    fn install(&self, _palette: &AnsiPalette, _theme_name: &str) -> Result<PathBuf> {
        todo!("Neovim backend install (ticket #22)")
    }

    fn write_to(&self, _palette: &AnsiPalette, _theme_name: &str, _path: &Path) -> Result<()> {
        todo!("Neovim backend write_to (ticket #22)")
    }
}
