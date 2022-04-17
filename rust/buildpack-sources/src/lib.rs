use std::fs::read_to_string;

use buildpack::{
    eyre,
    libcnb::{
        self,
        build::{BuildContext, BuildResult, BuildResultBuilder},
        data::build_plan::BuildPlan,
        detect::{DetectContext, DetectResult, DetectResultBuilder},
        generic::{GenericMetadata, GenericPlatform},
        Buildpack,
    },
    tracing, BuildpackTrait,
};
use sources::Sources;

pub struct SourcesBuildpack;

impl BuildpackTrait for SourcesBuildpack {
    fn toml() -> &'static str {
        include_str!("../buildpack.toml")
    }
}

const PROJECT_JSON: &str = "project.json";

impl Buildpack for SourcesBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = eyre::Report;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        let sources = read_to_string(PROJECT_JSON)
            .ok()
            .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok())
            .and_then(|project| project.get("sources").cloned())
            .and_then(|sources| serde_json::from_value::<Sources>(sources).ok());

        if let Some(sources) = sources {
            tracing::info!("Importing project sources");
            sources
                .import_sync(&context.app_dir)
                .map_err(libcnb::Error::BuildpackError)?;

            // Although this buildpack does not do anything in the build phase we
            // add to the build plan mainly for documentation / debugging purposes
            let (require, provide) =
                Self::require_and_provide("sources", PROJECT_JSON, "Import project sources", None);
            let mut build_plan = BuildPlan::new();
            build_plan.requires = vec![require];
            build_plan.provides = vec![provide];
            return DetectResultBuilder::pass().build_plan(build_plan).build();
        }

        DetectResultBuilder::fail().build()
    }

    fn build(&self, _context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        BuildResultBuilder::new().build()
    }
}
