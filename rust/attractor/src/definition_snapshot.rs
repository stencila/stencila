//! Create and extract zstd-compressed tar archives of definition directories.
//!
//! Each workflow, agent, or skill lives in a directory (e.g.
//! `.stencila/workflows/my-wf/`) that may contain the main markdown file
//! plus supporting files. A definition snapshot captures the **entire
//! directory** as a zstd-compressed tar, content-addressed by SHA-256.
//!
//! Feature-gated behind `sqlite` (same gate as [`super::sqlite_backend`]).

use std::io::Cursor;
use std::path::Path;

use sha2::{Digest, Sha256};

/// Zstd compression level for definition snapshots.
///
/// Level 3 (the zstd default) offers a good balance for small text-heavy
/// archives — roughly matching gzip-9 ratios with much faster decompression.
const ZSTD_LEVEL: i32 = 3;

/// Create a zstd-compressed tar archive of `dir` and return `(sha256_hex, blob)`.
///
/// All entries are stored with paths relative to `dir`. Hidden files and
/// directories (names starting with `.`) are excluded so that editor/VCS
/// metadata doesn't pollute the snapshot.
///
/// The SHA-256 hash is computed over the compressed blob, so identical
/// directory contents always produce the same hash.
///
/// # Errors
///
/// Returns `std::io::Error` if the directory cannot be read or any file
/// within it cannot be opened.
pub fn snapshot_dir(dir: &Path) -> std::io::Result<(String, Vec<u8>)> {
    let tar_bytes = create_tar(dir)?;
    let compressed = zstd::encode_all(Cursor::new(&tar_bytes), ZSTD_LEVEL)?;

    let hash = sha256_hex(&compressed);
    Ok((hash, compressed))
}

/// Extract a zstd-compressed tar archive into `dest`.
///
/// Creates `dest` and any parent directories if they don't exist.
///
/// # Errors
///
/// Returns `std::io::Error` if decompression or extraction fails.
pub fn extract_snapshot(blob: &[u8], dest: &Path) -> std::io::Result<()> {
    let decompressed = zstd::decode_all(Cursor::new(blob))?;
    let mut archive = tar::Archive::new(Cursor::new(decompressed));
    std::fs::create_dir_all(dest)?;
    archive.unpack(dest)?;
    Ok(())
}

fn create_tar(dir: &Path) -> std::io::Result<Vec<u8>> {
    let mut builder = tar::Builder::new(Vec::new());
    append_dir_recursive(&mut builder, dir, dir)?;
    builder.finish()?;
    builder.into_inner()
}

fn append_dir_recursive(
    builder: &mut tar::Builder<Vec<u8>>,
    root: &Path,
    current: &Path,
) -> std::io::Result<()> {
    let mut entries: Vec<_> = std::fs::read_dir(current)?.filter_map(Result::ok).collect();
    // Sort for deterministic archive order → deterministic hash.
    entries.sort_by_key(std::fs::DirEntry::file_name);

    for entry in entries {
        let name = entry.file_name();
        if name.to_string_lossy().starts_with('.') {
            continue;
        }

        let path = entry.path();
        let ft = entry.file_type()?;

        if ft.is_file() {
            let rel = path.strip_prefix(root).map_err(std::io::Error::other)?;
            builder.append_path_with_name(&path, rel)?;
        } else if ft.is_dir() {
            append_dir_recursive(builder, root, &path)?;
        }
    }

    Ok(())
}

fn sha256_hex(data: &[u8]) -> String {
    let digest = Sha256::digest(data);
    let mut hex = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn round_trip_single_file() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let dir = tmp.path().join("my-workflow");
        fs::create_dir_all(&dir).expect("mkdir");
        fs::write(dir.join("WORKFLOW.md"), "---\nname: test\n---\n").expect("write");

        let (hash, blob) = snapshot_dir(&dir).expect("snapshot");
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA-256 hex = 64 chars
        assert!(!blob.is_empty());

        let dest = tmp.path().join("restored");
        extract_snapshot(&blob, &dest).expect("extract");
        let content = fs::read_to_string(dest.join("WORKFLOW.md")).expect("read");
        assert!(content.contains("name: test"));
    }

    #[test]
    fn round_trip_multiple_files() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let dir = tmp.path().join("my-agent");
        fs::create_dir_all(dir.join("examples")).expect("mkdir");
        fs::write(dir.join("AGENT.md"), "# Agent\n").expect("write");
        fs::write(dir.join("examples/sample.md"), "example content").expect("write");

        let (hash, blob) = snapshot_dir(&dir).expect("snapshot");
        assert!(!hash.is_empty());

        let dest = tmp.path().join("restored");
        extract_snapshot(&blob, &dest).expect("extract");
        assert_eq!(
            fs::read_to_string(dest.join("AGENT.md")).expect("read"),
            "# Agent\n"
        );
        assert_eq!(
            fs::read_to_string(dest.join("examples/sample.md")).expect("read"),
            "example content"
        );
    }

    #[test]
    fn hidden_files_excluded() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let dir = tmp.path().join("skill");
        fs::create_dir_all(&dir).expect("mkdir");
        fs::write(dir.join("SKILL.md"), "skill").expect("write");
        fs::write(dir.join(".DS_Store"), "junk").expect("write");
        fs::create_dir_all(dir.join(".git")).expect("mkdir");
        fs::write(dir.join(".git/config"), "git stuff").expect("write");

        let (_hash, blob) = snapshot_dir(&dir).expect("snapshot");

        let dest = tmp.path().join("restored");
        extract_snapshot(&blob, &dest).expect("extract");
        assert!(dest.join("SKILL.md").exists());
        assert!(!dest.join(".DS_Store").exists());
        assert!(!dest.join(".git").exists());
    }

    #[test]
    fn deterministic_hash() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let dir = tmp.path().join("wf");
        fs::create_dir_all(&dir).expect("mkdir");
        fs::write(dir.join("WORKFLOW.md"), "hello").expect("write");

        let (h1, b1) = snapshot_dir(&dir).expect("snapshot 1");
        let (h2, b2) = snapshot_dir(&dir).expect("snapshot 2");

        assert_eq!(h1, h2);
        assert_eq!(b1, b2);
    }

    #[test]
    fn different_content_different_hash() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let dir = tmp.path().join("wf");
        fs::create_dir_all(&dir).expect("mkdir");

        fs::write(dir.join("WORKFLOW.md"), "version 1").expect("write");
        let (h1, _) = snapshot_dir(&dir).expect("snapshot 1");

        fs::write(dir.join("WORKFLOW.md"), "version 2").expect("write");
        let (h2, _) = snapshot_dir(&dir).expect("snapshot 2");

        assert_ne!(h1, h2);
    }
}
