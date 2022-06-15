use std::{
    collections::HashMap,
    fs,
    hash::Hasher,
    io,
    path::{Path, PathBuf},
};

use jwalk::WalkDirGeneric;
use oci_spec::image::{Descriptor, MediaType};
use seahash::SeaHasher;

use common::{
    eyre::{bail, Result},
    tracing,
};

// Serialization framework defaults to `rkyv` with fallback to `serde` JSON

#[cfg(feature = "rkyv")]
use rkyv::{Archive, Deserialize, Serialize};

#[cfg(feature = "rkyv-safe")]
use bytecheck::CheckBytes;

#[cfg(not(feature = "rkyv"))]
use serde::{Deserialize, Serialize};

use crate::change_set::{Change, ChangeSet};

/// An entry for a file, directory, or symlink, in a snapshot
///
/// Stores data necessary to detect a change in the file.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "rkyv", derive(Archive))]
#[cfg_attr(feature = "rkyv-safe", archive_attr(derive(CheckBytes)))]
pub struct SnapshotEntry {
    /// Metadata on the file, directory, or symlink
    ///
    /// Should only be `None` if there was an error getting the metadata
    /// while creating the snapshot.
    metadata: Option<SnapshotEntryMetadata>,

    /// Hash of the content of the file
    ///
    /// Used to detect if the content of a file is changed.
    /// Will be `None` if the entry is a directory or symlink.
    fingerprint: Option<u64>,

    /// The target of the symlink
    ///
    /// Used to detect if the target of the symlink has changed.
    /// Will be `None` if the entry is a file or directory.
    target: Option<String>,
}

/// Filesystem metadata for a snapshot entry
///
/// Only includes the metadata that needs to be differences. For that reason,
/// does not record `modified` time since that would create a false positive
/// difference (if all other attributes were the same).
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "rkyv", derive(Archive))]
#[cfg_attr(feature = "rkyv-safe", archive_attr(derive(CheckBytes)))]
pub struct SnapshotEntryMetadata {
    uid: u32,
    gid: u32,
    readonly: bool,
}

impl SnapshotEntry {
    /// Create a new snapshot entry
    fn new(
        path: &Path,
        file_type: &std::fs::FileType,
        metadata: Option<std::fs::Metadata>,
    ) -> Self {
        let metadata = metadata.map(|metadata| {
            #[cfg(target_family = "unix")]
            let (uid, gid) = {
                use std::os::unix::prelude::MetadataExt;
                (metadata.uid(), metadata.gid())
            };

            #[cfg(not(target_family = "unix"))]
            let (uid, gid) = (1000u32, 1000u32);

            SnapshotEntryMetadata {
                uid,
                gid,
                readonly: metadata.permissions().readonly(),
            }
        });

        let fingerprint = if file_type.is_file() {
            match Self::file_fingerprint::<SeaHasher>(path) {
                Ok(fingerprint) => Some(fingerprint),
                Err(error) => {
                    tracing::error!("While fingerprinting file `{}`: {}", path.display(), error);
                    None
                }
            }
        } else {
            None
        };

        let target = if file_type.is_symlink() {
            match fs::read_link(path) {
                Ok(target) => Some(target.to_string_lossy().to_string()),
                Err(error) => {
                    tracing::error!(
                        "While reading target of symlink `{}`: {}",
                        path.display(),
                        error
                    );
                    None
                }
            }
        } else {
            None
        };

        Self {
            metadata,
            fingerprint,
            target,
        }
    }

    /// Generate a hash of a file's content
    ///
    /// Used to generate a fingerprint
    ///
    /// Based on https://github.com/jRimbault/yadf/blob/04205a57882ffa7d6a9ca05016e18214a38079b6/src/fs/hash.rs#L29
    fn file_fingerprint<H>(path: &Path) -> io::Result<u64>
    where
        H: Hasher + Default,
    {
        struct HashWriter<H>(H);
        impl<H: Hasher> io::Write for HashWriter<H> {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                self.0.write(buf);
                Ok(buf.len())
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let mut hasher = HashWriter(H::default());
        io::copy(&mut std::fs::File::open(path)?, &mut hasher)?;
        Ok(hasher.0.finish())
    }
}

/// A snapshot of the files and directories in a directory
///
/// A snapshot is created at the start of a session and stored to disk. Another snapshot
/// is taken at the end of session. The changes between the snapshots are used to create
/// an image layer.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "rkyv", derive(Archive))]
#[cfg_attr(feature = "rkyv-safe", archive_attr(derive(CheckBytes)))]
pub struct Snapshot {
    /// The source directory, on the local filesystem, for the snapshot
    pub source_dir: String,

