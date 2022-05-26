use std::path::PathBuf;

use structopt::StructOpt;

use cli_utils::{async_trait::async_trait, result, Result, Run};

use crate::build::build;

/// Build and distribute container images
#[derive(Debug, StructOpt)]
#[structopt(
    alias = "images",
    setting = structopt::clap::AppSettings::ColoredHelp,
    setting = structopt::clap::AppSettings::DeriveDisplayOrder,
    setting = structopt::clap::AppSettings::VersionlessSubcommands
)]
pub enum Command {
    Build(Build),
}

#[async_trait]
impl Run for Command {
    async fn run(&self) -> Result {
        match self {
            Command::Build(cmd) => cmd.run().await,
        }
    }
}

/// Build an image
#[derive(Debug, StructOpt)]
#[structopt(
    setting = structopt::clap::AppSettings::ColoredHelp
)]
pub struct Build {
    /// The project directory to build
    ///
    /// Defaults to the current project.
    project: Option<PathBuf>,

    /// The base image to build from
    ///
    /// Use the image name and optionally a tag e.g. `docker.io/library/ubuntu:22.04`, `ubuntu:22.04`, `ubuntu`
    /// Equivalent to the `FROM` directive in a Dockerfile. Defaults to `stencila/femto`.
    #[structopt(long, short, env = "STENCILA_IMAGE_FROM")]
    from: Option<String>,

    /// Directories that should be added as separate layers to the image
    ///
    /// Use a colon separated list of globs. Defaults to "<project>" and "/layers/*/*" (i.e. a layer for the project
    /// directory and one for each of the sub-sub-directories of "/layers" which are created by buildpacks).
    #[structopt(long, short, env = "STENCILA_IMAGE_LAYERS")]
    layers: Option<String>,

    /// The directory to write the image to
    ///
    /// Defaults to `.stencila/image` in the project. When building within a container
    /// it can be useful to bind mount this volume from the host.
    #[structopt(long, short, env = "STENCILA_IMAGE_DIR")]
    dir: Option<PathBuf>,

    /// The name for the built image
    ///
    /// Defaults to the name of the directory plus a hash of its path (to maintain uniqueness).
    #[structopt(long, short, env = "STENCILA_IMAGE_TO")]
    to: Option<String>,
}

#[async_trait]
impl Run for Build {
    async fn run(&self) -> Result {
        let layers_dir: Vec<String> = self
            .layers
            .as_ref()
            .map(|str| str.split(':').map(String::from).collect())
            .unwrap_or_default();

        build(
            self.project.as_deref(),
            &layers_dir,
            self.from.as_deref(),
            self.to.as_deref(),
            self.dir.as_deref(),
        )
        .await?;

        result::nothing()
    }
}
