use std::{
    fs,
    path::{Path, PathBuf},
};

use binary_node::{BinaryInstallation, BinaryTrait, NodeBinary};
use buildpack::{
    eyre::{self, bail, eyre},
    fs_utils::{clear_dir_all, copy_dir_all, symlink_dir, symlink_file},
    libcnb::{
        self,
        build::{BuildContext, BuildResult, BuildResultBuilder},
        data::{build_plan::BuildPlan, layer_content_metadata::LayerTypes, layer_name},
        detect::{DetectContext, DetectResult, DetectResultBuilder},
        generic::{GenericMetadata, GenericPlatform},
        layer::{Layer, LayerResult, LayerResultBuilder},
        Buildpack,
        Error::BuildpackError,
    },
    platform_is_stencila, tracing, BuildpackTrait, SYSTEM_INSTALLED,
};

pub struct NodeBuildpack;

impl BuildpackTrait for NodeBuildpack {
    fn toml() -> &'static str {
        include_str!("../buildpack.toml")
    }
}

const NVMRC: &str = ".nvmrc";
const PACKAGE_JSON: &str = "package.json";
const PACKAGE_LOCK: &str = "package-lock.json";
const TOOL_VERSIONS: &str = ".tool-versions";

impl Buildpack for NodeBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = eyre::Report;

    fn detect(&self, _context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        // Read `.tool-versions` for Node.js version
        let tool_versions = Self::tool_versions();

        // Read `.nvmrc` for Node.js version
        let nvmrc = fs::read_to_string(NVMRC)
            .map(|content| content.trim().to_string())
            .ok();

        // Read `package.json` for Node.js version
        let package_json = fs::read_to_string(PACKAGE_JSON)
            .ok()
            .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok());

        // Detect `package-lock.json`
        let package_lock = PathBuf::from(PACKAGE_LOCK);

        // Fail early
        if !(tool_versions.contains_key("nodejs")
            || tool_versions.contains_key("node")
            || package_json.is_some()
            || package_lock.exists()
            || nvmrc.is_some()
            || Self::any_exist(&["main.js", "index.js"]))
        {
            return DetectResultBuilder::fail().build();
        }

        let mut requires = Vec::new();
        let mut provides = Vec::new();

        // Resolve Node.js version from `.tool-versions`, `.nvmrc`, `package.json`, or installed `node` version
        let (version, source) = if let Some(version) = tool_versions
            .get("nodejs")
            .or_else(|| tool_versions.get("node"))
        {
            (version.to_string(), TOOL_VERSIONS)
        } else if let Some(versionish) = nvmrc {
            (versionish, NVMRC)
        } else if let Some(semver) = package_json.as_ref().and_then(|package| {
            package
                .pointer("/engines/node")
                .and_then(|semver| semver.as_str().map(|semver| semver.to_string()))
        }) {
            (semver, PACKAGE_JSON)
        } else if let Some(version) = (NodeBinary {}).installed_version(None) {
            (version, SYSTEM_INSTALLED)
        } else {
            ("".to_string(), "")
        };

        // Require and provide Node.js
        let (require, provide) = Self::require_and_provide(
            format!("node {}", version).trim(),
            source,
            format!("Install Node.js {}", version).trim(),
        );
        requires.push(require);
        provides.push(provide);

        // Determine how NPM packages are to be installed
        if package_lock.exists() || package_json.is_some() {
            let (require, provide) = Self::require_and_provide(
                "npm install",
                if package_lock.exists() {
                    PACKAGE_LOCK
                } else {
                    PACKAGE_JSON
                },
                "Install NPM packages",
            );
            requires.push(require);
            provides.push(provide);
        }

        let mut build_plan = BuildPlan::new();
        build_plan.requires = requires;
        build_plan.provides = provides;
        DetectResultBuilder::pass().build_plan(build_plan).build()
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        for entry in &context.buildpack_plan.entries {
            let (name, args) = Self::split_entry_name(&entry.name);
            match name.as_str() {
                "node" => {
                    context.handle_layer(layer_name!("node"), NodeLayer::new(args))?;
                }
                "npm" => {
                    context.handle_layer(layer_name!("npm"), NpmLayer)?;
                }
                _ => {
                    return Err(BuildpackError(eyre!(
                        "Unhandled buildpack plan entry: {}",
                        name
                    )))
                }
            };
        }

        BuildResultBuilder::new().build()
    }
}

struct NodeLayer {
    /// A string describing the version, of Node.js to install
    ///
    /// This could be a well formed semantic version (e.g 14.0.1),
    /// a semver requirement (e.g. ^14.0), or an alias (e.g. `lts`).
    /// The `create` method aim to convert them all to a semver requirement.
    versionish: String,
}

