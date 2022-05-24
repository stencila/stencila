use std::{
    collections::HashMap,
    env,
    ffi::OsString,
    fs::{self, File, FileType, Metadata},
    hash::Hasher,
    io,
    os::unix::prelude::MetadataExt,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::Utc;
use jwalk::WalkDirGeneric;
use oci_spec::image::{
    Descriptor, DescriptorBuilder, ImageConfiguration, ImageConfigurationBuilder,
    ImageIndexBuilder, ImageManifest, ImageManifestBuilder, MediaType, SCHEMA_VERSION,
};
use seahash::SeaHasher;

use archive_utils::{flate2, tar};
use buildpack::{
    eyre::{eyre, Result},
    hash_utils::sha2::{Digest, Sha256},
    serde::{Deserialize, Serialize},
    serde_json::{self, json},
    tracing,
};

struct Image {}

impl Image {
    /// Create a new image
    fn new() -> Self {
        Self {}
    }

    /// Write an image config blob
    ///
    /// # Arguments
    ///
    /// - `image_dir`: the image directory
    fn write_config<P: AsRef<Path>>(&self, image_dir: P) -> Result<Descriptor> {
        let config = ImageConfigurationBuilder::default()
            .created(Utc::now().to_rfc3339())
            .os(env::consts::OS)
            .architecture(env::consts::ARCH)
            .build()?;

        BlobWriter::write_json(image_dir, MediaType::ImageConfig, &config)
    }

    /// Write an image manifest blob
    ///
    /// # Arguments
    ///
    /// - `image_dir`: the image directory
    fn write_manifest<P: AsRef<Path>>(&self, image_dir: P) -> Result<Descriptor> {
        let config = self.write_config(&image_dir)?;

        let layers = []; // TODO

        let manifest = ImageManifestBuilder::default()
            .schema_version(SCHEMA_VERSION)
            .config(config)
            .layers(layers)
            .build()?;

        BlobWriter::write_json(image_dir, MediaType::ImageManifest, &manifest)
    }

    /// Create a directory with an OCI image layout
    ///
    /// Implements the [OCI Image Layout](https://github.com/opencontainers/image-spec/blob/main/image-layout.md)
    /// specification.
    ///
    /// # Arguments
    ///
    /// - `image_dir`: the image directory
    fn write<P: AsRef<Path>>(&self, image_dir: P) -> Result<()> {
        let image_dir = image_dir.as_ref();

        let oci_layout = image_dir.join("oci-layout");
        fs::write(oci_layout, r#"{"imageLayoutVersion": "1.0.0"}"#)?;

        let manifest = self.write_manifest(image_dir)?;

        let index_json = image_dir.join("index.json");
        let index = ImageIndexBuilder::default()
            .schema_version(SCHEMA_VERSION)
            .manifests([manifest])
            .build()?;
        fs::write(index_json, serde_json::to_string(&index)?)?;

        Ok(())
    }
}

/// The set of changes between two snapshots
///
/// Represents the set of changes between two filesystem snapshots as described in
/// [OCI Image Layer Filesystem Changeset](https://github.com/opencontainers/image-spec/blob/main/layer.md)
struct ChangeSet {
    /// The directory that these changes are for
    dir: PathBuf,

    /// The change items
    items: Vec<Change>,
}

impl ChangeSet {
    /// Create a new set of snapshot changes
    fn new<P: AsRef<Path>>(dir: P, changes: Vec<Change>) -> Self {
        Self {
            dir: dir.as_ref().to_path_buf(),
            items: changes,
        }
    }

    /// Get the number of changes in this set
    fn len(&self) -> usize {
        self.items.len()
    }

    /// Creates an OCI layer for the set of changes
    ///
    /// This implements the [Representing Changes](https://github.com/opencontainers/image-spec/blob/main/layer.md#representing-changes)
    /// section of the OCI image spec:
    ///
    /// - `Added` and `Modified` paths are added to the archive.
    /// - `Removed` paths are represented as "whiteout" files.
    ///
    /// # Arguments
    ///
    /// - `dir`: the image directory to write the layer to (to the `blob/sha256` subdirectory)
    fn write_layer<P: AsRef<Path>>(self, image_dir: P) -> Result<Descriptor> {
        let mut writer = BlobWriter::new(&image_dir, MediaType::ImageLayerGzip)?;
        {
            // Block required to drop encoder and its borrow of `writer` before returning
            let encoder = flate2::write::GzEncoder::new(&mut writer, flate2::Compression::best());

            let mut archive = tar::Builder::new(encoder);
            for change in self.items {
                match change {
                    Change::Added(path) | Change::Modified(path) => {
                        archive.append_path_with_name(self.dir.join(&path), path)?;
                    }
                    Change::Removed(path) => {
                        let path = PathBuf::from(path);
                        let basename = path
                            .file_name()
                            .ok_or_else(|| eyre!("Path has no file name"))?;
                        let mut whiteout = OsString::from(".wh.".to_string());
                        whiteout.push(basename);
                        let path = match path.parent() {
                            Some(parent) => parent.join(whiteout),
                            None => PathBuf::from(whiteout),
                        };
                        let mut header = tar::Header::new_gnu();
                        header.set_path(path)?;
                        header.set_size(0);
                        header.set_cksum();
                        let data: &[u8] = &[];
                        archive.append(&header, data)?;
                    }
                };
            }
        }
        writer.finish()
    }

    /// Get the path of a layer blob within an image directory
    ///
    /// # Arguments
    ///
    /// - `image_dir`: the image directory
    /// - `digest`: the digest of the layer (with or without the "sha256:" prefix)
    fn layer_path<P: AsRef<Path>>(image_dir: P, digest: &str) -> PathBuf {
        image_dir
            .as_ref()
            .join("blobs")
            .join("sha256")
            .join(digest.strip_prefix("sha256:").unwrap_or(&digest))
    }

    /// Read a layer blob (a compressed tar archive) from an image directory
    ///
    /// At this stage, mainly just used for testing.
    ///
    /// # Arguments
    ///
    /// - `image_dir`: the image directory
    /// - `digest`: the digest of the layer (with or without the "sha256:" prefix)
    fn read_layer<P: AsRef<Path>>(
        image_dir: P,
        digest: &str,
    ) -> Result<tar::Archive<flate2::read::GzDecoder<File>>> {
        let path = Self::layer_path(image_dir, digest);
        let file = fs::File::open(&path)?;
        let decoder = flate2::read::GzDecoder::new(file);
        let archive = tar::Archive::new(decoder);
        Ok(archive)
    }
}

/// A change in a path between two snapshots
///
/// This enum represents the [Change Types](https://github.com/opencontainers/image-spec/blob/main/layer.md#change-types)
/// described in the OCI spec.
#[derive(Debug, PartialEq)]
enum Change {
    Added(String),
    Modified(String),
    Removed(String),
}

/// A snapshot of the files and directories in a directory
///
/// A snapshot is created at the start of a session and stored to disk. Another snapshot
/// is taken at the end of session. The changes between the snapshots are used to create
/// an image layer.
///
/// Currently this uses `serde_json` for serializing to/from disk. An alternative
/// serialization such as `rkyv` would be a lot more efficient but, at the time of writing,
/// does not support `HashMap` with `PathBuf` as the key.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(crate = "buildpack::serde")]
struct Snapshot {
    /// The directory to snapshot
    dir: PathBuf,

    /// Entries in the snapshot
    entries: HashMap<String, SnapshotEntry>,
}

impl Snapshot {
    /// Create a new snapshot of a directory
    fn new<P: AsRef<Path>>(dir: P) -> Self {
        let dir = dir.as_ref().to_path_buf();
        let entries = WalkDirGeneric::<((), SnapshotEntry)>::new(&dir)
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
                    let relative_path = path
                        .strip_prefix(&dir)
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
                    tracing::error!("While snapshotting `{}`: {}", dir.display(), error);
                    None
                }
            })
            .collect();
        Self { dir, entries }
    }

    /// Create a new snapshot by repeating the current one
    fn repeat(&self) -> Self {
        Self::new(&self.dir)
    }

    /// Write a snapshot to a file
    fn write<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Read a snapshot from a file
    fn read<P: AsRef<Path>>(path: P) -> Result<Self> {
        let json = fs::read_to_string(&path)?;
        let snapshot = serde_json::from_str(&json)?;
        Ok(snapshot)
    }

    /// Create a set of changes by calculating the difference between two snapshots
    fn diff(&self, other: &Snapshot) -> ChangeSet {
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
        ChangeSet::new(&self.dir, changes)
    }

    /// Create a set of changes by repeating the current snapshot
    ///
    /// Convenience function for combining calls to `repeat` and `diff.
    fn changes(&self) -> ChangeSet {
        self.diff(&self.repeat())
    }

    /// Create a layer by repeating the current snapshot
    ///
    /// Convenience function for combining calls to `changes` and `layer` on those changes.
    fn write_layer<P: AsRef<Path>>(self, dest: P) -> Result<Descriptor> {
        self.changes().write_layer(dest)
    }
}

