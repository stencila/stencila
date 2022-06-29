use std::{
    collections::HashMap,
    ffi::OsString,
    fs, io,
    path::{Path, PathBuf},
};

use oci_spec::image::{Descriptor, DescriptorBuilder, MediaType};

use archive_utils::{flate2, tar, zstd};
use common::{
    chrono::Utc,
    eyre::{bail, eyre, Result},
    itertools::Itertools,
    tracing,
};
use hash_utils::{sha2::Digest, sha2::Sha256};

use crate::{blob_writer::BlobWriter, storage::digest_to_parts};

/// A change in a path between two snapshots
///
/// This enum represents the [Change Types](https://github.com/opencontainers/image-spec/blob/main/layer.md#change-types)
/// described in the OCI spec.
#[derive(Debug, PartialEq, Eq, Ord, PartialOrd)]
pub enum Change {
    Added(String),
    Modified(String),
    Removed(String),
}

/// The set of changes between two snapshots
///
/// Represents the set of changes between two filesystem snapshots as described in
/// [OCI Image Layer Filesystem Changeset](https://github.com/opencontainers/image-spec/blob/main/layer.md)
pub struct ChangeSet {
    /// The source directory, on the local filesystem, for the changes
    source_dir: PathBuf,

    /// The destination directory, within the image's root filesystem, for the changes
    dest_dir: PathBuf,

    /// The change items
    pub(crate) items: Vec<Change>,
}

impl ChangeSet {
    /// Create a new set of snapshot changes
    pub fn new<P: AsRef<Path>>(source_dir: P, dest_dir: P, items: Vec<Change>) -> Self {
        let source_dir = source_dir.as_ref().to_path_buf();

        // Parths in tar archive must be relative so stri any leading slash
        let dest_dir = dest_dir.as_ref().to_path_buf();
        let dest_dir = match dest_dir.strip_prefix("/") {
            Ok(dir) => dir.to_owned(),
            Err(_) => dest_dir,
        };

        Self {
            source_dir,
            dest_dir,
            items,
        }
    }

