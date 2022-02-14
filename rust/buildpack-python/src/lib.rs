use std::{
    fs,
    path::{Path, PathBuf},
};

use binary_poetry::PoetryBinary;
use binary_python::{BinaryTrait, PythonBinary};
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
    platform_is_stencila, toml, BuildpackTrait,
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

const POETRY_INSTALL: &str = "install";
const POETRY_ADDREQ: &str = "addreq";

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
                ["poetry ", POETRY_INSTALL].concat(),
                if poetry_lock.exists() {
                    POETRY_LOCK
                } else {
                    PYPROJECT_TOML
                },
                "Install PyPI packages using `poetry install`",
            );
            requires.push(require);
            provides.push(provide);
        } else if requirements_txt.exists() {
            let (require, provide) = Self::require_and_provide(
                ["poetry ", POETRY_ADDREQ].concat(),
                REQUIREMENTS_TXT,
                "Install PyPI packages using `poetry add`",
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
                "python" => {
                    context.handle_layer(layer_name!("python"), PythonLayer::new(args))?;
                }
                "poetry" => {
                    context.handle_layer(layer_name!("poetry"), PoetryLayer::new(args))?;
                }
                _ => (),
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
            cache: true,
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

        // Symlink/copy the installation into the layer
        let dest = layer_path.join(python.version()?);
        if platform_is_stencila(&context.platform) {
            if python.is_stencila_install() {
                let source = python.grandparent()?;
                symlink_dir(source, &dest)?;
            } else {
                fs::create_dir_all(&dest)?;
                symlink_file(python.path, dest.join(python.name))?;
            }
        } else {
            #[allow(clippy::collapsible_else_if)]
            if python.is_stencila_install() {
                let source = python.grandparent()?;
                copy_dir_all(source, &dest)?;
            } else {
                bail!("Only able to build `python` layer if it has been installed by Stencila")
            }
        }

        LayerResultBuilder::new(GenericMetadata::default()).build()
    }
}

struct PoetryLayer {
    arg: String,
}

impl PoetryLayer {
    fn new(args: Vec<String>) -> Self {
        PoetryLayer {
            arg: args.first().cloned().unwrap_or_default(),
        }
    }
}

impl Layer for PoetryLayer {
    type Buildpack = PythonBuildpack;
    type Metadata = GenericMetadata;

    fn types(&self) -> LayerTypes {
        // Layer is available at build time and is cached but is not needed for
        // launch time because packages are installed into the `venv` of the
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
        // Require Poetry
        let mut poetry = PoetryBinary {}.require_sync(Some(">=1".to_string()), true)?;

        // If this is not a local build then make the layer the Poetry cache
        if !platform_is_stencila(&context.platform) {
            let cache_dir = layer_path.canonicalize()?;
            poetry.envs(&[("POETRY_CACHE_DIR", cache_dir.as_os_str())]);
        }

        // Do the install
        match self.arg.as_str() {
            POETRY_INSTALL => {
                poetry.run_sync(&["install"])?;
            }
            POETRY_ADDREQ => {
                // Poetry requires that `pyproject.toml` file exists for `add` to work
                // so we create one as minimal as possible. A relatively high lower bound
                // for Python is chosen because some packages will fail to install if they
                // require something higher.
                // TODO: Make the version, the version that is installed by this buildpack
                let pyproject_toml = PathBuf::from(PYPROJECT_TOML);
                if !pyproject_toml.exists() {
                    let name = context
                        .app_dir
                        .file_name()
                        .map(|name| name.to_string_lossy().to_string())
                        .unwrap_or_else(|| "temp".to_string());
                    let toml = format!(
                        r#"[tool.poetry]
name = "{}"
description = ""
version = "0.1.0"
authors = []

[tool.poetry.dependencies]
python = ">=3.8"
"#,
                        name
                    );
                    fs::write(pyproject_toml, toml)?;
                }
                for line in fs::read_to_string(REQUIREMENTS_TXT)?.lines() {
                    poetry.run_sync(&["add", line])?;
                }
            }
            _ => bail!("Unhandled arg: {}", self.arg),
        }

        LayerResultBuilder::new(GenericMetadata::default()).build()
    }
}
