use std::{
    collections::HashMap,
    io,
    path::{Path, PathBuf},
};

use eyre::Result;
use oci_spec::image::{Descriptor, DescriptorBuilder, MediaType};

use hash_utils::sha2::{Digest, Sha256};

use crate::utils::unique_string;

/// A writer that calculates the size and SHA256 hash of files as they are written
///
/// Writes blobs into the `blobs/sha256` subdirectory of an image directory and returns
/// an [OCI Content Descriptor](https://github.com/opencontainers/image-spec/blob/main/descriptor.md).
///
/// Allows use to do a single pass when writing files instead of reading them after writing in order
/// to generate the SHA256 signature.
pub struct BlobWriter {
    /// The path to the `blobs/sha256` subdirectory where the blob is written to
    blobs_dir: PathBuf,

    /// The media type of the blob
    media_type: MediaType,

    /// The temporary file name of the blob (used before we know its final name, which is its SHA256 checksum)
    file_name: PathBuf,

    /// The file the blob is written to
    file: std::fs::File,

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
    pub fn new<P: AsRef<Path>>(image_dir: P, media_type: MediaType) -> Result<Self> {
        use std::fs::{self, File};

        let blobs_dir = image_dir.as_ref().join("blobs").join("sha256");
        fs::create_dir_all(&blobs_dir)?;

        let filename = PathBuf::from(format!("temporary-{}", unique_string()));
        let file = File::create(blobs_dir.join(&filename))?;

        Ok(Self {
            blobs_dir,
            media_type,
            file_name: filename,
            file,
            bytes: 0,
            hash: Sha256::new(),
        })
    }

    /// Finish writing the blob
    ///
    /// Finalizes the SHA256 hash, renames the file to the hex digest of that hash,
    /// and returns a descriptor of the blob.
    pub fn finish(self, annotations: Option<HashMap<String, String>>) -> Result<Descriptor> {
        use std::fs;

        let sha256 = format!("{:x}", self.hash.finalize());

        fs::rename(
            self.blobs_dir.join(self.file_name),
            self.blobs_dir.join(&sha256),
        )?;

        let mut descriptor = DescriptorBuilder::default()
            .media_type(self.media_type)
            .size(self.bytes as i64)
            .digest(format!("sha256:{}", sha256));
        if let Some(annotations) = annotations {
            descriptor = descriptor.annotations(annotations)
        }
        let descriptor = descriptor.build()?;

        Ok(descriptor)
    }

    /// Write an object as a JSON based media type
    pub fn write_json<P: AsRef<Path>, S: serde::Serialize>(
        path: P,
        media_type: MediaType,
        object: &S,
        annotations: Option<HashMap<String, String>>,
    ) -> Result<Descriptor> {
        let mut writer = Self::new(path, media_type)?;
        serde_json::to_writer_pretty(&mut writer, object)?;
        writer.finish(annotations)
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