    /// Creates an OCI layer for the set of changes
    ///
    /// This implements the [Representing Changes](https://github.com/opencontainers/image-spec/blob/main/layer.md#representing-changes)
    /// section of the OCI image spec:
    ///
    /// - `Added` and `Modified` paths are added to the archive.
    /// - `Removed` paths are represented as "whiteout" files.
    ///
    /// Note that two SHA256 hashes are calculated, one for the `DiffID` of a changeset (calculated in this function
    /// and used in the image config file) and one for the digest which (calculated by the [`BlobWriter`] and used in the image manifest).
    /// A useful diagram showing how these are calculated and used is available
    /// [here](https://github.com/google/go-containerregistry/blob/main/pkg/v1/remote/README.md#anatomy-of-an-image-upload).
    ///
    /// # Arguments
    ///
    /// - `layout_dir`: the image directory to write the layer to (to the `blob/sha256` subdirectory)
    pub fn write_layer<P: AsRef<Path>>(
        self,
        media_type: &MediaType,
        layout_dir: P,
    ) -> Result<(String, Descriptor)> {
        if self.items.is_empty() {
            return Ok((
                "<empty>".to_string(),
                DescriptorBuilder::default()
                    .media_type(media_type.clone())
                    .digest("<none>")
                    .size(0)
                    .build()?,
            ));
        }

        tracing::info!(
            "Writing image layer from changeset for `{}`",
            self.source_dir.display()
        );

        let mut diffid_hash = Sha256::new();
        let mut blob_writer = BlobWriter::new()?;

        let changes = self.items.len();
        let mut additions: Vec<String> = Vec::new();
        let mut modifications: Vec<String> = Vec::new();
        let mut deletions: Vec<String> = Vec::new();

        {
            enum LayerEncoder<'lt> {
                Plain(&'lt mut BlobWriter),
                Gzip(flate2::write::GzEncoder<&'lt mut BlobWriter>),
                Zstd(zstd::stream::AutoFinishEncoder<'lt, &'lt mut BlobWriter>),
            }

            struct LayerWriter<'lt> {
                diffid_hash: &'lt mut Sha256,
                layer_encoder: LayerEncoder<'lt>,
            }

            impl<'lt> io::Write for LayerWriter<'lt> {
                fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                    self.diffid_hash.update(buf);
                    match &mut self.layer_encoder {
                        LayerEncoder::Plain(encoder) => encoder.write_all(buf)?,
                        LayerEncoder::Gzip(encoder) => encoder.write_all(buf)?,
                        LayerEncoder::Zstd(encoder) => encoder.write_all(buf)?,
                    }
                    Ok(buf.len())
                }

                fn flush(&mut self) -> io::Result<()> {
                    Ok(())
                }
            }

            let layer_encoder = match media_type {
                MediaType::ImageLayer => LayerEncoder::Plain(&mut blob_writer),
                MediaType::ImageLayerGzip => LayerEncoder::Gzip(flate2::write::GzEncoder::new(
                    &mut blob_writer,
                    flate2::Compression::new(4),
                )),
                MediaType::ImageLayerZstd => LayerEncoder::Zstd(
                    zstd::stream::Encoder::new(&mut blob_writer, 4)?.auto_finish(),
                ),
                _ => bail!("Unhandled media type for layer: {}", media_type.to_string()),
            };

            let mut layer_writer = LayerWriter {
                diffid_hash: &mut diffid_hash,
                layer_encoder,
            };

            let mut archive = tar::Builder::new(&mut layer_writer);

            // Add an entry for the `dest_dir` (and any of its parent) so that ownership (and other
            // metadata) of `source_dir` is maintained. If not done then there are issues with non-root
            // users writing to the `workspace` and `layers` directories and  their subdirectories.
            let mut path = PathBuf::new();
            for part in self.dest_dir.components() {
                path = path.join(part);
                archive.append_path_with_name(&self.source_dir, &path)?;
            }

            // Add each change in sorted order so that the digest of the generated archive
            // does not change for a given change set
            for change in self.items.into_iter().sorted() {
                match change {
                    Change::Added(ref path) | Change::Modified(ref path) => {
                        let source_path = self.source_dir.join(path);
                        let dest_path = self.dest_dir.join(path);

                        let result = if source_path.is_symlink() {
                            match fs::read_link(&source_path).and_then(|target| {
                                fs::metadata(&source_path).map(|metadata| (target, metadata))
                            }) {
                                Ok((target, metadata)) => {
                                    #[cfg(target_family = "unix")]
                                    let (uid, gid) = {
                                        use std::os::unix::prelude::MetadataExt;
                                        (metadata.uid(), metadata.gid())
                                    };

                                    #[cfg(not(target_family = "unix"))]
                                    let (uid, gid) = (1000u32, 1000u32);

                                    let mut header = tar::Header::new_gnu();
                                    header.set_uid(uid.into());
                                    header.set_gid(gid.into());
                                    header.set_entry_type(tar::EntryType::Symlink);
                                    header.set_size(0);
                                    archive.append_link(&mut header, dest_path, target)
                                }
                                Err(error) => Err(error),
                            }
                        } else {
                            archive.append_path_with_name(source_path, dest_path)
                        };

                        if let Err(error) = result {
                            tracing::debug!(
                                "While appending item for added or modified path `{}`: {}",
                                path,
                                error
                            )
                        } else {
                            match change {
                                Change::Added(..) => additions.push(path.to_string()),
                                Change::Modified(..) => modifications.push(path.to_string()),
                                _ => unreachable!(),
                            }
                        }
                    }
                    Change::Removed(path) => {
                        let path_buf = PathBuf::from(&path);
                        let basename = path_buf
                            .file_name()
                            .ok_or_else(|| eyre!("Path has no file name"))?;
                        let mut whiteout = OsString::from(".wh.".to_string());
                        whiteout.push(basename);
                        let path_buf = match path_buf.parent() {
                            Some(parent) => parent.join(whiteout),
                            None => PathBuf::from(whiteout),
                        };
                        let path_buf = self.dest_dir.join(path_buf);

                        let mut header = tar::Header::new_gnu();
                        header.set_path(path_buf)?;
                        header.set_size(0);
                        header.set_cksum();
                        let data: &[u8] = &[];

                        if let Err(error) = archive.append(&header, data) {
                            tracing::debug!(
                                "While appending item for deleted path `{}`: {}",
                                path,
                                error
                            )
                        } else {
                            deletions.push(path)
                        }
                    }
                };
            }
        }

