///! File system utilities, particularly functionality that requires
///! alternative implementations for alternative operating systems.
use eyre::{eyre, Result};
use std::{fs, io, os, path::Path};

/// Set permissions on a file
#[allow(unused_variables)]
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

/// Clear a directory
pub fn clear_dir_all(dir: impl AsRef<Path>) -> Result<()> {
    fs::remove_dir_all(&dir)?;
    fs::create_dir_all(&dir)?;
    Ok(())
}

/// Recursively copy a directory to another
pub fn copy_dir_all(src: impl AsRef<Path>, dest: impl AsRef<Path>) -> Result<()> {
    fs::create_dir_all(&dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            copy_dir_all(entry.path(), dest.as_ref().join(entry.file_name()))?;
        } else if let Err(error) = fs::copy(entry.path(), dest.as_ref().join(entry.file_name())) {
            // Ignore "the source path is neither a regular file nor a symlink to a regular file" errors
            if !matches!(error.kind(), io::ErrorKind::InvalidInput) {
                return Err(eyre!(error));
            }
        }
    }
    Ok(())
}
