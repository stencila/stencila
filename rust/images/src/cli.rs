use std::path::PathBuf;

use cli_utils::{
    clap::{self, Parser},
    common::async_trait::async_trait,
    result, Result, Run,
};
use common::tracing;

use crate::{
    change_set::{Change, ChangeSet},
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
    Pull(Pull),
    Push(Push),
    Remove(Remove),
    Snap(Snap),
    Save(Save),
}

#[async_trait]
impl Run for Command {
    async fn run(&self) -> Result {
        match &self.action {
            Action::List(action) => action.run().await,
            Action::Pull(action) => action.run().await,
            Action::Push(action) => action.run().await,
            Action::Remove(action) => action.run().await,
            Action::Snap(action) => action.run().await,
            Action::Save(action) => action.run().await,
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

/// Save a container as an image layer
#[derive(Parser)]
struct Save {
    /// The registry, repository and tag to give the image
    ///
    /// Equivalent to the `--tag` option to Docker build.
    /// Must be a valid image reference e.g. `localhost:5000/my-project`
    #[clap(long, short, aliases = &["ref", "tag"], env = "STENCILA_IMAGE_REF")]
    reference: String,

    /// The base image to build from
    ///
    /// Equivalent to the `FROM` directive in a Dockerfile. Defaults to the `STENCILA_IMAGE_REF` (i.e. the
    /// current image, if Stencila is running in a container), falling back to `stencila/stencila:nano` if not.
    /// Must be a valid image reference e.g. `docker.io/library/ubuntu:22.04`, `ubuntu:22.04`, `ubuntu`
    #[clap(long, short, alias = "from")]
    base: Option<String>,

    /// The path of the snapshot to use as the base for the layer changeset
    ///
    /// Defaults to `/root.snap` which is the default path for a snapshot generated
    /// from the `snap` command.
    #[clap(short, long)]
    snapshot: Option<PathBuf>,

    /// The format to use for image layers
    ///
    /// The Open Container Image spec allows for layers to be in several formats.
    /// The default `tar+zstd` format provides performance benefits over the others but may not be
    /// supported by older versions of some container tools.
    #[clap(
        long,
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
        default_value = "oci",
        possible_values = &["oci", "v2s2"]
    )]
    manifest_format: String,

    /// Do not push the image to the repository after writing it
    ///
    /// Mainly useful during development for testing that the image can be built without
    /// having to have a registry to push it to.
    #[clap(long)]
    no_push: bool,
}

#[async_trait]
impl Run for Save {
    async fn run(&self) -> Result {
        let snapshot_path = self
            .snapshot
            .clone()
            .unwrap_or_else(|| PathBuf::from("/root.snap"));

        // If there was no snapshot exit early. This also acts as a safeguard from running
        // this command on local machines (which is probably not wanted)
        if !snapshot_path.exists() {
            tracing::warn!(
                "The base snapshot `{}` does not exist so changes can not be saved",
                snapshot_path.display()
            );
            return result::nothing();
        }

        // Repeat the snapshot and generate a change set for it
        let snapshot1 = Snapshot::read(&snapshot_path)?;
        let snapshot2 = snapshot1.repeat();
        let root_change_set = snapshot1.diff(&snapshot2);

        // Write the new snapshot and create a separate changeset for it
        // so it can be added as a new layer. This means that when the image is started
        // the next time that we do not need to wait for the first snapshot to be generated.
        snapshot2.write(&snapshot_path)?;
        let mut snapshot_change_set = ChangeSet::new(
            "/",
            Some("/"),
            vec![Change::Modified(
                snapshot_path
                    .file_name()
                    .expect("To have a file name")
                    .to_string_lossy()
                    .to_string(),
            )],
        );
        snapshot_change_set.comment =
            Some(format!("Snapshot for / with {} items", snapshot2.size()));

        let mut image = Image::new(
            &self.reference,
            self.base.as_deref(),
            vec![root_change_set, snapshot_change_set],
            Some(self.layer_format.as_str()),
            Some(self.manifest_format.as_str()),
        )?;

        image.write().await?;

        if !self.no_push {
            image.push().await?;
            tracing::info!(
                "Image built and pushed to `{}`.",
                image.reference().to_string()
            );
        } else {
            tracing::info!("Image built and written to storage");
        }

        result::nothing()
    }
}