        let diff_id = format!("sha256:{:x}", diffid_hash.finalize());

        let mut annotations: HashMap<String, String> = [
            ("io.stencila.version", env!("CARGO_PKG_VERSION").to_string()),
            ("io.stencila.layer.created", Utc::now().to_rfc3339()),
            (
                "io.stencila.layer.directory",
                self.dest_dir.to_string_lossy().to_string(),
            ),
            ("io.stencila.layer.changes", changes.to_string()),
        ]
        .map(|(name, value)| (name.to_string(), value))
        .into();

        fn first_100(vec: Vec<String>) -> String {
            vec[..(std::cmp::min(vec.len(), 100))].join(":")
        }
        if !additions.is_empty() {
            annotations.insert(
                "io.stencila.layer.additions".to_string(),
                first_100(additions),
            );
        }
        if !modifications.is_empty() {
            annotations.insert(
                "io.stencila.layer.modifications".to_string(),
                first_100(modifications),
            );
        }
        if !deletions.is_empty() {
            annotations.insert(
                "io.stencila.layer.deletions".to_string(),
                first_100(deletions),
            );
        }

        let descriptor = blob_writer.finish(
            media_type.to_owned(),
            Some(annotations),
            Some(layout_dir.as_ref()),
        )?;

        Ok((diff_id, descriptor))
    }

    /// Get the path of a layer blob within an image directory
    ///
    /// # Arguments
    ///
    /// - `image_dir`: the image directory
    /// - `digest`: the digest of the layer (with or without the "sha256:" prefix)
    fn layer_path<P: AsRef<Path>>(image_dir: P, digest: &str) -> PathBuf {
        let (algo, hash) = digest_to_parts(digest);
        image_dir.as_ref().join("blobs").join(algo).join(hash)
    }

    /// Read a layer blob (a compressed tar archive) from an image directory
    ///
    /// At this stage, mainly just used for testing.
    ///
    /// # Arguments
    ///
    /// - `image_dir`: the image directory
    /// - `digest`: the digest of the layer (with or without the "sha256:" prefix)
    pub fn read_layer<P: AsRef<Path>>(
        image_dir: P,
        digest: &str,
    ) -> Result<tar::Archive<flate2::read::GzDecoder<std::fs::File>>> {
        let path = Self::layer_path(image_dir, digest);
        let file = fs::File::open(&path)?;
        let decoder = flate2::read::GzDecoder::new(file);
        let archive = tar::Archive::new(decoder);
        Ok(archive)
    }
}

#[cfg(test)]
mod tests {
    use hash_utils::file_sha256_hex;
    use test_utils::{common::tempfile::tempdir, skip_ci_os};

    use crate::{snapshot::Snapshot, storage::digest_to_parts};

    use super::*;

    /// Test that the descriptor for a layer is accurate (SHA256 and size are same as
    /// when independently calculated)
    #[test]
    fn changes_layer() -> Result<()> {
        if skip_ci_os(
            "macos",
            "Currently failing with Error: No such file or directory (os error 2)",
        ) {
            return Ok(());
        }

        let source_dir = tempdir()?;
        let layout_dir = tempdir()?;

        let snap = Snapshot::new(&source_dir, "workspace");

        fs::write(&source_dir.path().join("some-file.txt"), "Hello")?;

        // Create a layer archive, diffid and descriptor
        let changes = snap.changes();
        let (diff_id, descriptor) = changes.write_layer(&MediaType::ImageLayerGzip, &layout_dir)?;

        assert_eq!(diff_id.len(), 7 + 64);
        assert!(diff_id.starts_with("sha256:"));

        // Test that size and digest in the descriptor is as for the file
        let (algo, hash) = digest_to_parts(descriptor.digest());
        let archive = layout_dir.path().join("blobs").join(algo).join(hash);

        let size = fs::metadata(&archive)?.len() as i64;
        assert_eq!(descriptor.size(), size);

        let digest = format!("sha256:{}", file_sha256_hex(&archive)?);
        assert_eq!(descriptor.digest(), &digest);

        Ok(())
    }
}
