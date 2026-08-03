use crate::commands::flags::Flag;
use color_eyre::Result;
use std::path::PathBuf;

pub struct EngineArgs {
    pub target: PathBuf,
    pub output: PathBuf,
    pub flags: Vec<Flag>,
    pub verbose: bool,
}

/// Trait for LaTeX project compilation functionality.
pub trait LatexEngine {
    /// Compiles a given LaTeX project from a specified target .tex file.
    ///
    /// Returns the path to the compiled PDF.
    fn compile(&self, args: EngineArgs) -> Result<PathBuf>;
}
