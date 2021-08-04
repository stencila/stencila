///! File system utilities, particularly functionality that requires
///! alternative implementations for alternative operating systems.
use eyre::Result;
use std::{
    fs, os,
    path::{Path, PathBuf},
};

/// Merge paths
///
/// Differs from `path.join()` in that it joins the parent of `path1`, not `path1` itself.
pub fn merge_paths<P1: AsRef<Path>, P2: AsRef<Path>>(path1: P1, path2: P2) -> PathBuf {
    let path1 = path1.as_ref();
    let path2 = path2.as_ref();
    if let Some(parent) = path1.parent() {
        parent.join(path2)
    } else {
        path1.join(path2)
    }
}

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
