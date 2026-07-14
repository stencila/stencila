use std::path::Path;

use eyre::{OptionExt, Result};
use tempfile::NamedTempFile;

/// Create a temporary file suitable for atomically replacing a destination.
///
/// The file is created alongside the destination so that it can be atomically
/// persisted. On Unix, a new destination uses the permissions of a normal file
/// creation, filtered by the process umask, while an existing destination's
/// permissions are preserved exactly.
pub fn temp_file_for_atomic_write(path: &Path) -> Result<NamedTempFile> {
    let parent = path.parent().ok_or_eyre("Destination has no parent")?;

    #[cfg(unix)]
    {
        use std::{fs::Permissions, os::unix::fs::PermissionsExt};

        let existing_permissions = match path.metadata() {
            Ok(metadata) => Some(metadata.permissions()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(error) => return Err(error.into()),
        };

        let temp_file = tempfile::Builder::new()
            .permissions(Permissions::from_mode(0o666))
            .tempfile_in(parent)?;
        if let Some(permissions) = existing_permissions {
            temp_file.as_file().set_permissions(permissions)?;
        }

        Ok(temp_file)
    }

    #[cfg(not(unix))]
    Ok(NamedTempFile::new_in(parent)?)
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use super::*;

    #[test]
    fn creates_file_alongside_destination() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let destination = dir.path().join("destination");
        let temp_file = temp_file_for_atomic_write(&destination)?;

        assert_eq!(temp_file.path().parent(), Some(dir.path()));
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn uses_normal_permissions_for_new_destination() -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir()?;
        let normal_path = dir.path().join("normal");
        File::create(&normal_path)?;

        let destination = dir.path().join("destination");
        let temp_file = temp_file_for_atomic_write(&destination)?;

        assert_eq!(
            temp_file.as_file().metadata()?.permissions().mode() & 0o777,
            normal_path.metadata()?.permissions().mode() & 0o777
        );
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn preserves_existing_destination_permissions() -> Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let dir = tempfile::tempdir()?;
        let destination = dir.path().join("destination");
        File::create(&destination)?;
        destination.set_permissions(std::fs::Permissions::from_mode(0o640))?;

        let temp_file = temp_file_for_atomic_write(&destination)?;

        assert_eq!(
            temp_file.as_file().metadata()?.permissions().mode() & 0o777,
            0o640
        );
        Ok(())
    }
}
