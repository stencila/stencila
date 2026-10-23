use std::path::Path;

use flate2::read::GzDecoder;
use glob::glob;
use stencila_node_supplements::embed_supplements;
use tar::Archive;
use tempfile::tempdir;

use stencila_codec::{
    Codec, DecodeInfo, DecodeOptions,
    eyre::{OptionExt, Result, WrapErr},
    stencila_schema::Node,
};
use stencila_codec_jats::JatsCodec;
use stencila_node_media::{embed_media, relocate_media, side_dir};

/// Decode a PMC OA Package (tar.gz) to a Stencila [`Node`]
#[tracing::instrument]
pub(super) async fn decode_tar(
    path: &Path,
    options: Option<DecodeOptions>,
) -> Result<(Node, Option<Node>, DecodeInfo)> {
    // Create temporary directory to extract into
    // if path is not already a directory (e.g. an unzipped PMC OA Package)
    let tempdir = tempdir()?;
    let dir = if path.is_dir() { path } else { tempdir.path() };

    if path.is_file() {
        tracing::debug!("Extracting PMC OA package");
        let file = std::fs::File::open(path)?;
        let tar = GzDecoder::new(file);
        let mut archive = Archive::new(tar);
        archive.unpack(dir)?;
    }

    // Find the PMCXXXX directory within the dir
    let dir = glob(&dir.join("PMC*").to_string_lossy())?
        .flatten()
        .next()
        .ok_or_eyre("Unable to find PMC subdirectory in archive")?;

    // Find the JATS file in the dir
    let jats_path = glob(&dir.join("*.nxml").to_string_lossy())?
        .next()
        .and_then(|res| res.ok())
        .ok_or_eyre("Unable to find JATS XML file in PMC OA PAckage")?;

    // Decode the JATS
    let mut jats_options = options.clone().unwrap_or_default();
    jats_options.embed_media = Some(false);
    jats_options.embed_supplements = Some(false);
    let (mut node, .., info) = JatsCodec.from_path(&jats_path, Some(jats_options)).await?;

    // Embed media by default
    if options
        .as_ref()
        .and_then(|opts| opts.embed_media)
        .unwrap_or(true)
    {
        embed_media(&mut node, Some(&dir))?;
    }

    // Embed supplements by default
    if options
        .as_ref()
        .and_then(|opts| opts.embed_supplements)
        .unwrap_or(true)
    {
        embed_supplements(&mut node, &dir).await?;
    }

    let reference_media = options.as_ref().and_then(|opts| opts.embed_media) == Some(false);
    let reference_supplements =
        options.as_ref().and_then(|opts| opts.embed_supplements) == Some(false);
    if reference_media || reference_supplements {
        let media_dir = (reference_media && path.is_file()).then(|| side_dir(path, ".media"));
        let supplements_dir =
            (reference_supplements && path.is_file()).then(|| side_dir(path, ".supplements"));
        relocate_media(
            &mut node,
            Some(&jats_path),
            Some(path),
            media_dir.as_deref(),
            supplements_dir.as_deref(),
        )
        .wrap_err_with(|| retain_error(path))?;
    }

    Ok((node, None, info))
}

/// Error context when originals can not be retained beside an archive
fn retain_error(path: &Path) -> String {
    format!(
        "Could not retain original media and supplements beside `{}`; is its directory writable? \
         Decode with embedding enabled (e.g. `--embed-media --embed-supplements`) or decode a \
         writable copy of the archive",
        path.display()
    )
}
