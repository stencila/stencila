use std::{
    collections::HashMap,
    env::{self, current_dir, set_current_dir},
    ffi::OsString,
    fs::{create_dir_all, read_to_string, remove_file},
    path::{Path, PathBuf},
};

use binary_poetry::PoetryBinary;
use binary_python::{BinaryInstallation, BinaryTrait, PythonBinary};
use buildpack::{
    eyre::{self, bail},
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
    toml, tracing, BuildpackTrait, LayerHashMetadata, LayerVersionMetadata, SYSTEM_INSTALLED,
};

pub struct PythonBuildpack;

impl BuildpackTrait for PythonBuildpack {
    fn toml() -> &'static str {
        include_str!("../buildpack.toml")
    }
}

const POETRY_LOCK: &str = "poetry.lock";
const PYPROJECT_TOML: &str = "pyproject.toml";
const REQUIREMENTS_TXT: &str = "requirements.txt";
const RUNTIME_TXT: &str = "runtime.txt";
const TOOL_VERSIONS: &str = ".tool-versions";
const VENV: &str = ".venv";

impl Buildpack for PythonBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = eyre::Report;

    fn detect(&self, _context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        // Read `.tool-versions` for Python version
        let tool_versions = Self::tool_versions();

        // Read `pyproject.toml` for Python version and packages
        let pyproject_toml = read_to_string(PYPROJECT_TOML)
            .ok()
            .and_then(|json| toml::from_str::<toml::Value>(&json).ok());

        // May read `runtime.txt` for Python version
        let runtime_txt = PathBuf::from(RUNTIME_TXT);

        // Detect `poetry.lock`
        let poetry_lock = PathBuf::from(POETRY_LOCK);

        // Detect `requirements.txt`
        let requirements_txt = PathBuf::from(REQUIREMENTS_TXT);

        // Fail early
        if !(tool_versions.contains_key("python")
            || pyproject_toml.is_some()
            || runtime_txt.exists()
            || poetry_lock.exists()
            || requirements_txt.exists()
            || Self::any_exist(&["main.py", "index.py"]))
        {
            return DetectResultBuilder::fail().build();
        }

        let mut requires = Vec::new();
        let mut provides = Vec::new();

        // Resolve Python version from `.tool-versions`, `runtime.txt` or `pyproject.toml`
        let (version, source) = if let Some(version) = tool_versions.get("python") {
            (version.to_string(), TOOL_VERSIONS)
        } else if let Some(semver_req) = pyproject_toml
            .as_ref()
            .and_then(|project| project.get("tool"))
            .and_then(|tool| tool.get("poetry"))
            .and_then(|poetry| poetry.get("dependencies"))
            .and_then(|dependencies| dependencies.get("python"))
            .and_then(|semver_req| semver_req.as_str())
        {
            (semver_req.to_string(), PYPROJECT_TOML)
        } else if let Some(version) = read_to_string(runtime_txt).ok().and_then(|content| {
            content
                .trim()
                .strip_prefix("python-")
                .map(|version| version.to_string())
        }) {
            (version, RUNTIME_TXT)
        } else if let Some(version) = (PythonBinary {}).installed_version(None) {
            (version, SYSTEM_INSTALLED)
        } else {
            ("".to_string(), "")
        };

        // Require and provide Python
        let (require, provide) = Self::require_and_provide(
            format!("python {}", version).trim(),
            source,
            format!("Install Python {}", version).trim(),
        );
        requires.push(require);
        provides.push(provide);

        // Determine how packages are to be installed
        if pyproject_toml.is_some() || poetry_lock.exists() {
            let source = if poetry_lock.exists() {
                POETRY_LOCK
            } else {
                PYPROJECT_TOML
            };

            let (require, provide) = Self::require_and_provide("poetry", source, "Install Poetry");
            requires.push(require);
            provides.push(provide);

            let (require, provide) = Self::require_and_provide(
                "venv poetry",
                source,
                "Install Python packages into virtual environment using Poetry",
            );
            requires.push(require);
            provides.push(provide);
        } else if requirements_txt.exists() {
            let (require, provide) = Self::require_and_provide(
                "venv pip",
                REQUIREMENTS_TXT,
                "Install Python packages into virtual environment using Pip",
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

        if let Some(args) = entries.get("python") {
            context.handle_layer(layer_name!("python"), PythonLayer::new(args.clone()))?;
        }

        if entries.contains_key("poetry") {
            context.handle_layer(layer_name!("poetry"), PoetryLayer::new())?;
        }

        if let Some(args) = entries.get("venv") {
            context.handle_layer(layer_name!("venv"), VenvLayer::new(args.clone()))?;
        }

        BuildResultBuilder::new().build()
    }
}

struct PythonLayer {
    /// The semantic version requirement for the Python binary
    requirement: String,
}

impl PythonLayer {
    fn new(args: Vec<String>) -> Self {
        // Join args with commas because semver requirement parser expects that is
        // how parts of a requirement are separated
        let requirement = args.join(",");
        PythonLayer { requirement }
    }
}

impl Layer for PythonLayer {
    type Buildpack = PythonBuildpack;
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
        let installed = PythonBinary {}.semver_version_matches(
            &layer_data.content_metadata.metadata.version,
            &self.requirement,
        )?;
        let strategy = if installed {
            tracing::info!(
                "Existing `python` layer has `./bin/python` matching semver requirement `{}`; will keep",
                self.requirement
            );
            ExistingLayerStrategy::Keep
        } else {
            tracing::info!(
                "Existing `python` layer does not have `./bin/python` matching semver requirement `{}`; will recreate",
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
            "Creating `python` layer with semver requirement `{}`",
            self.requirement
        );

        let python = PythonBinary {}.require_sync(Some(self.requirement.clone()), true)?;
        let version = python.version()?.to_string();

        let mut layer_env = LayerEnv::new();

        if is_local_build(context) {
            if python.is_stencila_install() {
                tracing::info!("Linking to `python {}` installed by Stencila", version);
                let source = python.grandparent()?;

                symlink_dir(source.join("bin"), &layer_path.join("bin"))?;
                symlink_dir(source.join("lib"), &layer_path.join("lib"))?;
            } else {
                tracing::info!("Linking to `python {}` installed on system", version);

                let bin_path = layer_path.join("bin");
                create_dir_all(&bin_path)?;
                symlink_file(python.path, bin_path.join(python.name))?;
            }
        } else {
            #[allow(clippy::collapsible_else_if)]
            if python.is_stencila_install() {
                tracing::info!("Moving `python {}` installed by Stencila", version);
                let source = python.grandparent()?;

                move_dir_all(source, &layer_path)?;
                layer_env.insert(
                    Scope::All,
                    ModificationBehavior::Override,
                    "PYTHONHOME",
                    layer_path,
                );
            } else {
                tracing::info!("Linking to `python {}` installed on stack image", version);

                let bin_path = layer_path.join("bin");
                create_dir_all(&bin_path)?;
                symlink_file(python.path, bin_path.join(python.name))?;
            }
        }

        LayerResultBuilder::new(LayerVersionMetadata { version })
            .env(layer_env)
            .build()
    }
}

struct PoetryLayer {
    // The semantic version requirement for the Poetry binary
    // Currently fixed
    requirement: String,
}

impl PoetryLayer {
    fn new() -> Self {
        let requirement = ">=1,<2".to_string();
        PoetryLayer { requirement }
    }
}

impl Layer for PoetryLayer {
    type Buildpack = PythonBuildpack;
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
        let installed = PoetryBinary {}.semver_version_matches(
            &layer_data.content_metadata.metadata.version,
            &self.requirement,
        )?;
        let strategy = if installed {
            tracing::info!(
                "Existing `poetry` layer has `./bin/poetry` matching semver requirement `{}`; will keep",
                self.requirement
            );
            ExistingLayerStrategy::Keep
        } else {
            tracing::info!(
                "Existing `poetry` layer does not have `./bin/poetry` matching semver requirement `{}`; will recreate",
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
            "Creating `poetry` layer with semver requirement `{}`",
            self.requirement
        );

        let version = if is_local_build(context) {
            let poetry = PoetryBinary {}.require_sync(Some(self.requirement.clone()), true)?;
            let version = poetry.version()?.to_string();

            if poetry.is_stencila_install() {
                tracing::info!("Linking to `poetry {}` installed by Stencila", version);
                let source = poetry.grandparent()?;

                symlink_dir(source.join("bin"), &layer_path.join("bin"))?;
            } else {
                tracing::info!("Linking to `poetry {}` installed on system", version);

                let bin_path = layer_path.join("bin");
                create_dir_all(&bin_path)?;
                symlink_file(poetry.path, bin_path.join(poetry.name))?;
            }

            version
        } else {
            let poetry = PoetryBinary {}.find_version(&self.requirement).ok();

            if let Some(poetry) = poetry {
                let version = poetry.version()?.to_string();
                tracing::info!("Using `poetry {}` installed on stack image", version);

                version
            } else {
                tracing::info!(
                    "Installing `poetry` with semver requirement `{}`",
                    self.requirement
                );

                // Because of how Poetry installs itself, we need to install directly into the
                // layer, rather than copy it from somewhere else. Install may fail if we don't
                // use a recent enough version of Python with `ensurepip` module.
                // So prepend `PATH` with the version installed in sibling layer which
                // the `PoetryBinary::install_version` should pick up and use.
                let python_layer_bin = layer_path
                    .canonicalize()?
                    .parent()
                    .expect("Should have parent")
                    .join("python")
                    .join("bin");
                env::set_var("PATH", PythonBuildpack::prepend_path(&python_layer_bin)?);

                PoetryBinary {}.install_in_sync(
                    Some(self.requirement.clone()),
                    Some(layer_path.to_path_buf()),
                )?
            }
        };

        LayerResultBuilder::new(LayerVersionMetadata { version }).build()
    }
}

struct VenvLayer {
    /// The tool used to do the installation of packages ("pip" or "poetry")
    tool: String,
}

impl VenvLayer {
    fn new(args: Vec<String>) -> Self {
        let tool = args.first().cloned().unwrap_or_else(|| "pip".to_string());
        VenvLayer { tool }
    }
}

/// Generate hash for Poetry & Pip related files in an app directory
///
/// The hash is of the combined contents of `poetry.lock`, `pyproject.toml`, `requirements.txt`.
/// This means that if any one is changed or removed that the hash will change.
fn generate_packages_hash(app_dir: &Path) -> String {
    let content = [
        read_to_string(app_dir.join(POETRY_LOCK)).unwrap_or_default(),
        read_to_string(app_dir.join(PYPROJECT_TOML)).unwrap_or_default(),
        read_to_string(app_dir.join(REQUIREMENTS_TXT)).unwrap_or_default(),
    ]
    .concat();
    str_sha256_hex(&content)
}

impl Layer for VenvLayer {
    type Buildpack = PythonBuildpack;
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
        let package_hash = generate_packages_hash(&context.app_dir);
        let strategy = if package_hash == layer_data.content_metadata.metadata.hash {
            tracing::info!("Existing `venv` layer has same packages hash; will keep",);
            ExistingLayerStrategy::Keep
        } else {
            tracing::info!("Existing `venv` layer has different packages hash; will update");
            ExistingLayerStrategy::Update
        };
        Ok(strategy)
    }

    fn create(
        &self,
        context: &BuildContext<Self::Buildpack>,
        layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, eyre::Report> {
        tracing::info!("Creating `venv` layer");
        self.install(context, layer_path)
    }

    fn update(
        &self,
        context: &BuildContext<Self::Buildpack>,
        layer_data: &libcnb::layer::LayerData<Self::Metadata>,
    ) -> Result<LayerResult<Self::Metadata>, <Self::Buildpack as Buildpack>::Error> {
        tracing::info!("Updating `venv` layer");
        self.install(context, &layer_data.path)
    }
}

impl VenvLayer {
    fn install(
        &self,
        context: &BuildContext<PythonBuildpack>,
        layer_path: &Path,
    ) -> Result<LayerResult<LayerHashMetadata>, eyre::Report> {
        let app_path = &context.app_dir.canonicalize()?;
        let layer_path = &layer_path.canonicalize()?;

        let venv_path = if is_local_build(context) {
            // Attempt to use an existing virtual environment
            // The name `.venv` is most commonly used (e.g. Poetry uses that) but
            // we also look for `venv` and `env`.
            let dotvenv = app_path.join(VENV);
            let venv = app_path.join("venv");
            let env = app_path.join("env");
            if dotvenv.join("bin").join("python3").exists() {
                dotvenv
            } else if venv.join("bin").join("python3").exists() {
                venv
            } else if env.join("bin").join("python3").exists() {
                env
            } else {
                dotvenv
            }
        } else {
            // For CNB builds always use `.venv` in the layer
            layer_path.join(VENV)
        };

        let python_layer_path = layer_path
            .parent()
            .expect("Should have parent")
            .join("python");

        if !venv_path.exists() {
            // Create a `.venv` virtual environment using the installed version of Python
            // This is important because if affects the binary in the `.venv`
            let python = BinaryInstallation::new(
                "python",
                python_layer_path.join("bin").join("python"),
                None,
                vec![],
            );

            python.run_sync(&[
                OsString::from("-m"),
                "venv".into(),
                "--clear".into(),
                venv_path.clone().into(),
            ])?;
        }

        if self.tool == "pip" {
            tracing::info!(
                "Installing packages into `{}` using Pip",
                venv_path.display()
            );

            // Use the Python in the virtual environment
            let mut python = BinaryInstallation::new(
                "python",
                venv_path.join("bin").join("python"),
                None,
                vec![],
            );

            // If Stencila is not the platform use the layer as the Pip cache
            if is_cnb_build(context) {
                python.env_list(&[("PIP_CACHE_DIR", layer_path.as_os_str())]);
            }

            // By using the python in the `.venv` we get packages installed into it
            python.run_sync(&["-m", "pip", "install", "-r", REQUIREMENTS_TXT])?;
        } else {
            tracing::info!(
                "Installing packages into `{}` using Poetry",
                venv_path.display()
            );

            let mut envs: Vec<(OsString, OsString)> = vec![
                // Set env vars to keep Poetry happy
                (
                    "PATH".into(),
                    PythonBuildpack::prepend_path(&python_layer_path.join("bin"))?,
                ),
                ("PYTHONHOME".into(), python_layer_path.into()),
                // Ensure that a `.venv` folder in the working directory is used (instead of a system level one)
                ("POETRY_VIRTUALENVS_IN_PROJECT".into(), "true".into()),
            ];

            // If a CNB build use the `layer_path/cache` as the Poetry cache
            if is_cnb_build(context) {
                envs.push(("POETRY_CACHE_DIR".into(), layer_path.join("cache").into()));
            }

            // Use `poetry` installed in the sibling `poetry` layer
            let poetry = BinaryInstallation::new(
                "poetry",
                layer_path
                    .parent()
                    .expect("Should have parent")
                    .join("poetry")
                    .join("bin")
                    .join("poetry"),
                None,
                envs,
            );

            if is_local_build(context) {
                // Do the install in the app directory as normal
                poetry.run_sync(["install"])?;
            } else {
                // Do the install in the layer.
                // Because we can't tell poetry where to install, we need to copy the
                // files into the layer, and `cd` into it.
                // See https://github.com/python-poetry/poetry/pull/799

                copy_if_exists(app_path.join(POETRY_LOCK), layer_path.join(POETRY_LOCK))?;
                copy_if_exists(
                    app_path.join(PYPROJECT_TOML),
                    layer_path.join(PYPROJECT_TOML),
                )?;

                let current_dir = current_dir()?;
                set_current_dir(layer_path)?;
                let result = poetry.run_sync(["install"]);
                set_current_dir(current_dir)?;

                // Remove the files, so they are not there next time
                remove_file(layer_path.join(POETRY_LOCK)).ok();
                remove_file(layer_path.join(PYPROJECT_TOML)).ok();

                result?;
            }
        };

        // Add the virtual env packages to the PYTHONPATH
        let lib_dir = venv_path.join("lib");
        let python_minor = match lib_dir
            .read_dir()?
            .find_map(|entry| entry.ok().map(|entry| entry.file_name()))
        {
            Some(dir_name) => dir_name,
            None => bail!("Could not resolve pythonX.X library directory"),
        };
        let layer_env = LayerEnv::new().chainable_insert(
            Scope::All,
            ModificationBehavior::Prepend,
            "PYTHONPATH",
            lib_dir.join(python_minor).join("site-packages"),
        );

        // Generate a 'packages hash' to detect if layer is stale in `existing_layer_strategy()`
        let metadata = LayerHashMetadata {
            hash: generate_packages_hash(app_path),
        };

        LayerResultBuilder::new(metadata).env(layer_env).build()
    }
}
