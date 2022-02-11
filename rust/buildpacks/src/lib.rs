use buildpack::{
    buildpacks_dir,
    eyre::{bail, Result},
    platform_is_stencila, toml, tracing, BuildPlan, BuildpackPlan, BuildpackToml, BuildpackTrait,
};
use once_cell::sync::Lazy;
use std::{
    collections::BTreeMap,
    env::{current_dir, set_current_dir},
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

/// The set of registered buildpacks in the current process
static PACKS: Lazy<Arc<Buildpacks>> = Lazy::new(|| Arc::new(Buildpacks::new()));

/// A set of registered buildpacks
///
/// At present all buildpacks are builtin, but, as for `codecs`, `parsers` and other
/// collections it is possible that plugins could also provide buildpacks.
struct Buildpacks {
    inner: BTreeMap<String, BuildpackToml>,
}

/// A macro to dispatch methods to builtin buildpacks
macro_rules! dispatch_builtins {
    ($label:expr, $method:ident $(,$arg:expr)*) => {
        match $label {
            #[cfg(feature = "buildpack-node")]
            "node" => buildpack_node::NodeBuildpack{}.$method($($arg),*),
            #[cfg(feature = "buildpack-python")]
            "python" => buildpack_python::PythonBuildpack{}.$method($($arg),*),
            #[cfg(feature = "buildpack-r")]
            "r" => buildpack_r::RBuildpack{}.$method($($arg),*),
            _ => bail!("No buildpack with label `{}`", $label)
        }
    };
}

impl Buildpacks {
    /// Create a new buildpack registry
    ///
    /// Note that the string keys are "labels" for the buildpack which
    /// should be the same as the buildpack id sans the `stencila/` prefix.
    /// They aim to be more convenient to use in commands such as `stencila buildpacks show`.
    pub fn new() -> Self {
        let inner = vec![
            #[cfg(feature = "buildpack-node")]
            ("node", buildpack_node::NodeBuildpack::spec()),
            #[cfg(feature = "buildpack-python")]
            ("python", buildpack_python::PythonBuildpack::spec()),
            #[cfg(feature = "buildpack-r")]
            ("r", buildpack_r::RBuildpack::spec()),
        ]
        .into_iter()
        .map(|(label, buildpack): (_, _)| (label.to_string(), buildpack))
        .collect();

        Self { inner }
    }

    /// List the available buildpacks
    fn list(&self) -> &BTreeMap<String, BuildpackToml> {
        &self.inner
    }

    /// Generate a Markdown table of the available buildpacks
    fn table(&self) -> String {
        let cols = "|-----|--|----|-------|--------|";
        let head = "|Label|Id|Name|Version|Keywords|";
        let body = self
            .inner
            .iter()
            .map(|(label, buildpack_toml)| {
                let buildpack = &buildpack_toml.buildpack;
                format!(
                    "|{}|{}|{}|{}|{}|",
                    label,
                    buildpack.id,
                    buildpack.name,
                    buildpack.version,
                    buildpack
                        .keywords
                        .as_ref()
                        .map_or_else(String::new, |keywords| keywords.join(", "))
                )
            })
            .collect::<Vec<String>>()
            .join("\n");
        format!(
            "{top}\n{head}\n{align}\n{body}\n{bottom}\n",
            top = cols,
            head = head,
            align = cols,
            body = body,
            bottom = cols
        )
    }

    /// Get the buildpack with the given label or id
    fn get(&self, label: &str) -> Result<&BuildpackToml> {
        let label_lower = label.to_ascii_lowercase();
        match self.inner.get(&label_lower) {
            Some(buildpack) => Ok(buildpack),
            None => {
                for buildpack in self.inner.values() {
                    if buildpack.buildpack.id == label_lower {
                        return Ok(buildpack);
                    }
                }
                bail!("No buildpack with label or id `{}`", label)
            }
        }
    }

    /// Get the path to the CNB platform directory
    ///
    /// This directory path is passed to buildpacks in the `detect` and `build` phases.
    /// As a CNB platform, Stencila has its own platform directory which contains a
    /// file named `STENCILA_VERSION` containing the current version. This allows build packs
    /// to act differently depending if the platform is Stencila (local installs) versus
    /// Pack (install into Docker images).
    fn platform_dir() -> Result<PathBuf> {
        let dir = buildpacks_dir()?.join("platform");

        let env = dir.join("env");
        fs::create_dir_all(&env)?;

        let stencila_version = env.join("STENCILA_VERSION");
        fs::write(&stencila_version, env!("CARGO_PKG_VERSION"))?;

        Ok(dir)
    }

    /// Create a CNB layers directory for a buildpack
    ///
    /// Used in `build` when the `layers_dir` argument is not supplied.
    fn layers_dir_default(buildpack_label: &str) -> Result<PathBuf> {
        let dir = PathBuf::from(".stencila")
            .join("layers")
            .join(buildpack_label);
        fs::create_dir_all(&dir)?;

        Ok(dir)
    }

    /// Create a CNB Build Plan file (if it does not yet exist) for a buildpack
    ///
    /// Used in `detect` when the `build_plan` argument is not supplied.
    /// Rather than generate this path in a temporary directory (as Pack does) we generate
    /// it within the `.stencila` directory for transparency and easier debugging.
    ///
    /// See https://github.com/buildpacks/spec/blob/main/buildpack.md#build-plan-toml
    fn build_plan_default(buildpack_label: &str) -> Result<PathBuf> {
        let dir = PathBuf::from(".stencila")
            .join("build")
            .join(buildpack_label);
        fs::create_dir_all(&dir)?;

        let file = dir.join("build-plan.toml");
        if !file.exists() {
            fs::File::create(&file)?;
        }

        Ok(file)
    }

    /// Create a CNB Buildpack Plan file (if it does not yet exist) for a buildpack
    ///
    /// Used in `build` when the `buildpack_plan` argument is not supplied.
    /// Rather than generate this path in a temporary directory (as Pack does) we generate
    /// it within the `.stencila` directory for transparency and easier debugging.
    ///
    /// See https://github.com/buildpacks/spec/blob/main/buildpack.md#buildpack-plan-toml
    fn buildpack_plan_default(buildpack_label: &str) -> Result<PathBuf> {
        let dir = PathBuf::from(".stencila")
            .join("build")
            .join(buildpack_label);
        fs::create_dir_all(&dir)?;

        // Get the `build-plan.toml`
        let build_plan_path = dir.join("build-plan.toml");
        let build_plan = if build_plan_path.exists() {
            let toml = fs::read_to_string(build_plan_path)?;
            toml::from_str::<BuildPlan>(&toml)?
        } else {
            BuildPlan::default()
        };

        // Write the `buildpack-plan.toml` using the plan's `requires` as `entries`
        let buildpack_plan_path = dir.join("buildpack-plan.toml");
        let buildpack_plan = BuildpackPlan {
            entries: build_plan.requires.unwrap_or_default(),
        };
        let toml = toml::to_string(&buildpack_plan)?;
        fs::write(&buildpack_plan_path, &toml)?;

        Ok(buildpack_plan_path)
    }

    /// Detect whether a buildpack should build a working directory
    ///
    /// # Arguments
    ///
    /// - `buildpack_label`: The label of the buildpack to detect with.
    ///
    /// - `working_dir`: The directory to detect; defaults to the current directory.
    ///
    /// - `platform_dir`: The platform directory for the platform (e.g. Pack or Stencila);
    ///                   defaults to `buildpacks/platform` in the Stencila app data directory.
    ///
    /// - `build_plan`: The path of the `plan.toml` file;
    ///                 defaults to `.stencila/build/<buildpack_label>/build-plan.toml` in the working directory.
    fn detect(
        &self,
        buildpack_label: &str,
        working_dir: Option<&Path>,
        platform_dir: Option<&Path>,
        build_plan: Option<&Path>,
    ) -> Result<i32> {
        let current_dir = current_dir()?;
        if let Some(working_dir) = working_dir {
            set_current_dir(working_dir)?;
        }

        let platform_dir = match platform_dir {
            Some(dir) => dir.to_owned(),
            None => Self::platform_dir()?,
        };

        let build_plan = match build_plan {
            Some(path) => path.to_owned(),
            None => Self::build_plan_default(buildpack_label)?,
        };

        let result = dispatch_builtins!(buildpack_label, detect_with, &platform_dir, &build_plan);

        set_current_dir(current_dir)?;
        result
    }

    /// Build image layers for a working directory using a buildpack
    ///
    /// # Arguments
    ///
    /// - `buildpack_label`: The label of the buildpack to build with.
    ///
    /// - `working_dir`: The directory to build; defaults to the current directory.
    ///
    /// - `layers_dir`: A directory that may contain subdirectories representing each layer created
    ///                 by the buildpack in the final image or build cache;
    ///                 defaults to `.stencila/layers/<buildpack_label>` in the working directory.
    ///
    /// - `platform_dir`: The platform directory for the platform (e.g. Pack or Stencila);
    ///                   defaults to `buildpacks/platform` in the Stencila app data directory.
    ///
    /// - `buildpack_plan`: The path of the Buildpack Plan file;
    ///                      defaults to `.stencila/build/<buildpack_label>/buildpack-plan.toml` in the working directory.
    fn build(
        &self,
        buildpack_label: &str,
        working_dir: Option<&Path>,
        layers_dir: Option<&Path>,
        platform_dir: Option<&Path>,
        buildpack_plan: Option<&Path>,
    ) -> Result<i32> {
        let current_dir = current_dir()?;
        let working_dir = match working_dir {
            Some(dir) => {
                set_current_dir(dir)?;
                dir.to_owned()
            }
            None => current_dir.clone(),
        };

        let layers_dir = match layers_dir {
            Some(dir) => dir.to_owned(),
            None => Self::layers_dir_default(buildpack_label)?,
        };

        let platform_dir = match platform_dir {
            Some(dir) => dir.to_owned(),
            None => Self::platform_dir()?,
        };

        let buildpack_plan = match buildpack_plan {
            Some(path) => path.to_owned(),
            None => Self::buildpack_plan_default(buildpack_label)?,
        };

        if platform_is_stencila(&platform_dir) {
            tracing::debug!("Buildpack platform is Stencila");

            let code = self.detect(buildpack_label, None, Some(&platform_dir), None)?;
            if code != 0 {
                tracing::warn!(
                    "Directory `{}` does not match buildpack `{}` so will not be built",
                    working_dir.display(),
                    buildpack_label
                );
                return Ok(100);
            }
        }

        let result = dispatch_builtins!(
            buildpack_label,
            build_with,
            &layers_dir,
            &platform_dir,
            &buildpack_plan
        );

        set_current_dir(current_dir)?;
        result
    }

    /// Clean build directories for one, or all, buildpacks
    fn clean(&self, label: Option<&str>, working_dir: Option<&Path>) -> Result<()> {
        let working_dir = match working_dir {
            Some(dir) => dir.to_owned(),
            None => current_dir()?,
        };

        let stencila_dir = working_dir.join(".stencila");
        let mut build_dir = stencila_dir.join("build");
        let mut layers_dir = stencila_dir.join("layers");

        if let Some(label) = label {
            if label != "all" {
                build_dir.push(label);
                layers_dir.push(label);
            }
        }

        if build_dir.exists() {
            fs::remove_dir_all(build_dir)?;
        }
        if layers_dir.exists() {
            fs::remove_dir_all(layers_dir)?;
        }

        Ok(())
    }
}

impl Default for Buildpacks {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "cli")]
pub mod commands {
    use super::*;
    use cli_utils::{async_trait::async_trait, result, Result, Run};
    use std::{path::PathBuf, process};
    use structopt::StructOpt;

