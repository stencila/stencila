use binary_pack::{BinaryTrait, PackBinary};
use buildpack::{
    buildpacks_dir,
    eyre::{bail, Result},
    libcnb::data::{buildpack::BuildpackId, buildpack_id},
    platform_dir_is_stencila, tag_for_path, toml, tracing, BuildPlan, BuildpackPlan, BuildpackToml,
    BuildpackTrait,
};
use once_cell::sync::Lazy;
use regex::Regex;
use std::{
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
    inner: Vec<BuildpackToml>,
}

/// A macro to dispatch methods to builtin buildpacks
macro_rules! dispatch_builtins {
    ($label:expr, $method:ident $(,$arg:expr)*) => {
        match $label {
            #[cfg(feature = "buildpack-apt")]
            "stencila/apt" => buildpack_apt::AptBuildpack{}.$method($($arg),*),
            #[cfg(feature = "buildpack-dockerfile")]
            "stencila/dockerfile" => buildpack_dockerfile::DockerfileBuildpack{}.$method($($arg),*),
            #[cfg(feature = "buildpack-node")]
            "stencila/node" => buildpack_node::NodeBuildpack{}.$method($($arg),*),
            #[cfg(feature = "buildpack-python")]
            "stencila/python" => buildpack_python::PythonBuildpack{}.$method($($arg),*),
            #[cfg(feature = "buildpack-r")]
            "stencila/r" => buildpack_r::RBuildpack{}.$method($($arg),*),
            #[cfg(feature = "buildpack-sources")]
            "stencila/sources" => buildpack_sources::SourcesBuildpack{}.$method($($arg),*),
            #[cfg(feature = "buildpack-stencila")]
            "stencila/stencila" => buildpack_stencila::StencilaBuildpack{}.$method($($arg),*),
            _ => bail!("No buildpack with label `{}`", $label)
        }
    };
}

impl Buildpacks {
    /// Create a new buildpack registry
    pub fn new() -> Self {
        Self {
            inner: [
                // The `SourcesBuildpack` should be first because it will import files
                // during `detect` which other buildpacks will run their `detect` against
                #[cfg(feature = "buildpack-sources")]
                buildpack_sources::SourcesBuildpack::spec().ok(),
                // The `Dockerfile` buildpack should come second because it excludes the
                // other buildpacks from running
                #[cfg(feature = "buildpack-dockerfile")]
                buildpack_dockerfile::DockerfileBuildpack::spec().ok(),
                #[cfg(feature = "buildpack-apt")]
                buildpack_apt::AptBuildpack::spec().ok(),
                #[cfg(feature = "buildpack-node")]
                buildpack_node::NodeBuildpack::spec().ok(),
                #[cfg(feature = "buildpack-python")]
                buildpack_python::PythonBuildpack::spec().ok(),
                #[cfg(feature = "buildpack-r")]
                buildpack_r::RBuildpack::spec().ok(),
                // Stencila CLI buildpack
                #[cfg(feature = "buildpack-stencila")]
                buildpack_stencila::StencilaBuildpack::spec().ok(),
            ]
            .into_iter()
            .flatten()
            .collect(),
        }
    }

    /// List the available buildpacks
    fn list(&self) -> &Vec<BuildpackToml> {
        &self.inner
    }

