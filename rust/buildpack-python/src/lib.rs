use buildpack::{
    libcnb::{
        build::{BuildContext, BuildResult, BuildResultBuilder},
        detect::{DetectContext, DetectResult, DetectResultBuilder},
        generic::{GenericError, GenericMetadata, GenericPlatform},
        Buildpack, Result,
    },
    BuildpackTrait,
};

pub struct PythonBuildpack;

impl BuildpackTrait for PythonBuildpack {
    fn toml() -> &'static str {
        include_str!("../buildpack.toml")
    }
}

impl Buildpack for PythonBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = GenericError;

    fn detect(&self, _context: DetectContext<Self>) -> Result<DetectResult, Self::Error> {
        if Self::any_exist(&["main.py", "index.py", "requirements.txt", "pyproject.toml"])
            || Self::file_contains(".tool-versions", "python")
            || Self::file_contains("runtime.txt", "python")
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
