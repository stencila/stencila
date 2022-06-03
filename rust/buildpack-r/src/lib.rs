use std::{
    env,
    fs::{self, create_dir_all, read_to_string, remove_file, write},
    io::Write,
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
    tracing, BuildpackContext, BuildpackTrait, LayerOptions, LayerVersionMetadata,
};
use buildpack_apt::AptPackagesLayer;
use serde::{Deserialize, Serialize};

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

    fn detect(&self, _context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        // Read `.tool-versions` for R version
        let tool_versions = Self::tool_versions();

        // Read `renv.lock` for R version
        let renv_lock = read_to_string(RENV_LOCK)
            .ok()
            .and_then(|json| serde_json::from_str::<serde_json::Value>(&json).ok());

        // Read `DESCRIPTION` for parsing for R version
        let description = read_to_string(DESCRIPTION).ok();

        // Check for any R files. This is used to detect if to add a Renv layer (which
        // is not required is R is only in `.tool-versions`)
        let r_files_exist = Self::any_exist(&[
            RENV,
            RENV_LOCK,
            INSTALL_R,
            DESCRIPTION,
            &INSTALL_R.to_lowercase(),
            "main.R",
            "main.r",
            "index.R",
            "index.r",
        ]);

        // Fail early
        if !(tool_versions.contains_key("r")
            || tool_versions.contains_key("R")
            || renv_lock.is_some()
            || description.is_some()
            || r_files_exist)
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
        if Self::any_exist(&[INSTALL_R, &INSTALL_R.to_lowercase()]) {
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
        } else if renv_lock.is_some() {
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
        } else if r_files_exist {
            // Default behavior is to use `renv`'s snapshot method that scans files,
            // including DESCRIPTION files and `library` statements, for R packages
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
            context.handle_layer(
                layer_name!("renv"),
                RenvLayer::new(options, &context.app_dir),
            )?;
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

        let layer_path = &layer_path.canonicalize()?;

        // WARNING: It is not always clear what env vars should be set for R and which are set by
        // R on startup (see those "Set by R" in `EnvVar.html`) and thus ignored if you try to set them.
        // Generally it seems best to set R-specific variables in `Renviron.site` (see below)
        //   https://stat.ethz.ch/R-manual/R-devel/library/base/html/Startup.html
        //   https://stat.ethz.ch/R-manual/R-devel/library/base/html/EnvVar.html
        //   https://stat.ethz.ch/R-manual/R-devel/library/base/html/Rhome.html
        let mut layer_env = LayerEnv::new();

        let r_binary = RBinary {};

        let version = if context.is_local() {
            let r = r_binary.ensure_version_sync(&self.requirement)?;
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
        } else if let Some(r) = r_binary.installed(Some(self.requirement.clone()))? {
            let version = r.version()?.to_string();

            tracing::info!("Linking to `r {}` installed on stack image", version);
            let source = r.grandparent()?;

            symlink_dir(source.join("bin"), &layer_path.join("bin"))?;
            symlink_dir(source.join("lib"), &layer_path.join("lib"))?;

            version
        } else {
            let release = sys_info::linux_os_release()
                .ok()
                .and_then(|info| info.version_codename)
                .unwrap_or_default();

            tracing::info!("Installing `R` using `apt` repositories for `{}`", release);

            // Determine the highest version meeting semver requirement
            let versions = r_binary.versions_sync(env::consts::OS)?;
            let version = match r_binary
                .semver_versions_matching(&versions, &self.requirement)
                .first()
            {
                Some(version) => version.clone(),
                None => {
                    tracing::warn!("Unable to find version of R meeting semver requirement `{}`; will use latest", self.requirement);
                    versions
                        .first()
                        .cloned()
                        .ok_or_else(|| eyre!("No versions available for R"))?
                }
            };

            // Determine apt repository to use
            let repos = if version.starts_with('4') {
                format!(
                    "deb [trusted=yes] https://cloud.r-project.org/bin/linux/ubuntu {}-cran40/",
                    release
                )
            } else {
                "".to_string()
            };

            // Packages to install
            let packages = [
                // Install `r-base` and `r-base-dev` to allow users to install packages (at runtime) that require building
                format!("r-base={}-*", version),
                format!("r-base-dev={}-*", version),
            ]
            .join(",");

            // Do install
            let options: LayerOptions = hashmap! {
                "repos".to_string() => repos,
                "packages".to_string() => packages
            };
            let apt_layer = AptPackagesLayer::new(&options, None);
            let build_result = apt_layer.install(layer_path)?;
            if let Some(env) = build_result.env {
                layer_env = env;
            }

            tracing::info!("Patching `R` installed by `apt` buildpack");

            let r_home_dir = layer_path.join("usr").join("lib").join("R");

            // Patch the R_HOME_DIR variable in the R startup script
            let r_path = r_home_dir.join("bin").join("R");
            let content = read_to_string(&r_path)?.replace(
                "R_HOME_DIR=/usr/lib/R",
                &format!("R_HOME_DIR={}", r_home_dir.display()),
            );
            write(&r_path, content)?;

            // Replace Rscript otherwise it will fail with the error "Rscript execution error: No such file or directory"
            // because the install path of R is hardcoded into it.
            // See https://stackoverflow.com/questions/39832110/rscript-execution-error-no-such-file-or-directory
            let rscript_path = r_home_dir.join("bin").join("Rscript");
            #[cfg(target_family = "unix")]
            let mut file = {
                use std::os::unix::fs::OpenOptionsExt;
                fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .mode(0o775)
                    .open(rscript_path)?
            };
            #[cfg(not(target_family = "unix"))]
            let mut file = {
                fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(rscript_path)?
            };
            file.write_all(include_str!("Rscript.sh").as_bytes())?;

            // Replace executables in `layer_path/usr/bin` with symlinks to patched/replaced
            // files in `layer_path/usr/lib/R/bin`
            let to = layer_path.join("usr").join("bin");
            for file in ["R", "Rscript"] {
                let target = to.join(file);
                remove_file(&target)?;
                symlink_file(r_home_dir.join("bin").join(file), target)?;
            }

            // Modify `Renviron.site` as needed (the preferred way to set environment variables at R startup)
            // https://stat.ethz.ch/R-manual/R-devel/library/base/html/Startup.html
            let r_environ = layer_path.join("etc").join("R").join("Renviron.site");
            let layer_usr_share_r = layer_path.join("usr").join("share").join("R");
            let mut content = read_to_string(&r_environ)?;
            content.push_str(&format!(
                r#"
R_SHARE_DIR={}
R_INCLUDE_DIR={}
"#,
                layer_usr_share_r.join("share").display(),
                layer_usr_share_r.join("include").display()
            ));
            write(&r_environ, content)?;

            // Also set `R_SHARE_DIR` which is needed when building packages
            layer_env.insert(
                Scope::All,
                ModificationBehavior::Override,
                "R_SHARE_DIR",
                layer_usr_share_r.join("share"),
            );

            // Create a `Rprofile.site` file to use the RStudio package manager as repo
            let r_profile = layer_path.join("etc").join("R").join("Rprofile.site");
            write(
                r_profile,
                format!(
                    r#"
options(
    repos = c(RSTUDIO_PACKAGE_MANAGER = "https://packagemanager.rstudio.com/all/__linux__/{release}/latest"),
    HTTPUserAgent = sprintf("R/%s R (%s)", getRversion(), paste(getRversion(), R.version["platform"], R.version["arch"], R.version["os"]))
)
"#,
                    release = "bionic"
                ),
            )?;

            // Add /etc/R/Renviron if missing (not sure why this is missing?)
            // At present only env vars found to be necessary are added
            let r_environ = layer_path.join("etc").join("R").join("Renviron");
            if !r_environ.exists() {
                write(
                    r_environ,
                    r#"
EDITOR=nano
"#,
                )?;
            }

            // Replace broken symlinks
            let from = layer_path.join("etc").join("R");
            let to = r_home_dir.join("etc");
            for file in [
                "Makeconf",
                "Renviron",
                "Renviron.site",
                "Rprofile.site",
                "ldpaths",
                "repositories",
            ] {
                let target = to.join(file);
                remove_file(&target)?;
                symlink_file(from.join(file), target)?;
            }

            // Additional library paths needed by R at launch-time and when we check the version below
            // This overrides the `LD_LIBRARY_PATH` prepend defined by the apt layer above (thats why it
            // repeats the paths prepended there).
            const LD_LIBRARY_PATH: &str = "LD_LIBRARY_PATH";
            let ld_library_path = format!(
                "{}:", // Need to end with a colon to delimit from rest of path
                env::join_paths([
                    layer_path.join("lib"),
                    layer_path.join("lib/x86_64-linux-gnu"),
                    layer_path.join("usr/lib"),
                    layer_path.join("usr/lib/x86_64-linux-gnu"),
                    layer_path.join("usr/lib/x86_64-linux-gnu/blas"),
                    layer_path.join("usr/lib/x86_64-linux-gnu/lapack"),
                ])?
                .to_string_lossy()
            );
            layer_env.insert(
                Scope::All,
                ModificationBehavior::Prepend,
                LD_LIBRARY_PATH,
                ld_library_path.clone(),
            );
            env::set_var(
                LD_LIBRARY_PATH,
                ld_library_path
                    + &env::var_os(LD_LIBRARY_PATH)
                        .unwrap_or_default()
                        .to_string_lossy(),
            );

            // The R installation should now work, verify that is does and get the version
            let r = Binary::named("R").find_in(layer_path.join("usr").join("bin").as_os_str())?;
            match r.version() {
                Ok(version) => version,
                Err(error) => {
                    tracing::warn!("Unable to get version of R: {}", error);
                    // Return a version-ish string so that the image can at least be built
                    // and run for debugging purposes
                    "0.0.0"
                }
            }
            .to_string()
        };

        // Store version in metadata to detect if layer is stale in `existing_layer_strategy()`
        let metadata = LayerVersionMetadata { version };

        LayerResultBuilder::new(metadata).env(layer_env).build()
    }
}

