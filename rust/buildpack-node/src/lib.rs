use std::{
    collections::HashMap,
    ffi::OsString,
    fs::{create_dir_all, read_to_string},
    path::{Path, PathBuf},
};

use binary_node::{BinaryInstallation, BinaryTrait, NodeBinary};
use buildpack::{
    eyre,
    fs_utils::{copy_if_exists, move_dir_all, symlink_dir, symlink_file},
    hash_utils::str_sha256_hex,
    is_cnb_build, is_local_build,
    libcnb::{
        self,
        build::{BuildContext, BuildResult, BuildResultBuilder},
        data::{build_plan::BuildPlan, layer_content_metadata::LayerTypes, layer_name},
        detect::{DetectContext, DetectResult, DetectResultBuilder},
        generic::{GenericMetadata, GenericPlatform},
        layer::{ExistingLayerStrategy, Layer, LayerResult, LayerResultBuilder},
        layer_env::{LayerEnv, ModificationBehavior, Scope},
        Buildpack,
    },
    tracing, BuildpackTrait, LayerHashMetadata, LayerVersionMetadata, SYSTEM_INSTALLED,
};

pub struct NodeBuildpack;

impl BuildpackTrait for NodeBuildpack {
    fn toml() -> &'static str {
        include_str!("../buildpack.toml")
    }
}

const NODE_MODULES: &str = "node_modules";
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
        let nvmrc = read_to_string(NVMRC)
            .map(|content| content.trim().to_string())
            .ok();

        // Read `package.json` for Node.js version
        let package_json = read_to_string(PACKAGE_JSON)
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
                "node_modules",
                if package_lock.exists() {
                    PACKAGE_LOCK
                } else {
                    PACKAGE_JSON
                },
                "Install Node.js packages into `node_modules`",
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
        let entries: HashMap<_, _> = context
            .buildpack_plan
            .entries
            .iter()
            .map(|entry| Self::split_entry_name(&entry.name))
            .collect();

        if let Some(args) = entries.get("node") {
            context.handle_layer(layer_name!("node"), NodeLayer::new(args.clone()))?;
        }

        if entries.contains_key("node_modules") {
            context.handle_layer(layer_name!("node_modules"), NodeModulesLayer)?;
        }

        BuildResultBuilder::new().build()
    }
}

struct NodeLayer {
    /// The semantic version requirement for the `node` binary
    requirement: String,
}

impl NodeLayer {
    fn new(args: Vec<String>) -> Self {
        // Join args with commas because semver requirement parser expects that is
        // how parts of a requirement are separated
        let versionish = args.join(",");

        // Determine the semver requirement from versionish which could be a well formed semantic version (e.g 14.0.1),
        // a semver requirement (e.g. ^14.0), or an alias (e.g. `lts`)
        let requirement = if let Some(version) = versionish.strip_prefix('v') {
            version.to_string()
        } else if versionish == "lts" {
            // TODO: Determine LTS without doing a fetch, perhaps based on date
            // https://nodejs.org/en/about/releases/
            "^16".to_string()
        } else {
            versionish.clone()
        };

        NodeLayer { requirement }
    }
}

impl Layer for NodeLayer {
    type Buildpack = NodeBuildpack;
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
        let installed = NodeBinary {}.semver_version_matches(
            &layer_data.content_metadata.metadata.version,
            &self.requirement,
        )?;

        let strategy = if installed {
            tracing::info!(
                "Existing `node` layer has `./bin/node` matching semver requirement `{}`; will keep",
                self.requirement
            );
            ExistingLayerStrategy::Keep
        } else {
            tracing::info!(
                "Existing `node` layer does not have `./bin/node` matching semver requirement `{}`; will recreate",
                self.requirement
            );
            ExistingLayerStrategy::Recreate
        };
        Ok(strategy)
    }

    fn create(
        &self,
        context: &BuildContext<Self::Buildpack>,
        layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, eyre::Report> {
        tracing::info!(
            "Creating `node` layer with semver requirement `{}`",
            self.requirement
        );

        let node = NodeBinary {}.require_sync(Some(self.requirement.clone()), true)?;
        let version = node.version()?.to_string();

        if is_local_build(context) {
            if node.is_stencila_install() {
                tracing::info!("Linking to `node {}` installed by Stencila", version);
                let source = node.grandparent()?;

                symlink_dir(source.join("bin"), &layer_path.join("bin"))?;
                symlink_dir(source.join("lib"), &layer_path.join("lib"))?;
            } else {
                tracing::info!("Linking to `node {}` installed on system", version);
                let source = node.parent()?;

                let bin_path = layer_path.join("bin");
                create_dir_all(&bin_path)?;
                symlink_file(node.path, bin_path.join(node.name))?;
                symlink_file(source.join("corepack"), bin_path.join("corepack"))?;
                symlink_file(source.join("npm"), bin_path.join("npm"))?;
                symlink_file(source.join("npx"), bin_path.join("npx"))?;

                let lib_path = layer_path.join("lib");
                create_dir_all(&lib_path)?;
                symlink_dir(
                    source.join("..").join("lib").join(NODE_MODULES),
                    lib_path.join(NODE_MODULES),
                )?;
            }
        } else {
            #[allow(clippy::collapsible_else_if)]
            if node.is_stencila_install() {
                tracing::info!("Moving `node {}` installed by Stencila", version);
                let source = node.grandparent()?;

                move_dir_all(&source, layer_path)?;
            } else {
                tracing::info!("Linking to `node {}` installed on stack image", version);
                let source = node.grandparent()?;

                symlink_dir(source.join("bin"), &layer_path.join("bin"))?;
                symlink_dir(source.join("lib"), &layer_path.join("lib"))?;
            }
        }

        LayerResultBuilder::new(LayerVersionMetadata { version }).build()
    }
}

