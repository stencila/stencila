use std::{
    env::{self, current_dir, set_current_dir},
    ffi::OsString,
    fs::{create_dir_all, read_to_string, remove_file},
    path::{Path, PathBuf},
};

use binary_poetry::PoetryBinary;
use binary_python::{Binary, BinaryInstallation, BinaryTrait, PythonBinary};
use buildpack::{
    eyre::{self, bail},
    fs_utils::{copy_if_exists, symlink_dir, symlink_file},
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
    toml, tracing, BuildpackContext, BuildpackTrait, LayerHashMetadata, LayerOptions,
    LayerVersionMetadata,
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

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
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
        } else {
            ("*".to_string(), "")
        };

        if context.is_cnb() {
            // Ensure `version` is a valid version and not a semver requirement so that we can
            // specify the `apt` package versions to install. Log errors and have fallbacks rather than returning
            // `Err` because the latter can cause silent detect fails
            let fallback_version = "3.10.2".to_string();
            let python_binary = PythonBinary {};
            let versions = match python_binary.versions_sync(env::consts::OS) {
                Ok(versions) => versions,
                Err(error) => {
                    tracing::warn!("While getting Python versions: {}", error);
                    vec![fallback_version.clone()]
                }
            };
            let (version, minor_version) = match python_binary
                .semver_versions_matching(versions.clone(), &version)
                .first()
            {
                Some(version) => (
                    version.to_string(),
                    python_binary
                        .semver_version_minor(version)
                        .unwrap_or_default(),
                ),
                None => {
                    tracing::warn!("Unable to find version of Python meeting semver requirement `{}`; will use latest", version);
                    let version = versions.first().unwrap_or(&fallback_version).to_string();
                    let version_minor = python_binary
                        .semver_version_minor(&version)
                        .unwrap_or_default();
                    (version, version_minor)
                }
            };

            let release = sys_info::linux_os_release()
                .ok()
                .and_then(|info| info.version_codename)
                .unwrap_or_default();
            let repos = format!(
                "deb [trusted=yes] https://ppa.launchpadcontent.net/deadsnakes/ppa/ubuntu {} main",
                release
            );

            let packages = [
                format!("python{}={}-*", minor_version, version),
                format!("python{}-venv={}-*", minor_version, version),
            ]
            .join(",");

            requires.push(Self::require(
                "apt_packages",
                "",
                "Install `apt` packages required for Python",
                Some(hashmap! {
                    "repos" => repos,
                    "packages" => packages
                }),
            ));
        }

        // Require and provide Python
        let (require, provide) = Self::require_and_provide(
            "python",
            source,
            format!("Install Python {}", version).trim(),
            Some(hashmap! {
                "version" => version
            }),
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

            let (require, provide) =
                Self::require_and_provide("poetry", source, "Install Poetry", None);
            requires.push(require);
            provides.push(provide);

            let (require, provide) = Self::require_and_provide(
                "venv",
                source,
                "Install Python packages into virtual environment using Poetry",
                Some(hashmap! {
                    "package_manager" => "poetry".to_string()
                }),
            );
            requires.push(require);
            provides.push(provide);
        } else if requirements_txt.exists() {
            let (require, provide) = Self::require_and_provide(
                "venv",
                REQUIREMENTS_TXT,
                "Install Python packages into virtual environment using Pip",
                Some(hashmap! {
                    "package_manager" => "pip".to_string()
                }),
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

        if let Some(options) = entries.get("python") {
            let layer_data =
                context.handle_layer(layer_name!("python"), PythonLayer::new(options))?;
            self.set_layer_env_vars(&layer_data.env);
        }

        if entries.contains_key("poetry") {
            let layer_data = context.handle_layer(layer_name!("poetry"), PoetryLayer::new())?;
            self.set_layer_env_vars(&layer_data.env);
        }

        if let Some(options) = entries.get("venv") {
            context.handle_layer(layer_name!("venv"), VenvLayer::new(options))?;
        }

        self.restore_env_vars(env_vars);
        BuildResultBuilder::new().build()
    }
}

struct PythonLayer {
    /// The semantic version requirement for the Python binary
    requirement: String,
}

