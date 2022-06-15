//! Storage of images and blobs on local disk

use std::{
    collections::HashMap,
    env,
    fs::{create_dir_all, read_to_string, write},
    path::{Path, PathBuf},
};

use common::{
    chrono::{DateTime, Utc},
    chrono_humanize::HumanTime,
    dirs,
    eyre::{bail, eyre, Result},
    itertools::Itertools,
    once_cell::sync::Lazy,
    serde::{Deserialize, Serialize},
    serde_json,
    tokio::sync::RwLock,
    tracing,
};
use fs_utils::symlink_file;
use oci_spec::image::ImageManifest;
use path_utils::pathdiff::diff_paths;

use crate::image_reference::ImageReference;

/// Get the directory of the image and blob storage cache
pub fn cache_dir() -> PathBuf {
    let user_cache_dir = dirs::cache_dir().unwrap_or_else(|| env::current_dir().unwrap());
    match env::consts::OS {
        "macos" | "windows" => user_cache_dir.join("Stencila").join("Images-Cache"),
        _ => user_cache_dir.join("stencila").join("images"),
    }
}

/// Split a digest string into its algorithm and hash parts
///
/// See https://github.com/opencontainers/image-spec/blob/main/descriptor.md#digests
pub fn digest_to_parts(digest: &str) -> (&str, &str) {
    let mut parts = digest.splitn(2, ':');
    let first = parts.next();
    let second = parts.next();
    match (first, second) {
        (Some(algo), Some(hash)) => (algo, hash),
        _ => ("sha256", digest),
    }
}

/// Path of root directory for image layout directories
static IMAGES_DIR: Lazy<PathBuf> = Lazy::new(|| cache_dir().join("images"));

/// Get the path to a image based on its digest
pub fn image_path(digest: &str) -> PathBuf {
    let (algo, hash) = digest_to_parts(digest);
    IMAGES_DIR.join(algo).join(hash)
}

/// Get the path to a image based on its digest and ensure it is created
pub fn image_path_safe(digest: &str) -> Result<PathBuf> {
    let path = image_path(digest);
    path.parent().map(create_dir_all);
    Ok(path)
}

pub fn write_oci_layout_file(layout_dir: &Path) -> Result<()> {
    std::fs::write(
        layout_dir.join("oci-layout"),
        r#"{"imageLayoutVersion": "1.0.0"}"#,
    )?;
    Ok(())
}

/// A persistent mapping of image references to hash (hex encoded SHA256 hash i.e. not prefixed with `sha256`)
pub struct ImagesMap {
    inner: HashMap<String, ImageInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct ImageInfo {
    /// The id of the image (the digest of the config)
    pub(crate) id: String,

    /// When the image was created locally
    pub(crate) created: DateTime<Utc>,
}

pub static IMAGES_MAP: Lazy<RwLock<ImagesMap>> = Lazy::new(|| RwLock::new(ImagesMap::read()));

impl ImagesMap {
    /// Get the path of the image map
    fn path() -> PathBuf {
        cache_dir().join("images.json")
    }

    /// Read the image map from disk
    fn read() -> Self {
        let path = Self::path();

        let inner = if path.exists() {
            match read_to_string(&path)
                .map_err(|error| eyre!(error))
                .and_then(|json| serde_json::from_str(&json).map_err(|error| eyre!(error)))
            {
                Ok(inner) => inner,
                Err(error) => {
                    tracing::warn!("While reading {}: {}", path.display(), error);
                    HashMap::new()
                }
            }
        } else {
            HashMap::new()
        };

        ImagesMap { inner }
    }

    /// Write the image map to disk
    fn write(&self) -> Result<()> {
        let path = Self::path();
        create_dir_all(path.parent().expect("Path should always have a parent"))?;

        let json = serde_json::to_string_pretty(&self.inner)?;
        write(&path, json)?;

        Ok(())
    }

    /// List all images in the images map (as a hash map and a Markdown table; for CLI inspection)
    pub fn list(&self) -> (HashMap<String, ImageInfo>, String) {
        let rows = self
            .inner
            .iter()
            .sorted_by(|a, b| a.1.created.cmp(&b.1.created))
            .rev()
            .map(|(reference, info)| {
                format!(
                    "|{}|{}|{}|",
                    reference,
                    &info.id[..19],
                    HumanTime::from(info.created)
                )
            })
            .collect_vec();

        let md = if rows.is_empty() {
            "*No images built or pulled yet*".to_string()
        } else {
            format!(
                r"
|-----------|----|---------|
| Reference | Id | Created |
|-----------|----|---------|
{}
|-----------|----|---------|
",
                rows.join("\n")
            )
        };

        (self.inner.clone(), md)
    }

    /// Insert an entry into the images map
    pub fn insert(&mut self, reference: &str, digest: &str) -> Result<()> {
        let reference: ImageReference = reference.parse()?;
        let image = ImageInfo {
            id: digest.to_string(),
            created: Utc::now(),
        };
        self.inner
            .insert(reference.to_string_tag_or_latest(), image);
        self.write()?;

        Ok(())
    }

    /// Remove an entry from the images map
    pub fn remove(&mut self, reference: &str) -> Result<Option<String>> {
        if let Ok(reference) = reference.parse::<ImageReference>() {
            if let Some(info) = self.inner.remove(&reference.to_string_tag_or_latest()) {
                self.write()?;
                return Ok(Some(info.id));
            }
        }

        let (algo, hash) = digest_to_parts(reference);
        let id = [algo, ":", hash].concat();
        if hash.len() < 2 {
            bail!("Please provide a valid image reference or at least two characters of image id hash")
        }

        let mut remove_reference = String::new();
        for (reference, info) in &self.inner {
            if info.id.starts_with(&id) {
                remove_reference = reference.clone();
                break;
            }
        }
        if !remove_reference.is_empty() {
            if let Some(info) = self.inner.remove(&remove_reference) {
                self.write()?;
                return Ok(Some(info.id));
            }
        }

        Ok(None)
    }

