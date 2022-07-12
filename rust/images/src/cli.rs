use std::path::PathBuf;

use cli_utils::{
    clap::{self, Parser},
    common::async_trait::async_trait,
    result, Result, Run,
};
use common::tracing;

use crate::{
    distribution::{pull, push},
    image::Image,
    snapshot::Snapshot,
    storage::IMAGES_MAP,
};

/// Build and distribute container images
///
/// This subcommand provides a limited version of the functionality provided by
/// `docker` and `podman` CLI tools. It is not a general purpose container tool.
/// Only those commands needed by Stencila have been implemented.
#[derive(Parser)]
pub struct Command {
    #[clap(subcommand)]
    action: Action,
}

#[derive(Parser)]
enum Action {
    List(List),
    Build(Build),
    Pull(Pull),
    Push(Push),
    Remove(Remove),
    Snap(Snap),
}

#[async_trait]
impl Run for Command {
    async fn run(&self) -> Result {
        match &self.action {
            Action::List(action) => action.run().await,
            Action::Build(action) => action.run().await,
            Action::Pull(action) => action.run().await,
            Action::Push(action) => action.run().await,
            Action::Remove(action) => action.run().await,
            Action::Snap(action) => action.run().await,
        }
    }
}

/// List images in the local image store
///
/// Similar to `docker images` or `podman images` but only includes
/// container images that have been built or pulled by Stencila.
#[derive(Parser)]
struct List;

#[async_trait]
impl Run for List {
    async fn run(&self) -> Result {
        let images = IMAGES_MAP.read().await;
        let (images, table) = images.list();
        result::new("md", &table, images)
    }
}

/// Build an image
#[derive(Parser)]
struct Build {
    /// The directory to build an image for
    ///
    /// Defaults to the current directory.
    dir: Option<PathBuf>,

    /// The base image to build from
    ///
    /// Equivalent to the `FROM` directive in a Dockerfile. Defaults to the `STENCILA_IMAGE_REF` (i.e. the
    /// current image, if Stencila is running in a container), falling back to `stencila/stencila:nano` if not.
    /// Must be a valid image reference e.g. `docker.io/library/ubuntu:22.04`, `ubuntu:22.04`, `ubuntu`
    #[clap(long, short, env = "STENCILA_IMAGE_FROM")]
    from: Option<String>,

    /// The registry, repository and tag to push to
    ///
    /// Equivalent to the `--tag` option to Docker build.
    /// Must be a valid image reference e.g. `localhost:5000/my-project`.
    /// Defaults to the name of the directory plus a hash of its path (to maintain uniqueness).
    #[clap(long, short, env = "STENCILA_IMAGE_TAG")]
    tag: Option<String>,

    /// The format to use for image layers
    ///
    /// The Open Container Image spec allows for layers to be in several formats.
    /// The default `tar+zstd` format provides performance benefits over the others but may not be
    /// supported by older versions of some container tools.
    #[clap(
        long,
        env = "STENCILA_IMAGE_LAYER_FORMAT",
        default_value = "tar+zstd",
        possible_values = &["tar", "tar+gzip", "tgz", "tar+zstd", "tzs"]
    )]
    layer_format: String,

    /// The format to use for the image manifest
    ///
    /// Defaults to `oci`, however for compatibility with older version of some image registries it
    /// may be necessary to use `v2s2` (Docker Version 2 Schema 2).
    #[clap(
        long,
        env = "STENCILA_IMAGE_MANIFEST_FORMAT",
        default_value = "oci",
        possible_values = &["oci", "v2s2"]
    )]
    manifest_format: String,

    /// Do not create a layer for the workspace (i.e. ignore the `<dir>` argument)
    ///
    /// Mainly if you simply want to apply add `.env` and/or `.labels` files to the `--from` image
    /// and give it a new `--tag`.
    #[clap(long)]
    no_workspace: bool,

    /// Do not run any buildpacks
    ///
    /// Mainly useful during development for testing the writing of images, without waiting for
    /// potentially long buildpack build times.
    #[clap(long)]
    no_buildpacks: bool,

    /// Do not calculate a changeset for each layer directory and instead represent them in their entirety.
    ///
    /// The default behavior is to take snapshots of directories before and after the buildpacks build
    /// and generate a changeset representing the difference. This replicates the behavior of Dockerfile `RUN` directives.
    ///
    /// This option instead forces the layer to represent the entire directory after the build.
    #[clap(long)]
    no_diffs: bool,

    /// Do not write the image to disk after building it
    ///
    /// Mainly useful during development for testing that the image can be built without
    /// waiting for the base image manifest to be fetched or snapshot changesets to be calculated.
    #[clap(long)]
    no_write: bool,

    /// Do not push the image to the repository after writing it
    ///
    /// Mainly useful during development for testing that the image can be built without
    /// waiting for it to be pushed to the registry.
    #[clap(long)]
    no_push: bool,

    /// The directory where buildpacks build layers and which will are written into the image
    ///
    /// Defaults to a `/layers` (the usual in CNB images) or `<dir>/.stencila/layers` (the fallback
    /// for local builds).
    #[clap(long)]
    layers_dir: Option<PathBuf>,

    /// The directory to write the image to
    ///
    /// Defaults to a temporary directory. Use this option if you want to inspect the contents
    /// of the image directory. e.g. `stencila images build ... --no-build --no-push --layout-dir temp`.
    ///
    /// If the `layout_dir` already exists, its contents are deleted - so use with care!
    #[clap(long)]
    layout_dir: Option<PathBuf>,

    /// Whether the layout directory should be written with all layers
    ///
    /// As an optimization, base layers are only written to the layout directory as needed
    /// (i.e. when a registry does not have the layer yet). Use this option to ensure that layout directory
    /// includes all layers  (e.g. when wanting to run the image locally).
    #[clap(long)]
    layout_complete: bool,
}

