use std::{env, path::Path};

use bytes::{Bytes, BytesMut};
use bytesize::MIB;
use oci_spec::image::{
    Descriptor, ImageConfiguration, ImageIndex, ImageIndexBuilder, ImageManifest, MediaType,
    ToDockerV2S2, SCHEMA_VERSION,
};

use common::{
    eyre::{bail, Result},
    futures,
    serde::{de::DeserializeOwned, Deserialize},
    serde_json,
    tempfile::tempdir,
    tokio::{
        fs::{create_dir_all, metadata, read_to_string, remove_dir_all, rename, write, File},
        io::{self, AsyncReadExt, AsyncWriteExt, BufWriter},
    },
    tracing,
};
use hash_utils::str_sha256_hex;
use http_utils::{reqwest::Method, reqwest_middleware::RequestBuilder, CLIENT};

use crate::{
    blob_writer::BlobWriter,
    image_reference::ImageReference,
    storage::{
        blob_path, blob_path_safe, blob_symlink, image_path, image_path_safe,
        write_oci_layout_file, BLOBS_MAP, IMAGES_MAP,
    },
    utils::unique_string,
};

/// Pull an image to local storage from a registry
pub async fn pull(from: &str) -> Result<ImageReference> {
    let mut from: ImageReference = from.parse()?;

    let client = Client::new(&from.registry, &from.repository, None).await?;
    let digest = client.pull_image(&from.digest_or_tag_or_latest()).await?;

    from.digest = Some(digest);

    Ok(from)
}

/// Push an image from local storage, or from another registry, to a registry
pub async fn push(from: &str, to: Option<&str>, force_direct: bool) -> Result<ImageReference> {
    let from: ImageReference = from.parse()?;

    let to: ImageReference = match to {
        Some(to) => to.parse()?,
        None => from.clone(),
    };

    let client = Client::new(&to.registry, &to.repository, None).await?;

    // Check if image with `from` reference is available locally and if so use it
    if !force_direct {
        let images_map = IMAGES_MAP.read().await;
        if let Some(info) = images_map.get_id(&from.to_string_tag_or_latest()) {
            let layout_dir = image_path(&info.id);
            if layout_dir.exists() {
                client
                    .push_image(&layout_dir, &to.digest_or_tag_or_latest())
                    .await?;
                return Ok(to);
            }
        }
    }

    // Push image directly from one repository to another
    client
        .push_image_direct(&from, &to.digest_or_tag_or_latest())
        .await?;
    Ok(to)
}

/// A client that implements the [OCI Distribution Specification](https://github.com/opencontainers/distribution-spec/blob/main/spec.md)
/// for pulling and pushing images from a container registry
pub struct Client {
    /// URL of the image registry e.g. `registry.fly.io`, `localhost:5000`
    registry: String,

    /// Name of the image repository e.g. `library/hello-world`
    repository: String,

    /// Token used to authenticate requests
    token: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(crate = "common::serde")]
#[allow(dead_code)]
struct DockerAuthToken {
    token: String,
    expires_in: u32,
    issued_at: String,
}

impl Client {
    /// Create a new client
    pub async fn new(registry: &str, repository: &str, token: Option<&str>) -> Result<Self> {
        let token = match token {
            None => match registry {
                "docker.io" => {
                    // Get a temporary access token (at time of writing they last 5 minutes)
                    let mut request = CLIENT.get(
                            format!("https://auth.docker.io/token?service=registry.docker.io&scope=repository:{}:pull", repository)
                        );
                    // If possible use a Docker Hub username and password (the password is preferably an access token) for
                    // higher rate limits and push access
                    let username =
                        env::var("DOCKER_HUB_USERNAME").or_else(|_| env::var("DOCKER_USERNAME"));
                    let password =
                        env::var("DOCKER_HUB_PASSWORD").or_else(|_| env::var("DOCKER_PASSWORD"));
                    if let (Ok(username), Ok(password)) = (username, password) {
                        request = request.basic_auth(username, Some(password));
                    }
                    let response = request.send().await?.error_for_status()?;
                    let token: DockerAuthToken = response.json().await?;
                    Some(token.token)
                }
                "fly.io" => env::var("FLY_API_TOKEN")
                    .or_else(|_| env::var("FLY_TOKEN"))
                    .ok(),
                _ => None,
            },
            Some(token) => Some(token.to_string()),
        };

        Ok(Self {
            registry: registry.to_string(),
            repository: repository.to_string(),
            token,
        })
    }

