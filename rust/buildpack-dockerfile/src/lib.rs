//! A buildpack for Dockerfiles

use binary_podman::{binary::BinaryTrait, PodmanBinary};
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

/// A buildpack for projects containing a `Dockerfile` (or `Containerfile`)
///
/// Uses `podman`, rather than `docker`, to build the image because the former runs in
/// userspace and is thus more secure.
///
/// This is not a Cloud Native Buildpack (e.g. it lacks a `detect` or `build` binary).
/// However, it uses the same API (e.g. has a `buildpack.toml`) so that, for example, it
/// appears in the list at `stencila buildpacks list`.
///
/// We never point an external CNB platform, such as Pack, at this buildpack.
/// Instead ,in the buildpacks` crate, we run its `detect` method before any other buildpacks and build an
/// image from the Dockerfile if it passes.
impl Buildpack for DockerfileBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = Report;

    fn detect(&self, _context: DetectContext<Self>) -> Result<DetectResult, Self::Error> {
        if self.any_exist(&["Dockerfile", "Containerfile"]) {
            DetectResultBuilder::pass().build()
        } else {
            DetectResultBuilder::fail().build()
        }
    }

    fn build(&self, context: BuildContext<Self>) -> Result<BuildResult, Self::Error> {
        let tag = tag_for_path(&context.app_dir);

        PodmanBinary {}
            .require_sync(None, true)
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
