use std::fs;

use nitrile::sys::file::copy_dir_recursive;

use color_eyre::Result;
use color_eyre::eyre::ensure;

#[test]
fn test_copies_top_level_files_and_nested_directories() -> Result<()> {
    let src = tempfile::tempdir()?;
    let dst = tempfile::tempdir()?;

    fs::write(src.path().join("top.txt"), "top")?;
    fs::create_dir(src.path().join("nested"))?;
    fs::write(src.path().join("nested").join("inner.txt"), "inner")?;

    copy_dir_recursive(src.path(), dst.path())?;

    ensure!(
        fs::read_to_string(dst.path().join("top.txt"))? == "top",
        "top-level file was not copied"
    );
    ensure!(
        fs::read_to_string(dst.path().join("nested").join("inner.txt"))? == "inner",
        "nested file was not copied"
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn test_skips_symlinked_files_and_directories() -> Result<()> {
    use std::os::unix::fs::symlink;

    let src = tempfile::tempdir()?;
    let dst = tempfile::tempdir()?;

    fs::write(src.path().join("real.txt"), "real")?;
    fs::create_dir(src.path().join("real-dir"))?;
    fs::write(src.path().join("real-dir").join("inner.txt"), "inner")?;

    symlink(src.path().join("real.txt"), src.path().join("link.txt"))?;
    symlink(src.path().join("real-dir"), src.path().join("link-dir"))?;

    copy_dir_recursive(src.path(), dst.path())?;

    ensure!(
        dst.path().join("real.txt").is_file(),
        "regular file should still be copied"
    );
    ensure!(
        dst.path().join("real-dir").join("inner.txt").is_file(),
        "regular directory contents should still be copied"
    );
    ensure!(
        !dst.path().join("link.txt").exists(),
        "symlinked file should be skipped"
    );
    ensure!(
        !dst.path().join("link-dir").exists(),
        "symlinked directory should be skipped"
    );
    Ok(())
}
