///! File system utilities, particularly functionality that requires
///! alternative implementations for alternative operating systems.
use eyre::Result;
use std::{fs, os, path::Path};

/// Set permissions on a file
pub fn set_perms<File: AsRef<Path>>(path: File, mode: u32) -> Result<()> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        use os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    }

    Ok(())
}

/// Create a symbolic (soft) link to a file
pub fn symlink_file<Original: AsRef<Path>, Link: AsRef<Path>>(
    original: Original,
    link: Link,
) -> Result<()> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    os::unix::fs::symlink(original, link)?;

    #[cfg(target_os = "windows")]
    os::windows::fs::symlink_file(original, link)?;

    Ok(())
}

/// Create a symbolic (soft) link to a directory
pub fn symlink_dir<Original: AsRef<Path>, Link: AsRef<Path>>(
    original: Original,
    link: Link,
) -> Result<()> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    os::unix::fs::symlink(original, link)?;

    #[cfg(target_os = "windows")]
    os::windows::fs::symlink_dir(original, link)?;

    Ok(())
}
