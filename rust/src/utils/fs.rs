use std::path::Path;

use eyre::Result;

/// Create a symbolic (soft) link to a file
pub fn symlink_file<Original: AsRef<Path>, Link: AsRef<Path>>(
    original: Original,
    link: Link,
) -> Result<()> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    std::os::unix::fs::symlink(original, link)?;

    #[cfg(target_os = "windows")]
    std::os::windows::fs::symlink_file(original, link)?;

    Ok(())
}

/// Create a symbolic (soft) link to a directory
pub fn symlink_dir<Original: AsRef<Path>, Link: AsRef<Path>>(
    original: Original,
    link: Link,
) -> Result<()> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    std::os::unix::fs::symlink(original, link)?;

    #[cfg(target_os = "windows")]
    std::os::windows::fs::symlink_dir(original, link)?;

    Ok(())
}
