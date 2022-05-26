use std::{env, path::Path};

use eyre::Result;
use tokio::fs::create_dir_all;

use crate::{distribution::Client, image::ImageSpec};

pub async fn build<L>(
    project_dir: Option<&Path>,
    layer_dirs: &[L],
    from_image: Option<&str>,
    to_image: Option<&str>,
    image_dir: Option<&Path>,
) -> Result<()>
where
    L: AsRef<str>,
{
    let project_dir = project_dir
        .map(|dir| dir.to_path_buf())
        .unwrap_or_else(|| env::current_dir().expect("Unable to get cwd"));

    let image_dir = image_dir
        .map(|dir| dir.to_path_buf())
        .unwrap_or_else(|| project_dir.join(".stencila").join("image"));
    create_dir_all(image_dir).await?;

    // Get the diffids of the base image so we can include them in the new image

    let from_image = from_image
        .map(|image| image.to_string())
        .unwrap_or_else(|| "stencila/femto".to_string());
    let from_image: ImageSpec = from_image.parse()?;

    let from_client = Client::new(&from_image.registry, &from_image.name, None);
    let from_config = from_client.get_config(from_image.reference()).await?;
    let from_diffids = from_config.rootfs().diff_ids();

    println!("{:?}", from_diffids);

    // Create a snapshot of each of the `layer_dirs`
    
    Ok(())
}