impl PythonLayer {
    fn new(options: &LayerOptions) -> Self {
        let requirement = options
            .get("version")
            .cloned()
            .unwrap_or_else(|| "*".to_string());

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
        let version = &layer_data.content_metadata.metadata.version;
        let installed = PythonBinary {}.semver_version_matches(version, &self.requirement)?;
        let strategy = if installed {
            tracing::info!(
                "Existing `python` layer has `python {}` which match semver requirement `{}`; will keep",
                version,
                self.requirement
            );
            ExistingLayerStrategy::Keep
        } else {
            tracing::info!(
                "Existing `python` layer has `python {}` which does not match semver requirement `{}`; will recreate",
                version,
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

        let layer_env = LayerEnv::new();

        let version = if context.is_local() {
            let python = PythonBinary {}.ensure_version_sync(&self.requirement)?;
            let version = python.version()?.to_string();

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

            version
        } else {
            tracing::info!("Patching `python` installed by `apt` buildpack");

            let apt_packages = layer_path
                .join("..")
                .join("..")
                .join("stencila_apt")
                .join("apt_packages")
                .canonicalize()?;

            // Symlink from the installed binary to both `python` and `python3`
            let bin_dir = apt_packages.join("usr").join("bin");
            for entry in bin_dir.read_dir()?.flatten() {
                let path = entry.path();
                let is_python = path
                    .file_name()
                    .map_or(false, |name| name.to_string_lossy().starts_with("python3"));
                if is_python {
                    symlink_file(bin_dir.join(&path), bin_dir.join("python"))?;
                    symlink_file(bin_dir.join(path), bin_dir.join("python3"))?;
                    break;
                }
            }

            // The Python installation should now work, verify that is does and get the version
            let python = Binary::named("python")
                .find_in(apt_packages.join("usr").join("bin").as_os_str())?;
            let version = match python.version() {
                Ok(version) => version,
                Err(error) => {
                    tracing::warn!("Unable to get version of Python: {}", error);
                    // Return a version-ish string so that the image can at least be built
                    // and run for debugging purposes
                    "0.0.0"
                }
            }
            .to_string();

            // Ensure `pip` is installed
            python.run_sync(&["-m", "ensurepip"])?;

            version
        };

        // Store version in metadata to detect if layer is stale in `existing_layer_strategy()`
        let metadata = LayerVersionMetadata { version };

        LayerResultBuilder::new(metadata).env(layer_env).build()
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

        let version = if context.is_local() {
            let poetry = PoetryBinary {}.ensure_version_sync(&self.requirement)?;
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
                // layer, rather than copy it from somewhere else. Install may fail if a
                // recent enough version of Python with `ensurepip` module is not available.
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
    /// The package manager used to do the installation of packages
    ///
    /// Currently can be "pip" or "poetry"
    package_manager: String,
}

impl VenvLayer {
    fn new(options: &LayerOptions) -> Self {
        let package_manager = options
            .get("package_manager")
            .cloned()
            .unwrap_or_else(|| "pip".to_string());

        VenvLayer { package_manager }
    }

    /// Generate hash for Poetry & Pip related files in an app directory
    ///
    /// The hash is the combined contents of `poetry.lock`, `pyproject.toml`, `requirements.txt`.
    /// This means that if any one is changed or removed that the hash will change.
    fn generate_packages_hash(&self, app_dir: &Path) -> String {
        let content = [
            read_to_string(app_dir.join(POETRY_LOCK)).unwrap_or_default(),
            read_to_string(app_dir.join(PYPROJECT_TOML)).unwrap_or_default(),
            read_to_string(app_dir.join(REQUIREMENTS_TXT)).unwrap_or_default(),
        ]
        .concat();
        str_sha256_hex(&content)
    }
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
        let package_hash = self.generate_packages_hash(&context.app_dir);
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

        let venv_path = if context.is_local() {
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

        if self.package_manager == "pip" {
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

            // If a CNB build use the layer as the Pip cache
            if context.is_cnb() {
                python.env_list(&[("PIP_CACHE_DIR", layer_path.as_os_str())]);
            }

            // By using the python in the `.venv` we get packages installed into it
            python.run_sync(&["-m", "pip", "install", "-r", REQUIREMENTS_TXT])?;
        } else {
            tracing::info!(
                "Installing packages into `{}` using Poetry",
                venv_path.display()
            );

            let mut poetry = PoetryBinary {}.require_version_sync(">=1")?;
            let mut envs: Vec<(OsString, OsString)> = vec![
                // Ensure that a `.venv` folder in the working directory is used (instead of a system level one)
                ("POETRY_VIRTUALENVS_IN_PROJECT".into(), "true".into()),
            ];

            if context.is_local() {
                // Do the install in the app directory as normal
                poetry.env_list(&envs);
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

                // Use the `layer_path/cache` as the Poetry cache
                envs.push(("POETRY_CACHE_DIR".into(), layer_path.join("cache").into()));
                poetry.env_list(&envs);

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

        // Generate packages hash to detect if layer is stale in `existing_layer_strategy()`
        let metadata = LayerHashMetadata {
            hash: self.generate_packages_hash(app_path),
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

        LayerResultBuilder::new(metadata).env(layer_env).build()
    }
}
