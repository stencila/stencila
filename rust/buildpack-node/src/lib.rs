use std::{
    fs::{create_dir_all, read_to_string, remove_file},
    path::{Path, PathBuf},
};

use binary_node::{BinaryTrait, NodeBinary};
use buildpack::{
    eyre,
    fs_utils::{copy_if_exists, move_dir_all, symlink_dir, symlink_file},
    hash_utils::str_sha256_hex,
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
    maplit::hashmap,
    tracing, BuildpackContext, BuildpackTrait, LayerHashMetadata, LayerOptions,
    LayerVersionMetadata,
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
        } else {
            ("lts".to_string(), "")
        };

        // Require and provide Node.js
        let (require, provide) = Self::require_and_provide(
            "node",
            source,
            format!("Install Node.js {}", version).trim(),
            Some(hashmap! {
                "version" => version
            }),
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
                None,
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
        let env_vars = self.get_env_vars();
        let entries = self.buildpack_plan_entries(&context.buildpack_plan);

        if let Some(options) = entries.get("node") {
            let layer_data = context.handle_layer(layer_name!("node"), NodeLayer::new(options))?;
            self.set_layer_env_vars(&layer_data.env);
        }

        if entries.contains_key("node_modules") {
            context.handle_layer(layer_name!("node_modules"), NodeModulesLayer)?;
        }

        self.restore_env_vars(env_vars);
        BuildResultBuilder::new().build()
    }
}

struct NodeLayer {
    /// The semantic version requirement for the `node` binary
    requirement: String,
}

impl NodeLayer {
    fn new(options: &LayerOptions) -> Self {
        let requirement = options
            .get("version")
            .cloned()
            .unwrap_or_else(|| "lts".to_string());

        let requirement = if requirement == "lts" {
            // TODO: Determine LTS without doing a fetch, perhaps based on date
            // https://nodejs.org/en/about/releases/
            "^16".to_string()
        } else {
            requirement
        };

        Self { requirement }
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

        let node = NodeBinary {}.ensure_version_sync(&self.requirement)?;
        let version = node.version()?.to_string();

        if context.is_local() {
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

        // Store version in metadata to detect if layer is stale in `existing_layer_strategy()`
        let metadata = LayerVersionMetadata { version };

        // Set `NODE_PATH` so that the `lib/node_modules` (which has `npm` for example) can be found
        let layer_env = LayerEnv::new().chainable_insert(
            Scope::All,
            ModificationBehavior::Prepend,
            "NODE_PATH",
            layer_path.join("lib").join(NODE_MODULES),
        );

        LayerResultBuilder::new(metadata).env(layer_env).build()
    }
}

struct NodeModulesLayer;

impl NodeModulesLayer {
    /// Generate hash of files that affect which packages are installed into `node_modules`
    ///
    /// The hash is the combined contents of `package-lock.json` and `package.json`.
    /// This means that if either one is changed or removed that the hash will change.
    fn generate_packages_hash(&self, app_dir: &Path) -> String {
        let content = [
            read_to_string(app_dir.join(PACKAGE_LOCK)).unwrap_or_default(),
            read_to_string(app_dir.join(PACKAGE_JSON)).unwrap_or_default(),
        ]
        .concat();
        str_sha256_hex(&content)
    }
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
        let package_hash = self.generate_packages_hash(&context.app_dir);
        let strategy = if package_hash == layer_data.content_metadata.metadata.hash {
            tracing::info!("Existing `node_modules` layer has same packages hash; will keep",);
            ExistingLayerStrategy::Keep
        } else {
            tracing::info!(
                "Existing `node_modules` layer has different packages hash; will update"
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
        let app_path = &context.app_dir.canonicalize()?;
        let layer_path = &layer_path.canonicalize()?;

        // Call the `npm-cli.js` script installed in the `node` layer
        // This is done, rather than executing `bin/npm` directly, there are issues with node `require`
        // module resolution when the latter is done.
        let mut node = NodeBinary {}.require_sync()?;
        let npm = layer_path
            .parent()
            .expect("Should have parent")
            .join("node")
            .join("lib")
            .join(NODE_MODULES)
            .join("npm")
            .join("bin")
            .join("npm-cli.js")
            .into_os_string();

        if context.is_local() {
            // Do the install in the app directory as normal
            node.run_sync([npm, "install".into()])?;
        } else {
            // Do the install in the layer.
            // Alternative, more complicated approaches to this e.g. doing a local install and then copying
            // over to layers and/or symlinking are problematic.

            // Despite some confusion online it seems that at present it is necessary to copy over these
            // files when using `--prefix`
            copy_if_exists(app_path.join(PACKAGE_JSON), layer_path.join(PACKAGE_JSON))?;
            copy_if_exists(app_path.join(PACKAGE_LOCK), layer_path.join(PACKAGE_LOCK))?;

            // Use `layer_path/cache` as the NPM cache
            node.env_list(&[(
                "NPM_CONFIG_CACHE",
                layer_path.join("cache").into_os_string(),
            )]);

            node.run_sync([npm, "install".into(), "--prefix".into(), layer_path.into()])?;

            // Remove the files, so they are not there next time
            remove_file(layer_path.join(PACKAGE_JSON)).ok();
            remove_file(layer_path.join(PACKAGE_LOCK)).ok();
        }

        // Generate packages hash to detect if layer is stale in `existing_layer_strategy()`
        let metadata = LayerHashMetadata {
            hash: self.generate_packages_hash(app_path),
        };

        // Set the `NODE_PATH` so that the `node_modules` can be found
        let layer_env = LayerEnv::new().chainable_insert(
            Scope::All,
            ModificationBehavior::Prepend,
            "NODE_PATH",
            layer_path.join(NODE_MODULES),
        );

        LayerResultBuilder::new(metadata).env(layer_env).build()
    }
}