#[derive(Clone, Deserialize, Serialize)]
struct RenvLayer {
    /// The package manager used to do the installation of packages
    ///
    /// - "restore": restore from a `renv.lock` file
    /// - "snapshot": restore from a `renv.lock` file
    /// - "rscript": run an R script (usually `install.R`)
    method: String,

    /// The minor version of R to install packages for e.g. `4.0`
    ///
    /// Used to bust cached `renv` if the R minor version changes.
    minor_version: String,

    // Hash of files that affect which packages/versions are installed into `renv`
    //
    // The hash is the combined contents of `renv.lock` and `install.R`.
    // This means that if any one is changed or removed that the hash will change.
    packages_hash: String,
}

impl RenvLayer {
    fn new(options: &LayerOptions, app_path: &Path) -> Self {
        let method = options
            .get("method")
            .cloned()
            .unwrap_or_else(|| "snapshot".to_string());

        let minor_version = RBinary {}
            .require_sync()
            .and_then(|r| r.version().map(|v| v.to_string()))
            .and_then(|version| RBinary {}.semver_version_minor(&version))
            .unwrap_or_default();

        let packages_hash = str_sha256_hex(
            &[
                read_to_string(app_path.join(RENV_LOCK)).unwrap_or_default(),
                read_to_string(app_path.join(INSTALL_R)).unwrap_or_default(),
            ]
            .concat(),
        );

        RenvLayer {
            method,
            minor_version,
            packages_hash,
        }
    }
}

