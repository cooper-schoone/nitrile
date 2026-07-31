use crate::commands::flags::Flag;
use color_eyre::Result;
use std::path::{Path, PathBuf};

pub struct EngineArgs<'a> {
    pub target: &'a Path,
    pub output: Option<&'a Path>,
    pub flags: Vec<Flag>,
}

/// Trait for LaTeX project compilation functionality.
pub trait LatexEngine {
    /// Compiles a given LaTeX project from a specified target .tex file.
    ///
    /// Returns the path to the compiled PDF. The location depends on `output` and
    /// on the engine's own defaults, so the engine is responsible for resolving it.
    fn compile(&self, args: EngineArgs) -> Result<PathBuf>;
}
