use buildpack::{
    libcnb::{
        build::{BuildContext, BuildResult, BuildResultBuilder},
        detect::{DetectContext, DetectResult, DetectResultBuilder},
        generic::{GenericError, GenericMetadata, GenericPlatform},
        Buildpack, Result,
    },
    BuildpackTrait,
};

pub struct NodeBuildpack;

impl Buildpack for NodeBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = GenericError;

    fn detect(&self, _context: DetectContext<Self>) -> Result<DetectResult, Self::Error> {
        DetectResultBuilder::pass().build()
    }

    fn build(&self, _context: BuildContext<Self>) -> Result<BuildResult, Self::Error> {
        BuildResultBuilder::new().build()
    }
}

impl BuildpackTrait for NodeBuildpack {
    fn toml() -> &'static str {
        include_str!("../buildpack.toml")
    }
}
