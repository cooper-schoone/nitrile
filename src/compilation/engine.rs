use color_eyre::Result;
use std::path::Path;

/// Trait for LaTeX project compilation functionality.
pub trait LatexEngine {
    /// Compiles a given LaTeX project from a specified target .tex file.
    fn compile(&self, target: &Path, output: Option<&Path>) -> Result<()>;
}
