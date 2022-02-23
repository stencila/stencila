use std::{
    env,
    fs::{create_dir_all, read_to_string, remove_file, write},
    path::{Path, PathBuf},
};

use binary_r::{Binary, BinaryTrait, RBinary};
use buildpack::{
    eyre::{self, eyre},
    fs_utils::{symlink_dir, symlink_file},
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

pub struct RBuildpack;

impl BuildpackTrait for RBuildpack {
    fn toml() -> &'static str {
        include_str!("../buildpack.toml")
    }
}

const DESCRIPTION: &str = "DESCRIPTION";
const INSTALL_R: &str = "install.R";
const RENV: &str = "renv";
const RENV_LOCK: &str = "renv.lock";
const TOOL_VERSIONS: &str = ".tool-versions";

impl Buildpack for RBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = eyre::Report;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        // Read `.tool-versions` for R version
        let tool_versions = Self::tool_versions();

        // Read `renv.lock` for R version
        let renv_lock = read_to_string(RENV_LOCK)
            .ok()
            .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok());

        // Read `DESCRIPTION` for parsing for R version
        let description = read_to_string(DESCRIPTION).ok();

        // Fail early
        if !(tool_versions.contains_key("r")
            || tool_versions.contains_key("R")
            || renv_lock.is_some()
            || description.is_some()
            || Self::any_exist(&[
                RENV,
                INSTALL_R,
                &INSTALL_R.to_lowercase(),
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
        let (version, source) =
            if let Some(version) = tool_versions.get("r").or_else(|| tool_versions.get("R")) {
                (version.to_string(), TOOL_VERSIONS)
            } else if let Some(version) = renv_lock.as_ref().and_then(|renv_lock| {
                renv_lock
                    .pointer("/R/Version")
                    .and_then(|version| version.as_str().map(|version| version.to_string()))
            }) {
                (version, RENV_LOCK)
            } else {
                ("*".to_string(), "")
            };

        if context.is_cnb() {
            // Ensure `version` is a valid version and not a semver requirement so that we can
            // specify the `apt` package versions to install. Log errors and have fallbacks rather than returning
            // `Err` because the latter can cause silent detect fails
            let fallback_version = "4.1.2".to_string();
            let r_binary = RBinary {};
            let versions = match r_binary.versions_sync(env::consts::OS) {
                Ok(versions) => versions,
                Err(error) => {
                    tracing::warn!("While getting R versions: {}", error);
                    vec![fallback_version.clone()]
                }
            };
            let version = match r_binary
                .semver_versions_matching(versions.clone(), &version)
                .first()
            {
                Some(version) => version.to_string(),
                None => {
                    tracing::warn!("Unable to find version of R meeting semver requirement `{}`; will use latest", version);
                    versions.first().unwrap_or(&fallback_version).to_string()
                }
            };

            let repos = if version.starts_with('4') {
                let release = sys_info::linux_os_release()
                    .ok()
                    .and_then(|info| info.version_codename)
                    .unwrap_or_default();
                format!(
                    "deb [trusted=yes] https://cloud.r-project.org/bin/linux/ubuntu {}-cran40/",
                    release
                )
            } else {
                "".to_string()
            };

            // Installing `r-base` alone appears to cause version conflicts (for some versions)
            // so specify `r-base-core` and `r-recommended` separately
            let packages = ["r-base-core"]; //, "r-recommended"];
            let packages = packages
                .iter()
                .map(|pkg| format!("{}={}-*", pkg, version))
                .collect::<Vec<String>>()
                .join(",");

            requires.push(Self::require(
                "apt_packages",
                "",
                "Install `apt` packages required for R",
                Some(hashmap! {
                    "repos" => repos,
                    "packages" => packages
                }),
            ));
        }

        // Require and provide R
        let (require, provide) = Self::require_and_provide(
            "r",
            source,
            format!("Install R {}", version),
            Some(hashmap! {
                "version" => version
            }),
        );
        requires.push(require);
        provides.push(provide);

        // Determine how packages are to be installed
        if renv_lock.is_some() {
            let (require, provide) = Self::require_and_provide(
                "renv",
                RENV_LOCK,
                "Install R packages into `renv` by restoring from lockfile",
                Some(hashmap! {
                    "method" => "restore".to_string()
                }),
            );
            requires.push(require);
            provides.push(provide);
        } else if description.is_some() {
            let (require, provide) = Self::require_and_provide(
                "renv",
                DESCRIPTION,
                "Install R packages into `renv` by generating a lockfile",
                Some(hashmap! {
                    "method" => "snapshot".to_string()
                }),
            );
            requires.push(require);
            provides.push(provide);
        } else if Self::any_exist(&[INSTALL_R, &INSTALL_R.to_lowercase()]) {
            let (require, provide) = Self::require_and_provide(
                "renv",
                INSTALL_R,
                "Install R packages into `renv` using RScript",
                Some(hashmap! {
                    "method" => "rscript".to_string()
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

        if let Some(options) = entries.get("r") {
            let layer_data = context.handle_layer(layer_name!("r"), RLayer::new(options))?;
            self.set_layer_env_vars(&layer_data.env);
        }

        if let Some(options) = entries.get("renv") {
            context.handle_layer(layer_name!("renv"), RenvLayer::new(options))?;
        }

        self.restore_env_vars(env_vars);
        BuildResultBuilder::new().build()
    }
}

struct RLayer {
    /// The semantic version requirement for the `R` binary
    requirement: String,
}

impl RLayer {
    fn new(options: &LayerOptions) -> Self {
        let requirement = options
            .get("version")
            .cloned()
            .unwrap_or_else(|| "*".to_string());

        RLayer { requirement }
    }
}

impl Layer for RLayer {
    type Buildpack = RBuildpack;
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
        let installed = RBinary {}
            .semver_version_matches(version, &self.requirement)
            .unwrap_or(false);
        let strategy = if installed {
            tracing::info!(
                "Existing `r` layer has `R {}` which matches semver requirement `{}`; will keep",
                version,
                self.requirement
            );
            ExistingLayerStrategy::Keep
        } else {
            tracing::info!(
                "Existing `r` layer has `R {}` which does not match semver requirement `{}`; will recreate",
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
            "Creating `r` layer with semver requirement `{}`",
            self.requirement
        );

        let version = if context.is_local() {
            let r = RBinary {}.ensure_version_sync(&self.requirement)?;
            let version = r.version()?.to_string();

            if r.is_stencila_install() {
                tracing::info!("Linking to `R {}` installed by Stencila", version);
                let source = r.grandparent()?;

                symlink_dir(source.join("bin"), &layer_path.join("bin"))?;
                symlink_dir(source.join("lib"), &layer_path.join("lib"))?;
            } else {
                tracing::info!("Linking to `R {}` installed on system", version);
                let source = r.parent()?;

                let bin_path = layer_path.join("bin");
                create_dir_all(&bin_path)?;
                symlink_file(&r.path, bin_path.join(&r.name))?;
                symlink_file(source.join("Rscript"), bin_path.join("Rscript"))?;
            }

            version
        } else {
            tracing::info!("Patching `R` installed by `apt` buildpack");

            let apt_packages = layer_path
                .join("..")
                .join("..")
                .join("stencila_apt")
                .join("apt_packages")
                .canonicalize()?;

            // Patch the R_HOME_DIR variable in the R startup script
            let r_path = apt_packages.join("usr").join("bin").join("R");
            let r_home_dir = apt_packages.join("usr").join("lib").join("R");
            let mut r_script = read_to_string(&r_path)?;
            r_script = r_script.replace(
                "R_HOME_DIR=/usr/lib/R",
                &format!("R_HOME_DIR={}", r_home_dir.display()),
            );
            write(&r_path, r_script)?;

            // Replace broken symlinks
            let from = apt_packages.join("etc").join("R");
            let to = r_home_dir.join("etc");
            for file in [
                "Makeconf",
                "Renviron.site",
                "Rprofile.site",
                "ldpaths",
                "repositories",
            ] {
                let target = to.join(file);
                remove_file(&target)?;
                symlink_file(from.join(file), target)?;
            }
            remove_file(to.join("Renviron"))?;
            symlink_file(to.join("Renviron.orig"), to.join("Renviron"))?;

            // The R installation should now work, verify and get the version
            let r = Binary::named("R").find_in(apt_packages.join("usr").join("bin").as_os_str())?;
            r.version()?.to_string()
        };

        // Store version in metadata to detect if layer is stale in `existing_layer_strategy()`
        let metadata = LayerVersionMetadata { version };

        // A present not setting any env vars. This is just a placeholder in case we need to.
        // See https://stat.ethz.ch/R-manual/R-devel/library/base/html/Startup.html for more
        // on which env vars are used by R
        let layer_env = LayerEnv::new();

        LayerResultBuilder::new(metadata).env(layer_env).build()
    }
}

struct RenvLayer {
    /// The package manager used to do the installation of packages
    ///
    /// - "restore": restore from a `renv.lock` file
    /// - "snapshot": restore from a `renv.lock` file
    /// - "rscript": run an R script (usually `install.R`)
    method: String,
}

impl RenvLayer {
    fn new(options: &LayerOptions) -> Self {
        let method = options
            .get("method")
            .cloned()
            .unwrap_or_else(|| "snapshot".to_string());

        RenvLayer { method }
    }
}

/// Generate hash of files that affect which packages are installed into `renv`
///
/// The hash is the combined contents of `renv.lock` and `install.R`.
/// This means that if any one is changed or removed that the hash will change.
fn generate_packages_hash(app_dir: &Path) -> String {
    let content = [
        read_to_string(app_dir.join(RENV_LOCK)).unwrap_or_default(),
        read_to_string(app_dir.join(INSTALL_R)).unwrap_or_default(),
    ]
    .concat();
    str_sha256_hex(&content)
}

impl Layer for RenvLayer {
    type Buildpack = RBuildpack;
    type Metadata = LayerHashMetadata;

    fn types(&self) -> LayerTypes {
        // Because renv symlinks to its cache both launch and cache must be true
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
            tracing::info!("Existing `renv` layer has same packages hash; will keep",);
            ExistingLayerStrategy::Keep
        } else {
            tracing::info!("Existing `renv` layer has different packages hash; will update");
            ExistingLayerStrategy::Update
        };
        Ok(strategy)
    }

    fn create(
        &self,
        context: &BuildContext<Self::Buildpack>,
        layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, eyre::Report> {
        tracing::info!("Creating `renv` layer");
        self.install(context, layer_path)
    }

    fn update(
        &self,
        context: &BuildContext<Self::Buildpack>,
        layer_data: &libcnb::layer::LayerData<Self::Metadata>,
    ) -> Result<LayerResult<Self::Metadata>, <Self::Buildpack as Buildpack>::Error> {
        tracing::info!("Updating `renv` layer");
        self.install(context, &layer_data.path)
    }
}

impl RenvLayer {
    fn install(
        &self,
        context: &BuildContext<RBuildpack>,
        layer_path: &Path,
    ) -> Result<LayerResult<LayerHashMetadata>, eyre::Report> {
        let app_path = &context.app_dir.canonicalize()?;
        let layer_path = &layer_path.canonicalize()?;

        tracing::info!("Installing packages using `renv::{}`", self.method);

        // Get `Rscript` (should have been installed in `RLayer`)
        let mut rscript = Binary::named("Rscript").require_sync()?;

        // Reuse or create a `renv/library` folder.
        let library_path = if context.is_local() {
            app_path
        } else {
            layer_path
        }
        .join("renv")
        .join("library");

        if self.method == "rscript" {
            // Determine which file to run
            let file = if PathBuf::from(INSTALL_R).exists() {
                INSTALL_R.to_string()
            } else {
                INSTALL_R.to_lowercase()
            };

            tracing::info!(
                "Installing packages into `{}` directory by running `{}`",
                library_path.display(),
                file
            );

            // Run a script that monkey patches `install.packages` so that `renv/library`
            // is the lib and using the RStudio package manager (for pre-built packages)
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
                library_path.display(),
                file
            );
            rscript.run_sync(&["-e", &script])?;
        } else {
            tracing::info!(
                "Installing packages into `renv` using `renv::{}",
                self.method
            );

            // If a CNB build use the layer as the renv cache
            if context.is_cnb() {
                rscript.env_list(&[("RENV_PATHS_CACHE", layer_path.canonicalize()?.as_os_str())]);
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
        }

        // Generate packages hash to detect if layer is stale in `existing_layer_strategy()`
        let metadata = LayerHashMetadata {
            hash: generate_packages_hash(app_path),
        };

        // Add `renv/library` to the R_LIBS_USER
        // See https://stat.ethz.ch/R-manual/R-devel/library/base/html/libPaths.html for more
        // on R library paths
        let layer_env = LayerEnv::new().chainable_insert(
            Scope::All,
            ModificationBehavior::Prepend,
            "R_LIBS_USER",
            library_path,
        );

        LayerResultBuilder::new(metadata).env(layer_env).build()
    }
}
