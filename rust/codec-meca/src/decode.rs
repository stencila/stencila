use std::path::Path;

use codec::{
    Codec, DecodeInfo, DecodeOptions,
    eyre::{OptionExt, Result},
    schema::Node,
};
use glob::glob;
use codec_jats::JatsCodec;
use media_embed::embed_media;
use zip::ZipArchive;

/// Decode a MECA file to a Stencila [`Node`]
#[tracing::instrument]
pub(super) async fn decode_path(
    path: &Path,
    options: Option<DecodeOptions>,
) -> Result<(Node, Option<Node>, DecodeInfo)> {
    // Create temporary directory to extract into
    // if path is not already a directory (e.g. an unzipped MECA)
    let tempdir = tempfile::TempDir::new()?;
    let dir = if path.is_dir() { path } else { tempdir.path() };

    if path.is_file() {
        tracing::debug!("Extracting MECA");
        let file = std::fs::File::open(path)?;
        let mut zip = ZipArchive::new(file)?;
        zip.extract(dir)?;
    }

    let dir = dir.join("content");

    // Find the JATS file in the dir
    let jats_path = glob(&dir.join("*.xml").to_string_lossy())?
        .next()
        .and_then(|res| res.ok())
        .ok_or_eyre("Unable to find JATS XML file")?;

    // Decode the JATS
    let (mut node, .., info) = JatsCodec.from_path(&jats_path, options).await?;

    // Embed any images
    embed_media(&mut node, &dir)?;

    Ok((node, None, info))
}
