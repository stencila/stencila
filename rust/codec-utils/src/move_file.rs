use std::{fs, io::ErrorKind, path::Path};

use eyre::{Context, Result};

/// Cross-device safe file move operation.
///
/// This function provides a robust alternative to `std::fs::rename` that handles the common
/// "Invalid cross-device link (os error 18)" error encountered in containerized environments.
///
/// ## Problem Background
///
/// The standard `std::fs::rename` function performs an atomic move operation by updating
/// filesystem metadata, but this only works when both source and destination are on the
/// same filesystem. In containers, temporary directories (like `/tmp`) and working
/// directories often exist on different filesystems or mount points, causing rename
/// operations to fail with `ErrorKind::CrossDevice`.
///
/// ## Solution
///
/// This function attempts the fast rename operation first, and if it encounters a cross-device
/// error, falls back to a copy-and-delete strategy. While not atomic like rename, this
/// approach works across filesystem boundaries.
///
/// ## Behavior
///
/// 1. **Fast path**: Attempts `std::fs::rename` for same-filesystem moves
/// 2. **Fallback**: On cross-device error:
///    - Creates destination directory structure if needed
///    - Copies the file to the destination
///    - Preserves file permissions (best effort)
///    - Removes the source file
/// 3. **Error handling**: Provides context for each operation that might fail
///
/// ## Limitations
///
/// - The fallback copy-and-delete operation is **not atomic**
/// - Only works with regular files (not directories)
/// - Permission preservation is best-effort and may not work on all platforms
/// - Temporary disk space is required for the copy operation
pub fn move_file<S: AsRef<Path>, D: AsRef<Path>>(src: S, dst: D) -> Result<()> {
    let src = src.as_ref();
    let dst = dst.as_ref();

    match fs::rename(src, dst) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::CrossesDevices => {
            // Cross-device link detected, fall back to copy and delete

            // Ensure destination directory exists
            if let Some(parent) = dst.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!(
                        "Failed to create destination directory: {}",
                        parent.display()
                    )
                })?;
            }

            // Copy the file
            fs::copy(src, dst).with_context(|| {
                format!("Failed to copy from {} to {}", src.display(), dst.display())
            })?;

            // Preserve file permissions (best effort,  ignore failures)
            if let Ok(metadata) = fs::metadata(src) {
                fs::set_permissions(dst, metadata.permissions()).ok();
            }

            // Remove the source file
            fs::remove_file(src)
                .with_context(|| format!("Failed to remove source file: {}", src.display()))?;

            Ok(())
        }
        Err(e) => {
            Err(e).with_context(|| format!("Failed to move {} to {}", src.display(), dst.display()))
        }
    }
}
