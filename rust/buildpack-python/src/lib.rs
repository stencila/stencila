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

impl Buildpack for PythonBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = GenericError;

    fn detect(&self, _context: DetectContext<Self>) -> Result<DetectResult, Self::Error> {
        if self.any_exist(&["main.py", "index.py", "requirements.txt", "pyproject.toml"])
            || self.file_contains(".tool-versions", "python")
            || self.file_contains("runtime.txt", "python")
        {
            DetectResultBuilder::pass()
                .build_plan(self.build_plan(&["python"]))
                .build()
        } else {
            DetectResultBuilder::fail().build()
        }
    }

    fn build(&self, _context: BuildContext<Self>) -> Result<BuildResult, Self::Error> {
        BuildResultBuilder::new().build()
    }
}

impl BuildpackTrait for PythonBuildpack {
    fn toml() -> &'static str {
        include_str!("../buildpack.toml")
    }
}
