use std::path::PathBuf;

use structopt::StructOpt;

use cli_utils::{async_trait::async_trait, result, Result, Run};

use crate::image::Image;

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
    /// Equivalent to the `FROM` directive in a Dockerfile. Defaults to `stencila/femto`.
    /// Must be a valid image reference e.g. `docker.io/library/ubuntu:22.04`, `ubuntu:22.04`, `ubuntu`
    #[structopt(long, short, env = "STENCILA_IMAGE_FROM")]
    from: Option<String>,

    /// The registry, repository and tag to push to
    ///
    /// Equivalent to the `--tag` option to Docker build.
    /// Must be a valid image reference e.g. `localhost:5000/my-project`.
    /// Defaults to the name of the directory plus a hash of its path (to maintain uniqueness).
    #[structopt(long, short, multiple = true, env = "STENCILA_IMAGE_TAG")]
    tag: Option<String>,

    /// Directories that should be added as separate layers to the image
    ///
    /// Use a colon separated list of globs. Defaults to "<project>" and "/layers/*/*" (i.e. a layer for the project
    /// directory and one for each of the sub-sub-directories of "/layers" which are created by buildpacks).
    #[structopt(long, short, env = "STENCILA_IMAGE_LAYERS")]
    layers: Option<String>,

    /// Do not push the image
    #[structopt(long, short)]
    no_push: bool,

    /// The directory to write the image to
    ///
    /// Defaults to a temporary directory. Use this option when you want to inspect the contents
    /// of the image directory. When building within a container you can bind mount this volume from the host.
    /// 
    /// If the `layout_dir` already exists, its contents are deleted! Use this with care.
    #[structopt(long)]
    layout_dir: Option<PathBuf>,
}

#[async_trait]
impl Run for Build {
    async fn run(&self) -> Result {
        let layers_dirs: Vec<&str> = self
            .layers
            .as_ref()
            .map(|str| str.split(':').collect())
            .unwrap_or_default();

        let image = Image::new(
            self.project.as_deref(),
            self.tag.as_deref(),
            self.from.as_deref(),
            &layers_dirs,
            self.layout_dir.as_deref(),
        )?;

        image.build().await?;

        image.write().await?;

        if self.no_push {
            tracing::info!(
                "Image built and written to `{}`",
                image.layout_dir.display()
            );
        } else {
            image.push().await?;
            tracing::info!(
                "Image built and pushed to `{}/{}`",
                image.registry,
                image.repository
            );
        }

        result::value(image)
    }
}