    /// Get the image info
    pub fn get_id(&self, reference: &str) -> Option<ImageInfo> {
        if let Ok(reference) = reference.parse::<ImageReference>() {
            if let Some(info) = self.inner.get(&reference.to_string_tag_or_latest()) {
                return Some(info.clone());
            }
        }

        let (algo, hash) = digest_to_parts(reference);
        let id = [algo, ":", hash].concat();

        let _remove_reference = String::new();
        for info in self.inner.values() {
            if info.id.starts_with(&id) {
                return Some(info.clone());
            }
        }

        None
    }
}

/// Path of directory of shared blobs (that are symlinked to from image dirs)
static BLOBS_DIR: Lazy<PathBuf> = Lazy::new(|| cache_dir().join("blobs"));

/// Get the path to a blob based on its digest
pub fn blob_path(digest: &str) -> PathBuf {
    let (algo, hash) = digest_to_parts(digest);
    BLOBS_DIR.join(algo).join(hash)
}

/// Get the path to a blob based on its digest and ensure its parent directory is created
pub fn blob_path_safe(digest: &str) -> Result<PathBuf> {
    let path = blob_path(digest);
    path.parent().map(create_dir_all);
    Ok(path)
}

/// Create a symlink to a blob within a layout dir
pub fn blob_symlink(blob_path: &Path, layout_dir: &Path) -> Result<()> {
    let hex = blob_path
        .file_name()
        .expect("Blob path does not have filename");

    let layout_blobs_dir = layout_dir.join("blobs").join("sha256");
    create_dir_all(&layout_blobs_dir)?;

    let layout_blob_path = layout_blobs_dir.join(hex);

    let relative_path = match diff_paths(&blob_path, &layout_blob_path) {
        Some(link) => link,
        None => bail!("Unable to create relative link"),
    };
    let relative_path: PathBuf = relative_path.components().skip(1).collect();

    symlink_file(relative_path, &layout_blob_path)?;

    Ok(())
}

/// A persistent mapping of blobs to the registries and repositories they occur in
///
/// Used for [Cross Repository Blob Mounting](https://github.com/opencontainers/distribution-spec/blob/main/spec.md#mounting-a-blob-from-another-repository)
/// which allows for blobs to be pushed to a registry without them being pulled first (if they already exist on the registry).
pub struct BlobsMap {
    inner: HashMap<String, Vec<(String, String)>>,
}

pub static BLOBS_MAP: Lazy<RwLock<BlobsMap>> = Lazy::new(|| RwLock::new(BlobsMap::read()));

impl BlobsMap {
    /// Get the path of the blob map
    fn path() -> PathBuf {
        cache_dir().join("blobs.json")
    }

    /// Read the blob map from disk
    fn read() -> Self {
        let path = Self::path();

        let inner = if path.exists() {
            match read_to_string(&path)
                .map_err(|error| eyre!(error))
                .and_then(|json| serde_json::from_str(&json).map_err(|error| eyre!(error)))
            {
                Ok(inner) => inner,
                Err(error) => {
                    tracing::warn!("While reading {}: {}", path.display(), error);
                    HashMap::new()
                }
            }
        } else {
            HashMap::new()
        };

        BlobsMap { inner }
    }

    /// Write the blob map to disk
    fn write(&self) -> Result<()> {
        let path = Self::path();
        create_dir_all(path.parent().expect("Path should always have a parent"))?;

        let json = serde_json::to_string_pretty(&self.inner)?;
        write(&path, json)?;

        Ok(())
    }

    /// Insert an entry into the blob map
    ///
    /// # Arguments
    ///
    /// - `digest`: the digest of the blob
    /// - `registry`: the registry that the blob is known to exist on
    /// - `repository`: the repository that the blob is known to exist in
    fn insert(&mut self, digest: &str, registry: &str, repository: &str) {
        let pairs = self.inner.entry(digest.to_string()).or_default();
        let pair = (registry.to_string(), repository.to_string());
        if !pairs.contains(&pair) {
            pairs.push(pair);
        }
    }

    /// Insert entires into the blob map for an image manifest
    ///
    /// # Arguments
    ///
    /// - `manifest`: the manifest to insert
    /// - `digest`: the digest of the manifest
    /// - `registry`: the registry that the blob is known to exist on
    /// - `repository`: the repository that the blob is known to exist in
    pub fn insert_manifest(
        &mut self,
        manifest: &ImageManifest,
        digest: &str,
        registry: &str,
        repository: &str,
    ) -> Result<()> {
        self.insert(digest, registry, repository);
        self.insert(manifest.config().digest(), registry, repository);
        for descriptor in manifest.layers() {
            self.insert(descriptor.digest(), registry, repository)
        }
        self.write()
    }

    /// Get the registry and repository (if any) for a blob
    ///
    /// # Arguments
    ///
    /// - `digest`: the digest of the blob
    pub fn get_registry_and_repo(&self, digest: &str) -> Option<&(String, String)> {
        self.inner.get(digest).and_then(|pairs| pairs.first())
    }

    /// Get the repository (if any) for a blob on a given registry
    ///
    /// # Arguments
    ///
    /// - `digest`: the digest of the blob
    /// - `registry`: the registry the blob exists on
    pub fn get_repo(&self, digest: &str, registry: &str) -> Option<String> {
        self.inner.get(digest).and_then(|pairs| {
            pairs.iter().find_map(|pair| match pair.0 == registry {
                true => Some(pair.1.clone()),
                false => None,
            })
        })
    }
}