    /// The destination directory, on the image's root filesystem, for the snapshot
    pub dest_dir: String,

    /// Entries in the snapshot
    entries: HashMap<String, SnapshotEntry>,
}

impl Snapshot {
    /// Create a new snapshot of a directory
    ///
    /// If there is a `.dockerignore` or `.containerignore` file in source directory then it will be
    /// used to exclude paths, including those in child sub-directories.
    pub fn new<S: AsRef<Path>, D: AsRef<Path>>(source_dir: S, dest_dir: D) -> Self {
        let source_dir = source_dir.as_ref().to_path_buf();
        let dest_dir = dest_dir.as_ref().to_path_buf();

        let docker_ignore = source_dir.join(".dockerignore");
        let container_ignore = source_dir.join(".containerignore");
        fn parse_ignore_file(path: &Path) -> Option<gitignore::File> {
            match gitignore::File::new(path) {
                Ok(file) => Some(file),
                Err(error) => {
                    tracing::warn!(
                        "Error while parsing `{}`; will not be used: {}",
                        path.display(),
                        error
                    );
                    None
                }
            }
        }
        let ignore_file = if docker_ignore.exists() {
            parse_ignore_file(&docker_ignore)
        } else if container_ignore.exists() {
            parse_ignore_file(&container_ignore)
        } else {
            None
        };

        let entries = WalkDirGeneric::<((), SnapshotEntry)>::new(&source_dir)
            .skip_hidden(false)
            .process_read_dir(|_depth, _path, _read_dir_state, children| {
                children.iter_mut().flatten().for_each(|dir_entry| {
                    if !dir_entry.file_type.is_dir() {
                        dir_entry.client_state = SnapshotEntry::new(
                            &dir_entry.path(),
                            &dir_entry.file_type(),
                            dir_entry.metadata().ok(),
                        );
                    }
                })
            })
            .into_iter()
            .filter_map(|entry_result| match entry_result {
                Ok(entry) => {
                    let path = entry.path();

                    // Check that the file should not be ignored
                    if let Some(true) = ignore_file
                        .as_ref()
                        .and_then(|ignore_file| ignore_file.is_excluded(&path).ok())
                    {
                        return None;
                    };

                    let relative_path = path
                        .strip_prefix(&source_dir)
                        .expect("Should always be able to strip the root dir");
                    match relative_path == PathBuf::from("") {
                        true => None, // This is the entry for the dir itself so ignore it
                        false => Some((
                            relative_path.to_string_lossy().to_string(), // Should be lossless on Linux (and MacOS)
                            entry.client_state,
                        )),
                    }
                }
                Err(error) => {
                    tracing::error!("While snapshotting `{}`: {}", source_dir.display(), error);
                    None
                }
            })
            .collect();

        let source_dir = source_dir.to_string_lossy().to_string();
        let dest_dir = dest_dir.to_string_lossy().to_string();
        Self {
            source_dir,
            dest_dir,
            entries,
        }
    }

    /// Create a new snapshot by repeating the current one
    pub fn repeat(&self) -> Self {
        Self::new(&self.source_dir, &self.dest_dir)
    }

    /// Write a snapshot to a file
    pub fn write<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        #[cfg(feature = "rkyv")]
        {
            let bytes = rkyv::to_bytes::<Self, 256>(self)?;
            fs::write(path, bytes)?;
        }

        #[cfg(not(feature = "rkyv"))]
        {
            let json = serde_json::to_string_pretty(self)?;
            fs::write(path, json)?;
        }

        Ok(())
    }