impl Layer for RenvLayer {
    type Buildpack = RBuildpack;
    type Metadata = RenvLayer;

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
        let existing = &layer_data.content_metadata.metadata;
        let strategy = if self.minor_version != existing.minor_version {
            tracing::info!(
                "Existing `renv` layer is for different R minor version (`{}` => `{}`); will recreate",
                existing.minor_version,
                self.minor_version,
            );
            ExistingLayerStrategy::Recreate
        } else if self.packages_hash != existing.packages_hash {
            tracing::info!("Existing `renv` layer has different packages hash; will update",);
            ExistingLayerStrategy::Update
        } else {
            tracing::info!("Existing `renv` layer meets requirements; will keep",);
            ExistingLayerStrategy::Keep
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
    ) -> Result<LayerResult<RenvLayer>, eyre::Report> {
        let app_path = &context.app_dir.canonicalize()?;
        let layer_path = &layer_path.canonicalize()?;

        tracing::info!("Installing R packages into `renv` layer");

        // Get `R` (should have been installed in `RLayer`)
        let mut r = Binary::named("R").require_sync()?;

        // Reuse or create a `renv/library` folder.
        let library_path = if context.is_local() {
            app_path
        } else {
            layer_path
        }
        .join("renv")
        .join("library");
        if !library_path.exists() {
            create_dir_all(&library_path)?;
        }

        let expr = if self.method == "rscript" {
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

            // Run a script that monkey patches `install.packages` so that `renv/library` is the lib
            format!(
                r#"
install.packages <- function(pkgs, lib = NULL, repos = NULL, ...) {{ utils::install.packages(pkgs, lib = "{lib_path}", ...) }}
source("{install_script}")
            "#,
                lib_path = library_path.display(),
                install_script = file
            )
        } else {
            tracing::info!(
                "Installing packages into `renv` using `renv::{}`",
                self.method
            );

            // If not a local build use the layer as the renv cache
            if !context.is_local() {
                r.env_list(&[("RENV_PATHS_CACHE", layer_path.canonicalize()?.as_os_str())]);
            }

            // Run a script that does the install including installing if necessary and options for non-interactive use
            format!(
                r#"
options(renv.consent = TRUE)
if (!suppressMessages(require(renv, quietly=TRUE))) install.packages("renv")
{snapshot}
renv::activate()
renv::restore(prompt = FALSE)"#,
                snapshot = if self.method == "snapshot" {
                    "renv::snapshot()"
                } else {
                    ""
                }
            )
        };
        // Do not use --vanilla or --no-site-file here because we want the `Rprofile.site` from above to be used
        r.run_sync(&["--slave", "--no-restore", "-e", &expr])?;

        // Add `renv/library` to the R_LIBS_USER
        // See https://stat.ethz.ch/R-manual/R-devel/library/base/html/libPaths.html for more
        // on R library paths
        let layer_env = LayerEnv::new().chainable_insert(
            Scope::All,
            ModificationBehavior::Prepend,
            "R_LIBS_USER",
            library_path,
        );

        LayerResultBuilder::new(self.clone()).env(layer_env).build()
    }
}
