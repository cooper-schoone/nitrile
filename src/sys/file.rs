use std::fs;
use std::path::Path;

use color_eyre::Result;

/// Recursively copy the contents of a directory to another directory.
///
/// Symlinks are skipped rather than dereferenced, so the destination never
/// follows links out of the source tree.
///
/// # Errors
///
/// Returns an error if:
///   - The destination directory could not be initialized
///   - The source directory could not be read
///   - Any file in the source directory could not be processed
pub fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        let dest_path = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}