    /// Read a snapshot from a file
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self> {
        #[cfg(feature = "rkyv")]
        {
            let bytes = fs::read(path)?;

            #[cfg(feature = "rkyv-safe")]
            let archived = match rkyv::check_archived_root::<Self>(&bytes[..]) {
                Ok(archived) => archived,
                Err(error) => {
                    bail!("While checking archive: {}", error)
                }
            };

            #[cfg(not(feature = "rkyv-safe"))]
            let archived = unsafe { rkyv::archived_root::<Self>(&bytes[..]) };

            let snapshot = archived.deserialize(&mut rkyv::Infallible)?;
            Ok(snapshot)
        }

        #[cfg(not(feature = "rkyv"))]
        {
            let json = fs::read_to_string(&path)?;
            let snapshot = serde_json::from_str(&json)?;
            Ok(snapshot)
        }
    }

    /// Create a set of changes that replicate the current snapshot using only additions
    fn replicate(&self) -> ChangeSet {
        let changes = self
            .entries
            .keys()
            .map(|path| Change::Added(path.into()))
            .collect();
        ChangeSet::new(&self.source_dir, &self.dest_dir, changes)
    }

    /// Create a set of changes by determining the difference between two snapshots
    pub fn diff(&self, other: &Snapshot) -> ChangeSet {
        let mut changes = Vec::new();
        for (path, entry) in self.entries.iter() {
            match other.entries.get(path) {
                Some(other_entry) => {
                    if entry != other_entry {
                        changes.push(Change::Modified(path.into()))
                    }
                }
                None => changes.push(Change::Removed(path.into())),
            }
        }
        for path in other.entries.keys() {
            if !self.entries.contains_key(path) {
                changes.push(Change::Added(path.into()))
            }
        }
        ChangeSet::new(&self.source_dir, &self.dest_dir, changes)
    }

    /// Create a set of changes by repeating the current snapshot
    ///
    /// Convenience function for combining calls to `repeat` and `diff.
    pub fn changes(&self) -> ChangeSet {
        self.diff(&self.repeat())
    }

    /// Create a layer by repeating the current snapshot
    ///
    /// # Arguments
    ///
    /// - `diff`: Whether to create the layer as the difference to the original snapshot
    ///           (the usual) or as a replicate.
    pub fn write_layer<P: AsRef<Path>>(
        &self,
        layout_dir: P,
        diff: bool,
        media_type: &MediaType,
    ) -> Result<(String, Descriptor)> {
        let new = self.repeat();
        let changeset = if diff {
            self.diff(&new)
        } else {
            new.replicate()
        };
        changeset.write_layer(media_type, layout_dir)
    }
}

#[cfg(test)]
mod tests {
    use common::{eyre::eyre, tempfile::tempdir};

    use test_snaps::fixtures;
    use test_utils::print_logs;

    use super::*;

    /// Test that snapshots are correctly written to and read back from disk
    #[test]
    fn snapshot_serialization() -> Result<()> {
        let working_dir = fixtures().join("projects").join("apt");

        let temp = tempdir()?;
        let snapshot_path = temp.path().join("test.snap");
        let snapshot1 = Snapshot::new(working_dir, "/workspace");

        snapshot1.write(&snapshot_path)?;

        let snapshot2 = Snapshot::read(&snapshot_path)?;
        assert_eq!(snapshot1, snapshot2);

        Ok(())
    }

    /// Test that snap-shotting takes into consideration .dockerignore and .containerignore files
    #[test]
    fn snapshot_ignores() -> Result<()> {
        use std::fs;

        let temp = tempdir()?;
        let source_dir = temp.path();

        fs::write(source_dir.join("a.txt"), "A")?;
        assert!(Snapshot::new(source_dir, "").entries.contains_key("a.txt"));

        fs::write(source_dir.join(".dockerignore"), "*\n")?;
        assert!(!Snapshot::new(source_dir, "").entries.contains_key("a.txt"));

        fs::remove_file(source_dir.join(".dockerignore"))?;
        fs::write(source_dir.join(".containerignore"), "*.txt\n")?;
        assert!(!Snapshot::new(source_dir, "").entries.contains_key("a.txt"));

        fs::remove_file(source_dir.join(".containerignore"))?;
        fs::write(source_dir.join("b.txt"), "B")?;
        fs::write(source_dir.join(".dockerignore"), "!a.txt\n")?;
        let snapshot = Snapshot::new(source_dir, "");
        assert!(snapshot.entries.contains_key("a.txt"));
        assert!(snapshot.entries.contains_key("b.txt"));

        Ok(())
    }

