use std::{collections::HashMap, env, path::Path};

use oci_spec::image::{
    Descriptor, History, HistoryBuilder, ImageConfiguration, ImageConfigurationBuilder,
    ImageIndexBuilder, ImageManifestBuilder, MediaType, RootFsBuilder, ToDockerV2S2,
    SCHEMA_VERSION,
};

use common::{
    chrono::Utc,
    eyre::{bail, Result},
    serde::Serialize,
    serde_json, tokio,
};

use crate::{
    blob_writer::BlobWriter,
    change_set::ChangeSet,
    distribution::{push, Client},
    image_reference::ImageReference,
    storage::{image_path, image_path_safe, write_oci_layout_file, IMAGES_MAP},
    utils::unique_string,
};

/// A container image
///
/// This is serializable mainly so that it can be inspected as JSON or YAML output from a CLI command.
#[derive(Serialize)]
#[serde(crate = "common::serde")]
pub struct Image {
    /// The [`ChangeSet`]s used to generate each image layer
    #[serde(skip)]
    change_sets: Vec<ChangeSet>,

    /// The image reference for this image
    reference: ImageReference,

    /// The image reference for the base image from which this image is derived
    ///
    /// Equivalent to the `FROM` directive of a Dockerfile.
    base: ImageReference,

    /// The format used when writing layers
    layer_format: MediaType,

    /// The format for the image manifest
    ///
    /// Defaults to `application/vnd.oci.image.manifest.v1+json`. However, for some registries it
    /// may be necessary to use `application/vnd.docker.distribution.manifest.v2+json` (which has
    /// the same underlying schema).
    manifest_format: String,
}

impl Image {
    /// Create a new image
    pub fn new(
        reference: &str,
        base: Option<&str>,
        change_sets: Vec<ChangeSet>,
        layer_format: Option<&str>,
        manifest_format: Option<&str>,
    ) -> Result<Self> {
        let reference = reference.parse()?;

        let base = match base
            .map(String::from)
            .or_else(|| std::env::var("STENCILA_IMAGE_BASE").ok())
            .or_else(|| std::env::var("STENCILA_IMAGE_REF").ok())
        {
            Some(var) => var.parse()?,
            None => bail!("Unable to resolve the base image"),
        };

        let layer_format = match layer_format {
            None | Some("tar+gzip") | Some("tgz") => MediaType::ImageLayerGzip,
            Some("tar+zstd") | Some("tzs") => MediaType::ImageLayerZstd,
            Some("tar") => MediaType::ImageLayer,
            _ => bail!("Unknown layer format"),
        };

        let manifest_format = match manifest_format {
            None | Some("oci") => MediaType::ImageManifest.to_string(),
            Some("v2s2") => MediaType::ImageManifest.to_docker_v2s2()?.to_string(),
            _ => bail!("Unknown manifest format"),
        };

        Ok(Self {
            change_sets,
            reference,
            base,
            layer_format,
            manifest_format,
        })
    }

    /// Get the [`ImageReference`] of the image
    pub fn reference(&self) -> &ImageReference {
        &self.reference
    }

    /// Get the [`ImageReference`] of the image's base
    pub fn base(&self) -> &ImageReference {
        &self.base
    }

    /// Fetches the manifest and configuration of the base image
    ///
    /// Used when writing the image because the DiffIDs (from the config) and the layers (from the
    /// manifest) and required for the config and manifest of this image.
    async fn get_base(&self) -> Result<(String, ImageConfiguration, Vec<Descriptor>)> {
        let client = Client::new(&self.base.registry, &self.base.repository, None).await?;
        let (manifest, manifest_descriptor) = client
            .pull_manifest(&self.base.digest_or_tag_or_latest(), None)
            .await?;

        let config = client.pull_config(&manifest, None).await?;
        let layers = manifest.layers().clone();

        Ok((manifest_descriptor.digest().to_string(), config, layers))
    }