    /// Pull an image from the registry to the local image store
    ///
    /// Returns the digest of the image (which is the digest of it's config).
    pub async fn pull_image(&self, reference: &str) -> Result<String> {
        // If reference is a digest, and the image already exists, return early
        if reference.starts_with("sha256:") {
            let image_path = image_path(reference);
            if image_path.exists() {
                tracing::info!("Image with digest `{}` already pulled", reference);
                return Ok(reference.to_string());
            }
        }

        // Create a temporary directory to the pull the image into
        let layout_dir = image_path_safe(&format!("temp:{}", unique_string()))?;

        // Pull the manifest to get descriptors for config and layers
        let (manifest, manifest_descriptor) =
            self.pull_manifest(reference, Some(&layout_dir)).await?;

        // If the image was already pulled, return early
        let config_digest = manifest.config().digest();
        let image_path = image_path(config_digest);
        if image_path.exists() {
            remove_dir_all(layout_dir).await?;
            tracing::info!("Image with digest `{}` already pulled", config_digest);
            return Ok(config_digest.to_string());
        }

        // Pull config and layers, in parallel, into the dir using descriptors
        let config = vec![manifest.config().clone()];
        let layers = manifest.layers();
        let descriptors = config.iter().chain(layers.iter());
        let futures =
            descriptors.map(|descriptor| self.pull_blob_via(descriptor, Some(&layout_dir)));
        futures::future::try_join_all(futures).await?;

        // Write the index.json file
        let index = ImageIndexBuilder::default()
            .schema_version(SCHEMA_VERSION)
            .media_type(MediaType::ImageIndex)
            .manifests([manifest_descriptor])
            .build()?;
        write(
            layout_dir.join("index.json"),
            serde_json::to_string_pretty(&index)?,
        )
        .await?;

        // Write the oci-layout file
        write_oci_layout_file(&layout_dir)?;

        // Now we know the image digest, rename the directory
        create_dir_all(&image_path).await?;
        rename(layout_dir, image_path).await?;

        // Add an entry in the images map with a complete reference
        let mut images = IMAGES_MAP.write().await;
        let reference = ImageReference {
            registry: self.registry.clone(),
            repository: self.repository.clone(),
            tag: Some(reference.to_string()),
            ..Default::default()
        };
        images.insert(&reference.to_string(), config_digest)?;

        Ok(config_digest.to_string())
    }