struct NodeModulesLayer;

/// Generate `package_hash` string for an app directory
///
/// The hash is of the combined contens of `package-lock.json` and `package.json`.
/// This means that if either one is changed or removed that the hash will change.
fn generate_package_hash(app_dir: &Path) -> String {
    let content = [
        read_to_string(app_dir.join(PACKAGE_LOCK)).unwrap_or_default(),
        read_to_string(app_dir.join(PACKAGE_JSON)).unwrap_or_default(),
    ]
    .concat();
    str_sha256_hex(&content)
}

impl Layer for NodeModulesLayer {
    type Buildpack = NodeBuildpack;
    type Metadata = LayerHashMetadata;

    fn types(&self) -> LayerTypes {
        LayerTypes {
            build: true,
            launch: true,
            cache: true,
        }
    }

    fn existing_layer_strategy(
        &self,
        context: &BuildContext<Self::Buildpack>,
        layer_data: &libcnb::layer::LayerData<Self::Metadata>,
    ) -> Result<libcnb::layer::ExistingLayerStrategy, <Self::Buildpack as Buildpack>::Error> {
        let package_hash = generate_package_hash(&context.app_dir);
        let strategy = if package_hash == layer_data.content_metadata.metadata.hash {
            tracing::info!("Existing `node_modules` layer has same package hash; will keep",);
            ExistingLayerStrategy::Keep
        } else {
            tracing::info!("Existing `node_modules` layer has different package hash; will update");
            ExistingLayerStrategy::Update
        };
        Ok(strategy)
    }

    fn create(
        &self,
        context: &BuildContext<Self::Buildpack>,
        layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, eyre::Report> {
        tracing::info!("Creating `node_modules` layer");
        self.install(context, layer_path)
    }

    fn update(
        &self,
        context: &BuildContext<Self::Buildpack>,
        layer_data: &libcnb::layer::LayerData<Self::Metadata>,
    ) -> Result<LayerResult<Self::Metadata>, <Self::Buildpack as Buildpack>::Error> {
        tracing::info!("Updating `node_modules` layer");
        self.install(context, &layer_data.path)
    }
}

impl NodeModulesLayer {
    fn install(
        &self,
        context: &BuildContext<NodeBuildpack>,
        layer_path: &Path,
    ) -> Result<LayerResult<LayerHashMetadata>, eyre::Report> {
        let app_path = &context.app_dir;

        // Use the `node` and `npm` binaries installed in the `node` layer
        let node_layer = layer_path
            .parent()
            .expect("Should have parent")
            .join("node");

        // Prepend to the PATH because some package install scripts
        // use `/usr/bin/env node` which looks for `node` on PATH
        let mut envs: Vec<(OsString, OsString)> = vec![(
            "PATH".into(),
            NodeBuildpack::prepend_path(&node_layer.join("bin"))?,
        )];

        // If this is a CNB build use `layer_path/cache` as the NPM cache
        if is_cnb_build(context) {
            envs.push(("NPM_CONFIG_CACHE".into(), layer_path.join("cache").into()));
        }

        // Call the `npm-cli.js` script installed in the `node` layer
        // This is done, rather than executing `bin/npm` directly, there are issues with node `require`
        // module resolution when the latter is done.
        let node = BinaryInstallation::new(
            "node".into(),
            node_layer.join("bin").join("node"),
            None,
            envs,
        );
        let npm = node_layer
            .join("lib")
            .join(NODE_MODULES)
            .join("npm")
            .join("bin")
            .join("npm-cli.js")
            .into_os_string();

        let mut layer_env = LayerEnv::new();
        if is_local_build(context) {
            // Do the install in the app directory as normal
            node.run_sync(&[npm, "install".into()])?;
        } else {
            // Do the install in the layer.
            // Alternative, more complicated approaches to this e.g. doing a local install and then copying
            // over to layers and/or symlinking are problematic.

            // Despite some confusion online it seems that at present it is necessary to copy over these
            // files when using `--prefix`
            copy_if_exists(app_path.join(PACKAGE_JSON), layer_path.join(PACKAGE_JSON))?;
            copy_if_exists(app_path.join(PACKAGE_LOCK), layer_path.join(PACKAGE_LOCK))?;

            node.run_sync(&[npm, "install".into(), "--prefix".into(), layer_path.into()])?;

            // Set the `NODE_PATH` so that the `node_modules` can be found
            layer_env.insert(
                Scope::All,
                ModificationBehavior::Prepend,
                "NODE_PATH",
                layer_path.join(NODE_MODULES),
            );
        }

        // Generate a 'package hash' to detect if layer is stale in `existing_layer_strategy()`
        let metadata = LayerHashMetadata {
            hash: generate_package_hash(app_path),
        };

        LayerResultBuilder::new(metadata).env(layer_env).build()
    }
}