    /// Manage buildpacks
    ///
    /// In Stencila, a "buildpack" is a Cloud Native Buildpack (https://buildpacks.io)
    /// that is responsible for adding support for a programming language or other type of application
    /// to a container image.
    #[derive(Debug, StructOpt)]
    #[structopt(
        alias = "buildpack",
        setting = structopt::clap::AppSettings::ColoredHelp,
        setting = structopt::clap::AppSettings::DeriveDisplayOrder,
        setting = structopt::clap::AppSettings::VersionlessSubcommands
    )]
    pub enum Command {
        List(List),
        Show(Show),
        Detect(Detect),
        Build(Build),
        Clean(Clean),
    }

    #[async_trait]
    impl Run for Command {
        async fn run(&self) -> Result {
            match self {
                Command::List(cmd) => cmd.run().await,
                Command::Show(cmd) => cmd.run().await,
                Command::Detect(cmd) => cmd.run().await,
                Command::Build(cmd) => cmd.run().await,
                Command::Clean(cmd) => cmd.run().await,
            }
        }
    }

    /// List the buildpacks available
    ///
    /// The list of available buildpacks includes those that are built into the Stencila
    /// binary (e.g. `python`) as well as any buildpacks provided by plugins.
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct List {}

    #[async_trait]
    impl Run for List {
        async fn run(&self) -> Result {
            let list = PACKS.list();
            let table = PACKS.table();
            result::new("md", &table, &list)
        }
    }

    /// Show the specifications of a buildpack
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Show {
        /// The label of the buildpack
        ///
        /// To get the list of buildpack labels use `stencila build packs`.
        label: String,
    }

    #[async_trait]
    impl Run for Show {
        async fn run(&self) -> Result {
            let buildpack = PACKS.get(&self.label)?;
            result::value(buildpack)
        }
    }

    /// Detect whether a buildpack should build the working directory
    ///
    /// The `platform` and `plan` arguments of this command correspond
    /// to the same named arguments in the Cloud Native Buildpacks API for `detect`
    /// executables. See https://github.com/buildpacks/spec/blob/main/buildpack.md#detect.
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Detect {
        /// The label of the buildpack
        ///
        /// To get the list of buildpack labels use `stencila build packs`.
        label: String,

        /// The working directory (defaults to the current directory)
        working: Option<PathBuf>,

        /// A directory containing platform provided configuration, such as environment variables
        platform: Option<PathBuf>,

        /// A path to a file containing the Build Plan
        ///
        /// See https://github.com/buildpacks/spec/blob/main/buildpack.md#build-plan-toml
        plan: Option<PathBuf>,
    }

    #[async_trait]
    impl Run for Detect {
        async fn run(&self) -> Result {
            let result = PACKS.detect(
                &self.label,
                self.working.as_deref(),
                self.platform.as_deref(),
                self.plan.as_deref(),
            );

            let working_dir = self
                .working
                .clone()
                .unwrap_or_else(|| current_dir().expect("Should always be able to get cwd"));
            let working_dir = working_dir.display();

            let code = match result {
                Ok(code) => {
                    let will = if code == 0 { "will" } else { "will NOT" };
                    tracing::info!(
                        "Buildpack `{}` {} build `{}`",
                        self.label,
                        will,
                        working_dir
                    );
                    code
                }
                Err(error) => {
                    tracing::error!(
                        "While detecting `{}` with buildpack `{}`: {}",
                        working_dir,
                        self.label,
                        error
                    );
                    100
                }
            };

            // To maintain compatibility with the CNB API exit codes this function exits
            // here rather than returning up the call stack for the `main` function to
            // return some other code.
            process::exit(code)
        }
    }

    /// Build image layers for the working directory using a buildpack
    ///
    /// The `layers`, `platform` and `plan` arguments of this command correspond
    /// to the same named arguments in the Cloud Native Buildpacks API for `build`
    /// executables. See https://github.com/buildpacks/spec/blob/main/buildpack.md#build.
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Build {
        /// The label of the buildpack
        ///
        /// To get the list of buildpack labels use `stencila build packs`.
        label: String,

        /// The working directory (defaults to the current directory)
        working: Option<PathBuf>,

        /// A directory that may contain subdirectories representing each layer created by the
        /// buildpack in the final image or build cache
        layers: Option<PathBuf>,

        /// A directory containing platform provided configuration, such as environment variables
        platform: Option<PathBuf>,

        /// A path to a file containing the Buildpack Plan
        ///
        /// See https://github.com/buildpacks/spec/blob/main/buildpack.md#buildpack-plan-toml
        build: Option<PathBuf>,
    }

    #[async_trait]
    impl Run for Build {
        async fn run(&self) -> Result {
            let result = PACKS.build(
                &self.label,
                self.working.as_deref(),
                self.layers.as_deref(),
                self.platform.as_deref(),
                self.build.as_deref(),
            );

            let working_dir = self
                .working
                .clone()
                .unwrap_or_else(|| current_dir().expect("Should always be able to get cwd"));
            let working_dir = working_dir.display();

            let code = match result {
                Ok(code) => {
                    if code == 0 {
                        tracing::info!(
                            "Successfully built `{}` with buildpack `{}`",
                            working_dir,
                            self.label
                        );
                    }
                    code
                }
                Err(error) => {
                    tracing::error!(
                        "While building `{}` with buildpack `{}`: {}",
                        working_dir,
                        self.label,
                        error
                    );
                    100
                }
            };

            // See `run` for `Detect` for why we call `process::exit` here
            process::exit(code)
        }
    }

    /// Remove buildpack related directories from the `.stencila` folder or a working directory
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Clean {
        /// The label of the buildpack
        ///
        /// If not supplied, or "all", will perform clean for all buildpacks
        label: Option<String>,

        /// The working directory (defaults to the current directory)
        working: Option<PathBuf>,
    }

    #[async_trait]
    impl Run for Clean {
        async fn run(&self) -> Result {
            PACKS.clean(self.label.as_deref(), self.working.as_deref())?;
            result::nothing()
        }
    }
}