    /// Pull a manifest from the registry to the local image store.
    ///
    /// Returns a [`ImageManifest`] and the digest of the manifest.
    ///
    /// In accordance with the OCI spec, if the `Docker-Content-Digest` header is provided, it is verified
    /// against the digest of the downloaded content of the manifest.
    /// See https://github.com/opencontainers/distribution-spec/blob/main/spec.md#pulling-manifests
    pub async fn pull_manifest(
        &self,
        reference: &str,
        layout_dir: Option<&Path>,
    ) -> Result<(ImageManifest, Descriptor)> {
        // If reference is a digest, and the blob already exists, return early
        if reference.starts_with("sha256:") {
            let blob_path = blob_path(reference);
            if blob_path.exists() {
                tracing::info!("Manifest with digest `{}` already pulled", reference);

                let json = read_to_string(blob_path).await?;
                let manifest: ImageManifest = serde_json::from_str(&json)?;

                let descriptor = Descriptor::new(
                    MediaType::ImageManifest,
                    json.len().try_into()?,
                    str_sha256_hex(&json),
                );

                return Ok((manifest, descriptor));
            }
        }

        tracing::info!(
            "Pulling manifest from `{}/{}:{}`",
            self.registry,
            self.repository,
            reference
        );

        // Fetch the manifest from the registry and check digest

        let response = self
            .get(&["/manifests/", reference].concat())
            .header("Accept", MediaType::ImageManifest.to_string())
            // Currently required for compatibility with Docker registry...
            .header("Accept", MediaType::ImageManifest.to_docker_v2s2()?)
            .send()
            .await?
            .error_for_status()?;

        let headers = response.headers().clone();
        let json = response.text().await?;

        let digest = format!("sha256:{}", str_sha256_hex(&json));
        if let Some(provided_digest) = headers.get("Docker-Content-Digest") {
            let provided_digest = provided_digest.to_str()?;
            if provided_digest != digest {
                bail!("Digest in the `Docker-Content-Digest` header differs to that of the manifest content ({} != {})", provided_digest, digest)
            }
        }

        let manifest: ImageManifest = serde_json::from_str(&json)?;

        // Write the manifest for reuse later
        let descriptor =
            BlobWriter::write_json(&manifest, MediaType::ImageManifest, None, layout_dir)?;

        // Register the manifest, and it's layers, in the blob map for CRBM
        let mut blob_map = BLOBS_MAP.write().await;
        blob_map.insert_manifest(&manifest, &digest, &self.registry, &self.repository)?;

        Ok((manifest, descriptor))
    }

    /// Get an image config from the registry
    pub async fn pull_config(
        &self,
        manifest: &ImageManifest,
        layout_dir: Option<&Path>,
    ) -> Result<ImageConfiguration> {
        let descriptor = manifest.config();
        let config = self
            .pull_blob_into::<ImageConfiguration>(descriptor, layout_dir)
            .await?;
        Ok(config)
    }

    /// Pull a blob from the registry to a local file
    ///
    /// Writes the blob into `BLOBS_DIR` and then symlinks to it from the image dir (if any).
    /// Uses a [`BufWriter`] to avoid many small writes to the file (for each downloaded chunk).
    ///
    /// If the blob has already been pulled then will not re-pull it.
    ///
    /// See https://github.com/opencontainers/distribution-spec/blob/main/spec.md#pulling-blobs
    pub async fn pull_blob(
        &self,
        digest: &str,
        size: Option<i64>,
        layout_dir: Option<&Path>,
    ) -> Result<()> {
        let blob_path = blob_path_safe(digest)?;
        if !blob_path.exists() {
            tracing::info!(
                "Getting blob `{}` from `{}/{}`",
                digest,
                self.registry,
                self.repository
            );

            let mut response = self
                .get(&["/blobs/", digest].concat())
                .header("Accept", "application/octet-stream")
                .send()
                .await?
                .error_for_status()?;

            if let Some(size) = size {
                if let Some(length) = response
                    .headers()
                    .get("Content-Length")
                    .and_then(|length| length.to_str().ok())
                    .and_then(|length| length.parse::<i64>().ok())
                {
                    if length != size {
                        bail!(
                            "Content-Length header is different from size in descriptor ({} != {})",
                            length,
                            size
                        )
                    }
                }
            }

            // Write the blob to file
            let file = File::create(&blob_path).await?;
            let mut buffer = BufWriter::new(file);
            while let Some(chunk) = response.chunk().await? {
                buffer.write_all(&chunk).await?;
            }
            buffer.flush().await?;
        } else {
            tracing::info!("Blob `{}` already pulled", digest);
        }

        // Create a symlink from the layout dir to the shared blobs dir (like `BlobWriter` does)
        if let Some(layout_dir) = layout_dir {
            blob_symlink(&blob_path, layout_dir)?;
        }

        Ok(())
    }

    /// Pull a blob from the registry via a [`Descriptor`]
    pub async fn pull_blob_via(
        &self,
        descriptor: &Descriptor,
        layout_dir: Option<&Path>,
    ) -> Result<()> {
        self.pull_blob(descriptor.digest(), Some(descriptor.size()), layout_dir)
            .await
    }

