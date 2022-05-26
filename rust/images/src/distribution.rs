use std::{env, fs, path::Path};

use bytes::Bytes;
use eyre::{bail, Result};
use oci_spec::image::{Descriptor, ImageConfiguration, ImageManifest};
use serde::Deserialize;
use tokio::{
    fs::File,
    io::{self, AsyncReadExt, AsyncWriteExt},
};

use http_utils::{
    reqwest::{Method, Response},
    reqwest_middleware::RequestBuilder,
    CLIENT,
};

pub const DOCKER_REGISTRY: &str = "registry.hub.docker.com";

pub const FLY_REGISTRY: &str = "registry.fly.io";

/// A client that implements the [OCI Distribution Specification](https://github.com/opencontainers/distribution-spec/blob/main/spec.md)
/// for pulling and pushing images from a container registry
pub struct Client {
    /// URL of the image registry e.g. `registry.fly.io`, `localhost:5000`
    registry: String,

    /// Name of the image e.g. `library/hello-world`
    image: String,

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
    pub async fn new(
        registry_host: &str,
        image_name: &str,
        registry_token: Option<&str>,
    ) -> Result<Self> {
        let registry_token = match registry_token {
            None => match registry_host {
                DOCKER_REGISTRY => {
                    // Get a temporary access token (at time of writing they last 5 minutes)
                    let mut request = CLIENT.get(
                            format!("https://auth.docker.io/token?service=registry.docker.io&scope=repository:{}:pull", image_name)
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
            registry: registry_host.to_string(),
            image: image_name.to_string(),
            token: registry_token,
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
    pub async fn push_image<P: AsRef<Path>>(&self, reference: &str, image_dir: P) -> Result<()> {
        let image_dir = image_dir.as_ref();

        let manifest_path = image_dir.join("index.json");
        if !manifest_path.exists() {
            bail!("The image directory does not have an `index.json` manifest file")
        }

        let blobs_dir = image_dir.join("blobs").join("sha256");
        if !blobs_dir.exists() {
            bail!("The image directory does not have as `blobs/sha256` sub-directory")
        }

        for entry in blobs_dir.read_dir()?.flatten() {
            self.push_blob(entry.path()).await?;
        }

        self.push_manifest(reference, manifest_path).await?;

        Ok(())
    }

    /// Pull an image
    ///
    /// Pulls an image from the registry to a local directory. The inverse of `push_image`.
    pub async fn pull_image<P: AsRef<Path>>(&self, reference: &str, image_dir: P) -> Result<()> {
        let image_dir = image_dir.as_ref();

        let manifest = self.pull_manifest(reference, image_dir).await?;

        let mut futures = vec![self.pull_blob(manifest.config(), image_dir)];
        for layer in manifest.layers() {
            let future = self.pull_blob(layer, image_dir);
            futures.push(future);
        }

        futures::future::try_join_all(futures).await?;

        Ok(())
    }

    /// Get a manifest from the registry
    ///
    /// See https://github.com/opencontainers/distribution-spec/blob/main/spec.md#pulling-manifests
    pub async fn get_manifest<S: AsRef<str>>(&self, reference: S) -> Result<ImageManifest> {
        let reference = reference.as_ref();

        tracing::info!(
            "Pulling manifest from repository `{}/{}:{}`",
            self.registry,
            self.image,
            reference
        );

        let response = self
            .get(&["/manifests/", reference].concat())
            .header(
                "Accept",
                "application/vnd.docker.distribution.manifest.v2+json",
            )
            .send()
            .await?
            .error_for_status()?;

        let manifest: ImageManifest = response.json().await?;
        Ok(manifest)
    }

    /// Pull a manifest from the registry to a local file and return it
    pub async fn pull_manifest<S: AsRef<str>, P: AsRef<Path>>(
        &self,
        reference: S,
        image_dir: P,
    ) -> Result<ImageManifest> {
        let manifest = self.get_manifest(reference).await?;

        fs::create_dir_all(&image_dir)?;
        let manifest_path = image_dir.as_ref().join("index.json");
        let mut manifest_file = fs::File::create(manifest_path)?;
        manifest.to_writer_pretty(&mut manifest_file)?;

        Ok(manifest)
    }

    /// Push a manifest from a local file to the registry
    ///
    /// See https://github.com/opencontainers/distribution-spec/blob/main/spec.md#pushing-manifests
    pub async fn push_manifest<S: AsRef<str>, P: AsRef<Path>>(
        &self,
        reference: S,
        mainfest_path: P,
    ) -> Result<()> {
        let reference = reference.as_ref();

        tracing::info!(
            "Pushing manifest to repository `{}/{}:{}`",
            self.registry,
            self.image,
            reference
        );

        let manifest = fs::read_to_string(mainfest_path)?;

        self.put(&["/manifests/", reference].concat())
            .header(
                "Content-Type",
                "application/vnd.docker.distribution.manifest.v2+json",
            )
            .body(manifest)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    /// Get an image config from the registry
    pub async fn get_config<S: AsRef<str>>(
        &self,
        reference: S,
        manifest: Option<&ImageManifest>,
    ) -> Result<ImageConfiguration> {
        let manifest = match manifest {
            Some(manifest) => manifest.to_owned(),
            None => self.get_manifest(reference).await?,
        };
        let descriptor = manifest.config();
        let response = self.get_blob(descriptor).await?;
        let config: ImageConfiguration = response.json().await?;
        Ok(config)
    }

    /// Get a blob from the registry
    ///
    /// See https://github.com/opencontainers/distribution-spec/blob/main/spec.md#pulling-blobs
    pub async fn get_blob(&self, descriptor: &Descriptor) -> Result<Response> {
        let digest = descriptor.digest();
        let size = descriptor.size();

        tracing::info!(
            "Pulling blob `{}` from repository `{}/{}`",
            digest,
            self.registry,
            self.image
        );

        let response = self
            .get(&["/blobs/", digest].concat())
            .header("Accept", "application/octet-stream")
            .send()
            .await?
            .error_for_status()?;

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

        Ok(response)
    }

    /// Pull a blob from the registry to a local file
    pub async fn pull_blob<P: AsRef<Path>>(
        &self,
        descriptor: &Descriptor,
        image_dir: P,
    ) -> Result<()> {
        let mut response = self.get_blob(descriptor).await?;

        let blobs_dir = image_dir.as_ref().join("blobs").join("sha256");
        fs::create_dir_all(&blobs_dir)?;

        let digest = descriptor.digest();
        let filename = match digest.strip_prefix("sha256:") {
            Some(sha256) => sha256,
            None => bail!(
                "Expected digest to be prefixed by 'sha256:' but got: {}",
                digest
            ),
        };
        let blob_path = blobs_dir.join(filename);
        let mut file = File::create(blob_path).await?;

        while let Some(chunk) = response.chunk().await? {
            file.write(chunk.as_ref()).await?;
        }

        Ok(())
    }

    /// Push a blob from a local file to the registry
    ///
    /// See https://docs.docker.com/registry/spec/api/#pushing-a-layer for a description of the flow.
    /// See also https://github.com/opencontainers/distribution-spec/blob/main/spec.md#pushing-blobs.
    pub async fn push_blob<P: AsRef<Path>>(&self, blob_path: P) -> Result<()> {
        let filename = blob_path
            .as_ref()
            .file_name()
            .expect("Path to always have a filename");
        let digest = format!("sha256:{}", filename.to_string_lossy());

        // Check that the blob actually needs to be pushed
        let response = self.get(["/blobs/", &digest].concat()).send().await?;
        if response.status() == 200 {
            tracing::info!(
                "Blob `{}` already exists in repository `{}/{}`, will not push",
                digest,
                self.registry,
                self.image
            );
            return Ok(());
        }

        // Get the UUID and URL for this upload
        let response = self
            .post("/blobs/uploads/")
            .header("Content-Length", "0")
            .send()
            .await?
            .error_for_status()?;
        let upload_uuid = match response.headers().get("Docker-Upload-UUID") {
            Some(url) => url.to_str()?,
            None => bail!("Did not receive upload UUID from registry"),
        };
        let upload_url = match response.headers().get("Location") {
            Some(url) => url.to_str()?,
            None => bail!("Did not receive upload URL from registry"),
        };

        tracing::info!(
            "Pushing blob `{}` to repository `{}/{}`",
            digest,
            self.registry,
            self.image
        );

        const MAX_CHUNK_SIZE: usize = 1000; //5_048_576;

        let mut file = File::open(&blob_path).await?;
        let length = fs::metadata(&blob_path)?.len() as usize;

        // If the blob is less than the maximum chunk size then upload it "monolithically"
        // rather than in chunks (to reduce the number of requests)
        if length <= MAX_CHUNK_SIZE {
            let mut bytes = Vec::with_capacity(length);
            io::copy(&mut file, &mut bytes).await?;

            let upload_url = [
                upload_url,
                if upload_url.contains('?') { "&" } else { "?" },
                "digest=",
                &digest,
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
        let mut start_byte = 0;
        while let Ok(chunk_length) = file.read(&mut buffer[..]).await {
            if chunk_length == 0 {
                break;
            }

            let chunk = Bytes::copy_from_slice(&buffer[0..chunk_length]);

            tracing::info!(
                "{} {} {} {}",
                start_byte,
                start_byte + chunk_length - 1,
                chunk_length,
                chunk.len()
            );

            let response = self
                .patch(upload_url)
                .header("Content-Type", "application/octet-stream")
                .header(
                    "Content-Range",
                    format!("{}-{}", start_byte, start_byte + chunk_length - 1),
                )
                .header("Content-Length", chunk.len())
                .body(chunk)
                .send()
                .await?;
            if response.status() != 202 {
                bail!(
                    "While uploading chunk: {} {}",
                    response.status(),
                    response.text().await?
                )
            } else {
                let range = response
                    .headers()
                    .get("Range")
                    .and_then(|range| range.to_str().ok())
                    .unwrap_or_default();
                tracing::info!("Uploaded chunk range `{}`", range);
            }

            start_byte += chunk_length;
        }

        // Notify the registry that the upload is complete
        self.put(["/blobs/uploads/", upload_uuid, "?digest=", &digest].concat())
            .send()
            .await?
            .error_for_status()?;

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
            [&registry_url, "/v2/", &self.image, path].concat()
        };

        let mut request = CLIENT.request(method, url);

        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }

        request
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
        client.pull_image("latest", &image_dir).await?;
        client.push_image("latest", &image_dir).await?;

        Ok(())
    }
}
