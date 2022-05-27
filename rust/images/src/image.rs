use std::{
    collections::HashMap,
    env::{self, temp_dir},
    ffi::OsString,
    fs::{self, File, FileType, Metadata},
    hash::Hasher,
    io,
    os::unix::prelude::MetadataExt,
    path::{Path, PathBuf},
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::Utc;
use eyre::{bail, eyre, Result};
use jwalk::WalkDirGeneric;
use oci_spec::image::{
    Descriptor, DescriptorBuilder, History, HistoryBuilder, ImageConfiguration,
    ImageConfigurationBuilder, ImageIndexBuilder, ImageManifestBuilder, MediaType, RootFsBuilder,
    SCHEMA_VERSION,
};
use seahash::SeaHasher;

// Serialization framework defaults to `rkyv` with fallback to `serde` JSON

#[cfg(feature = "rkyv")]
use rkyv::{Archive, Deserialize, Serialize};

#[cfg(feature = "rkyv-safe")]
use bytecheck::CheckBytes;

#[cfg(not(feature = "rkyv"))]
use buildpack::serde::{Deserialize, Serialize};

use archive_utils::{flate2, tar};
use hash_utils::{sha2::Digest, sha2::Sha256, str_sha256_hex};

use crate::{
    distribution::{Client, DOCKER_REGISTRY},
    utils::unique_string,
};

#[derive(Debug, Default, PartialEq, serde::Serialize)]
pub struct ImageReference {
    /// The registry the image is on. Defaults to `registry.hub.docker.com`
    pub registry: String,

    /// The repository the image is in e.g. `ubuntu`, `library/hello-world`
    pub repository: String,

    /// An image tag e.g. `sha256:...`. Conflicts with `digest`.
    pub tag: Option<String>,

    /// An image digest e.g. `sha256:e07ee1baac5fae6a26f3...`. Conflicts with `tag`.
    pub digest: Option<String>,
}

impl ImageReference {
    pub fn reference(&self) -> String {
        match self.digest.as_ref().or_else(|| self.tag.as_ref()) {
            Some(reference) => reference.clone(),
            None => "latest".to_string(),
        }
    }
}

impl FromStr for ImageReference {
    type Err = eyre::Report;

    /// Parse a string into an [`ImageSpec`]
    ///
    /// Based on the implementation in https://github.com/HewlettPackard/dockerfile-parser-rs/
    fn from_str(str: &str) -> Result<ImageReference> {
        let parts: Vec<&str> = str.splitn(2, '/').collect();

        let first = parts[0];
        let (registry, rest) = if parts.len() == 2
            && (first == "localhost" || first.contains('.') || first.contains(':'))
        {
            (Some(parts[0]), parts[1])
        } else {
            (None, str)
        };

        let registry = if matches!(registry, None) || matches!(registry, Some("docker.io")) {
            DOCKER_REGISTRY.to_string()
        } else {
            registry
                .expect("Should be Some because of the match above")
                .to_string()
        };

        let (name, tag, hash) = if let Some(at_pos) = rest.find('@') {
            let (name, hash) = rest.split_at(at_pos);
            (name.to_string(), None, Some(hash[1..].to_string()))
        } else {
            let parts: Vec<&str> = rest.splitn(2, ':').collect();
            let name = parts[0].to_string();
            let tag = parts.get(1).map(|str| str.to_string());
            (name, tag, None)
        };

        let name = if registry == DOCKER_REGISTRY && !name.contains('/') {
            ["library/", &name].concat()
        } else {
            name
        };

        Ok(ImageReference {
            registry,
            repository: name,
            tag,
            digest: hash,
        })
    }
}

impl ToString for ImageReference {
    fn to_string(&self) -> String {
        if let Some(digest) = &self.digest {
            [&self.registry, "/", &self.repository, "@", digest]
        } else {
            [
                &self.registry,
                "/",
                &self.repository,
                ":",
                self.tag.as_deref().unwrap_or("latest"),
            ]
        }
        .concat()
    }
}

/// A container image
///
/// This is serializable mainly so that it can be inspected as JSON or YAML output from a CLI command.
#[derive(Debug, serde::Serialize)]
pub struct Image {
    /// The project directory to build an image for
    ///
    /// This is the "working directory" that buildpacks will build layers
    /// for based on the source code within it. Defaults to the current directory.
    pub project_dir: PathBuf,

    /// The registry that the image is pushed to
    ///
    /// Defaults to `registry.hub.docker.com`.
    pub registry: String,

    /// The repository that the image is pushed to
    ///
    /// Defaults to the name of the `project_dir` suffixed with a hash of the
    /// absolute path of the `project_dir` (this ensures uniqueness on the current machine).
    pub repository: String,

    /// The base image from which this image is derived
    ///
    /// Equivalent to the `FROM` directive of a Dockerfile.
    pub base: ImageReference,

    /// The directories that will be snapshotted to generate individual layers for the image
    ///
    /// Defaults to the `project_dir` and any subdirectories of `/layers/*`.
    pub layer_dirs: Vec<PathBuf>,

    /// The snapshots for each layer directory, used to generated [`ChangeSet`]s and image layers
    #[serde(skip)]
    layer_snapshots: Vec<Snapshot>,

    /// The directory where this image will be written to
    ///
    /// The image will be written to this directory following the [OCI Image Layout Specification]
    /// (https://github.com/opencontainers/image-spec/blob/main/image-layout.md)
    pub layout_dir: PathBuf,
}

impl Image {
    /// Create a new image
    pub fn new(
        project_dir: Option<&Path>,
        reference: Option<&str>,
        base: Option<&str>,
        layer_dirs: &[&str],
        layout_dir: Option<&Path>,
    ) -> Result<Self> {
        let project_dir = project_dir
            .map(PathBuf::from)
            .unwrap_or_else(|| env::current_dir().expect("Unable to get cwd"));

        let (registry, repository) = match reference {
            Some(reference) => {
                let reference: ImageReference = reference.parse()?;
                (reference.registry, reference.repository)
            }
            None => {
                let registry = DOCKER_REGISTRY.to_string();
                let name = project_dir
                    .file_name()
                    .map(|name| name.to_string_lossy().to_string())
                    .unwrap_or_else(|| "unnamed".to_string());
                let hash = str_sha256_hex(&project_dir.to_string_lossy().to_string());
                let repository = [&name, "-", &hash[..12]].concat();
                (registry, repository)
            }
        };

        let base = base.unwrap_or("stencila/stencila:latest").parse()?;

        let patterns = if !layer_dirs.is_empty() {
            layer_dirs.iter().cloned().map(String::from).collect()
        } else {
            vec![project_dir.to_string_lossy().to_string()]
        };
        let mut layer_dirs = vec![];
        for pattern in patterns {
            for path in glob::glob(&pattern)?.flatten() {
                if path.is_dir() {
                    layer_dirs.push(path);
                }
            }
        }

        let layer_snapshots = layer_dirs.iter().map(Snapshot::new).collect();

        let layout_dir = match layout_dir {
            Some(path) => PathBuf::from(path),
            None => temp_dir().join(format!("stencila-image-layout-{}", unique_string())),
        };

        Ok(Self {
            project_dir,
            registry,
            repository,
            base,
            layer_dirs,
            layer_snapshots,
            layout_dir,
        })
    }

    /// Fetches the manifest and configuration of the base image
    ///
    /// Used when writing the image because the DiffIDs (from the config) and the layers (from the
    /// manifest) and required for the config and manifest of this image.
    async fn get_base(&self) -> Result<(ImageConfiguration, Vec<Descriptor>)> {
        let client = Client::new(&self.base.registry, &self.base.repository, None).await?;
        let manifest = client.get_manifest(self.base.reference()).await?;
        let config = client.get_config(&manifest).await?;
        let layers = manifest.layers().clone();
        Ok((config, layers))
    }

    /// Write the image layer blobs and returns vectors of DiffIDs and layer descriptors
    fn write_layers(
        &self,
        base_config: &ImageConfiguration,
        base_layers: Vec<Descriptor>,
    ) -> Result<(Vec<Descriptor>, Vec<String>, Vec<History>)> {
        let mut layers = base_layers;
        let mut diff_ids = base_config.rootfs().diff_ids().clone();
        let mut histories = base_config.history().clone();

        for snapshot in &self.layer_snapshots {
            let (diff_id, layer) = snapshot.write_layer(&self.layout_dir)?;

            let empty_layer = diff_id == "<empty>";

            if !empty_layer {
                diff_ids.push(diff_id);
                layers.push(layer);
            }

            let history = HistoryBuilder::default()
                .created(Utc::now().to_rfc3339())
                .created_by(format!(
                    "stencila {}",
                    env::args().skip(1).collect::<Vec<String>>().join(" ")
                ))
                .comment(format!("Layer for snapshotted directory {}", snapshot.dir))
                .empty_layer(empty_layer)
                .build()?;
            histories.push(history)
        }

        Ok((layers, diff_ids, histories))
    }

    /// Write the image config blob
    ///
    /// Implements the [OCI Image Configuration Specification](https://github.com/opencontainers/image-spec/blob/main/config.md).
    fn write_config(
        &self,
        base_config: &ImageConfiguration,
        diff_ids: Vec<String>,
        history: Vec<History>,
    ) -> Result<Descriptor> {
        // Start with the config of the base image and override as necessary
        let config = base_config.config().clone().unwrap_or_default();

        let rootfs = RootFsBuilder::default().diff_ids(diff_ids).build()?;

        let configuration = ImageConfigurationBuilder::default()
            .created(Utc::now().to_rfc3339())
            .os(env::consts::OS)
            .architecture(env::consts::ARCH)
            .config(config)
            .rootfs(rootfs)
            .history(history)
            .build()?;

        BlobWriter::write_json(&self.layout_dir, MediaType::ImageConfig, &configuration)
    }

    /// Write the image manifest blob
    ///
    /// Implements the [OCI Image Manifest Specification](https://github.com/opencontainers/image-spec/blob/main/manifest.md).
    /// Given that the manifest requires the descriptors for config and layers also calls `write_config` and `write_layers`.
    async fn write_manifest(&self) -> Result<Descriptor> {
        let (base_config, base_layers) = self.get_base().await?;

        let (layers, diff_ids, history) = self.write_layers(&base_config, base_layers)?;

        let config = self.write_config(&base_config, diff_ids, history)?;

        let manifest = ImageManifestBuilder::default()
            .schema_version(SCHEMA_VERSION)
            .media_type(MediaType::ImageManifest)
            .config(config)
            .layers(layers)
            .build()?;

        BlobWriter::write_json(&self.layout_dir, MediaType::ImageManifest, &manifest)
    }

    /// Write the image `index.json`
    ///
    /// Implements the [OCI Image Index Specification](https://github.com/opencontainers/image-spec/blob/main/image-index.md).
    /// Given that the index requires the image manifest descriptor, also calls `write_manifest`. At present the
    /// image only has one manifest (for a Linux image).
    async fn write_index(&self) -> Result<()> {
        let manifest = self.write_manifest().await?;

        let index = ImageIndexBuilder::default()
            .schema_version(SCHEMA_VERSION)
            .media_type(MediaType::ImageIndex)
            .manifests([manifest])
            .annotations([
                // Where appropriate use pre defined annotations
                // https://github.com/opencontainers/image-spec/blob/main/annotations.md#pre-defined-annotation-keys
                (
                    "org.opencontainers.image.created".to_string(),
                    Utc::now().to_rfc3339(),
                ),
                (
                    "org.opencontainers.image.base.name".to_string(),
                    self.base.to_string(),
                ),
                (
                    "org.opencontainers.image.ref.name".to_string(),
                    self.repository.clone(),
                ),
                (
                    "io.stencila.stencila.version".to_string(),
                    env!("CARGO_PKG_VERSION").to_string(),
                ),
            ])
            .build()?;

        fs::write(
            self.layout_dir.join("index.json"),
            serde_json::to_string_pretty(&index)?,
        )?;

        Ok(())
    }

    pub async fn build(&self) -> Result<()> {
        fs::write("./foo.txt", Utc::now().to_rfc3339())?;
        Ok(())
    }

    /// Write the image to `layout_dir`
    ///
    /// Implements the [OCI Image Layout Specification](https://github.com/opencontainers/image-spec/blob/main/image-layout.md).
    ///
    /// Note that the `blobs/sha256` subdirectory may not have blobs for the base image (these
    /// are only pulled into that directory if necessary i.e. if the registry does not yet have them).
    pub async fn write(&self) -> Result<()> {
        if self.layout_dir.exists() {
            fs::remove_dir_all(&self.layout_dir)?;
        }
        fs::create_dir_all(&self.layout_dir)?;

        self.write_index().await?;

        fs::write(
            self.layout_dir.join("oci-layout"),
            r#"{"imageLayoutVersion": "1.0.0"}"#,
        )?;

        Ok(())
    }

    /// Push the image to its registry
    ///
    /// The image must be written first (by a call to `self.write()`).
    pub async fn push(&self) -> Result<()> {
        let client = Client::new(&self.registry, &self.repository, None).await?;
        client.push_image("latest", &self.layout_dir).await?;

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
    ///
    /// Note that two SHA256 hashes are calculated, one for the `DiffID` of a changeset (calculated in this function
    /// and used in the image config file) and one for the digest which (calculated by the [`BlobWriter`] and used in the image manifest).
    /// A useful diagram showing how these are calculated and used is available
    /// [here](https://github.com/google/go-containerregistry/blob/main/pkg/v1/remote/README.md#anatomy-of-an-image-upload).
    ///
    /// # Arguments
    ///
    /// - `layout_dir`: the image directory to write the layer to (to the `blob/sha256` subdirectory)
    fn write_layer<P: AsRef<Path>>(self, layout_dir: P) -> Result<(String, Descriptor)> {
        if self.len() == 0 {
            return Ok(("<empty>".to_string(), DescriptorBuilder::default().build()?));
        }

        let mut diffid_hash = Sha256::new();
        let mut blob_writer = BlobWriter::new(&layout_dir, MediaType::ImageLayerGzip)?;

        {
            struct LayerWriter<'lt> {
                diffid_hash: &'lt mut Sha256,
                gzip_encoder: flate2::write::GzEncoder<&'lt mut BlobWriter>,
            }

            impl<'lt> io::Write for LayerWriter<'lt> {
                fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                    self.diffid_hash.update(buf);
                    self.gzip_encoder.write_all(buf)?;
                    Ok(buf.len())
                }

                fn flush(&mut self) -> io::Result<()> {
                    Ok(())
                }
            }

            let mut layer_writer = LayerWriter {
                diffid_hash: &mut diffid_hash,
                gzip_encoder: flate2::write::GzEncoder::new(
                    &mut blob_writer,
                    flate2::Compression::best(),
                ),
            };

            let mut archive = tar::Builder::new(&mut layer_writer);
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

        let diff_id = format!("sha256:{:x}", diffid_hash.finalize());
        let descriptor = blob_writer.finish()?;
        Ok((diff_id, descriptor))
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
            .join(digest.strip_prefix("sha256:").unwrap_or(digest))
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
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "rkyv", derive(Archive))]
#[cfg_attr(feature = "rkyv-safe", archive_attr(derive(CheckBytes)))]
#[cfg_attr(not(feature = "rkyv"), serde(crate = "buildpack::serde"))]
struct Snapshot {
    /// The directory to snapshot
    dir: String,

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
        let dir = dir.to_string_lossy().to_string();
        Self { dir, entries }
    }

    /// Create a new snapshot by repeating the current one
    fn repeat(&self) -> Self {
        Self::new(&self.dir)
    }

    /// Write a snapshot to a file
    fn write<P: AsRef<Path>>(&self, path: P) -> Result<()> {
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
    fn read<P: AsRef<Path>>(path: P) -> Result<Self> {
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
    /// Convenience function for combining calls to `changes` and `write_layer` on those changes.
    fn write_layer<P: AsRef<Path>>(&self, layout_dir: P) -> Result<(String, Descriptor)> {
        self.changes().write_layer(layout_dir)
    }
}

/// An entry for a file or directory in a snapshot
#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "rkyv", derive(Archive))]
#[cfg_attr(feature = "rkyv-safe", archive_attr(derive(CheckBytes)))]
#[cfg_attr(not(feature = "rkyv"), serde(crate = "buildpack::serde"))]
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
#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "rkyv", derive(Archive))]
#[cfg_attr(feature = "rkyv-safe", archive_attr(derive(CheckBytes)))]
#[cfg_attr(not(feature = "rkyv"), serde(crate = "buildpack::serde"))]
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

        let filename = PathBuf::from(format!("temporary-{}", unique_string()));
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
    fn write_json<P: AsRef<Path>, S: serde::Serialize>(
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
    use hash_utils::file_sha256_hex;
    use test_snaps::fixtures;
    use test_utils::{print_logs, tempfile::tempdir};

    use super::*;

    /// Test parsing image spec
    #[test]
    fn parse_image_ref() -> Result<()> {
        let ubuntu = ImageReference {
            registry: "registry.hub.docker.com".to_string(),
            repository: "library/ubuntu".to_string(),
            ..Default::default()
        };

        assert_eq!("ubuntu".parse::<ImageReference>()?, ubuntu);
        assert_eq!("docker.io/ubuntu".parse::<ImageReference>()?, ubuntu);
        assert_eq!(
            "registry.hub.docker.com/ubuntu".parse::<ImageReference>()?,
            ubuntu
        );

        let ubuntu_2204 = ImageReference {
            registry: "registry.hub.docker.com".to_string(),
            repository: "library/ubuntu".to_string(),
            tag: Some("22.04".to_string()),
            ..Default::default()
        };

        assert_eq!("ubuntu:22.04".parse::<ImageReference>()?, ubuntu_2204);
        assert_eq!(
            "docker.io/ubuntu:22.04".parse::<ImageReference>()?,
            ubuntu_2204
        );
        assert_eq!(
            "registry.hub.docker.com/ubuntu:22.04".parse::<ImageReference>()?,
            ubuntu_2204
        );

        let ubuntu_digest = ImageReference {
            registry: "registry.hub.docker.com".to_string(),
            repository: "library/ubuntu".to_string(),
            digest: Some("sha256:abcdef".to_string()),
            ..Default::default()
        };

        assert_eq!(
            "ubuntu@sha256:abcdef".parse::<ImageReference>()?,
            ubuntu_digest
        );
        assert_eq!(
            "docker.io/ubuntu@sha256:abcdef".parse::<ImageReference>()?,
            ubuntu_digest
        );
        assert_eq!(
            "registry.hub.docker.com/ubuntu@sha256:abcdef".parse::<ImageReference>()?,
            ubuntu_digest
        );

        Ok(())
    }

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

        let (.., descriptor) = changes.write_layer(&image_dir)?;

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

        let (.., descriptor) = changes.write_layer(&image_dir)?;
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

        let (.., descriptor) = changes.write_layer(&image_dir)?;
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

        // Create a layer archive, diffid and descriptor

        let changes = snap.changes();
        let (diff_id, descriptor) = changes.write_layer(&image_dir)?;

        assert_eq!(diff_id.len(), 7 + 64);
        assert!(diff_id.starts_with("sha256:"));

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
    #[tokio::test]
    async fn image_write() -> Result<()> {
        let project_dir = tempdir()?;
        let image = Image::new(Some(project_dir.path()), None, None, &[], None)?;

        image.write().await?;

        assert!(image.layout_dir.join("oci-layout").is_file());
        assert!(image.layout_dir.join("index.json").is_file());
        assert!(image.layout_dir.join("blobs").join("sha256").is_dir());

        Ok(())
    }
}
