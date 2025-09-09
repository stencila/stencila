use std::path::Path;

use glob::glob;
use stencila_node_supplements::embed_supplements;
use zip::ZipArchive;

use stencila_codec::{
    Codec, DecodeInfo, DecodeOptions,
    eyre::{OptionExt, Result},
    stencila_schema::Node,
};
use stencila_codec_jats::JatsCodec;
use stencila_node_media::embed_media;

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

    // Embed media and supplements
    embed_media(&mut node, &dir)?;
    embed_supplements(&mut node, &dir).await?;

    Ok((node, None, info))
}