    /// Pull a blob from the registry via a [`Descriptor`] and return it as a type
    pub async fn pull_blob_into<T: DeserializeOwned>(
        &self,
        descriptor: &Descriptor,
        layout_dir: Option<&Path>,
    ) -> Result<T> {
        self.pull_blob_via(descriptor, layout_dir).await?;

        let blob_path = blob_path(descriptor.digest());
        let json = read_to_string(blob_path).await?;
        let object = serde_json::from_str(&json)?;

        Ok(object)
    }

    /// Push an image from a local OCI layout directory
    ///
    /// Pushes an image from a local directory to the registry. The blobs in `<image_dir>/blobs/sha256` are uploaded first
    /// using `push_blob`. Once that is complete the manifest is uploaded using `push_manifest`.
    ///
    /// # Arguments
    ///
    /// - `layout_dir`: the image directory following the [OCI Image Layout](https://github.com/opencontainers/image-spec/blob/main/image-layout.md) spec
    /// - `reference`: a reference for the image (usually a tag)
    pub async fn push_image(&self, layout_dir: &Path, reference: &str) -> Result<()> {
        let index_path = layout_dir.join("index.json");
        let index_json = read_to_string(index_path).await?;
        let index: ImageIndex = serde_json::from_str(&index_json)?;

        let manifest_descriptor = index.manifests().get(0).unwrap();
        self.push_manifest(reference, layout_dir, manifest_descriptor.digest())
            .await?;

        Ok(())
    }

    /// Push an image from one repository to another
    pub async fn push_image_direct(&self, from: &ImageReference, reference: &str) -> Result<()> {
        let layout_dir_temp = tempdir()?;
        let layout_dir = layout_dir_temp.path();

        let client = Client::new(&from.registry, &from.repository, None).await?;
        let (.., descriptor) = client
            .pull_manifest(&from.digest_or_tag_or_latest(), Some(layout_dir))
            .await?;

        self.push_manifest(reference, layout_dir, descriptor.digest())
            .await?;

        Ok(())
    }

    /// Push a manifest from a local file to the registry
    ///
    /// See https://github.com/opencontainers/distribution-spec/blob/main/spec.md#pushing-manifests.
    pub async fn push_manifest(
        &self,
        reference: &str,
        layout_dir: &Path,
        digest: &str,
    ) -> Result<()> {
        tracing::info!(
            "Pushing manifest to `{}/{}:{}`",
            self.registry,
            self.repository,
            reference
        );

        let manifest_path = blob_path(digest);
        let manifest_json = read_to_string(manifest_path).await?;
        let manifest: ImageManifest = serde_json::from_str(&manifest_json)?;

        self.push_blob(layout_dir, manifest.config().digest())
            .await?;

        for layer_descriptor in manifest.layers() {
            self.push_blob(layout_dir, layer_descriptor.digest())
                .await?
        }

        let media_type = manifest
            .media_type()
            .clone()
            .unwrap_or(MediaType::ImageManifest)
            .to_string();

        let response = self
            .put(&["/manifests/", reference].concat())
            .header("Content-Type", media_type)
            .body(manifest_json)
            .send()
            .await?;
        if let Err(..) = response.error_for_status_ref() {
            bail!(
                "While pushing manifest: {} {}",
                response.status(),
                response.text().await?
            );
        }

        // Register the manifest, and it's layers, in the blob map for CRBM
        let mut blob_map = BLOBS_MAP.write().await;
        blob_map.insert_manifest(&manifest, digest, &self.registry, &self.repository)?;

        Ok(())
    }