/// An entry for a file or directory in a snapshot
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(crate = "buildpack::serde")]
struct SnapshotEntry {
    /// Metadata on the file or directory
    ///
    /// Should only be `None` if there was an error getting the metadata
    /// while creating the snapshot.
    metadata: Option<SnapshotEntryMetadata>,

    /// Hash of the content of the file
    ///
    /// Will be `None` if the entry is a directory
    fingerprint: Option<u64>,
}

/// Filesystem metadata for a snapshot entry
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(crate = "buildpack::serde")]
struct SnapshotEntryMetadata {
    created: Option<u64>,
    modified: Option<u64>,
    uid: u32,
    gid: u32,
    readonly: bool,
}

impl SnapshotEntry {
    /// Create a new snapshot entry
    fn new(path: &Path, file_type: &FileType, metadata: Option<Metadata>) -> Self {
        let metadata = metadata.map(|metadata| SnapshotEntryMetadata {
            created: Self::file_timestamp(metadata.created()),
            modified: Self::file_timestamp(metadata.modified()),
            uid: metadata.uid(),
            gid: metadata.gid(),
            readonly: metadata.permissions().readonly(),
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

        Self {
            metadata,
            fingerprint,
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
        io::copy(&mut File::open(path)?, &mut hasher)?;
        Ok(hasher.0.finish())
    }

    /// Get a timestamp from a file's created or modified system time
    fn file_timestamp(time: Result<SystemTime, io::Error>) -> Option<u64> {
        time.map(|system_time| {
            system_time
                .duration_since(UNIX_EPOCH)
                .expect("Time should not go backwards")
                .as_secs()
        })
        .ok()
    }
}

/// A writer that calculates the size and SHA256 hash of files as they are written
///
/// Writes blobs into the `blobs/sha256` subdirectory of an image directory and returns
/// an [OCI Content Descriptor](https://github.com/opencontainers/image-spec/blob/main/descriptor.md)
///
/// Allows use to do a single pass when writing files instead of reading them after writing in order
/// to generate the SHA256 signature.
struct BlobWriter {
    /// The path to the `blobs/sha256` subdirectory where the blob is written to
    blobs_dir: PathBuf,

    /// The media type of the blob
    media_type: MediaType,

    /// The temporary filename of the blob (used before we know its final name - which is its SHA256 checksum)
    filename: PathBuf,

    /// The file the blob is written tp
    file: File,

    /// The number of bytes in the blob content
    bytes: usize,

    /// The SHA256 hash of the blob content
    hash: Sha256,
}

impl BlobWriter {
    /// Create a new blob writer
    ///
    /// # Arguments
    ///
    /// - `image_dir`: the image directory (blobs are written to the `blobs/sha256` subdirectory of this)
    /// - `media_type`: the media type of the blob
    fn new<P: AsRef<Path>>(image_dir: P, media_type: MediaType) -> Result<Self> {
        let blobs_dir = image_dir.as_ref().join("blobs").join("sha256");
        fs::create_dir_all(&blobs_dir)?;

        // Create a unique temporary filename without use of external crates
        // Based on https://users.rust-lang.org/t/random-number-without-using-the-external-crate/17260/9
        let ptr = Box::into_raw(Box::new(0)) as usize;
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        let filename = PathBuf::from(format!(".{}{}", ptr, nanos));

        let file = File::create(blobs_dir.join(&filename))?;

        Ok(Self {
            blobs_dir,
            media_type,
            filename,
            file,
            bytes: 0,
            hash: Sha256::new(),
        })
    }

    /// Finish writing the blob
    ///
    /// Finalizes the SHA256 hash, renames the file to the hex digest of that hash,
    /// and returns a descriptor of the blob.
    fn finish(self) -> Result<Descriptor> {
        let sha256 = format!("{:x}", self.hash.finalize());

        fs::rename(
            self.blobs_dir.join(self.filename),
            self.blobs_dir.join(&sha256),
        )?;

        let descriptor = DescriptorBuilder::default()
            .media_type(self.media_type)
            .size(self.bytes as i64)
            .digest(format!("sha256:{}", sha256))
            .build()?;

        Ok(descriptor)
    }

    /// Write an object as a JSON based media type
    fn write_json<P: AsRef<Path>, S: Serialize>(
        path: P,
        media_type: MediaType,
        object: &S,
    ) -> Result<Descriptor> {
        let mut writer = Self::new(path, media_type)?;
        serde_json::to_writer_pretty(&mut writer, object)?;
        writer.finish()
    }
}

impl io::Write for BlobWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write_all(buf)?;
        self.bytes += buf.len();
        self.hash.update(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use buildpack::hash_utils::file_sha256_hex;
    use test_snaps::fixtures;
    use test_utils::{print_logs, tempfile::tempdir};

    use super::*;

    /// Test that snapshots are correctly written to and read back from disk
    #[test]
    fn snapshot_serialization() -> Result<()> {
        let working_dir = fixtures().join("projects").join("apt");

        let temp = tempdir()?;
        let snapshot_path = temp.path().join("test.snap");
        let snapshot1 = Snapshot::new(working_dir);

        snapshot1.write(&snapshot_path)?;

        let snapshot2 = Snapshot::read(&snapshot_path)?;
        assert_eq!(snapshot1, snapshot2);

        Ok(())
    }

    /// Test snap-shotting, calculation of changesets, and the generation of layers from them.
    #[test]
    fn snapshot_changes() -> Result<()> {
        print_logs();

        // Create a temporary directory as a text fixture and a tar file for writing / reading layers

        let working_dir = tempdir()?;
        let image_dir = tempdir()?;

        // Create an initial snapshot which should be empty and has no changes with self

        let snap1 = Snapshot::new(working_dir.path());
        assert_eq!(snap1.entries.len(), 0);

        let changes = snap1.diff(&snap1);
        assert_eq!(changes.len(), 0);

        // Add a file, create a new snapshot and check it has one entry and produces a change set
        // with `Added` and tar has entry for it

        let a_txt = "a.txt".to_string();
        fs::write(working_dir.path().join(&a_txt), "Hello from a.txt")?;

        let snap2 = snap1.repeat();
        assert_eq!(snap2.entries.len(), 1);
        assert_eq!(snap2.entries[&a_txt].fingerprint, Some(3958791156379554752));

        let changes = snap1.diff(&snap2);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes.items[0], Change::Added(a_txt.clone()));

        let descriptor = changes.write_layer(&image_dir)?;

        let mut layer = ChangeSet::read_layer(&image_dir, descriptor.digest())?;
        let mut entries = layer.entries()?;
        let entry = entries
            .next()
            .ok_or_else(|| eyre!("No entries in tar archive"))??;
        assert_eq!(entry.path()?, PathBuf::from(&a_txt));
        assert_eq!(entry.size(), 16);

        // Repeat

        let b_txt = "b.txt".to_string();
        fs::write(working_dir.path().join(&b_txt), "Hello from b.txt")?;

        let snap3 = snap1.repeat();
        assert_eq!(snap3.entries.len(), 2);
        assert_eq!(snap2.entries[&a_txt].fingerprint, Some(3958791156379554752));
        assert_eq!(
            snap3.entries[&b_txt].fingerprint,
            Some(15548480638800185371)
        );

        let changes = snap2.diff(&snap3);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes.items[0], Change::Added(b_txt.clone()));

        // Remove a.txt and check that the change set has a `Removed` and tar has
        // a whiteout entry of size 0

        fs::remove_file(working_dir.path().join(&a_txt))?;

        let snap4 = snap1.repeat();
        assert_eq!(snap4.entries.len(), 1);
        assert_eq!(
            snap4.entries[&b_txt].fingerprint,
            Some(15548480638800185371)
        );

        let changes = snap3.diff(&snap4);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes.items[0], Change::Removed(a_txt));

        let descriptor = changes.write_layer(&image_dir)?;
        let mut layer = ChangeSet::read_layer(&image_dir, descriptor.digest())?;
        let mut entries = layer.entries()?;
        let entry = entries.next().unwrap()?;
        assert_eq!(entry.path()?, PathBuf::from(".wh.a.txt"));
        assert_eq!(entry.size(), 0);

        // Modify b.txt and check that the change set has a `Modified` and tar has
        // entry with new content

        fs::write(working_dir.path().join(&b_txt), "Hello")?;

        let snap5 = snap1.repeat();
        assert_eq!(snap5.entries.len(), 1);
        assert_eq!(snap5.entries[&b_txt].fingerprint, Some(3297469917561599766));

        let changes = snap4.diff(&snap5);
        assert_eq!(changes.len(), 1);
        assert_eq!(changes.items[0], Change::Modified(b_txt.clone()));

        let descriptor = changes.write_layer(&image_dir)?;
        let mut archive = ChangeSet::read_layer(&image_dir, descriptor.digest())?;
        let mut entries = archive.entries()?;
        let entry = entries.next().unwrap()?;
        assert_eq!(entry.path()?, PathBuf::from(b_txt));
        assert_eq!(entry.size(), 5);

        Ok(())
    }

