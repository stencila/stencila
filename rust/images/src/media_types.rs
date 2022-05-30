use oci_spec::image::MediaType;

/**
 * Get the [Docker Image Manifest V2, Schema 2](https://docs.docker.com/registry/spec/manifest-v2-2/)
 * media type equivalent for an [OCI Media Type](https://github.com/opencontainers/image-spec/blob/main/media-types.md).
 *
 * This trait may be necessary for compatibility with tools that do not recognize the OCI Media Types.
 * Where a [`MediaType`] is expected (e.g. the `media_type` method of builders) you can use
 * `MediaType::ImageManifest.to_docker_v2s2()?` instead.
 * 
 * Not all OCI Media Types have an equivalent Docker V2S2 Media Type. In those cases, `to_docker_v2s2` will error.
 */
pub trait ToDockerV2S2 {
    fn to_docker_v2s2(&self) -> Result<&str, std::fmt::Error>;
}

impl ToDockerV2S2 for MediaType {
    fn to_docker_v2s2(&self) -> Result<&str, std::fmt::Error> {
        Ok(match self {
            Self::ImageIndex => "application/vnd.docker.distribution.manifest.list.v2+json",
            Self::ImageManifest => "application/vnd.docker.distribution.manifest.v2+json",
            Self::ImageConfig => "application/vnd.docker.container.image.v1+json",
            Self::ImageLayerGzip => "application/vnd.docker.image.rootfs.diff.tar.gzip",
            _ => return Err(std::fmt::Error),
        })
    }
}
