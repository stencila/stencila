use std::{
    fs,
    path::{Path, PathBuf},
};

use binary_r::{BinaryInstallation, BinaryTrait, RBinary};
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
    platform_is_stencila, tracing, BuildpackTrait,
};

pub struct RBuildpack;

impl BuildpackTrait for RBuildpack {
    fn toml() -> &'static str {
        include_str!("../buildpack.toml")
    }
}

const INSTALLED: &str = "<installed>";
const DESCRIPTION: &str = "DESCRIPTION";
const INSTALL_R: &str = "install.R";
const RENV: &str = "renv";
const RENV_LOCK: &str = "renv.lock";
const TOOL_VERSIONS: &str = ".tool-versions";

impl Buildpack for RBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = eyre::Report;

    fn detect(&self, _context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        // Read `.tool-versions` for R version
        let tool_versions = Self::tool_versions();

        // Read `renv.lock` for R version
        let renv_lock = fs::read_to_string(RENV_LOCK)
            .ok()
            .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok());

        // Fail early
        if !(tool_versions.contains_key("R")
            || renv_lock.is_some()
            || Self::any_exist(&[
                RENV,
                INSTALL_R,
                &INSTALL_R.to_lowercase(),
                DESCRIPTION,
                "main.R",
                "main.r",
                "index.R",
                "index.r",
            ]))
        {
            return DetectResultBuilder::fail().build();
        }

        let mut requires = Vec::new();
        let mut provides = Vec::new();

        // Resolve R version from `.tool-versions`, `DESCRIPTION` or `renv`
        let (version, source) = if let Some(version) = tool_versions.get("R") {
            (version.to_string(), TOOL_VERSIONS)
        } else if let Some(version) = renv_lock.as_ref().and_then(|renv_lock| {
            renv_lock
                .pointer("/R/Version")
                .and_then(|version| version.as_str().map(|version| version.to_string()))
        }) {
            (version, RENV_LOCK)
        } else if let Some(version) = (RBinary {}).installed_version(None) {
            (version, INSTALLED)
        } else {
            ("".to_string(), "")
        };

        // Require and provide R
        let (require, provide) = Self::require_and_provide(
            format!("r {}", version).trim(),
            source,
            format!("Install R {}", version).trim(),
        );
        requires.push(require);
        provides.push(provide);

        // Determine how packages are to be installed
        if renv_lock.is_some() {
            let (require, provide) = Self::require_and_provide(
                "renv restore",
                RENV_LOCK,
                "Restore project from a lockfile using `renv`",
            );
            requires.push(require);
            provides.push(provide);
        } else if Self::any_exist(&[INSTALL_R, &INSTALL_R.to_lowercase()]) {
            let (require, provide) = Self::require_and_provide(
                "rscript install",
                INSTALL_R,
                "Install packages using RScript",
            );
            requires.push(require);
            provides.push(provide);
        } else {
            let (require, provide) = Self::require_and_provide(
                "renv snapshot",
                RENV_LOCK,
                "Generate and install a project lockfile using `renv`",
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
                "r" => {
                    let layer = RLayer::new(args.first().cloned());
                    context.handle_layer(layer_name!("r"), layer)?;
                }
                "renv" => {
                    let layer = RenvLayer::new(args.first().cloned());
                    context.handle_layer(layer_name!("renv"), layer)?;
                }
                "rscript" => {
                    context.handle_layer(layer_name!("rscript"), RscriptLayer)?;
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

struct RLayer {
    /// The version of R to install
    version: Option<String>,
}

impl RLayer {
    fn new(version: Option<String>) -> Self {
        RLayer { version }
    }
}

impl Layer for RLayer {
    type Buildpack = RBuildpack;
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
        // Install the specified version of R
        let r = RBinary {}.require_sync(self.version.clone(), true)?;
        let version = r.version()?;

        // Symlink/copy the installation into the layer
        if platform_is_stencila(&context.platform) {
            if r.is_stencila_install() {
                tracing::info!("Linking to R {} installed by Stencila", version);
                clear_dir_all(&layer_path)?;
                let source = r.grandparent()?;
                let dest = layer_path;
                symlink_dir(source.join("bin"), &dest.join("bin"))?;
                symlink_dir(source.join("lib"), &dest.join("lib"))?;
            } else {
                tracing::info!("Linking to R {} installed on system", version);
                clear_dir_all(&layer_path)?;
                let source = r.parent()?;
                let dest = layer_path.join("bin");
                fs::create_dir_all(&dest)?;
                symlink_file(&r.path, dest.join(&r.name))?;
                symlink_file(source.join("Rscript"), dest.join("Rscript"))?;
            }
        } else {
            #[allow(clippy::collapsible_else_if)]
            if r.is_stencila_install() {
                tracing::info!("Using R {} installed by Stencila", version);
                clear_dir_all(&layer_path)?;
                let source = r.grandparent()?;
                let dest = layer_path;
                copy_dir_all(source, &dest)?;
            } else {
                bail!("Only able to build `r` layer if R has been installed by Stencila")
            }
        }

        LayerResultBuilder::new(GenericMetadata::default()).build()
    }
}

struct RenvLayer {
    /// The installation method
    method: String,
}

impl RenvLayer {
    fn new(method: Option<String>) -> Self {
        RenvLayer {
            method: method.unwrap_or_else(|| "snapshot".to_string()),
        }
    }
}

impl Layer for RenvLayer {
    type Buildpack = RBuildpack;
    type Metadata = GenericMetadata;

    fn types(&self) -> LayerTypes {
        // Because renv symlinks to its cache both launch and cache must be true
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
        tracing::info!("Installing packages using `renv::{}`", self.method);

        // Get `Rscript` installed in `RLayer`
        let mut rscript = installed_rscript(layer_path)?;

        // If Stencila is not the platform use the layer as the renv cache
        if !platform_is_stencila(&context.platform) {
            rscript.envs(&[("RENV_PATHS_CACHE", layer_path.canonicalize()?.as_os_str())]);
        }

        // Run a script that does the install including installing if necessary,
        // options for non-interactive use and using the RStudio package manager (for pre-built packages)
        let script = format!(
            r#"
options(
    renv.consent = TRUE,
    repos = c(CRAN = "https://packagemanager.rstudio.com/all/latest")
)
if (!require(renv, quietly=TRUE)) install.packages("renv")
{}
renv::activate()
renv::restore(prompt = FALSE)"#,
            if self.method == "snapshot" {
                "renv::snapshot()"
            } else {
                ""
            }
        );
        rscript.run_sync(&["-e", &script])?;

        LayerResultBuilder::new(GenericMetadata::default()).build()
    }
}

struct RscriptLayer;

impl Layer for RscriptLayer {
    type Buildpack = RBuildpack;
    type Metadata = GenericMetadata;

    fn types(&self) -> LayerTypes {
        LayerTypes {
            build: true,
            launch: false,
            cache: false,
        }
    }

    fn create(
        &self,
        _context: &BuildContext<Self::Buildpack>,
        layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, eyre::Report> {
        tracing::info!("Installing packages using Rscript");

        // Reuse or create a `renv/library` folder.
        let renv_library = PathBuf::from("renv").join("library");
        fs::create_dir_all(&renv_library)?;

        // Get `Rscript` installed in `RLayer`
        let rscript = installed_rscript(layer_path)?;

        // Run a script that monkey patches `install.packages` so that `renv/library`
        // is the lib and RStudio package manager (for pre-built packages)
        let file = if PathBuf::from(INSTALL_R).exists() {
            INSTALL_R.to_string()
        } else {
            INSTALL_R.to_lowercase()
        };
        let script = format!(
            r#"
install.packages <- function(pkgs, lib = NULL, repos = NULL, ...) {{
    utils::install.packages(
        pkgs, 
        lib = "{}", 
        repos = c(CRAN = "https://packagemanager.rstudio.com/all/latest", repos),
        ...
    )
}}
source("{}")
        "#,
            renv_library.display(),
            file
        );
        rscript.run_sync(&["-e", &script])?;

        LayerResultBuilder::new(GenericMetadata::default()).build()
    }
}

/// Get the `Rscript` executable installed by the `RLayer` from another layer
fn installed_rscript(layer_path: &Path) -> Result<BinaryInstallation, eyre::Report> {
    Ok(BinaryInstallation {
        name: "Rscript".into(),
        path: layer_path
            .canonicalize()?
            .parent()
            .expect("Should have parent")
            .join("r")
            .join("bin")
            .join("Rscript"),
        ..Default::default()
    })
}