    /// Write the image layer blobs and returns vectors of DiffIDs and layer descriptors
    async fn write_layers(
        &self,
        base_config: &ImageConfiguration,
        base_layers: Vec<Descriptor>,
        layout_dir: &Path,
    ) -> Result<(Vec<Descriptor>, Vec<String>, Vec<History>)> {
        let mut layers = base_layers;
        let mut diff_ids = base_config.rootfs().diff_ids().clone();
        let mut histories = base_config.history().clone();

        let client = Client::new(&self.base.registry, &self.base.repository, None).await?;
        for layer in &layers {
            client.pull_blob_via(layer, None).await?
        }

        for change_set in &self.change_sets {
            let (diff_id, layer) = change_set.write_layer(&self.layer_format, layout_dir)?;

            if diff_id == "<empty>" {
                continue;
            }

            diff_ids.push(diff_id);
            layers.push(layer);

            let (additions, deletions, modifications) = change_set.summarize();
            let comment = change_set.comment.clone().unwrap_or_else(|| {
                format!("Change set for {} with {additions} additions, {deletions} deletions, and {modifications} modifications.", change_set.source_dir.display())
            });
            let history = HistoryBuilder::default()
                .created(Utc::now().to_rfc3339())
                .created_by(format!(
                    "stencila {}",
                    env::args().skip(1).collect::<Vec<String>>().join(" ")
                ))
                .comment(comment)
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
        layout_dir: &Path,
    ) -> Result<Descriptor> {
        // Start with the config of the base image and override as necessary
        let mut config = base_config.config().clone().unwrap_or_default();

        // Set working directory
        config.set_working_dir(Some("/work".to_string()));

        // Get the environment variables in the base images
        let mut env: HashMap<String, String> = config
            .env()
            .clone()
            .unwrap_or_default()
            .iter()
            .map(|name_value| {
                let mut parts = name_value.splitn(2, '=');
                (
                    parts.next().unwrap_or_default().to_owned(),
                    parts.next().unwrap_or_default().to_owned(),
                )
            })
            .collect();

        // Add an env var for the ref of the image (used as the default `--from` image when building another image from this)
        env.insert("STENCILA_IMAGE_REF".to_string(), self.reference.to_string());

        let env: Vec<String> = env
            .iter()
            .map(|(name, value)| [name, "=", value].concat())
            .collect();
        config.set_env(Some(env));

        // Extend labels, including with the contents of an y `.image-labels` file in working dir
        let mut labels = config.labels().clone().unwrap_or_default();
        labels.insert(
            "io.stencila.version".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        );

        // Rootfs DiffIDs calculated above
        let rootfs = RootFsBuilder::default().diff_ids(diff_ids).build()?;

        let configuration = ImageConfigurationBuilder::default()
            .created(Utc::now().to_rfc3339())
            .os(env::consts::OS)
            // Not that arch should be one of the values listed at https://go.dev/doc/install/source#environment
            // and that `env::consts::ARCH` does not necessarily return that
            .architecture("amd64")
            .config(config)
            .rootfs(rootfs)
            .history(history)
            .build()?;

        BlobWriter::write_json(
            &configuration,
            MediaType::ImageConfig,
            None,
            Some(layout_dir),
        )
    }

    /// Write the image manifest blob
    ///
    /// Implements the [OCI Image Manifest Specification](https://github.com/opencontainers/image-spec/blob/main/manifest.md).
    /// Given that the manifest requires the descriptors for config and layers also calls `write_config` and `write_layers`.
    async fn write_manifest(&self, layout_dir: &Path) -> Result<(String, Descriptor, Descriptor)> {
        let (base_digest, base_config, base_layers) = self.get_base().await?;

        let (layers, diff_ids, history) = self
            .write_layers(&base_config, base_layers, layout_dir)
            .await?;

        let config_descriptor = self.write_config(&base_config, diff_ids, history, layout_dir)?;

        let manifest = ImageManifestBuilder::default()
            .schema_version(SCHEMA_VERSION)
            .media_type(self.manifest_format.as_str())
            .config(config_descriptor.clone())
            .layers(layers)
            .build()?;

        let manifest_descriptor =
            BlobWriter::write_json(&manifest, MediaType::ImageManifest, None, Some(layout_dir))?;

        Ok((base_digest, config_descriptor, manifest_descriptor))
    }

    /// Write the image `index.json`
    ///
    /// Implements the [OCI Image Index Specification](https://github.com/opencontainers/image-spec/blob/main/image-index.md).
    /// Updates both `self.ref_.digest` and `self.base.digest`.
    async fn write_index(&mut self, layout_dir: &Path) -> Result<Descriptor> {
        use tokio::fs;

        let (base_digest, config_descriptor, manifest_descriptor) =
            self.write_manifest(layout_dir).await?;

        self.base.digest = Some(base_digest.clone());
        self.reference.digest = Some(manifest_descriptor.digest().to_string());

        let annotations: HashMap<String, String> = [
            // Where appropriate use pre defined annotations
            // https://github.com/opencontainers/image-spec/blob/main/annotations.md#pre-defined-annotation-keys
            (
                "org.opencontainers.image.ref.name".to_string(),
                self.reference.to_string_tag_or_latest(),
            ),
            (
                "org.opencontainers.image.created".to_string(),
                Utc::now().to_rfc3339(),
            ),
            (
                "org.opencontainers.image.base.name".to_string(),
                self.base.to_string_tag_or_latest(),
            ),
            (
                "org.opencontainers.image.base.digest".to_string(),
                base_digest,
            ),
        ]
        .into();

        let index = ImageIndexBuilder::default()
            .schema_version(SCHEMA_VERSION)
            .media_type(MediaType::ImageIndex)
            .manifests([manifest_descriptor])
            .annotations(annotations)
            .build()?;
        fs::write(
            layout_dir.join("index.json"),
            serde_json::to_string_pretty(&index)?,
        )
        .await?;

        Ok(config_descriptor)
    }

    /// Write the image to the local image store
    ///
    /// Implements the [OCI Image Layout Specification](https://github.com/opencontainers/image-spec/blob/main/image-layout.md).
    ///
    /// Note that the `blobs/sha256` subdirectory may not have blobs for the base image (these
    /// are only pulled into that directory if necessary i.e. if the registry does not yet have them).
    pub async fn write(&mut self) -> Result<(String, String)> {
        use tokio::fs;

        // Create a temporary OCI layout directory
        let layout_dir = image_path(&format!("temp:{}", &unique_string()));

        // Write image into that directory
        let config_descriptor = self.write_index(&layout_dir).await?;
        write_oci_layout_file(&layout_dir)?;
        let config_digest = config_descriptor.digest();

        // Now we know the id of the image, rename the dir
        let image_dir = image_path_safe(config_digest)?;
        fs::rename(&layout_dir, image_dir).await?;

        // Add an entry in the images map
        let mut images = IMAGES_MAP.write().await;
        let reference = self.reference.to_string_tag_or_latest();
        let id = config_digest;
        images.insert(&reference, id)?;

        Ok((reference, id.to_owned()))
    }

    /// Push the image to a repository
    pub async fn push(&mut self) -> Result<()> {
        push(&self.reference.to_string_tag_or_latest(), None, false).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test that when an image is written to a directory that they directory conforms to
    /// the OCI Image Layout spec
    #[tokio::test]
    async fn image_write() -> Result<()> {
        let mut image = Image::new("test", Some("ubuntu"), vec![], None, None)?;
        let (reference, id) = image.write().await?;

        let layout_dir = image_path(&id);
        assert!(layout_dir.join("oci-layout").is_file());
        assert!(layout_dir.join("index.json").is_file());
        assert!(layout_dir.join("blobs").join("sha256").is_dir());

        let mut image_map = IMAGES_MAP.write().await;
        image_map.remove(&reference)?;

        Ok(())
    }
}
