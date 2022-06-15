use std::{
    collections::HashMap,
    fs::{rename, File},
    io,
    path::{Path, PathBuf},
};

use common::{eyre::Result, serde, serde_json};
use oci_spec::image::{Descriptor, DescriptorBuilder, MediaType};

use hash_utils::sha2::{Digest, Sha256};

use crate::{
    storage::{blob_path_safe, blob_symlink},
    utils::unique_string,
};

/// A writer that calculates the size and SHA256 hash of files as they are written
///
/// Allows a single pass when writing blobs (instead of reading them after writing in order
/// to generate the SHA256 signature). Writes blobs into a shared `blobs/sha256` subdirectory and then
/// symlinks to those in each individual image directory, thereby maximizing layer reuse and reducing
/// disk consumption.
pub struct BlobWriter {
    /// The temporary file path of the blob (used before we know its final name, which is its SHA256 checksum)
    file_path: PathBuf,

    /// The file the blob is written to
    file: std::fs::File,

    /// The number of bytes in the blob content
    bytes: usize,

    /// The SHA256 hash of the blob content
    hash: Sha256,
}

impl BlobWriter {
    /// Create a new blob writer
    pub fn new() -> Result<Self> {
        let file_path = blob_path_safe(&format!("temp:{}", unique_string()))?;
        let file = File::create(&file_path)?;

        Ok(Self {
            file_path,
            file,
            bytes: 0,
            hash: Sha256::new(),
        })
    }

    /// Finish writing the blob
    ///
    /// Finalizes the SHA256 hash, renames the file to the hex digest of that hash,
    /// and returns a descriptor of the blob.
    pub fn finish(
        self,
        media_type: MediaType,
        annotations: Option<HashMap<String, String>>,
        layout_dir: Option<&Path>,
    ) -> Result<Descriptor> {
        // Finalize the hash
        let sha256 = format!("{:x}", self.hash.finalize());

        // Rename the blob file to its hash
        let blob_path = blob_path_safe(&format!("sha256:{}", sha256))?;
        rename(self.file_path, &blob_path)?;

        // Create a symlink from the layout dir to the shared blobs dir
        if let Some(layout_dir) = layout_dir {
            blob_symlink(&blob_path, layout_dir)?;
        }

        // Build the descriptor to return
        let mut descriptor = DescriptorBuilder::default()
            .media_type(media_type)
            .size(self.bytes as i64)
            .digest(format!("sha256:{}", sha256));
        if let Some(annotations) = annotations {
            descriptor = descriptor.annotations(annotations)
        }
        let descriptor = descriptor.build()?;

        Ok(descriptor)
    }

    /// Write an object as a JSON based media type
    pub fn write_json<S: serde::Serialize>(
        object: &S,
        media_type: MediaType,
        annotations: Option<HashMap<String, String>>,
        layout_dir: Option<&Path>,
    ) -> Result<Descriptor> {
        let mut writer = Self::new()?;
        serde_json::to_writer_pretty(&mut writer, object)?;
        writer.finish(media_type, annotations, layout_dir)
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
