use std::{fs, path::Path};

use bytes::Bytes;
use eyre::{bail, Result};
use oci_spec::image::{Descriptor, ImageManifest};
use tokio::{
    fs::File,
    io::{self, AsyncReadExt, AsyncWriteExt},
};

use http_utils::{reqwest_middleware::RequestBuilder, CLIENT};

/// A client that implements the [OCI Distribution Specification](https://github.com/opencontainers/distribution-spec/blob/main/spec.md)
/// for pulling and pushing images from a container registry
pub struct RegistryClient {
    /// Base URL of the image registry API
    url: String,

    /// Name of the image repository e.g. library/hello-world
    name: String,

    /// Token used to authenticate requests
    token: String,
}

impl RegistryClient {
    /// Create a new client
    fn new(url: &str, name: &str, token: &str) -> Self {
        Self {
            url: url.to_string(),
            name: name.to_string(),
            token: token.to_string(),
        }
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

    /// Pull a manifest from the registry to a local file and return it
    ///
    /// See https://github.com/opencontainers/distribution-spec/blob/main/spec.md#pulling-manifests
    pub async fn pull_manifest<S: AsRef<str>, P: AsRef<Path>>(
        &self,
        reference: S,
        image_dir: P,
    ) -> Result<ImageManifest> {
        let reference = reference.as_ref();

        tracing::info!(
            "Pulling manifest from repository `{}/{}:{}`",
            self.url,
            self.name,
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
            self.url,
            self.name,
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

    /// Pull a blob from the registry to a local file
    ///
    /// See https://github.com/opencontainers/distribution-spec/blob/main/spec.md#pulling-blobs
    pub async fn pull_blob<P: AsRef<Path>>(
        &self,
        descriptor: &Descriptor,
        image_dir: P,
    ) -> Result<()> {
        let digest = descriptor.digest();
        let size = descriptor.size();

        tracing::info!(
            "Pulling blob `{}` from repository `{}/{}`",
            digest,
            self.url,
            self.name
        );

        let mut response = self
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

        let blobs_dir = image_dir.as_ref().join("blobs").join("sha256");
        fs::create_dir_all(&blobs_dir)?;

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
                self.url,
                self.name
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
            self.url,
            self.name
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

            let response = CLIENT
                .put(upload_url)
                .bearer_auth(self.token.clone())
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

            let response = CLIENT
                .patch(upload_url)
                .bearer_auth(self.token.clone())
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

    /// Make a GET request to the registry
    fn get<S: AsRef<str>>(&self, path: S) -> RequestBuilder {
        CLIENT
            .get([&self.url, "/v2/", &self.name, path.as_ref()].concat())
            .bearer_auth(self.token.clone())
    }

    /// Make a POST request to the registry
    fn post<S: AsRef<str>>(&self, path: S) -> RequestBuilder {
        CLIENT
            .post([&self.url, "/v2/", &self.name, path.as_ref()].concat())
            .bearer_auth(self.token.clone())
    }

    /// Make a PUT request to the registry
    fn put<S: AsRef<str>>(&self, path: S) -> RequestBuilder {
        CLIENT
            .put([&self.url, "/v2/", &self.name, path.as_ref()].concat())
            .bearer_auth(self.token.clone())
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

        let client = RegistryClient::new("http://localhost:5000", "library/hello-world", "");
        client.pull_image("latest", &image_dir).await?;
        client.push_image("latest", &image_dir).await?;

        Ok(())
    }
}
