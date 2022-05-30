use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
};

use bytes::Bytes;
use bytesize::MIB;
use chrono::Utc;
use eyre::{bail, eyre, Result};
use oci_spec::image::{Descriptor, ImageConfiguration, ImageIndex, ImageManifest, MediaType};
use once_cell::sync::Lazy;
use serde::Deserialize;
use tokio::{
    fs::{create_dir_all, metadata, read_to_string, File},
    io::{self, AsyncReadExt, AsyncWriteExt, BufWriter},
    sync::RwLock,
};

use hash_utils::str_sha256_hex;
use http_utils::{
    reqwest::{Method, Response},
    reqwest_middleware::RequestBuilder,
    CLIENT,
};

pub const DOCKER_REGISTRY: &str = "registry.hub.docker.com";
pub const FLY_REGISTRY: &str = "registry.fly.io";

/// Get the directory of the cache
pub fn cache_dir() -> PathBuf {
    let user_cache_dir = dirs::cache_dir().unwrap_or_else(|| env::current_dir().unwrap());
    match env::consts::OS {
        "macos" | "windows" => user_cache_dir.join("Stencila").join("Images-Cache"),
        _ => user_cache_dir.join("stencila").join("images-cache"),
    }
}

/// A persistent mapping of blobs and which registries and repositories they occur in
///
/// Used for [Cross Repository Blob Mounting](https://github.com/opencontainers/distribution-spec/blob/main/spec.md#mounting-a-blob-from-another-repository)
struct BlobMap {
    inner: HashMap<String, Vec<(String, String)>>,
}

impl BlobMap {
    fn path() -> PathBuf {
        cache_dir().join("blob-map.json")
    }

