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
    /// The directory to build an image for
    ///
    /// Defaults to the current directory.
    dir: Option<PathBuf>,

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
    #[structopt(long, short, env = "STENCILA_IMAGE_TAG")]
    tag: Option<String>,

    /// Directories that should be added as separate layers to the image
    ///
    /// Use a colon separated list of globs. Defaults to "<dir>" and "/layers/*/*" (i.e. a layer for the working
    /// directory and one for each of the sub-sub-directories of "/layers" which are created by buildpacks).
    #[structopt(long, short, env = "STENCILA_IMAGE_LAYERS")]
    layers: Option<String>,

    /// Do not actually build the image
    ///
    /// Mainly useful during development for testing the parsing of inputs such as `--from`
    /// and `--tag` image references.
    #[structopt(long)]
    no_build: bool,

    /// Do not write the image to disk after building it
    ///
    /// Mainly useful during development for testing that the image can be built without
    /// waiting for downloads of base image manifest or snapshot changesets to be calculated.
    #[structopt(long)]
    no_write: bool,

    /// Do not push the image to the repository after writing it
    ///
    /// Mainly useful during development for testing that the image can be built without
    /// waiting for it to be pushed to the registry.
    #[structopt(long)]
    no_push: bool,

    /// The directory to write the image to
    ///
    /// Defaults to a temporary directory. Use this option if you want to inspect the contents
    /// of the image directory. When building within a container you can bind mount this volume from the host.
    ///
    /// If the `layout_dir` already exists, its contents are deleted - so use with care!
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
            self.dir.as_deref(),
            self.tag.as_deref(),
            self.from.as_deref(),
            &layers_dirs,
            self.layout_dir.as_deref(),
        )?;

        if self.no_build {
            tracing::info!("Skipped build because --no-build option used");
        } else {
            image.build().await?;

            if self.no_write {
                tracing::info!("Image built successfully");
            } else {
                image.write().await?;

                if self.no_push {
                    tracing::info!(
                        "Image built and written to `{}`",
                        image.layout_dir.display()
                    );
                } else {
                    image.push().await?;
                    tracing::info!("Image built and pushed to `{}`", image.ref_.to_string());
                }
            }
        }

        result::value(image)
    }
}