    /// Generate a Markdown table of the list of available buildpacks
    fn list_as_markdown(list: &[BuildpackToml]) -> String {
        let cols = "|--|----|-------|--------|";
        let head = "|Id|Name|Version|Keywords|";
        let body = list
            .iter()
            .map(|buildpack_toml| {
                let buildpack = &buildpack_toml.buildpack;
                format!(
                    "|{}|{}|{}|{}|",
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

    /// Find the `id` of the first buildbpack matching the label
    fn find(&self, label: &str) -> Result<&BuildpackId> {
        let label_lower = label.to_ascii_lowercase();
        for buildpack_toml in &self.inner {
            let id = buildpack_toml.buildpack.id.to_string();
            if id == label_lower || id.split('/').last() == Some(&label_lower) {
                return Ok(&buildpack_toml.buildpack.id);
            }
        }
        bail!("No buildpack with id or label `{}`", label)
    }

    /// Get the buildpack with the given label or id
    fn get(&self, label: &str) -> Result<&BuildpackToml> {
        let buildpack_id = self.find(label)?;
        for buildpack_toml in &self.inner {
            if buildpack_toml.buildpack.id == *buildpack_id {
                return Ok(buildpack_toml);
            }
        }
        bail!("No buildpack with id or label `{}`", label)
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

    /// Slugify a `BuildpackId` for use within a path
    ///
    /// Converts non-alphanumeric characters to underscores as seems
    /// to be the convention used by Pack
    fn slugify_buildpack_id(buildpack_id: &BuildpackId) -> String {
        static REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new("[^A-Za-z0-9 ]").expect("Unable to create regex"));

        REGEX.replace(buildpack_id.as_str(), "_").to_string()
    }

    /// Create a CNB layers directory for a buildpack
    ///
    /// Used in `build` when the `layers_dir` argument is not supplied.
    fn layers_dir_default(working_dir: &Path, buildpack_id: &BuildpackId) -> Result<PathBuf> {
        let dir = working_dir
            .join(".stencila")
            .join("layers")
            .join(Self::slugify_buildpack_id(buildpack_id));
        fs::create_dir_all(&dir)?;

        Ok(dir)
    }

    /// Create a `.stencila/build/<buildpack>` directory in a working directory
    ///
    /// Rather than generate a temporary directory (as Pack does) we generate
    /// it within the `.stencila` directory for transparency and easier debugging and
    /// to be able to display the build plan to users.
    fn build_dir_default(working_dir: &Path, buildpack_id: &BuildpackId) -> Result<PathBuf> {
        let dir = working_dir
            .join(".stencila")
            .join("build")
            .join(Self::slugify_buildpack_id(buildpack_id));
        fs::create_dir_all(&dir)?;

        Ok(dir)
    }

    /// Create a CNB Build Plan file for a buildpack in a working directory
    ///
    /// Used in `detect` when the `build_plan` argument is not supplied.
    ///
    /// See https://github.com/buildpacks/spec/blob/main/buildpack.md#build-plan-toml
    fn build_plan_default(working_dir: &Path, buildpack_id: &BuildpackId) -> Result<PathBuf> {
        let dir = Self::build_dir_default(working_dir, buildpack_id)?;

        let build_plan_path = dir.join("build-plan.toml");
        if !build_plan_path.exists() {
            fs::File::create(&build_plan_path)?;
        }

        Ok(build_plan_path)
    }

    /// Create a CNB Buildpack Plan file for a buildpack in a working directory
    ///
    /// Used in `build` when the `buildpack_plan` argument is not supplied.
    ///
    /// See https://github.com/buildpacks/spec/blob/main/buildpack.md#buildpack-plan-toml
    fn buildpack_plan_default(working_dir: &Path, buildpack_id: &BuildpackId) -> Result<PathBuf> {
        let dir = Self::build_dir_default(working_dir, buildpack_id)?;

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
        buildpack_id: &BuildpackId,
        working_dir: Option<&Path>,
        platform_dir: Option<&Path>,
        build_plan: Option<&Path>,
    ) -> Result<i32> {
        let current_dir = current_dir()?;
        let working_dir = working_dir.unwrap_or(&current_dir).canonicalize()?;

        let platform_dir = match platform_dir {
            Some(dir) => dir.to_owned(),
            None => Self::platform_dir()?,
        };

        let build_plan = match build_plan {
            Some(path) => path.to_owned(),
            None => Self::build_plan_default(&working_dir, buildpack_id)?,
        };

        set_current_dir(working_dir)?;
        let result = dispatch_builtins!(
            buildpack_id.as_str(),
            detect_with,
            &platform_dir,
            &build_plan
        );
        set_current_dir(current_dir)?;

        if let Ok(code) = result {
            if platform_dir_is_stencila(&platform_dir) && code != 0 {
                build_plan.parent().map(|dir| fs::remove_dir_all(dir).ok());
            }
        }

        result
    }

    /// Run `detect` for all buildpacks and return a map of the results
    fn detect_all(
        &self,
        working_dir: Option<&Path>,
        platform_dir: Option<&Path>,
    ) -> Result<Vec<(BuildpackId, bool)>> {
        let mut matched = Vec::new();

        for buildpack_toml in &self.inner {
            let buildpack_id = buildpack_toml.buildpack.id.clone();
            let result = self.detect(&buildpack_id, working_dir, platform_dir, None)?;
            matched.push((buildpack_id, result == 0));
        }

        Ok(matched)
    }

    /// Run `detect` for all buildpacks and compile the resulting `build-plan`.toml files into a
    /// map of `BuildPlan`s.
    ///
    /// The primary use for this method is to display to users what the build plan for their project
    /// is so that they can make changes (e.g. to `.tool-versions` or `requirements.txt` files).
    fn plan_all(
        &self,
        working_dir: Option<&Path>,
        platform_dir: Option<&Path>,
    ) -> Result<Vec<(BuildpackId, Option<BuildPlan>)>> {
        let working_dir = match working_dir {
            Some(dir) => dir.to_owned(),
            None => current_dir()?,
        };

        let matched = self.detect_all(Some(&working_dir), platform_dir)?;

        let build_plans = matched
            .into_iter()
            .map(|(buildpack_id, matched)| {
                let build_plan = match matched {
                    true => Self::build_plan_default(&working_dir, &buildpack_id)
                        .ok()
                        .and_then(|path| fs::read_to_string(path).ok())
                        .and_then(|toml| toml::from_str(&toml).ok()),
                    false => None,
                };
                (buildpack_id, build_plan)
            })
            .collect();

        Ok(build_plans)
    }

    /// Generate a Markdown document of a set of build plans
    fn plan_as_markdown(plans: &[(BuildpackId, Option<BuildPlan>)], show_all: bool) -> String {
        let plans = plans
            .iter()
            .map(|(id, plan)| {
                let md = match plan {
                    Some(plan) => plan
                        .requires
                        .iter()
                        .flatten()
                        .map(|require| {
                            let mut md = format!("- `{:18}`", require.name);
                            if let Some(metadata) = &require.metadata {
                                if let Some(desc) = metadata.get("description") {
                                    md.push_str(&format!(
                                        "  *{}*",
                                        desc.as_str().unwrap_or_default()
                                    ))
                                }
                                if let Some(source) = metadata.get("source") {
                                    md.push_str(&format!(
                                        " (`{}`)",
                                        source.as_str().unwrap_or_default()
                                    ))
                                }
                            }
                            md
                        })
                        .collect::<Vec<String>>()
                        .join("\n"),
                    None => "*Nothing*".to_string(),
                };
                if plan.is_some() || show_all {
                    format!("## Buildpack '{}'\n\n{}\n\n", id, md)
                } else {
                    "".to_string()
                }
            })
            .collect::<Vec<String>>()
            .concat();
        ["# Build plan\n\n", &plans].concat()
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
        buildpack_id: &BuildpackId,
        working_dir: Option<&Path>,
        layers_dir: Option<&Path>,
        platform_dir: Option<&Path>,
        buildpack_plan: Option<&Path>,
    ) -> Result<i32> {
        let current_dir = current_dir()?;
        let working_dir = working_dir.unwrap_or(&current_dir).canonicalize()?;

        tracing::info!(
            "Building `{}` with buildpack `{}`",
            working_dir.display(),
            buildpack_id
        );

        let layers_dir = match layers_dir {
            Some(dir) => dir.to_owned(),
            None => Self::layers_dir_default(&working_dir, buildpack_id)?,
        };

        let platform_dir = match platform_dir {
            Some(dir) => dir.to_owned(),
            None => Self::platform_dir()?,
        };

        let buildpack_plan = match buildpack_plan {
            Some(path) => path.to_owned(),
            None => Self::buildpack_plan_default(&working_dir, buildpack_id)?,
        };

        set_current_dir(working_dir)?;
        let result = dispatch_builtins!(
            buildpack_id.as_str(),
            build_with,
            &layers_dir,
            &platform_dir,
            &buildpack_plan
        );
        set_current_dir(current_dir)?;

        result
    }

    /// Run `detect` for all buildpacks, run `build` for those that match, and return
    /// a map of the detection results.
    ///
    /// If the directory matches the `dockerfile` buildpack, no other buildpacks will
    /// be built for the directory.
    fn build_all(
        &self,
        working_dir: Option<&Path>,
        platform_dir: Option<&Path>,
    ) -> Result<Vec<(BuildpackId, bool)>> {
        let matches = self.detect_all(working_dir, platform_dir)?;

        let dockerfile_buildpack_id = buildpack_id!("stencila/dockerfile");
        for (buildpack_id, matched) in &matches {
            if *matched {
                self.build(buildpack_id, working_dir, None, platform_dir, None)?;
                if *buildpack_id == dockerfile_buildpack_id {
                    break;
                }
            }
        }

        Ok(matches)
    }

    /// Build a container image for a working directory
    async fn pack(&self, working_dir: Option<&Path>) -> Result<()> {
        let dockerfile_buildpack_id = buildpack_id!("stencila/dockerfile");
        let code = self
            .detect(&dockerfile_buildpack_id, working_dir, None, None)
            .unwrap_or(100);
        if code == 0 {
            self.build(&dockerfile_buildpack_id, working_dir, None, None, None)?;
            return Ok(());
        }

        let working_dir = match working_dir {
            Some(dir) => dir.to_owned(),
            None => current_dir()?,
        };

        let tag = tag_for_path(&working_dir);

        tracing::info!(
            "Building container image `{}` for `{}` with `pack` binary",
            tag,
            working_dir.display()
        );

        const CNB_BUILDER: &str = "stencila/builder:bionic";

        let pack = PackBinary {}.ensure().await?;
        pack.run(&[
            "build",
            &tag,
            "--builder",
            CNB_BUILDER,
            "--path",
            &working_dir.display().to_string(),
            "--pull-policy",
            "if-not-present",
            "--trust-builder",
        ])
        .await
    }

    /// Clean build directories for one, or all, buildpacks
    fn clean(&self, working_dir: Option<&Path>, buildpack: Option<&str>) -> Result<()> {
        let working_dir = match working_dir {
            Some(dir) => dir.to_owned(),
            None => current_dir()?,
        };

        let stencila_dir = working_dir.join(".stencila");
        let mut build_dir = stencila_dir.join("build");
        let mut layers_dir = stencila_dir.join("layers");

        if let Some(buildpack) = buildpack {
            if buildpack != "all" {
                build_dir.push(["stencila_", buildpack].concat());
                layers_dir.push(["stencila_", buildpack].concat());
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
    use cli_utils::{async_trait::async_trait, result, stdout_isatty, Result, Run};
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
        Plan(Plan),
        Build(Build),
        Pack(Pack),
        Clean(Clean),
    }

    #[async_trait]
    impl Run for Command {
        async fn run(&self) -> Result {
            match self {
                Command::List(cmd) => cmd.run().await,
                Command::Show(cmd) => cmd.run().await,
                Command::Detect(cmd) => cmd.run().await,
                Command::Plan(cmd) => cmd.run().await,
                Command::Build(cmd) => cmd.run().await,
                Command::Pack(cmd) => cmd.run().await,
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
            let md = Buildpacks::list_as_markdown(list);
            result::new("md", &md, &list)
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
    /// This command is designed to be able to be used in a Cloud Native Buildpack (CNB)
    /// `bin/detect` script e.g
    ///
    ///    #!/usr/bin/env bash
    ///    set -eo pipefail
    ///    
    ///    stencila buildpacks detect . python <platform> <plan>
    ///
    /// See https://github.com/buildpacks/spec/blob/main/buildpack.md#detection
    /// further details.
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Detect {
        /// The working directory (defaults to the current directory)
        working: Option<PathBuf>,

        /// The id or label of the buildpack to detect with
        ///
        /// If not supplied, or "all", all buildpacks will be tested against the working directory
        /// and a map of the results returned.
        ///
        /// To get the list of buildpacks available use `stencila buildpacks list`.
        label: Option<String>,

        /// A directory containing platform provided configuration, such as environment variables
        platform: Option<PathBuf>,

        /// A path to a file containing the Build Plan
        ///
        /// See https://github.com/buildpacks/spec/blob/main/buildpack.md#build-plan-toml
        plan: Option<PathBuf>,

        /// Simulate detection on a CNB platform such as Pack
        #[structopt(long)]
        cnb: bool,
    }

    #[async_trait]
    impl Run for Detect {
        async fn run(&self) -> Result {
            let label = self.label.clone().unwrap_or_else(|| "all".to_string());

            let platform_dir = self.platform.as_ref().cloned().or_else(|| {
                if self.cnb {
                    Some(PathBuf::from("/tmp/cnb"))
                } else {
                    None
                }
            });

            if label == "all" {
                let results = PACKS.detect_all(self.working.as_deref(), platform_dir.as_deref())?;
                return result::value(results);
            }

            let buildpack_id = PACKS.find(&label)?;
            let result = PACKS.detect(
                buildpack_id,
                self.working.as_deref(),
                platform_dir.as_deref(),
                self.plan.as_deref(),
            );

            let working_dir = self
                .working
                .clone()
                .unwrap_or_else(|| current_dir().expect("Should always be able to get cwd"));
            let working_dir = working_dir.display();

            let code = match result {
                Ok(code) => {
                    let will = if code == 0 { "does" } else { "does NOT" };
                    tracing::info!(
                        "Buildpack `{}` {} match against `{}`",
                        label,
                        will,
                        working_dir
                    );
                    code
                }
                Err(error) => {
                    tracing::error!(
                        "While detecting `{}` with buildpack `{}`: {}",
                        working_dir,
                        label,
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

    /// Show the build plan for a working directory
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Plan {
        /// The working directory (defaults to the current directory)
        path: Option<PathBuf>,

        /// Show all buildpacks, including those that failed to match against the working directory
        #[structopt(short, long)]
        all: bool,

        /// Simulate plan on a CNB platform such as Pack
        #[structopt(long)]
        cnb: bool,
    }

    #[async_trait]
    impl Run for Plan {
        async fn run(&self) -> Result {
            let platform_dir = if self.cnb {
                Some(PathBuf::from("/tmp/cnb"))
            } else {
                None
            };

            let plans = PACKS.plan_all(self.path.as_deref(), platform_dir.as_deref())?;
            let md = Buildpacks::plan_as_markdown(&plans, self.all);
            result::new("md", &md, plans)
        }
    }

    /// Build image layers for the working directory using a buildpack
    ///
    /// This command is designed to be able to be used in a Cloud Native Buildpack (CNB)
    /// `bin/build` script e.g
    ///
    ///    #!/usr/bin/env bash
    ///    set -eo pipefail
    ///    
    ///    stencila buildpacks build . python <layers> <platform> <plan>
    ///
    /// See https://github.com/buildpacks/spec/blob/main/buildpack.md#build for
    /// further details.
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Build {
        /// The working directory (defaults to the current directory)
        working: Option<PathBuf>,

        /// The id or label of the buildpack to build
        ///
        /// If not supplied, or "all", all buildpacks will be tested against the working directory
        /// and those that match will be built.
        ///
        /// To get the list of buildpacks available use `stencila buildpacks list`.
        label: Option<String>,

        /// A directory that may contain subdirectories representing each layer created by the
        /// buildpack in the final image or build cache
        layers: Option<PathBuf>,

        /// A directory containing platform provided configuration, such as environment variables
        platform: Option<PathBuf>,

        /// A path to a file containing the Buildpack Plan
        ///
        /// See https://github.com/buildpacks/spec/blob/main/buildpack.md#buildpack-plan-toml
        build: Option<PathBuf>,

        /// Simulate building on a CNB platform such as Pack
        ///
        /// This is useful to buildpack developers for local debugging.
        /// For example, in another terminal, run `watch tree ...` on a project,
        ///
        ///   watch tree -a -L 6 fixtures/projects/node/package-json/
        ///
        /// and then run build that project with the `--cnb` flag,
        ///
        ///   cargo run --bin stencila -- buildpacks build --cnb fixtures/projects/node/package-json/
        ///
        /// Equivalent to using `/tmp/cnb` as `platform` directory (so won't work on
        /// platforms without `/tmp`).
        #[structopt(long)]
        cnb: bool,
    }

    #[async_trait]
    impl Run for Build {
        async fn run(&self) -> Result {
            let label = self.label.clone().unwrap_or_else(|| "all".to_string());

            let platform_dir = self.platform.as_ref().cloned().or_else(|| {
                if self.cnb {
                    Some(PathBuf::from("/tmp/cnb"))
                } else {
                    None
                }
            });

            if label == "all" {
                let results = PACKS.build_all(self.working.as_deref(), platform_dir.as_deref())?;
                return if stdout_isatty() {
                    result::nothing()
                } else {
                    result::value(results)
                };
            }

            let buildpack_id = PACKS.find(&label)?;
            let result = PACKS.build(
                buildpack_id,
                self.working.as_deref(),
                self.layers.as_deref(),
                platform_dir.as_deref(),
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
                            label
                        );
                    }
                    code
                }
                Err(error) => {
                    tracing::error!(
                        "While building `{}` with buildpack `{}`: {}",
                        working_dir,
                        label,
                        error
                    );
                    100
                }
            };

            // See `run` for `Detect` for why we call `process::exit` here
            process::exit(code)
        }
    }

    /// Create a container image for a working directory
    ///
    /// If the directory has a `Dockerfile` (or `Containerfile`) then the image will be
    /// built directly from that. Otherwise, the image will be built using
    /// using [`pack`](https://buildpacks.io/docs/tools/pack/) and the Stencila `builder`
    /// container image which include the buildpacks listed at `stencila buildpacks list`.
    ///
    /// Of course, you can use either `docker` or `pack` directly. This command just provides
    /// a convenient means of testing Stencila's image building logic locally an is mainly
    /// intended for developers.
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Pack {
        /// The working directory (defaults to the current directory)
        path: Option<PathBuf>,
    }

    #[async_trait]
    impl Run for Pack {
        async fn run(&self) -> Result {
            PACKS.pack(self.path.as_deref()).await?;
            result::nothing()
        }
    }

    /// Remove buildpack related directories from the `.stencila` folder or a working directory
    ///
    /// At present the buildpack related directories are `.stencila/build` and `.stencila/layers`.
    #[derive(Debug, StructOpt)]
    #[structopt(
        setting = structopt::clap::AppSettings::ColoredHelp
    )]
    pub struct Clean {
        /// The working directory (defaults to the current directory)
        working: Option<PathBuf>,

        /// The label of the Stencila buildpack to clean
        ///
        /// If not supplied, or "all", will perform clean for all buildpacks
        #[structopt(short, long)]
        buildpack: Option<String>,
    }

    #[async_trait]
    impl Run for Clean {
        async fn run(&self) -> Result {
            PACKS.clean(self.working.as_deref(), self.buildpack.as_deref())?;
            result::nothing()
        }
    }
}