    /// Test that the descriptor for a layer is accurate (SHA256 and size are same as
    /// when independently calculated)
    #[test]
    fn changes_layer() -> Result<()> {
        let working_dir = tempdir()?;
        let image_dir = tempdir()?;

        let snap = Snapshot::new(&working_dir);

        fs::write(&working_dir.path().join("some-file.txt"), "Hello")?;

        // Create a layer archive and descriptor

        let changes = snap.changes();
        let descriptor = changes.write_layer(&image_dir)?;

        // Test that size and digest in the descriptor is as for the file
        let archive = image_dir
            .path()
            .join("blobs")
            .join("sha256")
            .join(descriptor.digest().strip_prefix("sha256:").unwrap());

        let size = fs::metadata(&archive)?.len() as i64;
        assert_eq!(descriptor.size(), size);

        let digest = format!("sha256:{}", file_sha256_hex(&archive)?);
        assert_eq!(descriptor.digest(), &digest);

        Ok(())
    }

    /// Test that when an image is written to a directory that they directory conforms to
    /// the OCI Image Layout spec
    #[test]
    fn image_write() -> Result<()> {
        let image_dir = tempdir()?;
        let image = Image::new();
        image.write(&image_dir)?;

        let path = image_dir.path();
        assert!(path.join("oci-layout").is_file());
        assert!(path.join("index.json").is_file());
        assert!(path.join("blobs").join("sha256").is_dir());

        Ok(())
    }
}