    fn read() -> Self {
        let path = Self::path();

        let inner = if path.exists() {
            match std::fs::read_to_string(&path)
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

        BlobMap { inner }
    }

    fn write(&self) -> Result<()> {
        let path = Self::path();
        std::fs::create_dir_all(path.parent().expect("Path should always have a parent"))?;

        let json = serde_json::to_string_pretty(&self.inner)?;
        std::fs::write(&path, json)?;

        Ok(())
    }

    fn insert(&mut self, digest: &str, registry: &str, repository: &str) {
        let pairs = self.inner.entry(digest.to_string()).or_default();
        let pair = (registry.to_string(), repository.to_string());
        if !pairs.contains(&pair) {
            pairs.push(pair);
        }
    }

    fn insert_layers(
        &mut self,
        layers: &[Descriptor],
        registry: &str,
        repository: &str,
    ) -> Result<()> {
        for descriptor in layers {
            self.insert(descriptor.digest(), registry, repository)
        }
        self.write()?;
        Ok(())
    }

    fn get_registry_and_repo(&self, digest: &str) -> Option<&(String, String)> {
        self.inner.get(digest).and_then(|pairs| pairs.first())
    }

    fn get_repo(&self, digest: &str, registry: &str) -> Option<String> {
        self.inner.get(digest).and_then(|pairs| {
            pairs.iter().find_map(|pair| match pair.0 == registry {
                true => Some(pair.1.clone()),
                false => None,
            })
        })
    }
}

static BLOB_MAP: Lazy<RwLock<BlobMap>> = Lazy::new(|| RwLock::new(BlobMap::read()));

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
                DOCKER_REGISTRY => {
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
                FLY_REGISTRY => env::var("FLY_API_TOKEN")
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

    /// Push an image
    ///
    /// Pushes an image from a local directory to the registry. The blobs in `<image_dir>/blobs/sha256` are uploaded first
    /// using `push_blob`. Once that is complete the manifest is uploaded using `push_manifest`.
    ///
    /// # Arguments
    ///
    /// - `reference`: a reference for the image (usually a tag)
    /// - `image_dir`: the image directory following the [OCI Image Layout](https://github.com/opencontainers/image-spec/blob/main/image-layout.md) spec
    pub async fn push_image(&self, reference: &str, layout_dir: &Path) -> Result<()> {
        let index_path = layout_dir.join("index.json");
        let index_json = read_to_string(index_path).await?;
        let index: ImageIndex = serde_json::from_str(&index_json)?;

        let manifest_descriptor = index.manifests().get(0).unwrap();
        self.push_manifest(reference, layout_dir, manifest_descriptor.digest())
            .await?;

        Ok(())
    }

    /// Pull an image
    ///
    /// Pulls an image from the registry to a local directory. The inverse of `push_image`.
    pub async fn pull_image(&self, reference: &str, layout_dir: &Path) -> Result<()> {
        let manifest = self.pull_manifest(reference, layout_dir).await?;

        let mut futures = vec![self.pull_blob_via(layout_dir, manifest.config())];
        for layer in manifest.layers() {
            let future = self.pull_blob_via(layout_dir, layer);
            futures.push(future);
        }

        futures::future::try_join_all(futures).await?;

        Ok(())
    }

    /// Get a manifest from the registry
    ///
    /// See https://github.com/opencontainers/distribution-spec/blob/main/spec.md#pulling-manifests
    ///
    /// In accordance with that spec, if the `Docker-Content_Digest` header is provided, it is verified
    /// against the digest of the downloaded content of the manifest.
    pub async fn get_manifest<S: AsRef<str>>(
        &self,
        reference: S,
    ) -> Result<(ImageManifest, String)> {
        let reference = reference.as_ref();

        tracing::info!(
            "Pulling manifest from `{}/{}:{}`",
            self.registry,
            self.repository,
            reference
        );

        let response = self
            .get(&["/manifests/", reference].concat())
            .header("Accept", MediaType::ImageManifest.to_string())
            // Required for current version of Docker registry..
            .header(
                "Accept",
                "application/vnd.docker.distribution.manifest.v2+json",
            )
            .send()
            .await?
            .error_for_status()?;

        let headers = response.headers().clone();
        let json = response.text().await?;

        let digest = format!("sha256:{}", str_sha256_hex(&json));
        if let Some(provided_digest) = headers.get("Docker-Content_Digest") {
            let provided_digest = provided_digest.to_str()?;
            if provided_digest != digest {
                bail!("Digest in the `Docker-Content_Digest` header differs to that of the manifest content ({} != {})", provided_digest, digest)
            }
        }

        let manifest: ImageManifest = serde_json::from_str(&json)?;

        let mut blob_map = BLOB_MAP.write().await;
        blob_map.insert_layers(manifest.layers(), &self.registry, &self.repository)?;

        Ok((manifest, digest))
    }

    /// Pull a manifest from the registry to a local file and return it
    pub async fn pull_manifest<S: AsRef<str>, P: AsRef<Path>>(
        &self,
        _reference: S,
        _layout_dir: P,
    ) -> Result<ImageManifest> {
        todo!()
    }

    /// Push a manifest from a local file to the registry
    ///
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

        let manifest_path = Self::blob_path(layout_dir, digest)?;
        let manifest_json = read_to_string(manifest_path).await?;
        let manifest: ImageManifest = serde_json::from_str(&manifest_json)?;

        self.push_blob(layout_dir, manifest.config().digest())
            .await?;

        for layer_descriptor in manifest.layers() {
            self.push_blob(layout_dir, layer_descriptor.digest())
                .await?
        }

        let response = self
            .put(&["/manifests/", reference].concat())
            .header("Content-Type", MediaType::ImageManifest.to_string())
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

        let mut blob_map = BLOB_MAP.write().await;
        blob_map.insert_layers(manifest.layers(), &self.registry, &self.repository)?;

        Ok(())
    }

    /// Get an image config from the registry
    pub async fn get_config(&self, manifest: &ImageManifest) -> Result<ImageConfiguration> {
        let descriptor = manifest.config();
        let response = self
            .get_blob(descriptor.digest(), Some(descriptor.size()))
            .await?;
        let config: ImageConfiguration = response.json().await?;
        Ok(config)
    }

    /// Get the local filesystem path to a blob based on its digest
    fn blob_path(layout_dir: &Path, digest: &str) -> Result<PathBuf> {
        if let Some(suffix) = digest.strip_prefix("sha256:") {
            Ok(layout_dir.join("blobs").join("sha256").join(suffix))
        } else {
            bail!("Digest is not prefixed by `sha256:`")
        }
    }

    /// Get a blob from the registry
    ///
    /// See https://github.com/opencontainers/distribution-spec/blob/main/spec.md#pulling-blobs
    pub async fn get_blob(&self, digest: &str, size: Option<i64>) -> Result<Response> {
        tracing::info!(
            "Getting blob `{}` from `{}/{}`",
            digest,
            self.registry,
            self.repository
        );

        let response = self
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

        Ok(response)
    }

