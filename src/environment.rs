use which::which;

/// Returns whether or not the specified executable is present on PATH.
pub fn is_on_path(exe: &str) -> bool {
    which(exe).is_ok()
}
