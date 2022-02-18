use std::{
    collections::HashMap,
    ffi::OsString,
    fs::{self, remove_dir_all},
    path::{Path, PathBuf},
};

use binary_node::{BinaryInstallation, BinaryTrait, NodeBinary};
use buildpack::{
    eyre,
    fs_utils::{copy_dir_all, symlink_dir, symlink_file},
    hash_utils::file_sha256_hex,
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
                fs::create_dir_all(&bin_path)?;
                symlink_file(node.path, bin_path.join(node.name))?;
                symlink_file(source.join("corepack"), bin_path.join("corepack"))?;
                symlink_file(source.join("npm"), bin_path.join("npm"))?;
                symlink_file(source.join("npx"), bin_path.join("npx"))?;

                let lib_path = layer_path.join("lib");
                fs::create_dir_all(&lib_path)?;
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

                copy_dir_all(&source, layer_path)?;
                remove_dir_all(source)?;
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
/// If the directory has a `package-lock.json` that will be used. If not
/// `package.json` will be used. This means that if the user removes
/// or updates `package-lock.json` the layer will be updated. If there is no lock file
/// then the layer will only be updated if there are changes in `package.json`.
fn generate_package_hash(app_dir: &Path) -> Result<String, eyre::Report> {
    file_sha256_hex(app_dir.join(PACKAGE_LOCK))
        .or_else(|_| file_sha256_hex(app_dir.join(PACKAGE_JSON)))
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
        let package_hash = generate_package_hash(&context.app_dir).unwrap_or_default();
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
        // Use `node` and `npm` installed in the `node` layer
        let node_layer = layer_path
            .canonicalize()?
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
        // This is done, rather than executing `bin/npm` directly there are issues with node `require`
        // module resolution when the latter is used.
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
            .display()
            .to_string();

        // Read any `package-lock.json` so that we can restore it, or remove the new one that
        // install creates. This needs to be done to avoid affecting the `package_hash` generated at the end
        // of this function (and because we should leave app_dir untouched)
        let package_lock_path = context.app_dir.join(PACKAGE_LOCK);
        let package_lock_contents = fs::read_to_string(&package_lock_path).ok();

        // Do the install
        node.run_sync(&[&npm, "install"])?;

        // Remove or restore `package-lock.json`
        match package_lock_contents {
            Some(contents) => fs::write(&package_lock_path, contents)?,
            None => {
                if package_lock_path.exists() {
                    fs::remove_file(&package_lock_path)?
                }
            }
        }

        // Generate a 'package hash' to be able to tell if layer is stale in `existing_layer_strategy`
        let hash = generate_package_hash(&context.app_dir)?;

        let mut layer_env = LayerEnv::new();

        // If this is a CNB build move `node_modules` to `layer_path/node_modules` because
        // "Implementations MUST NOT write to any other location than layer_path".
        //
        // It feels like there should be other ways to do this, but using `--prefix` flag or
        // setting `NPM_CONFIG_PREFIX` or `NODE_PATH` didn't work. At least one other buildpack does it this way too:
        // https://github.com/paketo-buildpacks/npm-install/blob/83ebc22dde31d3f7423a215c8eb7549a180bbf35/install_build_process.go#L60-L76
        //
        // Instead of creating a symlink to `node_modules` in the app_dir, add it to NODE_PATH.
        if is_cnb_build(context) {
            let app_node_modules = context.app_dir.join(NODE_MODULES);
            let layer_node_modules = layer_path.join(NODE_MODULES);
            copy_dir_all(&app_node_modules, &layer_node_modules)?;
            remove_dir_all(&app_node_modules)?;
            layer_env.insert(
                Scope::All,
                ModificationBehavior::Prepend,
                "NODE_PATH",
                layer_node_modules,
            );
        }

        LayerResultBuilder::new(LayerHashMetadata { hash })
            .env(layer_env)
            .build()
    }
}
