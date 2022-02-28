use binary_podman::{BinaryTrait, PodmanBinary};
use buildpack::{
    eyre::Report,
    libcnb::{
        build::{BuildContext, BuildResult, BuildResultBuilder},
        detect::{DetectContext, DetectResult, DetectResultBuilder},
        generic::{GenericMetadata, GenericPlatform},
        Buildpack, Error, Result,
    },
    tag_for_path, BuildpackTrait,
};

pub struct DockerfileBuildpack;

impl Buildpack for DockerfileBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = Report;

    fn detect(&self, _context: DetectContext<Self>) -> Result<DetectResult, Self::Error> {
        if Self::any_exist(&["Dockerfile", "Containerfile"]) {
            DetectResultBuilder::pass().build()
        } else {
            DetectResultBuilder::fail().build()
        }
    }

    fn build(&self, context: BuildContext<Self>) -> Result<BuildResult, Self::Error> {
        let tag = tag_for_path(&context.app_dir);

        PodmanBinary {}
            .ensure_version_sync(">=1")
            .map_err(Error::BuildpackError)?
            .run_sync(&["build", "--tag", &tag, "."])
            .map_err(Error::BuildpackError)?;

        BuildResultBuilder::new().build()
    }
}

impl BuildpackTrait for DockerfileBuildpack {
    fn toml() -> &'static str {
        include_str!("../buildpack.toml")
    }
}
