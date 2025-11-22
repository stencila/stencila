use std::{path::Path, time::UNIX_EPOCH};

use eyre::Result;
use walkdir::WalkDir;

/// Get the modification time of a file or directory
///
/// For files, returns the file's modification timestamp.
/// For directories, recursively finds the latest modification time
/// of any file within the directory.
///
/// Returns Unix timestamp (seconds since UNIX_EPOCH).
///
/// # Errors
///
/// Returns an error if the path does not exist or if there are issues
/// accessing file metadata.
pub fn modification_time(path: &Path) -> Result<u64> {
    if !path.exists() {
        return Err(eyre::eyre!("Path does not exist: {}", path.display()));
    }

    if path.is_file() {
        // For files, return the file's modification time
        get_mtime(path)
    } else if path.is_dir() {
        // For directories, find the latest modification time of any file within
        find_latest_mtime_in_dir(path)
    } else {
        // For other types (symlinks, etc.), try to get their mtime directly
        get_mtime(path)
    }
}

/// Get the modification time of a single file or directory
fn get_mtime(path: &Path) -> Result<u64> {
    let metadata = path.metadata()?;
    let modified = metadata.modified()?;
    let duration = modified.duration_since(UNIX_EPOCH)?;
    Ok(duration.as_secs())
}

/// Find the latest modification time of any file within a directory
fn find_latest_mtime_in_dir(dir: &Path) -> Result<u64> {
    let mut latest = 0u64;

    // Walk the directory recursively, but don't follow symlinks
    for entry in WalkDir::new(dir).follow_links(false) {
        // Skip entries that we can't access
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        // Only consider files (not directories or symlinks)
        if !entry.file_type().is_file() {
            continue;
        }

        // Try to get the modification time
        if let Ok(metadata) = entry.metadata()
            && let Ok(modified) = metadata.modified()
            && let Ok(duration) = modified.duration_since(UNIX_EPOCH)
        {
            latest = latest.max(duration.as_secs());
        }
    }

    // Fallback to directory's own mtime if no files were found
    if latest == 0 {
        latest = get_mtime(dir)?;
    }

    Ok(latest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::time::Duration;

    #[test]
    fn test_file_modification_time() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let file_path = temp_dir.path().join("test.txt");

        // Create a test file
        let mut file = fs::File::create(&file_path)?;
        writeln!(file, "test content")?;

        // Get modification time
        let mtime = modification_time(&file_path)?;

        // Should be a reasonable Unix timestamp (after 2020)
        assert!(mtime > 1577836800); // Jan 1, 2020

        Ok(())
    }

    #[test]
    fn test_directory_modification_time() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let dir_path = temp_dir.path();

        // Create a file in the directory
        let file_path = dir_path.join("test.txt");
        let mut file = fs::File::create(&file_path)?;
        writeln!(file, "test content")?;

        // Get directory modification time
        let mtime = modification_time(dir_path)?;

        // Should be a reasonable Unix timestamp
        assert!(mtime > 1577836800); // Jan 1, 2020

        // Sleep a bit and create another file
        std::thread::sleep(Duration::from_millis(10));
        let file2_path = dir_path.join("test2.txt");
        let mut file2 = fs::File::create(&file2_path)?;
        writeln!(file2, "newer content")?;

        // Get directory modification time again
        let mtime2 = modification_time(dir_path)?;

        // The second mtime should be >= the first (newer file)
        assert!(mtime2 >= mtime);

        Ok(())
    }

    #[test]
    fn test_empty_directory() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let empty_dir = temp_dir.path().join("empty");
        fs::create_dir(&empty_dir)?;

        // Should return the directory's own mtime
        let mtime = modification_time(&empty_dir)?;

        // Should be a reasonable Unix timestamp
        assert!(mtime > 1577836800); // Jan 1, 2020

        Ok(())
    }

    #[test]
    fn test_nonexistent_path() {
        let result = modification_time(Path::new("/nonexistent/path/that/does/not/exist"));
        assert!(result.is_err());
    }
}
