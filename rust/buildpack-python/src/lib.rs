use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use binary_poetry::PoetryBinary;
use binary_python::{BinaryInstallation, BinaryTrait, PythonBinary};
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
    platform_is_stencila, toml, tracing, BuildpackTrait,
};

pub struct PythonBuildpack;

impl BuildpackTrait for PythonBuildpack {
    fn toml() -> &'static str {
        include_str!("../buildpack.toml")
    }
}

const INSTALLED: &str = "<installed>";
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
        let pyproject_toml = fs::read_to_string(PYPROJECT_TOML)
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
        } else if let Some(version) = fs::read_to_string(runtime_txt).ok().and_then(|content| {
            content
                .trim()
                .strip_prefix("python-")
                .map(|version| version.to_string())
        }) {
            (version, RUNTIME_TXT)
        } else if let Some(version) = (PythonBinary {}).installed_version(None) {
            (version, INSTALLED)
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

        // Determine how PyPI packages are to be installed
        if pyproject_toml.is_some() || poetry_lock.exists() {
            let (require, provide) = Self::require_and_provide(
                "poetry install",
                if poetry_lock.exists() {
                    POETRY_LOCK
                } else {
                    PYPROJECT_TOML
                },
                "Install PyPI packages using Poetry",
            );
            requires.push(require);
            provides.push(provide);
        } else if requirements_txt.exists() {
            let (require, provide) = Self::require_and_provide(
                "pip install",
                REQUIREMENTS_TXT,
                "Install PyPI packages using Pip",
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
                "python" => {
                    context.handle_layer(layer_name!("python"), PythonLayer::new(args))?;
                }
                "poetry" => {
                    context.handle_layer(layer_name!("poetry"), PoetryLayer)?;
                }
                "pip" => {
                    context.handle_layer(layer_name!("pip"), PipLayer)?;
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

struct PythonLayer {
    /// The semver requirement for the Python binary
    requirement: String,
}

impl PythonLayer {
    fn new(args: Vec<String>) -> Self {
        PythonLayer {
            // Join args with commas because semver requirement parser expects it to be so
            requirement: args.join(",").trim().to_string(),
        }
    }
}

impl Layer for PythonLayer {
    type Buildpack = PythonBuildpack;
    type Metadata = GenericMetadata;

    fn types(&self) -> LayerTypes {
        LayerTypes {
            build: true,
            launch: true,
            cache: false,
        }
    }

    fn create(
        &self,
        context: &BuildContext<Self::Buildpack>,
        layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, eyre::Report> {
        // Ensure a version meeting the semver is installed
        let requirement = if self.requirement.is_empty() {
            None
        } else {
            Some(self.requirement.clone())
        };
        let python = PythonBinary {}.require_sync(requirement, true)?;
        let version = python.version()?;

        // Symlink/copy the installation into the layer
        if platform_is_stencila(&context.platform) {
            if python.is_stencila_install() {
                tracing::info!("Linking to Python {} installed by Stencila", version);
                clear_dir_all(&layer_path)?;
                let source = python.grandparent()?;
                let dest = layer_path;
                symlink_dir(source.join("bin"), &dest.join("bin"))?;
                symlink_dir(source.join("lib"), &dest.join("lib"))?;
            } else {
                tracing::info!("Linking to Python {} installed on system", version);
                clear_dir_all(&layer_path)?;
                let dest = layer_path.join("bin");
                fs::create_dir_all(&dest)?;
                symlink_file(python.path, dest.join(python.name))?;
            }
        } else {
            #[allow(clippy::collapsible_else_if)]
            if python.is_stencila_install() {
                tracing::info!("Using Python {} installed by Stencila", version);
                clear_dir_all(&layer_path)?;
                let source = python.grandparent()?;
                let dest = layer_path;
                copy_dir_all(source, &dest)?;
            } else {
                bail!("Only able to build `python` layer if Python has been installed by Stencila")
            }
        }

        LayerResultBuilder::new(GenericMetadata::default()).build()
    }
}

struct PoetryLayer;

impl Layer for PoetryLayer {
    type Buildpack = PythonBuildpack;
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
        // Although the `POETRY_VIRTUALENVS_IN_PROJECT = true` setting below should make
        // Potry create a `.vevn` if one does not exist - it doesn't. So do it here.
        if !PathBuf::from(VENV).exists() {
            tracing::info!("Creating virtual environment '{}'", VENV);
            create_venv(layer_path)?;
        }

        tracing::info!("Installing packages using Poetry");

        let mut poetry = PoetryBinary {}.require_sync(Some(">=1,<2".to_string()), true)?;

        // Ensure that a `.venv` folder in the working directory is used (instead of a system level one)
        let mut envs: Vec<(OsString, OsString)> =
            vec![("POETRY_VIRTUALENVS_IN_PROJECT".into(), "true".into())];

        // If Stencila is not the platform make use the layer as the Poetry cache
        if !platform_is_stencila(&context.platform) {
            envs.push(("POETRY_CACHE_DIR".into(), layer_path.canonicalize()?.into()));
        }

        // Protip: use `println!("{}", poetry.run_sync(&["config", "--list"])?);` to check
        // config set by env vars
        poetry.envs(&envs);

        poetry.run_sync(&["install"])?;

        LayerResultBuilder::new(GenericMetadata::default()).build()
    }
}

struct PipLayer;

impl Layer for PipLayer {
    type Buildpack = PythonBuildpack;
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
        // Reuse or create a virtualenv.
        // The name `.venv` is most commonly used for this (e.g. Poetry uses that) but
        // we also look for `venv` and `env`.
        let mut virtualenv = PathBuf::from(VENV);
        if !virtualenv.exists() {
            let venv = PathBuf::from("venv");
            let env = PathBuf::from("env");
            if venv.join("bin").join("python").exists() {
                virtualenv = venv;
            } else if env.join("bin").join("python").exists() {
                virtualenv = env;
            } else {
                tracing::info!("Creating virtual environment '{}'", VENV);
                create_venv(layer_path)?;
            }
        }
        let virtualenv = virtualenv.canonicalize()?;

        tracing::info!("Installing packages using Pip");

        // Use the Python in the virtualenv, assume it is `python3` and has `pip`
        let mut python = BinaryInstallation {
            name: "python3".into(),
            path: virtualenv.join("bin").join("python3"),
            ..Default::default()
        };

        // If Stencila is not the platform use the layer as the Pip cache
        if !platform_is_stencila(&context.platform) {
            python.envs(&[("PIP_CACHE_DIR", layer_path.as_os_str())]);
        }

        python.run_sync(&["-m", "pip", "install", "-r", REQUIREMENTS_TXT])?;

        LayerResultBuilder::new(GenericMetadata::default()).build()
    }
}

/// Create a `.venv` virtual environment using the installed version of Python
fn create_venv(layer_path: &Path) -> Result<(), eyre::Report> {
    // Get the `python` installed by `PythonLayer`.
    // This is important because if affects the binary in the `venv`
    let python = BinaryInstallation {
        name: "python".into(),
        path: layer_path
            .canonicalize()?
            .parent()
            .expect("Should have parent")
            .join("python")
            .join("bin")
            .join("python"),
        ..Default::default()
    };

    python.run_sync(&["-m", "venv", "--clear", VENV])?;

    Ok(())
}