impl NodeLayer {
    fn new(args: Vec<String>) -> Self {
        NodeLayer {
            // Join args with commas because semver requirement parser expects it to be so
            versionish: args.join(","),
        }
    }
}

impl Layer for NodeLayer {
    type Buildpack = NodeBuildpack;
    type Metadata = GenericMetadata;

    fn types(&self) -> LayerTypes {
        LayerTypes {
            build: true,
            launch: true,
            cache: true,
        }
    }

    fn create(
        &self,
        context: &BuildContext<Self::Buildpack>,
        layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, eyre::Report> {
        // Determine the semver requirement
        let mut requirement = if let Some(version) = self.versionish.strip_prefix('v') {
            version.to_string()
        } else if self.versionish == "lts" {
            // TODO: Determine LTS without doing a fetch, perhaps based on date
            // https://nodejs.org/en/about/releases/
            "^16".to_string()
        } else {
            self.versionish.clone()
        };
        if requirement.is_empty() {
            requirement = "*".to_string();
        }

        // First check if there is already a `node` binary meeting the semver requirement
        // in the expected place
        let installed = NodeBinary {}.installed_at(
            &layer_path.join("bin").join("node"),
            Some(requirement.clone()),
        )?;

        if installed {
            tracing::info!("Node.js {} already installed", requirement);
        } else {
            tracing::info!(
                "No version of Node.js meeting requirement `{}` yet installed",
                requirement
            );

            // Ensure a version meeting the semver requirement is installed on the builder
            let node = NodeBinary {}.require_sync(Some(requirement), true)?;
            let version = node.version()?;

            // Symlink/copy the installation into the layer
            if platform_is_stencila(&context.platform) {
                if node.is_stencila_install() {
                    tracing::info!("Linking to Node.js {} installed by Stencila", version);
                    clear_dir_all(&layer_path)?;
                    let source = node.grandparent()?;

                    symlink_dir(source.join("bin"), &layer_path.join("bin"))?;
                    symlink_dir(source.join("lib"), &layer_path.join("lib"))?;
                } else {
                    tracing::info!("Linking to Node.js {} installed on system", version);
                    clear_dir_all(&layer_path)?;
                    let source = node.parent()?;

                    let bin_path = layer_path.join("bin");
                    fs::create_dir_all(&bin_path)?;
                    symlink_file(node.path, bin_path.join(node.name))?;
                    symlink_file(source.join("npm"), bin_path.join("npm"))?;
                    symlink_file(source.join("npx"), bin_path.join("npx"))?;

                    let lib_path = layer_path.join("lib");
                    fs::create_dir_all(&lib_path)?;
                    symlink_dir(
                        source.join("..").join("lib").join("node_modules"),
                        lib_path.join("node_modules"),
                    )?;
                }
            } else {
                #[allow(clippy::collapsible_else_if)]
                if node.is_stencila_install() {
                    tracing::info!("Using Node.js {} installed by Stencila", version);
                    clear_dir_all(&layer_path)?;
                    let source = node.grandparent()?;

                    copy_dir_all(source, &layer_path)?;
                } else {
                    bail!("Only able to build `node` layer if Node has been installed by Stencila")
                }
            }
        }

        LayerResultBuilder::new(GenericMetadata::default()).build()
    }
}

struct NpmLayer;

impl Layer for NpmLayer {
    type Buildpack = NodeBuildpack;
    type Metadata = GenericMetadata;

    fn types(&self) -> LayerTypes {
        LayerTypes {
            build: true,
            launch: false,
            cache: true,
        }
    }

    fn create(
        &self,
        context: &BuildContext<Self::Buildpack>,
        layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, eyre::Report> {
        tracing::info!("Installing packages using NPM");

        // Use `node` from the previous layer
        let node_layer = layer_path
            .canonicalize()?
            .parent()
            .expect("Should have parent")
            .join("node");
        let mut node = BinaryInstallation {
            name: "node".into(),
            path: node_layer.join("bin").join("node"),
            ..Default::default()
        };

        // Use the `npm-cli.js` script from the previous layer
        // This is done because the `npm` script in `bin` has a
        // `/usr/bin/env node` shebang and so needs the right `node` in $PATH.
        // Even if that is done, there are issues with node `require`
        // module resolution when doing so. But this works...
        let npm = node_layer
            .join("lib")
            .join("node_modules")
            .join("npm")
            .join("bin")
            .join("npm-cli.js")
            .display()
            .to_string();

        // If Stencila is not the platform use the layer as the NPM cache
        if !platform_is_stencila(&context.platform) {
            node.envs(&[("NPM_CONFIG_CACHE", layer_path.canonicalize()?)]);
        }

        // Do the install
        node.run_sync(&[&npm, "install"])?;

        LayerResultBuilder::new(GenericMetadata::default()).build()
    }
}
