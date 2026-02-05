use std::path::{Path, PathBuf};

use anyhow::Result;

use crate::pipeline::assign::AnsiPalette;

use super::ThemeBackend;

/// Zellij terminal multiplexer theme backend (KDL format).
pub struct ZellijBackend;

impl ThemeBackend for ZellijBackend {
    fn name(&self) -> &str {
        "Zellij"
    }

    fn serialize(&self, _palette: &AnsiPalette, _theme_name: &str) -> String {
        todo!("Zellij backend serialization (ticket #21)")
    }

    fn install(&self, _palette: &AnsiPalette, _theme_name: &str) -> Result<PathBuf> {
        todo!("Zellij backend install (ticket #21)")
    }

    fn write_to(&self, _palette: &AnsiPalette, _theme_name: &str, _path: &Path) -> Result<()> {
        todo!("Zellij backend write_to (ticket #21)")
    }
}