#[async_trait]
impl Run for Build {
    async fn run(&self) -> Result {
        let working_dir = if self.no_workspace {
            None
        } else {
            Some(match self.dir.as_ref() {
                Some(dir) => dir.to_owned(),
                None => std::env::current_dir()?,
            })
        };

        let mut image = Image::new(
            working_dir.as_deref(),
            self.tag.as_deref(),
            self.from.as_deref(),
            self.layers_dir.as_deref(),
            Some(!self.no_diffs),
            Some(self.layer_format.as_str()),
            Some(self.manifest_format.as_str()),
        )?;

        if self.no_buildpacks {
            tracing::info!("Skipped build (--no-build option used).");
        } else {
            image.build().await?;
        }

        if self.no_write {
            tracing::info!(
                "Image built successfully. Skipping write and push (--no-write option used)."
            );
        } else {
            image.write().await?;

            if self.no_push {
                tracing::info!(
                    "Image built and written to ``.",
                    //image.layout_dir().display()
                );
            } else {
                //image.push().await?;
                tracing::info!(
                    "Image built and pushed to `{}`.",
                    image.reference().to_string()
                );
            }
        }

        result::value(image)
    }
}

/// Pull an image from a registry
///
/// Equivalent to `docker pull` and `podman pull`.
#[derive(Parser)]
struct Pull {
    /// The image to pull
    image: String,
}

#[async_trait]
impl Run for Pull {
    async fn run(&self) -> Result {
        let image = pull(&self.image).await?;
        result::value(image)
    }
}

/// Push an image to a registry
///
/// Similar to `podman pull` in that it allows an image to be pushed from
/// one image reference to another (without having to tag first as with `docker`).
#[derive(Parser)]
struct Push {
    /// The image to push
    image: String,

    /// The reference to push the image to (if different)
    to: Option<String>,

    /// Force a direct transfer from the source registry to the destination registry
    #[clap(long, short)]
    force_direct: bool,
}

#[async_trait]
impl Run for Push {
    async fn run(&self) -> Result {
        let to = push(&self.image, self.to.as_deref(), self.force_direct).await?;
        result::value(to)
    }
}

/// Remove an image from the local image store
///
/// Equivalent to `docker rmi` and `podman rmi`.
#[derive(Parser)]
struct Remove {
    /// The image to remove (a reference, id, or hash component of id)
    image: String,
}

#[async_trait]
impl Run for Remove {
    async fn run(&self) -> Result {
        let mut image_map = IMAGES_MAP.write().await;
        let ids = image_map.remove(&self.image)?;
        if ids.is_empty() {
            tracing::warn!("No images matching `{}` to remove", self.image);
            result::nothing()
        } else {
            tracing::info!("Removed {} images", ids.len());
            result::value(ids)
        }
    }
}

/// Take a snapshot of the filesystem
///
/// This command is used create a snapshot of the filesystem that can be used
/// by the `save` command to generate an image layer based on the changes since the snapshot.
///
/// Defaults to creating a snapshot of the entire filesystem but a directory can be specified.
/// Creates a `.snap` file next to the directory that is snap shotted (i.e. defaults to `/root.snap`)
///
/// Snapshots are usually made within a container or virtual machine and may be slow if run
/// on a large filesystem. To avoid inadvertent snapshots users are asked for confirmation
/// (this can be skipped by using the `--yes` option).
#[derive(Parser)]
struct Snap {
    /// Path of the directory to snapshot
    #[clap(default_value = "/")]
    dir: PathBuf,

    /// Do not ask for confirmation
    #[clap(short, long)]
    yes: bool,
}

#[async_trait]
impl Run for Snap {
    async fn run(&self) -> Result {
        let filename = self
            .dir
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "root".to_string())
            + ".snap";
        let path = self
            .dir
            .parent()
            .map(|parent| parent.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("/"))
            .join(filename);

        if !self.yes {
            println!(
                "Are you sure you want to snapshot directory `{}`? (y/n)",
                self.dir.display()
            );
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if input.trim().to_lowercase() != "y" {
                tracing::info!("Cancelling snapshot");
                return result::nothing();
            }
        }

        let snapshot = Snapshot::new(self.dir.clone());
        snapshot.write(&path)?;

        tracing::info!(
            "Successfully created snapshot `{}` of directory `{}`",
            path.display(),
            self.dir.display()
        );

        result::nothing()
    }
}
