//! A buildpack for Dockerfiles

use binaries::require_sync;
use buildpack::{
    eyre::Report,
    libcnb::{
        build::{BuildContext, BuildResult, BuildResultBuilder},
        detect::{DetectContext, DetectResult, DetectResultBuilder},
        generic::{GenericMetadata, GenericPlatform},
        Buildpack, Error, Result,
    },
    BuildpackTrait,
};

pub struct DockerfileBuildpack;

/// A buildpack for projects containing a `Dockerfile` (or `Containerfile`)
///
/// Build a container image with a tag `<dir-name>-<hash>` where `<dir-name>` is the
/// name of the directory containing the `Dockerfile` and `<hash>` is the 12-character
/// truncated SHA256 hash of its path (to avoid clashes between directories with the same name).
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
        let name = context
            .app_dir
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "unnamed".to_string());
        let mut hash = hash_utils::str_sha256_hex(&context.app_dir.display().to_string());
        hash.truncate(12);
        let tag = [&name, "-", &hash].concat();

        let podman = require_sync("podman", "*").map_err(Error::BuildpackError)?;
        podman
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