    /// Pull a blob from the registry to a local file
    /// 
    /// Uses a [`BufWriter`] to avoid many small writes to the file (for each downloaded chunk).
    pub async fn pull_blob(
        &self,
        layout_dir: &Path,
        digest: &str,
        size: Option<i64>,
    ) -> Result<()> {
        let mut response = self.get_blob(digest, size).await?;

        let blobs_dir = layout_dir.join("blobs").join("sha256");
        create_dir_all(&blobs_dir).await?;

        let blob_path = Self::blob_path(layout_dir, digest)?;
        let file = File::create(&blob_path).await?;

        tracing::info!(
            "Writing blob `{}` to `{}`",
            digest,
            blob_path.display()
        );
        
        let mut buffer = BufWriter::new(file);
        while let Some(chunk) = response.chunk().await? {
            buffer.write_all(&chunk).await?;
        }

        Ok(())
    }

    /// Pull a blob from the registry via a [`Descriptor`]
    pub async fn pull_blob_via(&self, layout_dir: &Path, descriptor: &Descriptor) -> Result<()> {
        self.pull_blob(layout_dir, descriptor.digest(), Some(descriptor.size()))
            .await
    }

    /// Push a blob from a local file to the registry
    ///
    /// See https://docs.docker.com/registry/spec/api/#pushing-a-layer for a description of the flow.
    /// See also https://github.com/opencontainers/distribution-spec/blob/main/spec.md#pushing-blobs.
    pub async fn push_blob(&self, layout_dir: &Path, digest: &str) -> Result<()> {
        // Check that the blob actually needs to be pushed. We avoid caching because if we do
        // use a cached response and the blob was deleted in the meantime (or a new locahost server
        // started during development, we'll get a "blob unknown to registry" error below.
        // In theory, the `Cache-Control: no-cache` header should work, but it didn't
        // so we use the current time as a query param to bust the cache.
        // See https://github.com/06chaynes/http-cache/issues/13
        let response = self
            .head(["/blobs/", digest, "?time=", &Utc::now().to_rfc3339()].concat())
            .header("Cache-Control", "no-cache")
            .send()
            .await?;
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
        let blob_map = BLOB_MAP.read().await;
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

        let blob_path = Self::blob_path(layout_dir, digest)?;

        if !blob_path.exists() {
            // The blob does not exist (usually because it is in a manifest for a base image and thus
            // not pulled in case it does not need to be). So, pull the blob using the BLOB_MAP's
            // record of the registry and repo it belongs to.
            let blob_map = BLOB_MAP.read().await;
            if let Some((registry, repository)) = blob_map.get_registry_and_repo(digest) {
                let client = Client::new(registry, repository, None).await?;
                client.pull_blob(layout_dir, digest, None).await?;
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
        const MAX_CHUNK_SIZE: usize = 100 * MIB as usize;

        // If the blob is less than the maximum chunk size then upload it "monolithically"
        // rather than in chunks (to reduce the number of requests)
        if blob_length <= MAX_CHUNK_SIZE {
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
        let mut buffer = vec![0u8; MAX_CHUNK_SIZE];
        let mut chunk_start = 0;
        while let Ok(chunk_length) = blob_file.read(&mut buffer[..]).await {
            if chunk_length == 0 {
                break;
            }

            let chunk = Bytes::copy_from_slice(&buffer[0..chunk_length]);

            let response = self
                .patch(upload_url)
                .header("Content-Type", "application/octet-stream")
                .header(
                    "Content-Range",
                    format!("{}-{}", chunk_start, chunk_start + chunk_length - 1),
                )
                .header("Content-Length", chunk_length)
                .body(chunk)
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
            } else {
                ["https://", &self.registry].concat()
            };
            [&registry_url, "/v2/", &self.repository, path].concat()
        };

        let mut request = CLIENT.request(method, url);

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
    use test_utils::{print_logs_level, skip_ci, tempfile::tempdir};

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

        print_logs_level(tracing::Level::INFO);

        let image_dir = tempdir()?;

        let client = Client::new("localhost:5000", "hello-world", None).await?;
        client.pull_image("latest", image_dir.path()).await?;
        client.push_image("latest", image_dir.path()).await?;

        Ok(())
    }
}
