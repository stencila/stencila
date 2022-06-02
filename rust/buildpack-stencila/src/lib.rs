use std::{fs::create_dir_all, path::Path};

use binary_stencila::{BinaryTrait, StencilaBinary};
use buildpack::{
    eyre,
    fs_utils::symlink_file,
    libcnb::{
        self,
        build::{BuildContext, BuildResult, BuildResultBuilder},
        data::{
            build_plan::BuildPlan,
            launch::{Launch, ProcessBuilder},
            layer_content_metadata::LayerTypes,
            layer_name, process_type,
        },
        detect::{DetectContext, DetectResult, DetectResultBuilder},
        generic::{GenericMetadata, GenericPlatform},
        layer::{ExistingLayerStrategy, Layer, LayerResult, LayerResultBuilder},
        Buildpack,
    },
    maplit::hashmap,
    tracing, BuildpackContext, BuildpackTrait, LayerOptions, LayerVersionMetadata,
};

pub struct StencilaBuildpack;

impl BuildpackTrait for StencilaBuildpack {
    fn toml() -> &'static str {
        include_str!("../buildpack.toml")
    }
}

const TOOL_VERSIONS: &str = ".tool-versions";

impl Buildpack for StencilaBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = eyre::Report;

    fn detect(&self, _context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        // Read `.tool-versions` for Stencila version
        let tool_versions = Self::tool_versions();

        // Resolve Stencila version from `.tool-versions`
        let (version, source) = if let Some(version) = tool_versions.get("stencila") {
            (version.to_string(), TOOL_VERSIONS)
        } else {
            // Default to at least 1.1.0 which is the first version with a `musl` binary suitable for
            // use in a CNB container
            (">=1.1.0".to_string(), "")
        };

        // Require and provide Stencila
        let (require, provide) = Self::require_and_provide(
            "stencila",
            source,
            format!("Install Stencila {}", version).trim(),
            Some(hashmap! {
                "version" => version
            }),
        );

        let mut build_plan = BuildPlan::new();
        build_plan.requires = vec![require];
        build_plan.provides = vec![provide];
        DetectResultBuilder::pass().build_plan(build_plan).build()
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        let entries = self.buildpack_plan_entries(&context.buildpack_plan);

        if let Some(options) = entries.get("stencila") {
            context.handle_layer(layer_name!("stencila"), StencilaLayer::new(options))?;
        }

        let launch = Launch::new().process(
            ProcessBuilder::new(process_type!("server"), "stencila")
                .args(["server", "start", "--url=0.0.0.0:9000"])
                .direct(true)
                .default(true)
                .build(),
        );

        BuildResultBuilder::new().launch(launch).build()
    }
}

struct StencilaLayer {
    /// The semantic version requirement for the `stencila` binary
    requirement: String,
}

impl StencilaLayer {
    fn new(options: &LayerOptions) -> Self {
        let requirement = options
            .get("version")
            .cloned()
            .unwrap_or_else(|| "*".to_string());

        Self { requirement }
    }
}

impl Layer for StencilaLayer {
    type Buildpack = StencilaBuildpack;
    type Metadata = LayerVersionMetadata;

    fn types(&self) -> LayerTypes {
        LayerTypes {
            build: true,
            launch: true,
            cache: true,
        }
    }

    fn existing_layer_strategy(
        &self,
        _context: &BuildContext<Self::Buildpack>,
        layer_data: &libcnb::layer::LayerData<Self::Metadata>,
    ) -> Result<libcnb::layer::ExistingLayerStrategy, <Self::Buildpack as Buildpack>::Error> {
        let version = &layer_data.content_metadata.metadata.version;
        let installed = StencilaBinary {}.semver_version_matches(version, &self.requirement)?;
        let strategy = if installed {
            tracing::info!(
                "Existing `stencila` layer has `stencila {}` which matches semver requirement `{}`; will keep",
                version,
                self.requirement
            );
            ExistingLayerStrategy::Keep
        } else {
            tracing::info!(
                "Existing `stencila` layer has `stencila {}` which does not match semver requirement `{}`; will update",
                version,
                self.requirement
            );
            ExistingLayerStrategy::Update
        };
        Ok(strategy)
    }

    fn create(
        &self,
        context: &BuildContext<Self::Buildpack>,
        layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, eyre::Report> {
        tracing::info!("Creating `stencila` layer");
        self.install(context, layer_path)
    }

    fn update(
        &self,
        context: &BuildContext<Self::Buildpack>,
        layer_data: &libcnb::layer::LayerData<Self::Metadata>,
    ) -> Result<LayerResult<Self::Metadata>, <Self::Buildpack as Buildpack>::Error> {
        tracing::info!("Updating `stencila` layer");
        self.install(context, &layer_data.path)
    }
}

impl StencilaLayer {
    fn install(
        &self,
        context: &BuildContext<StencilaBuildpack>,
        layer_path: &Path,
    ) -> Result<LayerResult<LayerVersionMetadata>, eyre::Report> {
        tracing::info!(
            "Creating `stencila` layer with semver requirement `{}`",
            self.requirement
        );

        let bin_path = layer_path.join("bin");
        create_dir_all(&bin_path)?;

        let stencila_binary = StencilaBinary {};

        let version = if context.is_local() {
            let stencila = stencila_binary.ensure_version_sync(&self.requirement)?;
            let version = stencila.version()?.to_string();

            tracing::info!("Linking to `stencila {}`", version);

            symlink_file(stencila.path, bin_path.join(stencila.name))?;

            version
        } else {
            let current = stencila_binary.find_version_in(&self.requirement, bin_path.as_os_str());
            match current {
                Ok(installation) => {
                    let version = installation.version.unwrap_or_default();

                    tracing::info!(
                        "Binary `stencila {}` is already installed and meets requirement `{}`",
                        version,
                        self.requirement
                    );

                    version
                }
                Err(..) => {
                    tracing::info!("Installing `stencila {}`", self.requirement);

                    stencila_binary
                        .install_in_sync(Some(self.requirement.clone()), Some(bin_path))?
                }
            }
        };

        // Store version in metadata to detect if layer is stale in `existing_layer_strategy()`
        let metadata = LayerVersionMetadata { version };

        LayerResultBuilder::new(metadata).build()
    }
}
