use std::{env::current_dir, path::PathBuf, process};

use structopt::StructOpt;

use buildpack::tracing;
use cli_utils::{async_trait::async_trait, result, stdout_isatty, Result, Run};

use crate::buildpacks::{Buildpacks, PACKS};

/// Manage and use container buildpacks
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
