use buildpack::{
    libcnb::{
        build::{BuildContext, BuildResult, BuildResultBuilder},
        detect::{DetectContext, DetectResult, DetectResultBuilder},
        generic::{GenericError, GenericMetadata, GenericPlatform},
        Buildpack, Result,
    },
    BuildpackTrait,
};

pub struct RBuildpack;

impl BuildpackTrait for RBuildpack {
    fn toml() -> &'static str {
        include_str!("../buildpack.toml")
    }
}

impl Buildpack for RBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = GenericError;

    fn detect(&self, _context: DetectContext<Self>) -> Result<DetectResult, Self::Error> {
        if Self::any_exist(&[
            "main.R",
            "index.R",
            "main.r",
            "index.r",
            "install.R",
            "install.r",
            "DESCRIPTION",
        ]) || Self::file_contains(".tool-versions", "R ")
        {
            DetectResultBuilder::pass().build()
        } else {
            DetectResultBuilder::fail().build()
        }
    }

    fn build(&self, _context: BuildContext<Self>) -> Result<BuildResult, Self::Error> {
        BuildResultBuilder::new().build()
    }
}