    /// Push a blob from a local file to the registry
    ///
    /// See https://docs.docker.com/registry/spec/api/#pushing-a-layer for a description of the flow.
    /// See also https://github.com/opencontainers/distribution-spec/blob/main/spec.md#pushing-blobs.
    pub async fn push_blob(&self, layout_dir: &Path, digest: &str) -> Result<()> {
        // Check that the blob actually needs to be pushed. We avoid caching because if we do
        // use a cached response and the blob was deleted in the meantime (or a new localhost registry
        // started during development) we'll get a "blob unknown to registry" error below.
        let response = self.head(["/blobs/", digest].concat()).send().await?;
        if response.status() == 200 {
            tracing::info!(
                "Blob `{}` already exists in `{}/{}`",
                digest,
                self.registry,
                self.repository
            );
            return Ok(());
        }

        // Check to see if the blob has a known repository on the registry
        let blob_map = BLOBS_MAP.read().await;
        let other_repository = blob_map.get_repo(digest, &self.registry);
        drop(blob_map);

        // Initiate an upload with Cross Repository Blob Mounting if possible
        // Note: according to the OCI spec "The registry MAY treat the from parameter as optional,
        // and it MAY cross-mount the blob if it can be found.". So it will always pay to try.
        let from = other_repository
            .clone()
            .map(|repository| ["&from=", &repository].concat())
            .unwrap_or_default();
        let path = format!("/blobs/uploads/?mount={}{}", digest, from);
        let response = self
            .post(path)
            .header("Content-Length", "0")
            .send()
            .await?
            .error_for_status()?;
        if response.status() == 201 {
            // Successful mount
            tracing::info!(
                "Mounted blob `{}` from `{}/{}`",
                digest,
                self.registry,
                other_repository
                    .clone()
                    .unwrap_or_else(|| "<unknown>".to_string())
            );
            return Ok(());
        }

        // Registry does not support cross-repository mounting or is unable to mount the blob
        // "This indicates that the upload session has begun and that the client MAY proceed with the upload."

        let blob_path = blob_path(digest);
        if !blob_path.exists() {
            // The blob does not exist (usually because it is in a manifest for a base image and thus
            // not pulled in case it does not need to be). So, pull the blob using the BLOB_MAP's
            // record of the registry and repo it belongs to.
            let blob_map = BLOBS_MAP.read().await;
            if let Some((registry, repository)) = blob_map.get_registry_and_repo(digest) {
                let client = Client::new(registry, repository, None).await?;
                client.pull_blob(digest, None, Some(layout_dir)).await?;
            } else {
                bail!(
                    "Blob with digest is not in image layout directory or blob map: {}",
                    digest
                )
            }
        }

        tracing::info!(
            "Pushing blob `{}` to `{}/{}`",
            digest,
            self.registry,
            self.repository
        );

        let mut blob_file = File::open(&blob_path).await?;
        let blob_length = metadata(&blob_path).await?.len() as usize;

        let mut upload_url = match response.headers().get("Location") {
            Some(url) => url.to_str()?.to_string(),
            None => bail!("Did not receive upload URL from registry"),
        };

        // It is not clear what the optimum chunk size here but if too small results
        // in many requests which is slow.
        const CHUNK_SIZE: usize = 50 * MIB as usize;

        // If the blob is less than the maximum chunk size then upload it "monolithically"
        // rather than in chunks (to reduce the number of requests)
        if blob_length <= CHUNK_SIZE {
            let mut bytes = Vec::with_capacity(blob_length);
            io::copy(&mut blob_file, &mut bytes).await?;

            let upload_url = [
                upload_url.as_str(),
                if upload_url.contains('?') { "&" } else { "?" },
                "digest=",
                digest,
            ]
            .concat();

            let response = self
                .put(upload_url)
                .header("Content-Type", "application/octet-stream")
                .body(bytes)
                .send()
                .await?;
            if response.status() != 201 {
                bail!("While uploading blob: {}", response.text().await?)
            }

            return Ok(());
        }

        // Read the blob in chunks and upload each
        let mut buffer = BytesMut::with_capacity(CHUNK_SIZE);
        let mut chunk_start = 0;
        while let Ok(bytes_read) = blob_file.read_buf(&mut buffer).await {
            // If there are still bytes to be read but the buffer is less than the chunk size
            // then continue the next iteration of the loop
            if bytes_read > 0 && buffer.len() < CHUNK_SIZE {
                continue;
            }

            let chunk_length = std::cmp::min(buffer.len(), CHUNK_SIZE);
            let chunk = buffer.split_to(chunk_length);

            let response = self
                .patch(upload_url)
                .header("Content-Type", "application/octet-stream")
                .header(
                    "Content-Range",
                    format!("{}-{}", chunk_start, chunk_start + chunk_length - 1),
                )
                .header("Content-Length", chunk_length)
                .body(Bytes::from(chunk))
                .send()
                .await?;
            if response.status() == 202 {
                // "Each consecutive chunk upload SHOULD use the <location> provided in the
                // response to the previous chunk upload."
                upload_url = match response.headers().get("Location") {
                    Some(value) => value.to_str()?.to_string(),
                    None => {
                        bail!("Response did not have `Location` header")
                    }
                }
            } else {
                bail!(
                    "While uploading chunk: {} {}",
                    response.status(),
                    response.text().await?
                )
            }

            // No more bytes to read so exit the loop
            if bytes_read == 0 {
                break;
            }

            chunk_start += chunk_length;
        }

        // Notify the registry that the upload is complete
        let upload_url = [
            upload_url.as_str(),
            if upload_url.contains('?') { "&" } else { "?" },
            "digest=",
            digest,
        ]
        .concat();
        self.put(upload_url).send().await?.error_for_status()?;

        Ok(())
    }

