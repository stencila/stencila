use std::{
    fs,
    path::{Path, PathBuf},
};

use binary_node::{Binary, BinaryTrait, NodeBinary};
use buildpack::{
    eyre::{self, bail},
    fs_utils::{copy_dir_all, symlink_dir, symlink_file},
    libcnb::{
        self,
        build::{BuildContext, BuildResult, BuildResultBuilder},
        data::{build_plan::BuildPlan, layer_content_metadata::LayerTypes, layer_name},
        detect::{DetectContext, DetectResult, DetectResultBuilder},
        generic::{GenericMetadata, GenericPlatform},
        layer::{Layer, LayerResult, LayerResultBuilder},
        Buildpack,
    },
    platform_is_stencila, BuildpackTrait,
};

pub struct NodeBuildpack;

impl BuildpackTrait for NodeBuildpack {
    fn toml() -> &'static str {
        include_str!("../buildpack.toml")
    }
}

impl Buildpack for NodeBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = eyre::Report;

    fn detect(&self, _context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        // Read `.tool-versions` for Node.js version
        const TOOL_VERSIONS: &str = ".tool-versions";
        let tool_versions = Self::tool_versions();

        // Read `.nvmrc` for Node.js version
        const NVMRC: &str = ".nvmrc";
        let nvmrc = fs::read_to_string(NVMRC)
            .map(|content| content.trim().to_string())
            .ok();

        // Read `package.json` for Node.js version
        const PACKAGE_JSON: &str = "package.json";
        let package_json = fs::read_to_string(PACKAGE_JSON)
            .ok()
            .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok());

        // Detect `package-lock.json`
        const PACKAGE_LOCK: &str = "package-lock.json";
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

        // Resolve Node.js version from `.tool-versions`, `.nvmrc` or `package.json`
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
            (String::new(), "")
        };

        // Require and provide Node.js
        let (require, provide) = Self::require_and_provide(
            format!("node {}", version),
            source,
            format!("Install Node.js {}", version),
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
                "Install NPM packages using `npm install`",
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
        let entries: Vec<String> = context
            .buildpack_plan
            .entries
            .iter()
            .map(|entry| entry.name.clone())
            .collect();

        for entry in entries {
            let (name, args) = Self::split_entry_name(&entry);
            match name.as_str() {
                "node" => {
                    context.handle_layer(layer_name!("node"), NodeLayer::new(args))?;
                }
                "npm" => {
                    context.handle_layer(layer_name!("npm"), NpmLayer::new(args))?;
                }
                _ => (),
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

        // Ensure a version meeting the semver is installed
        let node = NodeBinary {}.require_sync(Some(requirement), true)?;

        // Symlink/copy the installation into the layer
        let dest = layer_path.join(node.version()?);
        if platform_is_stencila(&context.platform) {
            if node.is_stencila_install() {
                let source = node.grandparent()?;
                symlink_dir(source, &dest)?;
            } else {
                let source = node.parent()?;
                fs::create_dir_all(&dest)?;
                symlink_file(node.path, dest.join(node.name))?;
                symlink_file(source.join("npm"), dest.join("npm"))?;
                symlink_file(source.join("npx"), dest.join("npx"))?;
            }
        } else {
            #[allow(clippy::collapsible_else_if)]
            if node.is_stencila_install() {
                let source = node.grandparent()?;
                copy_dir_all(source, &dest)?;
            } else {
                bail!("Only able to build `node` layer if it has been installed by Stencila")
            }
        }

        LayerResultBuilder::new(GenericMetadata::default()).build()
    }
}

struct NpmLayer;

impl NpmLayer {
    fn new(_args: Vec<String>) -> Self {
        NpmLayer
    }
}

impl Layer for NpmLayer {
    type Buildpack = NodeBuildpack;
    type Metadata = GenericMetadata;

    fn types(&self) -> LayerTypes {
        // Layer is available at build time and is cached but is not needed for
        // launch time because packages are installed into the `node_modules` of the
        // working directory
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
        // Require npm
        // TODO: Should we use a specific version of npm? The version installed by this pack?
        // Or one with expected behavior wrt to package-lock.json, peerDependencies etc.
        let mut npm = Binary::unregistered("npm").require_sync(None, false)?;

        // If this is not a local build then make the layer the NPM cache
        if !platform_is_stencila(&context.platform) {
            npm.set_env(&[("NPM_CONFIG_CACHE", layer_path.canonicalize()?.as_os_str())]);
        }

        // Do the install
        npm.run_sync(&["install"])?;

        LayerResultBuilder::new(GenericMetadata::default()).build()
    }
}