    /// Test snap-shotting, calculation of changesets, and the generation of layers from them.
    #[test]
    fn snapshot_changes() -> Result<()> {
        use std::fs;

        print_logs();

        // Create a temporary directory as a text fixture and a tar file for writing / reading layers

        let source_dir = tempdir()?;
        let dest_dir = PathBuf::from("workspace");
        let layout_dir = tempdir()?;

        // Create an initial snapshot which should be empty and has no changes with self

        let snap1 = Snapshot::new(source_dir.path(), &dest_dir);
        assert_eq!(snap1.entries.len(), 0);

        let changes = snap1.diff(&snap1);
        assert_eq!(changes.items.len(), 0);

        // Add a file, create a new snapshot and check it has one entry and produces a change set
        // with `Added` and tar has entry for it

        let a_txt = "a.txt".to_string();
        fs::write(source_dir.path().join(&a_txt), "Hello from a.txt")?;

        let snap2 = snap1.repeat();
        assert_eq!(snap2.entries.len(), 1);
        assert_eq!(snap2.entries[&a_txt].fingerprint, Some(3958791156379554752));

        let changes = snap1.diff(&snap2);
        assert_eq!(changes.items.len(), 1);
        assert_eq!(changes.items[0], Change::Added(a_txt.clone()));

        let (.., descriptor) = changes.write_layer(&MediaType::ImageLayerGzip, &layout_dir)?;

        let mut layer = ChangeSet::read_layer(&layout_dir, descriptor.digest())?;
        let mut entries = layer.entries()?;
        let entry = entries
            .nth(1)
            .ok_or_else(|| eyre!("No entries in tar archive"))??;
        assert_eq!(entry.path()?, dest_dir.join(&a_txt));
        assert_eq!(entry.size(), 16);

        // Repeat

        let b_txt = "b.txt".to_string();
        fs::write(source_dir.path().join(&b_txt), "Hello from b.txt")?;

        let snap3 = snap1.repeat();
        assert_eq!(snap3.entries.len(), 2);
        assert_eq!(snap2.entries[&a_txt].fingerprint, Some(3958791156379554752));
        assert_eq!(
            snap3.entries[&b_txt].fingerprint,
            Some(15548480638800185371)
        );

        let changes = snap2.diff(&snap3);
        assert_eq!(changes.items.len(), 1);
        assert_eq!(changes.items[0], Change::Added(b_txt.clone()));

        // Remove a.txt and check that the change set has a `Removed` and tar has
        // a whiteout entry of size 0

        fs::remove_file(source_dir.path().join(&a_txt))?;

        let snap4 = snap1.repeat();
        assert_eq!(snap4.entries.len(), 1);
        assert_eq!(
            snap4.entries[&b_txt].fingerprint,
            Some(15548480638800185371)
        );

        let changes = snap3.diff(&snap4);
        assert_eq!(changes.items.len(), 1);
        assert_eq!(changes.items[0], Change::Removed(a_txt));

        let (.., descriptor) = changes.write_layer(&MediaType::ImageLayerGzip, &layout_dir)?;
        let mut layer = ChangeSet::read_layer(&layout_dir, descriptor.digest())?;
        let mut entries = layer.entries()?;
        let entry = entries.nth(1).unwrap()?;
        assert_eq!(entry.path()?, dest_dir.join(".wh.a.txt"));
        assert_eq!(entry.size(), 0);

        // Modify b.txt and check that the change set has a `Modified` and tar has
        // entry with new content

        fs::write(source_dir.path().join(&b_txt), "Hello")?;

        let snap5 = snap1.repeat();
        assert_eq!(snap5.entries.len(), 1);
        assert_eq!(snap5.entries[&b_txt].fingerprint, Some(3297469917561599766));

        let changes = snap4.diff(&snap5);
        assert_eq!(changes.items.len(), 1);
        assert_eq!(changes.items[0], Change::Modified(b_txt.clone()));

        let (.., descriptor) = changes.write_layer(&MediaType::ImageLayerGzip, &layout_dir)?;
        let mut archive = ChangeSet::read_layer(&layout_dir, descriptor.digest())?;
        let mut entries = archive.entries()?;
        let entry = entries.nth(1).unwrap()?;
        assert_eq!(entry.path()?, dest_dir.join(b_txt));
        assert_eq!(entry.size(), 5);

        Ok(())
    }
}