    /// Make a request to the registry
    fn request<S: AsRef<str>>(&self, method: Method, path: S) -> RequestBuilder {
        let path = path.as_ref();

        let url = if path.starts_with("http") {
            path.to_string()
        } else {
            let registry_url = if self.registry.starts_with("https://") {
                self.registry.clone()
            } else if self.registry.starts_with("localhost") {
                ["http://", &self.registry].concat()
            } else if self.registry == "docker.io" {
                "https://registry.hub.docker.com".to_string()
            } else if self.registry == "fly.io" {
                "https://registry.fly.io".to_string()
            } else {
                ["https://", &self.registry].concat()
            };
            [&registry_url, "/v2/", &self.repository, path].concat()
        };

        // Because we implement a caching mechanism for blobs, to avoid "double storing"
        // blobs, turn off caching by the default `cacache`
        let mut request = CLIENT
            .request(method, url)
            .header("Cache-Control", "no-store");

        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }

        request
    }

    /// Make a HEAD request to the registry
    fn head<S: AsRef<str>>(&self, path: S) -> RequestBuilder {
        self.request(Method::HEAD, path)
    }

    /// Make a GET request to the registry
    fn get<S: AsRef<str>>(&self, path: S) -> RequestBuilder {
        self.request(Method::GET, path)
    }

    /// Make a POST request to the registry
    fn post<S: AsRef<str>>(&self, path: S) -> RequestBuilder {
        self.request(Method::POST, path)
    }

    /// Make a PATCH request to the registry
    fn patch<S: AsRef<str>>(&self, path: S) -> RequestBuilder {
        self.request(Method::PATCH, path)
    }

    /// Make a PUT request to the registry
    fn put<S: AsRef<str>>(&self, path: S) -> RequestBuilder {
        self.request(Method::PUT, path)
    }
}

#[cfg(test)]
mod tests {
    use test_utils::{
        common::{tempfile::tempdir, tokio},
        // print_logs_level,
        skip_ci,
    };

    use super::*;

    /// Pull and push back the Docker `hello-world` image from the local registry
    ///
    /// To set up this test, run a registry container:
    ///
    ///    docker run -p5000:5000 registry
    ///
    /// Then push `library/hello-world` to that registry:
    ///
    ///    docker pull library/hello-world
    ///    docker tag hello-world localhost:5000/library/hello-world:latest
    ///    docker push localhost:5000/library/hello-world:latest
    #[ignore]
    #[tokio::test]
    async fn hello_world() -> Result<()> {
        skip_ci("Requires an image registry to be running locally");

        // print_logs_level(tracing::Level::INFO);

        let image_dir = tempdir()?;

        let client = Client::new("localhost:5000", "hello-world", None).await?;
        client.pull_image("latest").await?;
        client.push_image(image_dir.path(), "latest").await?;

        Ok(())
    }
}
